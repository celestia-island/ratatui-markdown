//! # ratatui-markdown
//!
//! A Rust library providing markdown rendering, collapsible JSON/TOML tree views,
//! and a rich hybrid scroll system — all built on top of [ratatui].
//!
//! [ratatui]: https://github.com/ratatui/ratatui
//!
//! ## Features
//!
//! - **Markdown rendering** — parse and render markdown to styled [`ratatui::text::Line`]s,
//!   with support for headings, lists, code blocks, blockquotes, tables, and inline
//!   formatting (bold, italic, inline code)
//! - **Collapsible trees** — parse JSON or TOML into interactive collapsible trees with
//!   expand/collapse, styled keys, and keyboard navigation
//! - **Hybrid scroll system** — dual-mode scrolling: free-scroll for exploring content,
//!   engaged mode for navigating focusable items
//! - **`MarkdownPreview` widget** — unified widget combining markdown rendering, tree
//!   views, and action items into a single scrollable view
//! - **`RichTextTheme`** — fully themeable via a trait: 15+ color slots for text,
//!   borders, JSON values, popups, and more
//! - **CJK-aware text wrapping** — correct width calculation for CJK characters via
//!   `unicode-width`
//! - **TOML frontmatter support** — optionally strip `+++`-delimited TOML frontmatter
//!
//! ## Feature Flags
//!
//! All features are enabled by default. Disable default features to enable only what
//! you need:
//!
//! ```toml
//! [dependencies]
//! ratatui-markdown = { version = "0.2", default-features = false, features = ["markdown"] }
//! ```
//!
//! | Feature    | Requires                                | Description                               |
//! |------------|-----------------------------------------|-------------------------------------------|
//! | `markdown` | —                                       | Markdown parser and renderer              |
//! | `image`    | `image` crate                           | Image resolution via `ImageResolver`      |
//! | `scroll`   | —                                       | HybridScrollView, scrollable lists        |
//! | `tree`     | `scroll`, `serde_json`, `toml`          | Collapsible JSON/TOML tree                |
//! | `preview`  | `markdown`, `scroll`, `tree`            | `MarkdownPreview` unified widget          |
//! | `mermaid`  | `markdown`                              | Mermaid diagram rendering                 |
//! | `viewer`   | `markdown`, `scroll`                    | `MarkdownViewer` widget                   |
//!
//! ## Quick Start
//!
//! ```rust
//! use ratatui_markdown::preview::MarkdownPreview;
//!
//! let mut preview = MarkdownPreview::new();
//! preview.set_content("# Hello, world!\n\nThis is a paragraph.");
//! // render and handle input in your ratatui app loop
//! ```
//!
//! ### Markdown Rendering
//!
//! ```rust
//! use ratatui_markdown::markdown::MarkdownRenderer;
//!
//! let renderer = MarkdownRenderer::new(80);
//! let blocks = renderer.parse("# Title\n\nParagraph with **bold** text.");
//! let lines = renderer.render(&blocks, &my_theme);
//! ```
//!
//! ### Custom Rendering with Hooks
//!
//! ```rust
//! use ratatui_markdown::markdown::{MarkdownRenderer, RenderHooks};
//! use ratatui::text::Line;
//!
//! struct MyHooks;
//!
//! impl RenderHooks for MyHooks {
//!     fn heading1(&self, text: &str) -> Option<Line<'static>> {
//!         Some(Line::raw(format!(">>> {}", text)))
//!     }
//! }
//!
//! let renderer = MarkdownRenderer::new(80)
//!     .with_render_hooks(Box::new(MyHooks));
//! ```
//!
//! ### Collapsible Trees
//!
//! ```rust
//! use ratatui_markdown::tree::CollapsibleTree;
//!
//! let mut tree = CollapsibleTree::from_json_str(
//!     r#"{"key": "value", "nested": {"a": 1}}"#
//! ).unwrap();
//! let lines = tree.render_lines(80, &my_theme);
//! let items = tree.build_focusable_items();
//! tree.toggle("nested");
//! ```
//!
//! ### Scroll System
//!
//! ```rust
//! use ratatui_markdown::scroll::HybridScrollView;
//!
//! let mut scroll = HybridScrollView::new()
//!     .with_cursor_indicator(true);
//! // set content, handle input, render
//! ```
//!
//! ### Theming
//!
//! ```rust
//! use ratatui::style::Color;
//! use ratatui_markdown::theme::{Generation, RichTextTheme};
//!
//! struct MyTheme;
//!
//! impl RichTextTheme for MyTheme {
//!     fn generation(&self) -> Generation { Generation(1) }
//!     fn get_text_color(&self) -> Color { Color::White }
//!     fn get_muted_text_color(&self) -> Color { Color::Gray }
//!     fn get_primary_color(&self) -> Color { Color::Cyan }
//!     fn get_secondary_color(&self) -> Color { Color::Blue }
//!     fn get_info_color(&self) -> Color { Color::LightBlue }
//!     fn get_background_color(&self) -> Color { Color::Black }
//!     fn get_border_color(&self) -> Color { Color::DarkGray }
//!     fn get_focused_border_color(&self) -> Color { Color::White }
//!     fn get_popup_selected_background(&self) -> Color { Color::DarkGray }
//!     fn get_popup_selected_text_color(&self) -> Color { Color::White }
//!     fn get_json_key_color(&self) -> Color { Color::LightCyan }
//!     fn get_json_string_color(&self) -> Color { Color::Green }
//!     fn get_json_number_color(&self) -> Color { Color::Yellow }
//!     fn get_json_bool_color(&self) -> Color { Color::Magenta }
//!     fn get_json_null_color(&self) -> Color { Color::DarkGray }
//!     fn get_accent_yellow(&self) -> Color { Color::Yellow }
//! }
//! ```
//!
//! ## Modules
//!
//! | Module | Feature | Description |
//! |--------|---------|-------------|
//! | [`markdown`] | `markdown` | Parse and render markdown text |
//! | [`scroll`] | `scroll` | Hybrid scroll system with focusable items |
//! | [`tree`] | `tree` | Collapsible JSON/TOML tree view |
//! | [`preview`] | `preview` | Unified `MarkdownPreview` widget |
//! | [`mermaid`] | `mermaid` | Mermaid diagram rendering |
//! | [`viewer`] | `viewer` | `MarkdownViewer` widget |
//! | [`theme`] | always | `RichTextTheme` trait for theming |
//! | [`constants`] | always | Box-drawing chars, tree connectors, arrows |

pub mod constants;
#[cfg(feature = "highlight")]
pub mod highlight;
#[cfg(feature = "markdown")]
pub mod markdown;
#[cfg(feature = "mermaid")]
pub mod mermaid;
#[cfg(feature = "preview")]
pub mod preview;
#[cfg(feature = "scroll")]
pub mod scroll;
pub mod text_input;
pub mod theme;
#[cfg(feature = "tree")]
pub mod tree;
#[cfg(feature = "viewer")]
pub mod viewer;

#[allow(deprecated)]
pub use theme::DefaultTheme;
pub use theme::{RichTextTheme, ThemeBuilder, ThemeConfig};
