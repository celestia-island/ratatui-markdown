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

## 7. Custom-Span Tree & Cursor Delegation (P0 — Critical for TUI migration)

### Problem

The TUI's sidebar agent timeline, conversation message list, global todo, device panel,
and logs page all share the same rendering pattern:

1. Build a list of **multi-line entries** with custom `Vec<Span<'static>>` content per line
2. Draw tree connectors (├─ └─) as prefix on each line  
3. Render a **cursor indicator** on the first line of the selected entry
4. Apply **selection background** to all lines of the selected entry
5. Scroll the viewport to keep the selected entry visible

Currently each component reimplements this independently (~300 lines each).
`CollapsibleTree` is data-driven (JSON/TOML only, no custom Spans).
`ScrollableList` with `ListItemRenderer` supports custom Spans but only single-line items.
`HybridScrollView` has multi-line focusable items but hardcoded cursor rendering.

None of these can be used as-is for the sidebar/conversation panel.

### What the TUI needs to tell the library

The TUI side should only need to provide:

```rust
// 1. Build entries with custom Span content (per frame, since content is dynamic)
let entries = vec![
    SpanTreeEntry {
        id: "agent-1".into(),
        lines: vec![
            vec![Span::styled("▸", cursor_style), Span::raw(" "), Span::styled("hubris", bold)],
            vec![Span::raw("   "), Span::styled("│", muted), Span::styled(" streaming...", muted)],
        ],
    },
    SpanTreeEntry {
        id: "agent-2".into(),
        lines: vec![
            vec![Span::raw("  "), Span::raw(" "), Span::styled("apo-ria", bold)],
        ],
    },
];

// 2. Specify which entry is selected (by id)
tree.set_selected("agent-1");

// 3. Specify what the cursor looks like (as Span)
tree.set_cursor_span(Span::styled("▸", Style::default().fg(primary)));
tree.set_blank_cursor_span(Span::raw(" "));
```

The library handles: viewport clipping, selection background, cursor prefix injection,
scroll-to-visible, scrollbar.

### Proposed API

#### New struct: `SpanTree`

```rust
pub struct SpanTreeEntry {
    pub id: String,
    pub lines: Vec<Vec<Span<'static>>>,
}

pub struct SpanTree {
    entries: Vec<SpanTreeEntry>,
    selected_id: Option<String>,
    scroll_offset: usize,
    viewport_height: usize,
    cursor_span: Span<'static>,       // default: Span::styled("▸", primary)
    blank_cursor_span: Span<'static>,  // default: Span::raw(" ")
    cursor_column: usize,              // which span index to replace (default: 0)
    auto_follow: bool,
}
```

**Key design decisions:**
- `entries` is rebuilt every frame by the TUI (dynamic content: spinners, timers, streaming tail)
- The library does NOT draw tree prefixes — the TUI includes them in the Span content
- The library only handles: cursor injection, selection bg, viewport scroll, scrollbar
- `cursor_column: usize` — which Span position holds the cursor placeholder (default: 0)
  - The TUI puts a cursor placeholder Span at this position in each entry's first line
  - The library replaces it with `cursor_span` for the selected entry, `blank_cursor_span` otherwise

#### Constructor & builders

```rust
impl SpanTree {
    pub fn new() -> Self;
    pub fn with_cursor_style(mut self, cursor: Span<'static>, blank: Span<'static>) -> Self;
    pub fn with_cursor_column(mut self, col: usize) -> Self;
    pub fn with_auto_follow(mut self, follow: bool) -> Self;
}
```

#### Per-frame update

```rust
impl SpanTree {
    /// Set entries for this frame. Replaces all previous entries.
    pub fn set_entries(&mut self, entries: Vec<SpanTreeEntry>);

    /// Select entry by id. Scrolls to make it visible.
    pub fn set_selected(&mut self, id: &str);

    /// Clear selection.
    pub fn clear_selection(&mut self);

    /// Select by numeric index.
    pub fn set_selected_index(&mut self, index: usize);
}
```

#### Navigation (delegated from TUI key handler)

```rust
impl SpanTree {
    pub fn navigate_up(&mut self);
    pub fn navigate_down(&mut self);
    pub fn navigate_to_first(&mut self);
    pub fn navigate_to_last(&mut self);
    pub fn scroll_up(&mut self, lines: usize);
    pub fn scroll_down(&mut self, lines: usize);
}
```

#### Rendering

```rust
impl SpanTree {
    pub fn render(
        &self,
        f: &mut Frame,
        inner_area: Rect,
        outer_area: Rect,
        theme: &impl RichTextTheme,
    );
}
```

The render method:
1. Calculates viewport from `inner_area.height`
2. Finds selected entry, calculates its line range
3. Adjusts `scroll_offset` so selected entry is visible
4. For each visible line:
   - Clones the Spans from the entry
   - If this is the first line of the selected entry: replaces `cursor_column` span with `cursor_span`, applies `popup_selected_background` to all spans
   - If this is a continuation line of the selected entry: applies background color
   - Otherwise: replaces `cursor_column` span with `blank_cursor_span` (only on first line)
5. Renders as `Paragraph`
6. Renders `ArrowScrollbar` on `outer_area`

#### Query methods

```rust
impl SpanTree {
    pub fn selected_id(&self) -> Option<&str>;
    pub fn selected_index(&self) -> Option<usize>;
    pub fn total_lines(&self) -> usize;
    pub fn entry_count(&self) -> usize;
    pub fn scroll_offset(&self) -> usize;
}
```

### Cursor Column Protocol

The key insight is the **cursor column protocol**: the TUI always puts a cursor placeholder
at a fixed Span index in each entry's first line. The library replaces it:

```rust
// TUI builds entry:
let first_line = vec![
    Span::raw(" "),                          // index 0 = cursor placeholder
    Span::raw(tree_prefix),                   // "├─ "
    Span::styled("agent-name", bold_style),   // agent name
];

// Library during render:
if is_selected_first_line {
    spans[cursor_column] = cursor_span.clone();  // "▸"
    for span in &mut spans { span.style = span.style.bg(selection_bg); }
} else if is_first_line {
    spans[cursor_column] = blank_cursor_span.clone();  // " "
}
```

This way the TUI doesn't need to know about cursor rendering at all — it just reserves
a slot in the Span vector.

### Migration path for TUI components

| Component | Current lines | After migration | Strategy |
|-----------|---------------|-----------------|----------|
| **sidebar.rs** render() | 310 lines (180-441) | ~80 lines | Build `Vec<SpanTreeEntry>` from `visible_agents`, delegate render |
| **conversation/rendering.rs** render_message_list() | 140 lines (295-438) | ~30 lines | Build entries from message lines, delegate |
| **global_todo.rs** render() | 280 lines | ~60 lines | Build entries from todo items, delegate |
| **device_panel.rs** render() | 200 lines | ~50 lines | Build entries from device info, delegate |
| **logs_page** render_log_viewport() | 230 lines | ~50 lines | Build entries from log entries, delegate |

**Estimated TUI code reduction: ~900 lines**

### Files to create/modify in ratatui-markdown

| File | Action | Description |
|------|--------|-------------|
| `src/tree/span_tree.rs` | **New** | `SpanTree`, `SpanTreeEntry` structs |
| `src/tree/span_tree/render.rs` | **New** | Render logic (viewport clip, cursor inject, selection bg) |
| `src/tree/span_tree/scroll.rs` | **New** | Navigation + auto-follow scroll logic |
| `src/tree/mod.rs` | Modify | Export `SpanTree`, `SpanTreeEntry` |
| `src/lib.rs` | Modify | Re-export |

### Estimated effort

| Part | Lines (est.) |
|------|-------------|
| SpanTree struct + constructors | ~80 |
| Cursor injection + selection bg rendering | ~80 |
| Navigation + scroll-to-visible + auto-follow | ~80 |
| Tests | ~150 |
| **Total** | **~390** |

### Relationship to existing components

```
CollapsibleTree    → static JSON/TOML data, auto-expand/collapse
ScrollableList<T>  → single-line items, ListItemRenderer trait
HybridScrollView   → multi-line focusable regions, free/engaged scroll modes
SpanTree (NEW)     → dynamic multi-line entries, custom Span content, cursor delegation
```

`SpanTree` fills the gap between `ScrollableList` (single-line) and `HybridScrollView`
(static content with position-based regions). It's the right abstraction for any TUI
component that renders a dynamic list of styled entries with cursor navigation.

---

## Implementation Priority (Updated)

| Priority | Feature | Impact on entelecheia |
|----------|---------|-----------------------|
| P0 | **SpanTree: custom-Span tree + cursor delegation** | Replaces ~900 lines across 5 TUI components |
| P0 | Nested blockquote parser + renderer | Enables code-in-quote, multi-level quotes |
| P0 | CodeBlock header/footer override fields | Enables custom per-block headers |
| P1 | Mermaid hook interception | Allows TUI to render dependency graphs |
| P2 | Blockquote header/footer override | Consistent with CodeBlock override |
| P3 | Links, strikethrough, task lists | Nice-to-have completeness |

## Estimated Effort (Updated)

| Feature | New/Modified Files | Lines (est.) |
|---------|--------------------|---------------|
| **SpanTree (cursor + selection + scroll)** | `span_tree/*.rs`, `tree/mod.rs`, `lib.rs` | ~390 |
| Nested blockquote (parser) | `parser.rs` | ~150 |
| Nested blockquote (renderer) | `render.rs` | ~80 |
| Nested blockquote (types) | `types.rs` | ~20 |
| CodeBlock override fields | `types.rs`, `render.rs` | ~50 |
| Mermaid hook | `hooks.rs`, `render.rs` | ~30 |
| **Total** | | **~720** |
