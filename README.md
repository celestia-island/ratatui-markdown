<div align="center">
  <img src="https://raw.githubusercontent.com/celestia-island/ratatui-markdown/dev/examples/logo.webp" alt="ratatui-markdown logo" width="200" />
</div>

<div align="center"><h1>ratatui-markdown</h1></div>
<div align="center">
  <strong>Markdown rendering, Mermaid diagrams, syntax highlighting, collapsible trees, and rich scroll widgets for ratatui</strong>
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
    <a href="https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/en/index.md">Documentation</a>
    <span> | </span>
    <a href="https://docs.rs/ratatui-markdown">API Reference</a>
  </h3>
</div>

<div align="center">
  <p>
    <a href="https://github.com/celestia-island/ratatui-markdown/blob/dev/README.md">English</a> |
    <a href="https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/zhs/index.md">简体中文</a> |
    <a href="https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/zht/index.md">繁體中文</a> |
    <a href="https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/ja/index.md">日本語</a> |
    <a href="https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/ko/index.md">한국어</a> |
    <a href="https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/fr/index.md">Français</a> |
    <a href="https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/es/index.md">Español</a> |
    <a href="https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/ru/index.md">Русский</a> |
    <a href="https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/ar/index.md">العربية</a>
  </p>
</div>

<br/>

> A Rust library providing markdown rendering, Mermaid diagrams, syntax highlighting, collapsible JSON/TOML tree views, and a rich hybrid scroll system — all built on top of [ratatui](https://github.com/ratatui/ratatui).

## Features

- **Markdown rendering** — parse and render markdown to styled `ratatui::text::Line`s, with support for headings, lists, code blocks, blockquotes, tables, images, and inline formatting (bold, italic, inline code)
- **Mermaid diagrams** — render sequence, pie, gantt, and state diagrams from ` ```mermaid ` code blocks (feature-gated: `mermaid`)
- **Syntax highlighting** — tree-sitter based code block highlighting with per-language feature flags (feature-gated: `highlight-lang-*`)
- **Image support** — resolve `![alt](path)` images via the `ImageResolver` trait (feature-gated: `image`)
- **Custom rendering hooks** — override rendering of any block type (headings, code blocks, lists, tables, etc.) via the `RenderHooks` trait
- **Collapsible trees** — parse JSON or TOML into interactive collapsible trees with expand/collapse, styled keys, and keyboard navigation
- **Hybrid scroll system** — dual-mode scrolling: free-scroll for exploring content, engaged mode for navigating focusable items
- **MarkdownPreview / MarkdownViewer** — unified widgets combining markdown, tree views, and scroll into a single view
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
ratatui-markdown = "0.2"
```

For the full feature set (enabled by default):

```toml
[dependencies]
ratatui-markdown = { version = "0.2", features = ["preview"] }
```

Individual features can be enabled selectively:

| Feature              | Description                                          | Default |
|----------------------|------------------------------------------------------|---------|
| `markdown`           | Markdown parsing and rendering                       | ✓       |
| `image`              | Image resolution via `ImageResolver` trait           | ✓       |
| `scroll`             | Hybrid scroll and scrollable widgets                 | ✓       |
| `tree`               | JSON/TOML collapsible tree (requires `scroll`)       | ✓       |
| `preview`            | `MarkdownPreview` unified widget (requires `markdown`, `scroll`, `tree`) | ✓ |
| `mermaid`            | Mermaid diagram rendering (requires `markdown`)      | ✓       |
| `viewer`             | `MarkdownViewer` widget (requires `markdown`, `scroll`) | ✓    |
| `highlight`          | Syntax highlighting via tree-sitter                  |         |
| `highlight-lang-*`   | Individual language grammars (requires `highlight`)  |         |
| `highlight-lang-all` | All bundled language grammars                        |         |

### Examples

| Example              | Description                          | Features required             |
|----------------------|--------------------------------------|-------------------------------|
| `basic`              | Minimal markdown rendering           | —                             |
| `code`               | Syntax-highlighted code blocks       | `highlight-lang-all`          |
| `custom_code_block`  | Custom code block rendering hooks    | —                             |
| `image`              | Image embedding and zoom             | `image`                       |
| `mermaid`            | Mermaid diagram rendering            | `mermaid`                     |
| `tree_list`          | Collapsible JSON/TOML tree view      | —                             |

```bash
cargo run --example basic
cargo run --example code --features highlight-lang-all
cargo run --example image
cargo run --example mermaid
cargo run --example tree_list
```

## Documentation

- [Getting Started](https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/en/getting-started.md)
- [Markdown Module](https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/en/markdown.md)
- [Scroll System](https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/en/scroll.md)
- [Tree View](https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/en/tree.md)
- [Preview Widget](https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/en/preview.md)
- [Theme Customization](https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/en/theme.md)
- [Contributing](https://github.com/celestia-island/ratatui-markdown/blob/dev/docs/guides/en/contributing.md)
- [API Reference](https://docs.rs/ratatui-markdown)

## License

Dual-licensed under MIT OR Apache-2.0.
