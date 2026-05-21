#[cfg(feature = "highlight")]
mod config;
#[cfg(feature = "highlight")]
mod hooks;
#[cfg(feature = "highlight-pest")]
mod pest_bridge;
mod segment;
#[cfg(feature = "highlight")]
mod treesitter;

#[cfg(feature = "highlight")]
pub use config::{highlight_to_style, HIGHLIGHT_NAMES};
#[cfg(feature = "highlight")]
pub use hooks::HighlightHooks;
#[cfg(feature = "highlight-pest")]
pub use pest_bridge::pest_pairs_to_segments;
use ratatui::{style::Style, text::Line};
pub use segment::segments_to_lines;
#[cfg(feature = "highlight")]
pub use treesitter::TreeSitterHighlighter;

#[derive(Debug, Clone)]
pub struct StyleSegment {
    pub start: usize,
    pub end: usize,
    pub style: Style,
}

pub trait CodeHighlighter: Send + Sync {
    fn highlight(&self, lang: &str, code: &str) -> Vec<StyleSegment>;
}

pub fn highlight_to_lines(
    highlighter: &dyn CodeHighlighter,
    lang: &str,
    code: &str,
    prefix: &str,
    border_style: Style,
    max_width: usize,
) -> Vec<Line<'static>> {
    let code = code.replace('\t', "    ");
    let segments = highlighter.highlight(lang, &code);
    segment::segments_to_lines(&code, &segments, prefix, border_style, max_width)
}
