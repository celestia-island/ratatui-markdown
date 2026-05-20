mod render;
mod scroll;

use ratatui::{
    style::{Color, Style},
    text::Span,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorLineMode {
    #[default]
    HeaderOnly,
    AllLines,
}

#[derive(Debug, Clone)]
pub struct SpanTreeEntry {
    pub id: String,
    pub lines: Vec<Vec<Span<'static>>>,
}

impl SpanTreeEntry {
    pub fn new(id: impl Into<String>, lines: Vec<Vec<Span<'static>>>) -> Self {
        Self {
            id: id.into(),
            lines,
        }
    }

    pub fn total_lines(&self) -> usize {
        self.lines.len().max(1)
    }
}

pub struct SpanTree {
    entries: Vec<SpanTreeEntry>,
    selected_id: Option<String>,
    scroll_offset: usize,
    viewport_height: usize,
    cursor_span: Span<'static>,
    blank_cursor_span: Span<'static>,
    cursor_column: usize,
    auto_follow: bool,
    cursor_line_mode: CursorLineMode,
}

impl Default for SpanTree {
    fn default() -> Self {
        Self::new()
    }
}

impl SpanTree {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            selected_id: None,
            scroll_offset: 0,
            viewport_height: 10,
            cursor_span: Span::styled("▸", Style::default().fg(Color::Cyan)),
            blank_cursor_span: Span::raw(" "),
            cursor_column: 0,
            auto_follow: false,
            cursor_line_mode: CursorLineMode::default(),
        }
    }

    pub fn with_cursor_style(mut self, cursor: Span<'static>, blank: Span<'static>) -> Self {
        self.cursor_span = cursor;
        self.blank_cursor_span = blank;
        self
    }

    pub fn with_cursor_column(mut self, col: usize) -> Self {
        self.cursor_column = col;
        self
    }

    pub fn with_auto_follow(mut self, follow: bool) -> Self {
        self.auto_follow = follow;
        self
    }

    pub fn with_cursor_line_mode(mut self, mode: CursorLineMode) -> Self {
        self.cursor_line_mode = mode;
        self
    }

    pub fn set_entries(&mut self, entries: Vec<SpanTreeEntry>) {
        self.entries = entries;
        if self.auto_follow {
            self.scroll_to_last_entry();
        } else {
            self.clamp_scroll_offset();
        }
    }

    pub fn set_selected(&mut self, id: &str) {
        if self.entry_index_by_id(id).is_some() {
            self.selected_id = Some(id.to_string());
            self.scroll_to_selected();
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_id = None;
    }

    pub fn set_selected_index(&mut self, index: usize) {
        if index < self.entries.len() {
            self.selected_id = Some(self.entries[index].id.clone());
            self.scroll_to_selected();
        }
    }

    pub fn selected_id(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_id
            .as_ref()
            .and_then(|id| self.entry_index_by_id(id))
    }

    pub fn total_lines(&self) -> usize {
        if self.entries.is_empty() {
            return 0;
        }
        self.entries.iter().map(|e| e.total_lines()).sum()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset.min(self.max_scroll_offset());
    }

    pub fn viewport_height(&self) -> usize {
        self.viewport_height
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn cursor_line_mode(&self) -> CursorLineMode {
        self.cursor_line_mode
    }

    pub fn render(
        &mut self,
        f: &mut ratatui::Frame,
        inner_area: ratatui::layout::Rect,
        outer_area: ratatui::layout::Rect,
        theme: &impl crate::theme::RichTextTheme,
    ) {
        render::render(self, f, inner_area, outer_area, theme);
    }

    pub fn navigate_up(&mut self) {
        scroll::navigate_up(self);
    }

    pub fn navigate_down(&mut self) {
        scroll::navigate_down(self);
    }

    pub fn navigate_to_first(&mut self) {
        scroll::navigate_to_first(self);
    }

    pub fn navigate_to_last(&mut self) {
        scroll::navigate_to_last(self);
    }

    pub fn scroll_up(&mut self, lines: usize) {
        scroll::scroll_up(self, lines);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        scroll::scroll_down(self, lines);
    }

    pub(in crate::scroll) fn entry_index_by_id(&self, id: &str) -> Option<usize> {
        self.entries.iter().position(|e| e.id == id)
    }

    pub(in crate::scroll) fn line_offset_for_entry(&self, entry_idx: usize) -> usize {
        self.entries[..entry_idx]
            .iter()
            .map(|e| e.total_lines())
            .sum()
    }

    pub(in crate::scroll) fn line_count_up_to(&self, entry_idx: usize) -> usize {
        self.entries[..=entry_idx]
            .iter()
            .map(|e| e.total_lines())
            .sum()
    }

    pub(in crate::scroll) fn max_scroll_offset(&self) -> usize {
        let total = self.total_lines();
        total.saturating_sub(self.viewport_height)
    }

    pub(in crate::scroll) fn clamp_scroll_offset(&mut self) {
        let max = self.max_scroll_offset();
        if self.scroll_offset > max {
            self.scroll_offset = max;
        }
    }

    pub(in crate::scroll) fn scroll_to_selected(&mut self) {
        if let Some(idx) = self.selected_index() {
            let entry_start = self.line_offset_for_entry(idx);
            let entry_end = self.line_count_up_to(idx);
            let vp = self.viewport_height;

            if entry_start < self.scroll_offset {
                self.scroll_offset = entry_start;
            } else if entry_end > self.scroll_offset + vp {
                self.scroll_offset = entry_end.saturating_sub(vp);
            }
        }
    }

    pub fn center_on_selected(&mut self) {
        if let Some(idx) = self.selected_index() {
            let entry_start = self.line_offset_for_entry(idx);
            let entry_lines = self.entries[idx].total_lines();
            let entry_center = entry_start + entry_lines / 2;
            let target = entry_center.saturating_sub(self.viewport_height / 2);
            self.scroll_offset = target.min(self.max_scroll_offset());
        }
    }

    fn scroll_to_last_entry(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let total = self.total_lines();
        let vp = self.viewport_height;
        self.scroll_offset = total.saturating_sub(vp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::Span;

    fn make_entry(id: &str, line_count: usize) -> SpanTreeEntry {
        let lines = (0..line_count)
            .map(|i| vec![Span::raw(format!("{}-line-{}", id, i))])
            .collect();
        SpanTreeEntry::new(id, lines)
    }

    #[test]
    fn empty_tree_has_no_entries() {
        let tree = SpanTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.entry_count(), 0);
        assert_eq!(tree.total_lines(), 0);
    }

    #[test]
    fn set_entries_updates_count() {
        let mut tree = SpanTree::new();
        tree.set_entries(vec![make_entry("a", 2), make_entry("b", 3)]);
        assert_eq!(tree.entry_count(), 2);
        assert_eq!(tree.total_lines(), 5);
    }

    #[test]
    fn set_selected_finds_entry() {
        let mut tree = SpanTree::new();
        tree.set_entries(vec![
            make_entry("a", 1),
            make_entry("b", 1),
            make_entry("c", 1),
        ]);
        tree.set_selected("b");
        assert_eq!(tree.selected_id(), Some("b"));
        assert_eq!(tree.selected_index(), Some(1));
    }

    #[test]
    fn set_selected_unknown_id_ignored() {
        let mut tree = SpanTree::new();
        tree.set_entries(vec![make_entry("a", 1)]);
        tree.set_selected("b");
        assert_eq!(tree.selected_id(), None);
    }

    #[test]
    fn clear_selection_works() {
        let mut tree = SpanTree::new();
        tree.set_entries(vec![make_entry("a", 1)]);
        tree.set_selected("a");
        assert_eq!(tree.selected_id(), Some("a"));
        tree.clear_selection();
        assert_eq!(tree.selected_id(), None);
    }

    #[test]
    fn set_selected_index_works() {
        let mut tree = SpanTree::new();
        tree.set_entries(vec![make_entry("a", 1), make_entry("b", 1)]);
        tree.set_selected_index(1);
        assert_eq!(tree.selected_id(), Some("b"));
    }

    #[test]
    fn navigate_down_moves_selection() {
        let mut tree = SpanTree::new();
        tree.set_entries(vec![
            make_entry("a", 1),
            make_entry("b", 1),
            make_entry("c", 1),
        ]);
        tree.set_selected("a");
        tree.navigate_down();
        assert_eq!(tree.selected_id(), Some("b"));
        tree.navigate_down();
        assert_eq!(tree.selected_id(), Some("c"));
    }

    #[test]
    fn navigate_up_moves_selection() {
        let mut tree = SpanTree::new();
        tree.set_entries(vec![
            make_entry("a", 1),
            make_entry("b", 1),
            make_entry("c", 1),
        ]);
        tree.set_selected("c");
        tree.navigate_up();
        assert_eq!(tree.selected_id(), Some("b"));
        tree.navigate_up();
        assert_eq!(tree.selected_id(), Some("a"));
    }

    #[test]
    fn navigate_to_first_and_last() {
        let mut tree = SpanTree::new();
        tree.set_entries(vec![
            make_entry("a", 1),
            make_entry("b", 1),
            make_entry("c", 1),
        ]);
        tree.navigate_to_last();
        assert_eq!(tree.selected_id(), Some("c"));
        tree.navigate_to_first();
        assert_eq!(tree.selected_id(), Some("a"));
    }

    #[test]
    fn navigate_down_from_none_selects_first() {
        let mut tree = SpanTree::new();
        tree.set_entries(vec![make_entry("a", 1), make_entry("b", 1)]);
        tree.navigate_down();
        assert_eq!(tree.selected_id(), Some("a"));
    }

    #[test]
    fn scroll_offset_clamps_on_set_entries() {
        let mut tree = SpanTree::new();
        tree.viewport_height = 2;
        tree.set_entries(vec![make_entry("a", 5), make_entry("b", 5)]);
        tree.scroll_offset = 100;
        tree.set_entries(vec![make_entry("x", 1)]);
        assert!(tree.scroll_offset <= tree.max_scroll_offset());
    }

    #[test]
    fn auto_follow_keeps_at_bottom() {
        let mut tree = SpanTree::new().with_auto_follow(true);
        tree.viewport_height = 3;
        tree.set_entries(vec![make_entry("a", 2), make_entry("b", 2)]);
        let offset_before = tree.scroll_offset();
        tree.set_entries(vec![
            make_entry("a", 2),
            make_entry("b", 2),
            make_entry("c", 2),
        ]);
        assert!(tree.scroll_offset() >= offset_before);
    }

    #[test]
    fn total_lines_counts_multi_line_entries() {
        let mut tree = SpanTree::new();
        tree.set_entries(vec![make_entry("a", 3), make_entry("b", 2)]);
        assert_eq!(tree.total_lines(), 5);
    }

    #[test]
    fn cursor_column_customization() {
        let tree = SpanTree::new().with_cursor_column(2);
        assert_eq!(tree.cursor_column, 2);
    }

    #[test]
    fn cursor_style_customization() {
        let tree = SpanTree::new().with_cursor_style(Span::raw(">"), Span::raw(" "));
        assert_eq!(tree.cursor_span.content, ">");
        assert_eq!(tree.blank_cursor_span.content, " ");
    }

    #[test]
    fn scroll_up_and_down_adjust_offset() {
        let mut tree = SpanTree::new();
        tree.viewport_height = 2;
        tree.set_entries(vec![make_entry("a", 5), make_entry("b", 5)]);
        tree.scroll_down(3);
        assert_eq!(tree.scroll_offset(), 3);
        tree.scroll_up(2);
        assert_eq!(tree.scroll_offset(), 1);
    }

    #[test]
    fn cursor_line_mode_default_is_header_only() {
        let tree = SpanTree::new();
        assert_eq!(tree.cursor_line_mode(), CursorLineMode::HeaderOnly);
    }

    #[test]
    fn cursor_line_mode_all_lines_builder() {
        let tree = SpanTree::new().with_cursor_line_mode(CursorLineMode::AllLines);
        assert_eq!(tree.cursor_line_mode(), CursorLineMode::AllLines);
    }
}
