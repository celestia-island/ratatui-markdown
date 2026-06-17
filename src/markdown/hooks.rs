use ratatui::text::Line;

use super::types::MarkdownBlock;
#[cfg(feature = "image")]
use crate::markdown::image::image;

pub trait RenderHooks: Send + Sync {
    fn heading1(&self, _text: &str) -> Option<Line<'static>> {
        None
    }

    fn heading2(&self, _text: &str) -> Option<Line<'static>> {
        None
    }

    fn heading3(&self, _text: &str) -> Option<Line<'static>> {
        None
    }

    fn paragraph(&self, _lines: &[String]) -> Option<Vec<Line<'static>>> {
        None
    }

    fn render_code_block(&self, _lang: &str, _content: &str) -> Option<Vec<Line<'static>>> {
        None
    }

    fn code_block_header(&self, _lang: &str) -> Option<Line<'static>> {
        None
    }

    fn code_block_footer(&self, _lang: &str, _content_line_count: usize) -> Option<Line<'static>> {
        None
    }

    fn code_block_line(&self, _line: &str, _idx: usize, _total: usize) -> Option<Line<'static>> {
        None
    }

    fn code_block_line_prefix(&self, _lang: &str) -> Option<String> {
        None
    }

    fn inline_code(&self, _code: &str) -> Option<Line<'static>> {
        None
    }

    fn list_item_marker(
        &self,
        _indent: u8,
        _is_last_in_group: bool,
        _ancestors_are_last: &[bool],
        _index_in_group: usize,
    ) -> Option<String> {
        None
    }

    /// Total character width for each level of tree indentation (including continuation lines/blank fill).
    /// Returns `None` to disable tree-style list rendering.
    ///
    /// Internal conventions:
    /// - Continuation line = `│` + (unit - 1) spaces
    /// - Blank fill = unit spaces
    /// - Connector = `├─ ` / `└─ ` (fixed 3 characters, not included in this value)
    ///
    /// Examples:
    /// - `Some(3)` → compact: `│  ├─ ` (3 columns per level)
    /// - `Some(4)` → relaxed: `│   ├─ ` (4 columns per level)
    fn tree_indent_unit(&self) -> Option<usize> {
        None
    }

    /// Continuation prefix for wrapped lines (preserves ancestor-level `│` continuation lines).
    /// Parameters are the same as `list_item_marker`; the return value is used for the 2nd line
    /// and beyond after text wrapping.
    /// Returns `None` to fall back to equal-width blank spaces.
    fn tree_continuation_prefix(
        &self,
        _indent: u8,
        _ancestors_are_last: &[bool],
    ) -> Option<String> {
        None
    }

    fn list_item_content(&self, _text: &str, _indent: u8) -> Option<Vec<Line<'static>>> {
        None
    }

    fn blockquote(&self, _level: u8, _children: &[MarkdownBlock]) -> Option<Vec<Line<'static>>> {
        None
    }

    fn horizontal_rule(&self) -> Option<Line<'static>> {
        None
    }

    fn blank_line(&self) -> Option<Line<'static>> {
        None
    }

    fn table(&self, _headers: &[String], _rows: &[Vec<String>]) -> Option<Vec<Line<'static>>> {
        None
    }

    fn image_fallback(&self, _alt: &str, _path: &str) -> Option<Vec<Line<'static>>> {
        None
    }

    #[cfg(feature = "image")]
    fn render_mermaid_image(&self, _source: &str) -> Option<image::DynamicImage> {
        None
    }
}
