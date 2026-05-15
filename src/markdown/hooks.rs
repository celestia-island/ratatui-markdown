use ratatui::text::Line;

use super::types::MarkdownBlock;

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

    fn render_code_block(
        &self,
        _lang: &str,
        _content: &str,
    ) -> Option<Vec<Line<'static>>> {
        None
    }

    fn code_block_header(&self, _lang: &str) -> Option<Line<'static>> {
        None
    }

    fn code_block_footer(&self, _lang: &str, _content_line_count: usize) -> Option<Line<'static>> {
        None
    }

    fn code_block_line(
        &self,
        _line: &str,
        _idx: usize,
        _total: usize,
    ) -> Option<Line<'static>> {
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

    fn tree_indent_width(&self) -> Option<usize> {
        None
    }

    fn tree_text_gap(&self) -> Option<usize> {
        None
    }

    fn list_item_content(&self, _text: &str, _indent: u8) -> Option<Vec<Line<'static>>> {
        None
    }

    fn blockquote(
        &self,
        _level: u8,
        _children: &[MarkdownBlock],
    ) -> Option<Vec<Line<'static>>> {
        None
    }

    fn horizontal_rule(&self) -> Option<Line<'static>> {
        None
    }

    fn blank_line(&self) -> Option<Line<'static>> {
        None
    }

    fn table(
        &self,
        _headers: &[String],
        _rows: &[Vec<String>],
    ) -> Option<Vec<Line<'static>>> {
        None
    }

    fn image_fallback(&self, _alt: &str, _path: &str) -> Option<Vec<Line<'static>>> {
        None
    }
}
