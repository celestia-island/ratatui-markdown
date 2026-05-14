# ratatui-markdown 0.2.0 Enhancement Plan

## Overview

Enhance ratatui-markdown with image support, custom rendering hooks, and four examples.
Target version: **0.2.0**.

## Design Decisions

- **Image rendering**: Library returns `Vec<Line>` + `Vec<ImagePlacement>`, user renders with ratatui-image in draw loop
- **Hooks coverage**: All `MarkdownBlock` variants
- **Hooks injection**: Constructor pattern `MarkdownRenderer::new(w).with_render_hooks(hooks)`

## 1. Image System (feature-gated: `image`)

### 1.1 Cargo.toml

```toml
[features]
image = ["dep:image"]

[dependencies]
image = { version = "0.25", optional = true, default-features = false, features = ["png"] }
```

### 1.2 ImageResolver trait (`src/markdown/image.rs`)

```rust
pub trait ImageResolver {
    fn resolve(&mut self, path: &str) -> Option<image::DynamicImage>;
    fn fallback(&self, path: &str, alt: &str) -> Span<'static>;
}

pub struct NoopImageResolver;
// always returns None
```

### 1.3 ImagePlacement & MarkdownRenderOutput

```rust
pub struct ImagePlacement {
    pub row: usize,
    pub col: usize,
    pub width_cells: u16,
    pub height_cells: u16,
    pub image: image::DynamicImage,
}

pub struct MarkdownRenderOutput {
    pub lines: Vec<Line<'static>>,
    pub images: Vec<ImagePlacement>,
}
```

### 1.4 MarkdownBlock::Image variant

```rust
Image {
    alt: String,
    path: String,
    #[cfg(feature = "image")]
    image: Option<image::DynamicImage>,
}
```

### 1.5 Parser changes

- `parse()` — unchanged, images produce `Image { alt, path, image: None }`
- `parse_with_images(resolver)` — calls `resolver.resolve(path)`, stores result

Detect `![alt](path)` syntax in parser.

### 1.6 Renderer changes

- `render()` — unchanged, Image blocks render fallback span
- `render_full()` — returns `MarkdownRenderOutput` with image placements

## 2. RenderHooks trait (all block types)

```rust
pub trait RenderHooks {
    // Heading
    fn heading1(&self, text: &str) -> Option<Line<'static>>;
    fn heading2(&self, text: &str) -> Option<Line<'static>>;
    fn heading3(&self, text: &str) -> Option<Line<'static>>;

    // Paragraph
    fn paragraph(&self, lines: &[String]) -> Option<Vec<Line<'static>>>;

    // Code block
    fn code_block_header(&self, lang: &str) -> Option<Line<'static>>;
    fn code_block_footer(&self, lang: &str, content_line_count: usize) -> Option<Line<'static>>;
    fn code_block_line(&self, line: &str, idx: usize, total: usize) -> Option<Line<'static>>;
    fn code_block_line_prefix(&self, lang: &str) -> Option<String>;

    // Inline code
    fn inline_code(&self, code: &str) -> Option<Line<'static>>;

    // List item
    fn list_item_marker(&self, indent: u8, is_last_in_group: bool, ancestors_are_last: &[bool], index_in_group: usize) -> Option<String>;
    fn list_item_content(&self, text: &str, indent: u8) -> Option<Vec<Line<'static>>>;

    // Blockquote
    fn blockquote(&self, text: &str) -> Option<Vec<Line<'static>>>;

    // Horizontal rule
    fn horizontal_rule(&self) -> Option<Line<'static>>;

    // Blank line
    fn blank_line(&self) -> Option<Line<'static>>;

    // Table
    fn table(&self, headers: &[String], rows: &[Vec<String>]) -> Option<Vec<Line<'static>>>;

    // Image fallback
    fn image_fallback(&self, alt: &str, path: &str) -> Option<Vec<Line<'static>>>;
}
```

Injected via constructor: `MarkdownRenderer::new(w).with_render_hooks(Box::new(my_hooks))`

## 3. Examples

| Example | File | Purpose |
|---------|------|---------|
| `basic` | `examples/basic.rs` | Basic markdown rendering with Paragraph |
| `image` | `examples/image.rs` | ImageResolver + render_full + ratatui-image overlay |
| `tree_list` | `examples/tree_list.rs` | RenderHooks customizing lists to tree-style box-drawing |
| `custom_code_block` | `examples/custom_code_block.rs` | RenderHooks customizing code block header/footer |

## 4. File Changes

| File | Change |
|------|--------|
| `Cargo.toml` | version 0.2.0, image feature, image dep |
| `src/markdown/types.rs` | Add `Image` variant to `MarkdownBlock` |
| `src/markdown/image.rs` | **NEW**: `ImageResolver`, `NoopImageResolver`, `ImagePlacement`, `MarkdownRenderOutput` |
| `src/markdown/hooks.rs` | **NEW**: `RenderHooks` trait |
| `src/markdown/parser.rs` | Add `parse_with_images()`, `![alt](path)` detection |
| `src/markdown/render.rs` | Add `render_full()`, hooks integration, image block rendering |
| `src/markdown/mod.rs` | Register new modules, update `MarkdownRenderer` struct with hooks field |
| `src/preview/mod.rs` | Adapt for new block type + hooks passthrough |
| `src/lib.rs` | Update docs, re-exports |
| `examples/basic.rs` | **NEW** |
| `examples/image.rs` | **NEW** |
| `examples/tree_list.rs` | **NEW** |
| `examples/custom_code_block.rs` | **NEW** |

## 5. Backward Compatibility

- `MarkdownBlock` enum: new variant (downstream match needs new arm — compile error, not silent)
- `MarkdownRenderer::new()`: unchanged
- `parse()`: signature unchanged, image syntax produces Image{image:None}
- `render()`: signature unchanged, Image blocks render fallback span
- `MarkdownPreview`: needs internal adaptation

## 6. Implementation Order

1. PLAN.md
2. Cargo.toml (version bump + features)
3. types.rs (MarkdownBlock::Image)
4. image.rs (ImageResolver trait)
5. hooks.rs (RenderHooks trait)
6. mod.rs (register modules, update MarkdownRenderer struct)
7. parser.rs (parse_with_images + image syntax detection)
8. render.rs (render_full + hooks integration)
9. preview/mod.rs (adapt)
10. lib.rs (docs + re-exports)
11. examples/
12. Tests
