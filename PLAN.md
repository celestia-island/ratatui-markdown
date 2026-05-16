# ratatui-markdown Roadmap

All previously planned features have been implemented. This document tracks new
requirements that emerged during TUI integration work.

---

## Section 8 — Dual-Mode Text Input/Display Component (`TextInput`)

### Motivation

The current `TextInput` widget (in entelecheia) has several limitations:

1. **Cursor rendering is hardcoded** — only an inverted-color block cursor exists.
   No bar cursor, underline cursor, or configurable position logic.

2. **Cursor blink is hardcoded** — `AnimationManager` owns a `CursorBlinkState`
   enum with fixed 800ms/200ms intervals. The state machine can't be replaced
   or externally driven.

3. **No markdown awareness** — the input treats text as plain strings. When
   editing markdown, the user sees raw source with no visual distinction
   between headings, bold, links, etc.

4. **Duplicated rendering** — conversation input, modal input, and command
   palette all copy the cursor/selection rendering code instead of using a
   shared component.

5. **No read-only mode** — there's no way to display the same content area as
   beautifully rendered markdown (headings, lists, code blocks) and then switch
   to edit mode showing the raw source with light syntax highlighting.

### Requirements

#### 8.1 Dual Rendering Modes

The component must support two distinct rendering modes, switchable at runtime:

**Edit Mode** (source-visible):
- Displays the raw markdown/plain-text source code
- Applies lightweight syntax styling: headings in bold, `inline code` with
  background tint, `**bold**` in bold, `*italic*` in italic, `[links](url)`
  with colored brackets, etc.
- Does NOT render structural elements — `#` prefixes stay visible, list markers
  stay as-is, code fences remain as `` ``` `` lines
- Think "VS Code markdown preview while editing" — source is visible but has
  semantic coloring
- Cursor is active and interactive (typing, selection, movement)
- The cursor position is determined by `char_idx` in the source string

**Read Mode** (rendered display):
- Renders markdown fully via `MarkdownRenderer::parse()` + `render()`
- Headings styled, lists formatted, code blocks syntax-highlighted, tables
  rendered, blockquotes indented, etc.
- No cursor, no editing — purely for reading
- The rendered output is identical to what `MarkdownPreview` would produce
- Text selection for copy is supported via the existing `SelectableTextPanel`
  mechanism (external, not part of this component)

Mode switching is instantaneous — the same content string is interpreted
differently. The caller owns the content string and passes it to the
component along with the current mode.

#### 8.2 Configurable Cursor Rendering

The cursor visual representation must be fully configurable:

```rust
pub enum CursorShape {
    /// Filled block covering the character at cursor position.
    /// Renders the character with swapped fg/bg.
    Block,
    /// Vertical bar rendered as a separate span BEFORE the character at
    /// cursor position. Does not modify the character's appearance.
    /// Width: 1 cell.
    Bar,
    /// Underline rendered below the character at cursor position.
    /// Renders the character with an underline modifier.
    Underline,
    /// Hollow box outline around the character at cursor position.
    /// Uses box-drawing characters. Width: 1 cell (overtype).
    HollowBlock,
}

pub enum CursorPosition {
    /// Cursor is rendered BETWEEN characters — before the character at
    /// cursor `char_idx`. This is the standard bar/line cursor behavior.
    /// For Block shape, this means the block covers the gap (renders a
    /// colored space before the character).
    BeforeChar,
    /// Cursor is rendered ON the character at cursor `char_idx` — the
    /// character under the cursor is visually modified (inverted, underlined,
    /// etc). This is the standard block/underline cursor behavior.
    OnChar,
}
```

The cursor style (colors) should be configurable separately from shape:

```rust
pub struct CursorStyle {
    pub shape: CursorShape,
    pub position: CursorPosition,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub modifier: Modifier,
}
```

When `fg`/`bg` are `None`, the theme's default cursor colors are used
(`get_primary_color()` for the prominent color, `get_background_color()` for
    the swap).

#### 8.3 External Animation State Machine

The cursor blink animation must be **entirely external** to the component.

```rust
/// Trait for external cursor blink state.
/// The component calls `is_visible()` on each render frame to decide
/// whether to show or hide the cursor.
pub trait CursorBlinkController {
    fn is_visible(&self) -> bool;
}
```

The component accepts an `Option<Rc<dyn CursorBlinkController>>` or similar.
When `None`, the cursor is always visible (no blinking). When `Some`, the
controller determines visibility on each frame.

This allows the caller to implement any blink pattern:
- Fixed interval (current 800ms)
- Adaptive (pause on type, resume after idle)
- Fast/slow switching
- Accessibility-driven (no blink, or slow blink)
- Per-component independent blink (each input can blink independently)

The existing `AnimationManager` in entelecheia would implement this trait.

#### 8.4 Selection Rendering

Selection rendering should remain as-is (highlighted span with theme colors)
but with configurable colors:

```rust
pub struct SelectionStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}
```

When `None`, the theme's default selection colors are used.

#### 8.5 Edit-Mode Syntax Styling

In edit mode, inline markdown syntax gets light styling via the existing
`parse_inline_formatting()` function (already in `rendering/` module).
Additionally:

| Syntax | Styling |
|--------|---------|
| `# heading` | Bold + heading color on `#`, bold on heading text |
| `**bold**` | Bold on content, muted on `**` delimiters |
| `*italic*` | Italic on content, muted on `*` delimiters |
| `` `code` `` | Inline-code bg tint on content, muted on `` ` `` delimiters |
| `[text](url)` | Link color on text, muted on brackets/parens |
| `~~strike~~` | Crossed-out on content, muted on `~~` delimiters |
| `> quote` | Quoted-color on `>`, italic on content |
| `- item` / `* item` | Muted on marker |
| `` ```lang `` | Code-fence color on delimiters, syntax name in dim |
| Plain text | Default text color |

This is NOT full markdown rendering — structural elements remain as source.
Only inline-level tokens get visual distinction.

#### 8.6 API Sketch

```rust
pub struct TextInput {
    // content
    text: String,
    cursor_char_idx: usize,
    selection: Option<Selection>,

    // rendering
    mode: InputMode,
    cursor_style: CursorStyle,
    selection_style: SelectionStyle,
    blink_controller: Option<Rc<dyn CursorBlinkController>>,
    horizontal_scroll: usize,

    // config
    placeholder: Option<String>,
    password: bool,
    max_width: usize,
}

pub enum InputMode {
    /// Raw source with light syntax styling. Cursor active.
    Edit,
    /// Fully rendered markdown. No cursor, no editing.
    Read,
}

impl TextInput {
    pub fn new() -> Self;
    pub fn with_mode(mut self, mode: InputMode) -> Self;
    pub fn with_cursor_style(mut self, style: CursorStyle) -> Self;
    pub fn with_selection_style(mut self, style: SelectionStyle) -> Self;
    pub fn with_blink_controller(mut self, ctrl: Rc<dyn CursorBlinkController>) -> Self;
    pub fn with_placeholder(mut self, text: impl Into<String>) -> Self;
    pub fn with_password(mut self, password: bool) -> Self;
    pub fn with_max_width(mut self, width: usize) -> Self;

    /// Render the component. In Edit mode, renders the editable source.
    /// In Read mode, renders fully-formatted markdown.
    pub fn render(&self, f: &mut Frame, area: Rect, theme: &impl RichTextTheme);

    /// Calculate the layout rect needed for Read mode rendering.
    /// Useful for pre-calculating scroll areas.
    pub fn rendered_height(&self, width: usize, theme: &impl RichTextTheme) -> u16;
}
```

### Implementation Priority

1. **CursorShape + CursorPosition** — core cursor abstraction
2. **CursorBlinkController trait** — external blink state
3. **InputMode::Edit with syntax styling** — reuses existing `parse_inline_formatting()`
4. **InputMode::Read** — delegates to `MarkdownRenderer`
5. **SelectionStyle** — configurable colors

### Dependencies

- `MarkdownRenderer` — for Read mode rendering (already exists)
- `parse_inline_formatting()` — for Edit mode inline styling (already exists)
- `RichTextTheme` — for color resolution (already exists)

### Integration Impact

After implementation, the following entelecheia files should be updated:

| Current File | Change |
|---|---|
| `widgets/text_input/` | Replaced by this component (or wraps it) |
| `conversation/rendering.rs:129-274` | Remove duplicated input rendering, use component |
| `modal/modal_impl/render_impl.rs:491-573` | Remove duplicated input rendering, use component |
| `interaction/command/completer.rs:231-243` | Use component's cursor rendering |
| `widgets/animation/manager.rs` | Implement `CursorBlinkController` trait |

---

## Section 9 — SpanTree Enhancement: Per-Line Cursor Column

### Motivation

The current SpanTree only replaces the cursor span on `line_idx == 0` of each
entry. Some components (like the timeline) want the cursor indicator on ALL
lines of a selected group, not just the first line.

### Requirements

Add an option to SpanTree to control cursor rendering across all lines of an
entry:

```rust
pub enum CursorLineMode {
    /// Only replace cursor span on line 0 (header line) of each entry.
    /// Subsequent lines keep their original spans.
    HeaderOnly,
    /// Replace cursor span on ALL lines of the selected entry.
    /// Each line's span at `cursor_column` gets the cursor/blank replacement.
    AllLines,
}
```

Default: `HeaderOnly` (backward compatible).

```rust
impl SpanTree {
    pub fn with_cursor_line_mode(mut self, mode: CursorLineMode) -> Self;
}
```

### Integration Impact

- `timeline.rs` would use `AllLines` to show cursor on every line of the
  selected agent group
- Other components (sidebar, device_panel, global_todo) use default `HeaderOnly`

---

## Migration Priority Summary

| # | Target | Pattern | Estimated Savings | Priority |
|---|--------|---------|-------------------|----------|
| 8 | TextInput component | New library component | ~300 lines (dedup) | High |
| 9 | SpanTree cursor mode | Enhancement | ~10 lines | Low |
| — | `conversation/rendering.rs` input dedup | Use TextInput | ~145 lines | High |
| — | `modal/render_impl.rs` input dedup | Use TextInput | ~80 lines | High |
| — | `agent_config/render.rs` | ScrollableList | ~60 lines | Medium |
| — | `command_palette.rs` | ScrollableList | ~40 lines | Medium |
| — | `models_page` panels | ScrollableList/SpanTree | ~120 lines | Medium |
| — | `agents_page` list | ScrollableList | ~60 lines | Medium |
