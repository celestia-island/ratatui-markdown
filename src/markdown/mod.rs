mod hooks;
#[cfg(feature = "image")]
pub mod image;
mod inline;
mod parser;
mod render;
#[cfg(test)]
mod render_tests;
#[cfg(test)]
mod tests;
mod text;
mod types;

use std::boxed::Box;

pub use hooks::RenderHooks;
#[cfg(feature = "image")]
pub use image::{CropRect, ImagePlacement, ImageResolver, MarkdownRenderOutput, NoopImageResolver};
pub use inline::parse_inline_formatting;
pub use types::MarkdownBlock;

pub struct MarkdownRenderer {
    pub(crate) max_width: usize,
    pub(crate) hooks: Option<Box<dyn RenderHooks>>,
}

impl MarkdownRenderer {
    pub fn new(max_width: usize) -> Self {
        Self {
            max_width,
            hooks: None,
        }
    }

    pub fn with_render_hooks(mut self, hooks: Box<dyn RenderHooks>) -> Self {
        self.hooks = Some(hooks);
        self
    }
}
