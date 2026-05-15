<div align="center">
  <img src="../../../examples/logo.webp" alt="ratatui-markdown logo" width="200" />
</div>

# ratatui-markdown

> A Rust library providing markdown rendering, collapsible JSON/TOML trees, and rich scroll widgets for ratatui.
>
> **Build with**: [ratatui](https://github.com/ratatui/ratatui) 0.29 + pure Rust
>
> **Minimal Rust Version**: 1.74

<div align="center">
  <p>
    <a href="index.md">English</a> |
    <a href="../zhs/index.md">简体中文</a> |
    <a href="../zht/index.md">繁體中文</a> |
    <a href="../ja/index.md">日本語</a> |
    <a href="../ko/index.md">한국어</a> |
    <a href="../fr/index.md">Français</a> |
    <a href="../es/index.md">Español</a> |
    <a href="../ru/index.md">Русский</a> |
    <a href="../ar/index.md">العربية</a>
  </p>
</div>

## What is ratatui-markdown?

ratatui-markdown is a feature-rich rendering library for terminal user interfaces built with [ratatui](https://github.com/ratatui/ratatui). It provides four main functional modules that can be used independently or combined through the `MarkdownPreview` widget.

## Core Modules

### Markdown Rendering

Parse and render markdown text into styled terminal output:

- **Headings**: H1 (`#`), H2 (`##`), H3 (`###`)
- **Paragraphs** with automatic CJK-aware text wrapping
- **Inline formatting**: `**bold**`, `*italic*`, `***bold+italic***`, `` `inline code` ``
- **Fenced code blocks** with optional language labels (mermaid blocks are skipped)
- **Blockquotes** (`>`)
- **Unordered lists** (`-`, `*`, `+`) and ordered lists (`1.`, `2.`)
- **Horizontal rules** (`---`, `***`, `___`)
- **Tables** with proportional column widths and cell wrapping

### Collapsible Tree View

Parse and interactively browse structured data:

- Parse **JSON** and **TOML** into collapsible trees
- **Expand / collapse** individual nodes, expand all, collapse all, expand to depth
- **Styled keys**: JSON mode (quoted keys with `:`) or TOML mode (bare keys with `=`)
- **Keyboard navigation**: cursor-based selection and toggle
- **Value type coloring**: strings, numbers, booleans, null — each in their own theme color

### Hybrid Scroll System

Smart scrolling that handles both free browsing and item navigation:

- **Free-scroll mode**: scroll through content freely
- **Engaged mode**: auto-activates when focusable items enter the viewport
- **Cursor navigation**: move through focusable items with keyboard
- **Cursor indicator**: visual `> ` prefix on engaged lines
- **Scrollbar**: arrow-based overlay on the outer edge
- **Pagination**: `page_up` / `page_down` support

### MarkdownPreview Widget

The high-level widget that ties everything together:

- Renders markdown content, tree views, and action items in a single scrollable layout
- **Caching**: rebuilds output only when content, width, or theme generation changes
- **TOML frontmatter stripping**: automatically strips `+++`-delimited frontmatter
- **Action items**: keyboard-selectable labeled items with action IDs
- Delegates all navigation to `HybridScrollView`

## Quick Start

```toml
[dependencies]
ratatui-markdown = "0.1"
```

```rust
use ratatui_markdown::preview::MarkdownPreview;

let mut preview = MarkdownPreview::new();
preview.set_content("# Hello, world!\n\nThis is a paragraph.");
// render and handle input in your ratatui app loop
```

## Feature Flags

All features are enabled by default. Disable default features and enable only what you need:

```toml
[dependencies]
ratatui-markdown = { version = "0.1", default-features = false, features = ["markdown"] }
```

| Feature    | Depends on        | Description                                     |
|------------|--------------------|-------------------------------------------------|
| `markdown` | —                  | Markdown parser and renderer                    |
| `scroll`   | —                  | HybridScrollView, scrollable lists, scrollbar   |
| `tree`     | `scroll`, `serde_json`, `toml` | Collapsible JSON/TOML tree        |
| `preview`  | `markdown`, `scroll`, `tree` | MarkdownPreview unified widget      |

## Project Structure

```
ratatui-markdown/
  src/
   ├── lib.rs                  # Library entry: feature-gated modules
   ├── theme.rs                # RichTextTheme trait, Generation token
   ├── constants/
   │   ├── mod.rs              # Re-exports
   │   ├── box_chars.rs        # Box-drawing character constants
   │   └── list_prefix.rs      # Tree connectors, arrows, markers
   ├── markdown/
   │   ├── mod.rs              # MarkdownRenderer struct
   │   ├── parser.rs           # Block-level markdown parser
   │   ├── types.rs            # MarkdownBlock enum, TextToken
   │   ├── render.rs           # Block-level renderer (+ table)
   │   ├── inline.rs           # Inline formatting parser
   │   └── text.rs             # CJK-aware text wrapping
   ├── scroll/
   │   ├── mod.rs              # Re-exports
   │   ├── hybrid_scroll/      # HybridScrollView (core widget)
   │   ├── scrollable_list.rs  # Generic ScrollableList<T>
   │   ├── scrollable_panel.rs # Simple scrollable helper
   │   ├── focusable_list.rs   # FocusableItemList renderer
   │   ├── follow_scroll.rs    # FollowScrollState
   │   └── scrollbar.rs        # ArrowScrollbar widget
   ├── tree/
   │   ├── mod.rs              # Re-exports
   │   ├── tree_lines.rs       # Tree line construction
   │   └── collapsible_tree/   # CollapsibleTree + node ops + rendering
   └── preview/
       └── mod.rs              # MarkdownPreview unified widget
```

## Documentation

| Guide | Description |
|-------|-------------|
| [Getting Started](getting-started.md) | Setup and first render |
| [Markdown](markdown.md) | Parsing and rendering markdown |
| [Scroll System](scroll.md) | Hybrid scroll, navigation, scrollbars |
| [Tree View](tree.md) | JSON/TOML trees, expand/collapse |
| [Preview Widget](preview.md) | Combining everything with MarkdownPreview |
| [Theme](theme.md) | Implementing RichTextTheme |
| [Contributing](contributing.md) | Development and contribution guide |

## License

Dual-licensed under MIT OR Apache-2.0.
