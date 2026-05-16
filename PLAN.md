# ratatui-markdown Roadmap

All previously planned features have been implemented. This document tracks new
requirements that emerged during TUI integration work and their status.

---

## Section 8 — Dual-Mode Text Input/Display Component (`TextInput`) — DONE

**Status**: Implemented in `src/text_input/` (commits `1b23911`, `5cc646f`).

### What was implemented

| Component | File | Description |
|---|---|---|
| `CursorShape` | `types.rs` | Block, Bar (`│`), Underline, HollowBlock |
| `CursorPosition` | `types.rs` | OnChar (block/underline), BeforeChar (bar) |
| `CursorStyle` | `types.rs` | Builder: shape + position + fg/bg/modifier |
| `SelectionStyle` | `types.rs` | Builder: fg/bg (defaults to theme) |
| `InputMode` | `types.rs` | Edit (source-visible), Read (rendered markdown) |
| `Selection` | `types.rs` | start/end with ordered() |
| `CursorBlinkController` | `types.rs` | Trait with `is_visible() -> bool` |
| `TextInput` | `mod.rs` | Full component: builder API, editing ops, render dispatch |
| Cursor rendering | `cursor.rs` | `apply_cursor_and_selection()` — shape-aware cursor + selection bg |
| Edit-mode rendering | `edit_render.rs` | Source-visible with syntax styling (headings, bold, italic, code, links, strikethrough, blockquote, lists, code fences) |
| Read-mode rendering | `read_render.rs` | Delegates to `MarkdownRenderer` for full markdown rendering |
| Horizontal scroll | `edit_render.rs` | Unicode-aware width calculation + truncation |

### API

```rust
let mut input = TextInput::new()
    .with_mode(InputMode::Edit)
    .with_cursor_style(CursorStyle::new().with_shape(CursorShape::Bar))
    .with_blink_controller(Rc::new(my_blink_controller))
    .with_password(false)
    .with_placeholder("type here...")
    .with_max_width(80);

input.set_text("# Hello **world**");
input.set_cursor_char_idx(5);
input.render(f, area, &theme);
```

### Integration TODO (entelecheia side)

| File | Change | Priority |
|---|---|---|
| `widgets/text_input/` | Consider wrapping or replacing with library TextInput | Medium |
| `conversation/rendering.rs:129-274` | Remove duplicated input rendering | High |
| `modal/modal_impl/render_impl.rs:491-573` | Remove duplicated input rendering | High |
| `widgets/animation/manager.rs` | Implement `CursorBlinkController` trait | Medium |

---

## Section 9 — SpanTree Enhancement: Per-Line Cursor Column — DONE

**Status**: Implemented in `src/scroll/span_tree/` (commit `1b23911`).

### What was implemented

| Component | Description |
|---|---|
| `CursorLineMode` enum | `HeaderOnly` (default), `AllLines` |
| `SpanTree::with_cursor_line_mode()` | Builder method |
| `SpanTree::cursor_line_mode()` | Accessor |
| Render logic | `render.rs:44-48` — cursor replacement respects `CursorLineMode` |

### Usage

```rust
let tree = SpanTree::new()
    .with_cursor_line_mode(CursorLineMode::AllLines);
```

---

## Section 10 — Remaining TUI Migration Candidates

These are entelecheia-side migrations that use existing library components.

### High Priority

| Target | Pattern | Est. Savings |
|---|---|---|
| `conversation/rendering.rs:129-274` | Remove duplicated input rendering | ~145 lines |
| `modal/render_impl.rs:491-573` | Remove duplicated input rendering | ~80 lines |

### Medium Priority

| Target | Pattern | Est. Savings |
|---|---|---|
| `agent_config/render.rs` | ScrollableList / SpanTree | ~60 lines |
| `command_palette.rs` | ScrollableList | ~40 lines |
| `models_page/` panels | ScrollableList / SpanTree | ~120 lines |
| `agents_page` list | ScrollableList | ~60 lines |

### Low Priority

| Target | Pattern | Est. Savings |
|---|---|---|
| `help_page`, `theme_page`, `language_page` | ScrollableList | ~30 lines each |
| `completer.rs`, `mention_completer.rs` | ScrollableList | ~20 lines each |
