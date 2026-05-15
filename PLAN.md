# ratatui-markdown Enhancement Plan

This document describes the planned enhancements needed to support richer TUI rendering,
driven by the entelecheia project's migration from manual rendering to ratatui-markdown.

## 1. Nested Block Support (Critical)

### Problem

The parser is strictly flat — it produces `Vec<MarkdownBlock>` with no parent-child relationships.
This means the following markdown patterns are broken:

**Code block inside blockquote:**
```markdown
> Here's a code example:
> ```rust
> fn main() {}
> ```
```
`> ```rust` is parsed as `Blockquote`, not as a fenced code block start.

**Multi-level blockquotes:**
```markdown
> Level 1
> > Level 2
> > > Level 3
```
Each `>` line becomes an independent `Blockquote(text)` — no nesting information is preserved.

**Lists inside blockquotes, blockquotes inside lists, etc.** — none work.

### Proposed Solution

#### Option A: Recursive MarkdownBlock (Recommended)

Replace the flat string content in `Blockquote` with a recursive structure:

```rust
pub enum MarkdownBlock {
    // ... existing variants ...

    Blockquote {
        level: u8,                    // nesting depth (1, 2, 3, ...)
        children: Vec<MarkdownBlock>, // nested blocks
    },
}
```

For backward compatibility, keep `Blockquote(String)` as a deprecated alias that produces
`Blockquote { level: 1, children: vec![Paragraph(vec![text])] }`.

The parser needs to detect:
- Consecutive `>` lines that form a logical blockquote group
- `> ` prefix stripping before re-parsing inner content
- `>> ` as level-2 blockquote, `>>> ` as level-3, etc.
- `> ```lang` as start of fenced code inside blockquote

The renderer needs to:
- Recursively render children with indentation/prefix per level
- Accumulate vertical line prefixes: level 1 = `│ `, level 2 = `│ │ `, etc.
- Support the `RenderHooks::blockquote` hook with level information

#### Option B: Generic NestedBlock container

```rust
pub enum MarkdownBlock {
    // ... existing variants ...

    NestedBlock {
        kind: NestedKind,
        children: Vec<MarkdownBlock>,
    },
}

pub enum NestedKind {
    Blockquote { level: u8 },
    Decorated {
        header: Option<String>,
        footer: Option<String>,
        prefix: Option<String>,
    },
}
```

Option B is more general but adds complexity. Option A is simpler and covers the primary use case.

### Renderer Changes

`render.rs` needs a method like:

```rust
fn render_nested_blockquote(
    &self,
    level: u8,
    children: &[MarkdownBlock],
    theme: &impl RichTextTheme,
    lines: &mut Vec<Line<'static>>,
) {
    let prefix = "│ ".repeat(level as usize);
    // render children into temp buffer, then prefix each line
    let mut inner = Vec::new();
    for child in children {
        self.render_block(child, ..., &mut inner);
    }
    for mut line in inner {
        line.spans.insert(0, Span::styled(prefix.clone(), muted_style));
        lines.push(line);
    }
}
```

### Parser Changes

`parser.rs` needs:

1. **Blockquote group detection**: Collect consecutive `>` lines into a group, strip `> ` prefix, re-parse inner content.
2. **Nested `>` detection**: Count leading `>` characters to determine level.
3. **Fenced code inside blockquote**: When inside a blockquote group, detect `> ```lang` as code block start.

## 2. Manual Node Construction: Header/Footer Override (High Priority)

### Problem

`RenderHooks::code_block_header/footer` are global — they receive only `lang` and cannot
produce different headers for different code blocks of the same language.

Use case from entelecheia:
```
╭─ Input ────     ← code block titled "Input"
│ { ... }
╰────────────

╭─ Output ───     ← code block titled "Output"
│ { ... }
╰────────────
```

Currently impossible without encoding metadata in the `lang` field.

### Proposed Solution

Add optional override fields to `MarkdownBlock::CodeBlock`:

```rust
CodeBlock {
    lang: String,
    code: String,
    header_override: Option<String>,  // replaces default "╭─ {lang} ──"
    footer_override: Option<String>,  // replaces default "╰──────"
    prefix_override: Option<String>,  // replaces default "│ "
}
```

The renderer should check these fields before consulting hooks:

```rust
MarkdownBlock::CodeBlock { lang, code, header_override, footer_override, prefix_override } => {
    if let Some(ref header) = header_override {
        lines.push(Line::from(Span::styled(header.clone(), ...)));
    } else if let Some(custom) = hooks.and_then(|h| h.code_block_header(lang)) {
        lines.push(custom);
    } else {
        lines.push(self.default_code_block_header(lang, theme));
    }
    // ... similar for footer and prefix
}
```

This is a breaking change to the enum. To minimize breakage, provide constructor helpers:

```rust
impl MarkdownBlock {
    pub fn code_block(lang: impl Into<String>, code: impl Into<String>) -> Self {
        Self::CodeBlock {
            lang: lang.into(),
            code: code.into(),
            header_override: None,
            footer_override: None,
            prefix_override: None,
        }
    }
}
```

### Also: Blockquote header/footer

Similar override mechanism for blockquotes:

```rust
Blockquote {
    level: u8,
    children: Vec<MarkdownBlock>,
    header_override: Option<String>,  // line before children
    footer_override: Option<String>,  // line after children
}
```

## 3. Mermaid Block Handling (Medium Priority)

### Problem

Mermaid code blocks are silently skipped (`render.rs:294`). No fallback, no indication,
and no way for the caller to intercept via hooks (the check happens before hooks are consulted).

### Proposed Solution

Move the mermaid check to be a hook-like decision:

```rust
fn render_code_block(...) {
    // Let hooks decide first
    if let Some(h) = hooks {
        if let Some(custom) = h.code_block(lang, content) {
            lines.extend(custom);
            return;
        }
    }
    // Default: skip mermaid
    if lang == "mermaid" { return; }
    // ... normal rendering
}
```

Add a new hook method:

```rust
fn code_block(&self, lang: &str, content: &str) -> Option<Vec<Line<'static>>> {
    None  // None = use default renderer (which skips mermaid)
}
```

This allows callers to handle mermaid blocks (render a placeholder, invoke a diagram renderer, etc.).

## 4. Links Support (Low Priority)

`[text](url)` is not parsed or rendered. For TUI use, rendering as styled text with the
link label visible is useful:

```
[example](https://example.com)  →  example (underlined, primary color)
```

Not critical for entelecheia's current needs but would make the library more complete.

## 5. Strikethrough (Low Priority)

`~~text~~` is not in the inline parser. Easy to add alongside bold/italic.

## 6. Task Lists (Low Priority)

`- [ ] task` / `- [x] done` — useful for todo rendering. Could be a new `ListItem` variant
or a separate `TaskItem` block.

---

## Implementation Priority

| Priority | Feature | Impact on entelecheia |
|----------|---------|-----------------------|
| P0 | Nested blockquote parser + renderer | Enables code-in-quote, multi-level quotes |
| P0 | CodeBlock header/footer override fields | Enables custom per-block headers |
| P1 | Mermaid hook interception | Allows TUI to render dependency graphs |
| P2 | Blockquote header/footer override | Consistent with CodeBlock override |
| P3 | Links, strikethrough, task lists | Nice-to-have completeness |

## Estimated Effort

| Feature | New/Modified Files | Lines (est.) |
|---------|--------------------|---------------|
| Nested blockquote (parser) | `parser.rs` | ~150 |
| Nested blockquote (renderer) | `render.rs` | ~80 |
| Nested blockquote (types) | `types.rs` | ~20 |
| CodeBlock override fields | `types.rs`, `render.rs` | ~50 |
| Mermaid hook | `hooks.rs`, `render.rs` | ~30 |
| **Total** | | **~330** |
