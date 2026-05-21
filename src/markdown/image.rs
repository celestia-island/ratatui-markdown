#[cfg(feature = "image")]
pub use image;
use ratatui::{
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Span},
};

#[cfg(feature = "image")]
pub struct CropRect {
    pub col_offset: u16,
    pub row_offset: u16,
    pub visible_width: u16,
    pub visible_height: u16,
}

#[cfg(feature = "image")]
pub struct ImagePlacement {
    pub row: usize,
    pub col: usize,
    pub width_cells: u16,
    pub height_cells: u16,
    pub image: image::DynamicImage,
    pub crop: Option<CropRect>,
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

/// Approximate pixel-to-cell conversion for a typical monospace terminal font.
/// These are defaults; override [`ImageResolver::cell_dimensions`] for precision.
#[cfg(feature = "image")]
const DEFAULT_PIXELS_PER_CELL_W: u32 = 9;
#[cfg(feature = "image")]
const DEFAULT_PIXELS_PER_CELL_H: u32 = 18;

#[cfg(feature = "image")]
pub trait ImageResolver {
    fn resolve(&mut self, path: &str) -> Option<image::DynamicImage>;

    /// Determine the terminal **cell** dimensions for rendering a resolved image.
    ///
    /// Called by [`MarkdownRenderer::render_full`] after `resolve()` succeeds.
    /// The returned `(width_cells, height_cells)` controls:
    /// - How many blank lines are reserved in the text output so content doesn't overlap
    /// - The `width_cells` / `height_cells` stored in [`ImagePlacement`] for your draw loop
    ///
    /// **Default**: fits to `max_width` preserving aspect ratio using
    /// `~9 px / cell` width and `~18 px / cell` height heuristics.
    /// Override this when you have exact knowledge of your terminal's cell size
    /// or want to constrain images differently.
    fn cell_dimensions(
        &mut self,
        img: &image::DynamicImage,
        max_width: u16,
        _max_height: u16,
    ) -> (u16, u16) {
        let pw = img.width();
        let ph = img.height();
        if pw == 0 || ph == 0 || max_width == 0 {
            return (0, 0);
        }
        let w_cells = pw.div_ceil(DEFAULT_PIXELS_PER_CELL_W) as u16;
        let w = w_cells.min(max_width);
        let ratio = ph * w as u32 / (pw.max(1));
        let h = ratio.div_ceil(DEFAULT_PIXELS_PER_CELL_H) as u16;
        (w.max(1), h.max(1))
    }

    fn fallback(&self, path: &str, alt: &str) -> Span<'static> {
        let label = if alt.is_empty() {
            path.to_string()
        } else {
            alt.to_string()
        };
        let label = label.replace('\t', "    ");
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
