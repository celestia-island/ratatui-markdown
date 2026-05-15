mod hooks;
mod inline;
#[cfg(feature = "image")]
pub mod image;
mod parser;
mod render;
#[cfg(test)]
mod render_tests;
#[cfg(test)]
mod tests;
mod text;
mod types;

pub use hooks::RenderHooks;
pub use inline::parse_inline_formatting;
pub use types::MarkdownBlock;

#[cfg(feature = "image")]
pub use image::{CropRect, ImagePlacement, ImageResolver, MarkdownRenderOutput, NoopImageResolver};

use std::boxed::Box;

pub struct MarkdownRenderer {
    pub(crate) max_width: usize,
    pub(crate) hooks: Option<Box<dyn RenderHooks>>,
    pub(crate) tree_indent_width: usize,
    pub(crate) tree_text_gap: usize,
}

impl MarkdownRenderer {
    pub fn new(max_width: usize) -> Self {
        Self {
            max_width,
            hooks: None,
            tree_indent_width: 3,
            tree_text_gap: 0,
        }
    }

    pub fn with_render_hooks(mut self, hooks: Box<dyn RenderHooks>) -> Self {
        self.hooks = Some(hooks);
        self
    }

    pub fn with_tree_indent_width(mut self, width: usize) -> Self {
        self.tree_indent_width = width.max(1);
        self
    }

    pub fn with_tree_text_gap(mut self, gap: usize) -> Self {
        self.tree_text_gap = gap;
        self
    }
}
