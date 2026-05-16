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

    /// 每级树形缩进的字符总宽度（含延续线/空白）。
    /// 返回 `None` 表示不启用树形列表渲染。
    ///
    /// 内部约定：
    /// - 延续线 = `│` + (unit - 1) 个空格
    /// - 空白填充 = unit 个空格
    /// - 连接符 = `├─ ` / `└─ `（固定 3 字符，不含在此值内）
    ///
    /// 例如：
    /// - `Some(3)` → 紧凑：`│  ├─ `（每级差 3 列）
    /// - `Some(4)` → 宽松：`│   ├─ `（每级差 4 列）
    fn tree_indent_unit(&self) -> Option<usize> {
        None
    }

    /// 换行续行的前缀（保留祖先层级的 `│` 延续线）。
    /// 参数与 `list_item_marker` 相同，返回值用于文本换行后的第 2 行及之后。
    /// 返回 `None` 时渲染器回退到等宽纯空格。
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
}
