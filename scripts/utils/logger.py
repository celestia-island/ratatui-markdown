#!/usr/bin/env python3
"""Unified dev-script logger — columnar output aligned with Rust `tracing`.

Canonical layout (source first, so streamed Rust tracing lines — which already
carry TIME/LEVEL/TARGET — align by simply prepending the source tag):

    SOURCE     TIME      LEVEL  MODULE       MESSAGE
    backend    21:49:58  INFO   chest_cli    shittim-chest started on 0.0.0.0:3000
    scepter    21:49:58  OK     preflight    rootless fuse-overlayfs available
    i18n       21:50:00  WARN   check_i18n   3 unused keys detected

Levels (ANSI, severity-coloured; triple-gated on tty + NO_COLOR + TERM=dumb):
    DEBUG (dim)   INFO (cyan)   OK (green)   WARN (yellow)   ERROR (red)

Two emit modes:
  - log()/debug()/info()/ok()/warn()/error() : full 5-column line (own message)
  - tag(source, line)                        : prepend SOURCE column to a line
                                               streamed from a subprocess (ANSI
                                               escapes stripped, then re-aligned
                                               to the columnar layout)

Vendored copy — each repo hosts a byte-identical file. Keep copies in sync with
the project's sync_devkit tool. This is NOT a shared just-common-style path
import; every repo is self-contained.
"""
from __future__ import annotations

import os
import re
import shutil
import sys
import textwrap
from datetime import datetime
from typing import IO, Optional

__all__ = [
    "Logger",
    "configure",
    "log",
    "debug",
    "info",
    "ok",
    "warn",
    "error",
    "section",
    "tag",
    "blank",
    "rule",
]

_LEVELS = {
    "DEBUG": "dim",
    "INFO": "cyan",
    "OK": "green",
    "WARN": "yellow",
    "ERROR": "red",
}

# Canonical column widths — the single source of truth. Every caller shares
# these so the TIME/LEVEL/MODULE columns align across processes in one session
# (e.g. dev.py + its imported preflight + its check_i18n subprocess). Override
# only if a whole session genuinely needs different widths.
SOURCE_WIDTH = 13
MODULE_WIDTH = 12

_ANSI = {
    "dim": "\033[2m",
    "cyan": "\033[0;36m",
    "green": "\033[0;32m",
    "yellow": "\033[1;33m",
    "red": "\033[0;31m",
    "bold": "\033[1m",
    "reset": "\033[0m",
}

_TIME_FMT = "%H:%M:%S"

# Strip ANSI escape codes (tracing_subscriber emits them even when piped).
_ANSI_RE = re.compile(r'\033\[[0-9;]*m')


def _strip_ansi(text: str) -> str:
    return _ANSI_RE.sub('', text)


# Match Rust tracing output: "HH:MM:SS  LEVEL  target: message"
# (tracing's default Full format with ShortTimer, after ANSI stripping).
# Used by tag() to re-align subprocess output with the Python columnar layout.
_RUST_TRACE_RE = re.compile(
    r'^(\d{2}:\d{2}:\d{2})\s+'
    r'(DEBUG|INFO|WARN|ERROR|TRACE)\s+'
    r'(.+?):\s(.*)$'
)


def _color_enabled(stream: IO) -> bool:
    if os.environ.get("NO_COLOR"):
        return False
    if os.environ.get("TERM") == "dumb":
        return False
    isatty = getattr(stream, "isatty", None)
    return bool(isatty() if callable(isatty) else False)


def _term_width(default: int = 120) -> int:
    try:
        return shutil.get_terminal_size((default, 24)).columns
    except OSError:
        return default


def _script_module() -> str:
    name = os.path.splitext(os.path.basename(sys.argv[0]))[0] if sys.argv and sys.argv[0] else ""
    return name or "script"


def _wrap(text: str, width: int, indent: str) -> str:
    if width <= 1 or not text:
        return text
    # Wrap each paragraph to `width` (the message-column width), then prepend
    # `indent` to every line after the first. We deliberately do NOT pass
    # subsequent_indent to textwrap: its `width` is the TOTAL line width
    # INCLUDING the indent, so using it made continuation lines wrap
    # `len(indent)` columns narrower than the first line (the first line got
    # `width` chars of text, continuations got `width - len(indent)`).
    raw_lines: list[str] = []
    for paragraph in text.splitlines() or [""]:
        if not paragraph:
            raw_lines.append("")
            continue
        raw_lines.extend(
            textwrap.wrap(
                paragraph,
                width=width,
                break_long_words=True,
                break_on_hyphens=False,
            )
            or [""]
        )
    if not raw_lines:
        return text
    out = [raw_lines[0]]
    for line in raw_lines[1:]:
        out.append(indent + line if line else line)
    return "\n".join(out)


class Logger:
    """Columnar logger. Instantiate for a fixed source/module, or use the
    module-level functions (which share a process-wide default Logger)."""

    def __init__(
        self,
        source: str = "dev",
        module: Optional[str] = None,
        *,
        source_width: int = SOURCE_WIDTH,
        module_width: int = MODULE_WIDTH,
        stream: Optional[IO] = None,
    ) -> None:
        self.source = source
        self.module = module or _script_module()
        self.source_width = source_width
        self.module_width = module_width
        self.stream: IO = stream if stream is not None else sys.stdout
        self._color = _color_enabled(self.stream)

    def configure(
        self,
        *,
        source: Optional[str] = None,
        module: Optional[str] = None,
        source_width: Optional[int] = None,
        module_width: Optional[int] = None,
        stream: Optional[IO] = None,
    ) -> "Logger":
        if source is not None:
            self.source = source
        if module is not None:
            self.module = module
        if source_width is not None:
            self.source_width = source_width
        if module_width is not None:
            self.module_width = module_width
        if stream is not None:
            self.stream = stream
            self._color = _color_enabled(self.stream)
        return self

    def _paint(self, name: str, text: str) -> str:
        if not self._color or name not in _ANSI:
            return text
        return f"{_ANSI[name]}{text}{_ANSI['reset']}"

    def _now(self) -> str:
        return datetime.now().strftime(_TIME_FMT)

    @property
    def _msg_offset(self) -> int:
        # visible columns before the message: src + 2 + time(8) + 2 + level(5) + 2 + mod + 2
        return self.source_width + 2 + 8 + 2 + 5 + 2 + self.module_width + 2

    def _emit_line(self, prefix: str, msg: str, *, overflow: bool = False, err: bool = False) -> None:
        """Render prefix + message, wrapping the message at the correct column.

        When *overflow* is True some column (source or module) exceeded its
        allotted width; the prefix is emitted as its own line and the message
        starts on the next line aligned at the theoretical message offset so
        continuation lines stay aligned with other processes' output. A blank
        message emits no second line (avoids a whitespace-only row)."""
        offset = self._msg_offset
        term = _term_width()
        content_w = term - offset
        if content_w >= offset + 20:
            body = _wrap(msg, content_w, " " * offset)
        else:
            body = msg
        if overflow:
            self._emit(prefix, err=err)
            if body:
                self._emit(" " * offset + body, err=err)
        else:
            self._emit(prefix + ("  " + body if body else ""), err=err)

    def log(
        self,
        level: str,
        message: str,
        *,
        source: Optional[str] = None,
        module: Optional[str] = None,
    ) -> None:
        level = level.upper()
        if level not in _LEVELS:
            level = "INFO"
        color = _LEVELS[level]
        lvl = level.rjust(5)
        src_raw = source if source is not None else self.source
        mod_raw = module if module is not None else self.module
        overflow = len(src_raw) > self.source_width or len(mod_raw) > self.module_width
        prefix = "  ".join(
            [
                self._paint("dim", src_raw.ljust(self.source_width)),
                self._paint("dim", self._now()),
                self._paint(color, lvl),
                self._paint("dim", mod_raw.ljust(self.module_width)),
            ]
        )
        self._emit_line(prefix, str(message), overflow=overflow, err=(level == "ERROR"))

    def debug(self, message: str, **kw) -> None:
        self.log("DEBUG", message, **kw)

    def info(self, message: str, **kw) -> None:
        self.log("INFO", message, **kw)

    def ok(self, message: str, **kw) -> None:
        self.log("OK", message, **kw)

    def warn(self, message: str, **kw) -> None:
        self.log("WARN", message, **kw)

    def error(self, message: str, **kw) -> None:
        self.log("ERROR", message, **kw)

    def tag(self, source: str, line: str) -> None:
        """Prepend the columnar header to a line streamed from a subprocess.

        Rust ``tracing`` lines (``HH:MM:SS LEVEL TARGET: msg``) are reparsed
        and re-emitted with the same column widths as :meth:`log`, so the
        TIME/LEVEL/MODULE columns align across Python and Rust output. Plain
        lines (cargo progress, mock-LLM stdout, bare stderr) are emitted as
        full columnar lines too — current timestamp, INFO level, and the source
        as the module — so every streamed line carries the same columns instead
        of a bare source+text that breaks alignment."""
        text = _strip_ansi(line.rstrip("\n\r"))
        src = self._paint("dim", source.ljust(self.source_width))

        m = _RUST_TRACE_RE.match(text)
        if m:
            time_s, level, target, msg = m.groups()
            level = level.upper()
            if level == "TRACE":
                level = "DEBUG"
            # Strip trailing ':' from target (tracing appends it), then take
            # the leaf segment so long module paths (e.g.
            # "scepter::state_machine::skill_chain::execution::merge_coordinator")
            # don't push the message column far to the right.
            target = target.rstrip(":").split("::")[-1]
        else:
            # Non-tracing line (cargo, deno/node console.log, plain stderr):
            # synthesize the missing columns so it aligns with log() output.
            time_s, level, target, msg = self._now(), "INFO", source, text

        color = _LEVELS.get(level, "dim")
        lvl = level.rjust(5)
        overflow = len(source) > self.source_width or len(target) > self.module_width
        prefix = "  ".join([
            src,
            self._paint("dim", time_s),
            self._paint(color, lvl),
            self._paint("dim", target.ljust(self.module_width)),
        ])
        self._emit_line(prefix, msg, overflow=overflow, err=(level == "ERROR"))

    def section(self, title: str) -> None:
        self._emit(self._paint("bold", f"==> {title}"))

    def blank(self) -> None:
        self._emit("")

    def rule(self) -> None:
        self._emit(self._paint("dim", "-" * min(_term_width(), 80)))

    def _emit(self, text: str, *, err: bool = False) -> None:
        stream = self.stream if not err else (sys.stderr if self.stream is sys.stdout else self.stream)
        try:
            stream.write(text + "\n")
            stream.flush()
        except (BrokenPipeError, ValueError):
            pass


_default = Logger()


def configure(**kw) -> Logger:
    return _default.configure(**kw)


def log(level: str, message: str, **kw) -> None:
    _default.log(level, message, **kw)


def debug(message: str, **kw) -> None:
    _default.debug(message, **kw)


def info(message: str, **kw) -> None:
    _default.info(message, **kw)


def ok(message: str, **kw) -> None:
    _default.ok(message, **kw)


def warn(message: str, **kw) -> None:
    _default.warn(message, **kw)


def error(message: str, **kw) -> None:
    _default.error(message, **kw)


def section(title: str) -> None:
    _default.section(title)


def tag(source: str, line: str) -> None:
    _default.tag(source, line)


def blank() -> None:
    _default.blank()


def rule() -> None:
    _default.rule()
