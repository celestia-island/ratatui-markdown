mod segment;

#[cfg(feature = "highlight")]
mod treesitter;

#[cfg(feature = "highlight")]
mod hooks;

#[cfg(feature = "highlight-lang-mcfunction")]
mod mcfunction;

#[cfg(feature = "highlight-lang-brainfuck")]
mod brainfuck;

pub use segment::segments_to_lines;

#[cfg(feature = "highlight")]
pub use treesitter::TreeSitterHighlighter;

#[cfg(feature = "highlight")]
pub use hooks::HighlightHooks;

#[cfg(feature = "highlight-lang-mcfunction")]
pub use mcfunction::McfunctionHighlighter;

#[cfg(feature = "highlight-lang-brainfuck")]
pub use brainfuck::BrainfuckHighlighter;

use ratatui::style::Style;
use ratatui::text::Line;

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
    let segments = highlighter.highlight(lang, code);
    segment::segments_to_lines(code, &segments, prefix, border_style, max_width)
}
