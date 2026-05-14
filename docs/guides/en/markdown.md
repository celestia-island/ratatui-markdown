# Markdown Module

> Parse and render markdown text into styled `ratatui::text::Line`s.

## Overview

The `markdown` module provides a custom markdown parser and renderer designed for terminal output. It is **not** a CommonMark-compliant parser — it targets the subset of markdown most useful in terminal UIs.

Gated behind the `markdown` feature flag (enabled by default).

## MarkdownRenderer

```rust
pub struct MarkdownRenderer {
    max_width: usize,
}

impl MarkdownRenderer {
    pub fn new(max_width: usize) -> Self;
    pub fn parse(&self, markdown: &str) -> Vec<MarkdownBlock>;
    pub fn render(&self, blocks: &[MarkdownBlock], theme: &impl RichTextTheme) -> Vec<Line<'static>>;
}
```

### Constructor

`MarkdownRenderer::new(max_width)` takes the available content width in columns. This width is used for paragraph text wrapping (CJK-aware) and table column sizing.

### Parsing

`parse()` takes a `&str` of markdown text and returns a `Vec<MarkdownBlock>`. The parser is line-oriented and processes blocks in sequence.

### Rendering

`render()` takes the parsed blocks and a theme, producing a `Vec<Line<'static>>` suitable for direct use in ratatui widgets.

## MarkdownBlock Enum

```rust
pub enum MarkdownBlock {
    Heading1(String),              // # Heading
    Heading2(String),              // ## Heading
    Heading3(String),              // ### Heading
    Paragraph(Vec<String>),        // Wrapped paragraph lines
    CodeBlock(String, String),     // (language, content)
    InlineCode(String),            // `inline code`
    ListItem(String, u8),          // (content, indent_level)
    Blockquote(String),            // > quoted text
    HorizontalRule,                // --- or *** or ___
    BlankLine,                     // empty line
    Table {                        // | col1 | col2 |
        headers: Vec<String>,      //   |------|------|
        rows: Vec<Vec<String>>,    //   | val1 | val2 |
    },
}
```

### Block Details

**Headings** (H1-H3): Rendered with the `primary_color`, with H1 using `Modifier::BOLD`.

**Paragraphs**: Text is CJK-aware word-wrapped to `max_width`. Each wrapped line becomes an entry in the `Vec<String>`.

**Code Blocks** (fenced with ` ``` `): Rendered with `muted_text_color` inside bordered boxes using box-drawing characters. Mermaid code blocks are silently skipped.

**Inline Code**: Rendered with `secondary_color` and `Modifier::DIM`.

**Lists**: Unordered (`-`, `*`, `+`) and ordered (`1.`, `2.`). Each item preserves its indentation level. Sub-items are indented visually.

**Blockquotes**: Prefixed with a colored `│` bar and rendered in `muted_text_color`.

**Tables**: Columns are sized proportionally based on content width. Cells are wrapped, headers use `Modifier::BOLD`, and borders use box-drawing characters.

## Inline Formatting

Inline formatting is applied **inside** paragraph and list item text:

| Markdown        | Rendered Effect                |
|-----------------|--------------------------------|
| `**text**`      | **Bold** (`Modifier::BOLD`)   |
| `*text*`        | *Italic* (`Modifier::ITALIC`) |
| `***text***`    | ***Bold+Italic***              |
| `` `code` ``    | `Inline Code` style            |

```rust
pub fn parse_inline_formatting(text: &str, theme: &impl RichTextTheme) -> Vec<Span<'static>>;
```

This standalone function is also re-exported for use outside the `MarkdownRenderer`.

## Example

```rust
use ratatui_markdown::markdown::MarkdownRenderer;

let md = r#"
# Title

This is a paragraph with **bold** and *italic* text.

## Code

```rust
fn main() {
    println!("Hello!");
}
```

| Name | Version |
|------|---------|
| ratatui | 0.29 |
| serde | 1.0 |
"#;

let renderer = MarkdownRenderer::new(80);
let blocks = renderer.parse(md);
let lines = renderer.render(&blocks, theme);
// use lines in a ratatui widget
```
