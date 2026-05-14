<div align="center"><h1>ratatui-markdown</h1></div>
<div align="center">
  <strong>Markdown rendering, collapsible JSON/TOML trees, and rich scroll widgets for ratatui</strong>
</div>

<br />

<div align="center">
  <a href="https://github.com/celestia-island/ratatui-markdown/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/celestia-island/ratatui-markdown/ci.yml?branch=dev" alt="CI" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg" alt="License" />
  </a>
  <a href="https://crates.io/crates/ratatui-markdown">
    <img src="https://img.shields.io/crates/v/ratatui-markdown.svg" alt="Crates.io" />
  </a>
</div>

<div align="center">
  <h3>
    <a href="#quick-start">Quick Start</a>
    <span> | </span>
    <a href="docs/guides/en/index.md">Documentation</a>
    <span> | </span>
    <a href="https://docs.rs/ratatui-markdown">API Reference</a>
  </h3>
</div>

<div align="center">
  <p>
    <a href="../../README.md">English</a> |
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

<br/>

> A Rust library providing markdown rendering, collapsible JSON/TOML tree views, and a rich hybrid scroll system — all built on top of [ratatui](https://github.com/ratatui/ratatui).

## Features

- **Markdown rendering** — parse and render markdown to styled `ratatui::text::Line`s, with support for headings, lists, code blocks, blockquotes, tables, and inline formatting (bold, italic, inline code)
- **Collapsible trees** — parse JSON or TOML into interactive collapsible trees with expand/collapse, styled keys, and keyboard navigation
- **Hybrid scroll system** — dual-mode scrolling: free-scroll for exploring content, engaged mode for navigating focusable items
- **MarkdownPreview widget** — unified widget that combines markdown rendering, tree views, and action items into a single scrollable view
- **RichTheme** — fully themeable via the `RichTextTheme` trait: 15+ color slots for text, borders, JSON values, popups, and more
- **CJK-aware text wrapping** — correct width calculation for CJK characters via `unicode-width`
- **TOML frontmatter support** — optionally strip `+++`-delimited TOML frontmatter from rendered content

## Quick Start

### Prerequisites

- Rust 1.74+
- [ratatui](https://github.com/ratatui/ratatui) 0.29

### Installation

```toml
[dependencies]
ratatui-markdown = "0.1"
```

For the full feature set (enabled by default):

```toml
[dependencies]
ratatui-markdown = { version = "0.1", features = ["preview"] }
```

Individual features can be enabled selectively:

| Feature    | Description                            |
|------------|----------------------------------------|
| `markdown` | Markdown parsing and rendering         |
| `scroll`   | Hybrid scroll and scrollable widgets   |
| `tree`     | JSON/TOML collapsible tree (requires `scroll`) |
| `preview`  | `MarkdownPreview` unified widget (requires all) |

### Example

```rust
use ratatui_markdown::{
    markdown::MarkdownRenderer,
    preview::MarkdownPreview,
    theme::RichTextTheme,
};

struct App {
    preview: MarkdownPreview,
}

impl App {
    fn render(&mut self, f: &mut ratatui::Frame, theme: &impl RichTextTheme) {
        self.preview.set_content("# Hello\n\nThis is **markdown**!");
        self.preview.render(f, f.area(), f.area(), theme);
    }

    fn handle_input(&mut self, key: ratatui::crossterm::event::KeyCode) {
        match key {
            KeyCode::Up | KeyCode::Char('k') => self.preview.scroll_up(),
            KeyCode::Down | KeyCode::Char('j') => self.preview.scroll_down(),
            KeyCode::Enter => { self.preview.toggle_tree_node(); }
            _ => {}
        }
    }
}
```

```rust
// Parse and render markdown to ratatui Lines
use ratatui_markdown::markdown::MarkdownRenderer;

let md = MarkdownRenderer::new(80);
let blocks = md.parse("# Title\n\nParagraph text\n\n```rust\nlet x = 1;\n```");
let lines = md.render(&blocks, &my_theme);

// Render a collapsible JSON tree
use ratatui_markdown::tree::CollapsibleTree;

let mut tree = CollapsibleTree::from_json_str(r#"{"key": "value", "nested": {"a": 1}}"#).unwrap();
let lines = tree.render_lines(80, &my_theme);
let focusable = tree.build_focusable_items();
tree.toggle("nested"); // collapse/expand
```

## Documentation

- [Getting Started](docs/guides/en/getting-started.md)
- [Markdown Module](docs/guides/en/markdown.md)
- [Scroll System](docs/guides/en/scroll.md)
- [Tree View](docs/guides/en/tree.md)
- [Preview Widget](docs/guides/en/preview.md)
- [Theme Customization](docs/guides/en/theme.md)
- [Contributing](docs/guides/en/contributing.md)
- [API Reference](https://docs.rs/ratatui-markdown)

### Languages

| Language | Documentation |
|----------|---------------|
| English | [docs/guides/en/](index.md) |
| 简体中文 | [docs/guides/zhs/](../zhs/index.md) |
| 繁體中文 | [docs/guides/zht/](../zht/index.md) |
| 日本語 | [docs/guides/ja/](../ja/index.md) |
| 한국어 | [docs/guides/ko/](../ko/index.md) |
| Français | [docs/guides/fr/](../fr/index.md) |
| Español | [docs/guides/es/](../es/index.md) |
| Русский | [docs/guides/ru/](../ru/index.md) |
| العربية | [docs/guides/ar/](../ar/index.md) |

## License

Dual-licensed under MIT OR Apache-2.0.
