use ratatui::{
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Span},
};

#[cfg(feature = "image")]
pub use image;

#[cfg(feature = "image")]
pub struct ImagePlacement {
    pub row: usize,
    pub col: usize,
    pub width_cells: u16,
    pub height_cells: u16,
    pub image: image::DynamicImage,
}

#[cfg(feature = "image")]
pub struct ResolvedImage {
    pub path: String,
    pub image: image::DynamicImage,
}

#[cfg(feature = "image")]
pub struct MarkdownRenderOutput {
    pub lines: Vec<Line<'static>>,
    pub images: Vec<ImagePlacement>,
}

#[cfg(feature = "image")]
pub trait ImageResolver {
    fn resolve(&mut self, path: &str) -> Option<image::DynamicImage>;

    fn fallback(&self, path: &str, alt: &str) -> Span<'static> {
        let label = if alt.is_empty() {
            path.to_string()
        } else {
            alt.to_string()
        };
        Span::styled(
            format!("[image: {label}]"),
            Style::default().italic().fg(Color::Gray),
        )
    }
}

#[cfg(feature = "image")]
pub struct NoopImageResolver;

#[cfg(feature = "image")]
impl ImageResolver for NoopImageResolver {
    fn resolve(&mut self, _path: &str) -> Option<image::DynamicImage> {
        None
    }
}

#[cfg(feature = "image")]
impl MarkdownRenderOutput {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            images: Vec::new(),
        }
    }
}

#[cfg(feature = "image")]
impl Default for MarkdownRenderOutput {
    fn default() -> Self {
        Self::new()
    }
}
