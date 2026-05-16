# Getting Started

## Prerequisites

- **Rust** 1.74 or later
- **ratatui** 0.29 (automatically pulled as a dependency)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ratatui-markdown = "0.1"
```

This enables all features by default (`markdown`, `scroll`, `tree`, `preview`, `mermaid`, `image`, `viewer`).

### Selective Features

To reduce compile time and dependencies, enable only what you need:

```toml
# Markdown rendering only
ratatui-markdown = { version = "0.1", default-features = false, features = ["markdown"] }

# Scroll system only
ratatui-markdown = { version = "0.1", default-features = false, features = ["scroll"] }

# Tree view (pulls in scroll, serde_json, and toml)
ratatui-markdown = { version = "0.1", default-features = false, features = ["tree"] }
```

## Basic Usage

### Render Markdown

```rust
use ratatui_markdown::markdown::MarkdownRenderer;
use ratatui_markdown::theme::RichTextTheme;

// Create a renderer with maximum content width
let renderer = MarkdownRenderer::new(80);

// Parse markdown text into blocks
let blocks = renderer.parse("# Hello\n\nThis is **bold** and *italic* text.");

// Render blocks into ratatui::text::Line<'static>
let lines = renderer.render(&blocks, &my_theme);
```

### Browse a Tree

```rust
use ratatui_markdown::tree::CollapsibleTree;

// Parse JSON into a collapsible tree
let json_str = r#"{"name": "project", "deps": {"ratatui": "0.29", "serde": "1.0"}}"#;
let mut tree = CollapsibleTree::from_json_str(json_str).unwrap();

// Render tree lines
let lines = tree.render_lines(80, &my_theme);

// Get focusable items for navigation
let items = tree.build_focusable_items();

// Toggle a node
tree.toggle("deps/serde");
```

### Use the MarkdownPreview Widget

The `MarkdownPreview` widget combines everything into a single scrollable view:

```rust
use ratatui_markdown::preview::MarkdownPreview;
use ratatui_markdown::theme::RichTextTheme;

let mut preview = MarkdownPreview::new()
    .with_left_padding(true);

// Set markdown content
preview.set_content("# Welcome\n\n- Item one\n- Item two\n\n```rust\nlet x = 42;\n```");

// Set a collapsible tree (optional)
let tree = CollapsibleTree::from_json_str(r#"{"config": {"port": 8080}}"#).unwrap();
preview.set_tree(Some(tree));

// Handle keyboard input
preview.scroll_up();
preview.scroll_down();
preview.page_up(10);
preview.page_down(10);
preview.toggle_tree_node(); // Enter key

// Render in your ratatui draw loop
fn draw(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}
```

## Implementing a Theme

The library uses a trait to look up all colors:

```rust
use ratatui::style::Color;
use ratatui_markdown::theme::{Generation, RichTextTheme};

struct MyTheme;

impl RichTextTheme for MyTheme {
    fn generation(&self) -> Generation { Generation(1) }
    fn get_text_color(&self) -> Color { Color::White }
    fn get_muted_text_color(&self) -> Color { Color::Gray }
    fn get_primary_color(&self) -> Color { Color::Cyan }
    fn get_secondary_color(&self) -> Color { Color::Blue }
    fn get_info_color(&self) -> Color { Color::LightBlue }
    fn get_background_color(&self) -> Color { Color::Black }
    fn get_border_color(&self) -> Color { Color::DarkGray }
    fn get_focused_border_color(&self) -> Color { Color::White }
    fn get_popup_selected_background(&self) -> Color { Color::DarkGray }
    fn get_popup_selected_text_color(&self) -> Color { Color::White }
    fn get_json_key_color(&self) -> Color { Color::LightCyan }
    fn get_json_string_color(&self) -> Color { Color::Green }
    fn get_json_number_color(&self) -> Color { Color::Yellow }
    fn get_json_bool_color(&self) -> Color { Color::Magenta }
    fn get_json_null_color(&self) -> Color { Color::DarkGray }
    fn get_accent_yellow(&self) -> Color { Color::Yellow }
}
```

Change the `generation()` return value to invalidate the preview widget's cache and force a re-render (e.g., when the user switches themes at runtime).

## Next Steps

- [Markdown Module](markdown.md) — full markdown parsing and rendering API
- [Scroll System](scroll.md) — understand the hybrid scroll architecture
- [Tree View](tree.md) — JSON/TOML tree rendering and interaction
- [Preview Widget](preview.md) — the high-level unified widget
- [Theme](theme.md) — complete theme customization guide
