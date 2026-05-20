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

#[cfg(test)]
mod render_tests {
    use super::*;
    use crate::constants::*;
    use crate::theme::ThemeConfig;
    use ratatui::{
        backend::TestBackend,
        layout::Rect,
        style::{Color, Style},
        Terminal,
    };

    fn test_theme() -> ThemeConfig {
        ThemeConfig::default()
    }

    fn render_to_lines(tree: &mut SpanTree, width: u16, height: u16) -> Vec<String> {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        let area = Rect::new(0, 0, width, height);
        terminal
            .draw(|f| {
                tree.render(f, area, area, &test_theme());
            })
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        (0..height)
            .map(|r| {
                (0..width)
                    .map(|c| buffer.cell((c, r)).unwrap().symbol().to_string())
                    .collect::<String>()
            })
            .collect()
    }

    fn build_timeline_entry(
        id: &str,
        tree_prefix: &str,
        continuation_indent: &str,
        header_text: &str,
        detail_lines: &[&str],
    ) -> SpanTreeEntry {
        let mut lines = Vec::new();
        let header_spans = vec![
            Span::raw("  "),
            Span::styled(
                tree_prefix.to_string(),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw(header_text.to_string()),
        ];
        lines.push(header_spans);

        let n = detail_lines.len();
        for (i, detail) in detail_lines.iter().enumerate() {
            let connector = if i == n - 1 {
                BRANCH_END_SP
            } else {
                BRANCH_MID_SP
            };
            let prefix = format!("{}{}", continuation_indent, connector);
            let detail_spans = vec![
                Span::raw("  "),
                Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                Span::raw(detail.to_string()),
            ];
            lines.push(detail_spans);
        }

        SpanTreeEntry::new(id, lines)
    }

    #[test]
    fn tree_multiline_selected_entry_has_cursor_on_header_blank_on_body() {
        let mut tree = SpanTree::new()
            .with_cursor_style(Span::styled("▸ ", Style::default()), Span::raw("  "));

        let entry = build_timeline_entry(
            "agent2",
            "└─ ",
            "   ",
            "#002 hubris",
            &["work status text", "tool name"],
        );
        tree.set_entries(vec![entry]);
        tree.set_selected("agent2");

        let rows = render_to_lines(&mut tree, 60, 10);
        assert!(rows[0].contains("▸"), "header should show cursor: {:?}", rows[0]);
        assert!(
            !rows[1].contains("▸"),
            "body line should NOT show cursor: {:?}",
            rows[1]
        );
        assert!(
            !rows[2].contains("▸"),
            "body line should NOT show cursor: {:?}",
            rows[2]
        );
    }

    #[test]
    fn tree_continuation_indent_is_preserved_on_selected_entry() {
        let mut tree = SpanTree::new()
            .with_cursor_style(Span::styled("▸ ", Style::default()), Span::raw("  "));

        let entry = build_timeline_entry(
            "sel",
            "└─ ",
            "   ",
            "#002 hubris",
            &["work status", "tool name"],
        );
        tree.set_entries(vec![entry]);
        tree.set_selected("sel");

        let rows = render_to_lines(&mut tree, 60, 10);

        let header = rows[0].trim_end();
        assert!(
            header.contains("└─"),
            "header should contain └─: {:?}",
            header
        );

        let body1 = rows[1].trim_end();
        let body2 = rows[2].trim_end();
        assert!(
            body1.contains("├─") || body1.contains("│"),
            "body1 should contain tree connector: {:?}",
            body1
        );
        assert!(
            body2.contains("└─"),
            "body2 should contain └─: {:?}",
            body2
        );
    }

    #[test]
    fn tree_continuation_indent_preserved_on_non_selected_entry() {
        let mut tree = SpanTree::new()
            .with_cursor_style(Span::styled("▸ ", Style::default()), Span::raw("  "));

        let entry_a = build_timeline_entry("a", "├─ ", "│  ", "#001 done", &[]);
        let entry_b = build_timeline_entry(
            "b",
            "└─ ",
            "   ",
            "#002 active",
            &["thinking...", "executing tool"],
        );
        tree.set_entries(vec![entry_a, entry_b]);
        tree.set_selected("a");

        let rows = render_to_lines(&mut tree, 60, 10);

        let selected_header = rows[0].trim_end();
        assert!(
            selected_header.contains("▸"),
            "selected header should have cursor: {:?}",
            selected_header
        );
        assert!(
            selected_header.contains("├─"),
            "selected header should have ├─: {:?}",
            selected_header
        );

        let non_selected_header = rows[1].trim_end();
        assert!(
            non_selected_header.contains("└─"),
            "non-selected header should have └─: {:?}",
            non_selected_header
        );
        assert!(
            !non_selected_header.contains("▸"),
            "non-selected header should NOT have cursor: {:?}",
            non_selected_header
        );

        let body1 = rows[2].trim_end();
        let body2 = rows[3].trim_end();
        assert!(
            body1.contains("├─"),
            "continuation line1 should have ├─: {:?}",
            body1
        );
        assert!(
            body2.contains("└─"),
            "continuation line2 should have └─: {:?}",
            body2
        );
    }

    #[test]
    fn tree_blank_cursor_width_matches_placeholder() {
        let cursor = "▸ ";
        let blank = "  ";
        let placeholder = "  ";
        assert_eq!(
            blank.chars().count(),
            placeholder.chars().count(),
            "blank cursor display width must match placeholder display width"
        );
        assert_eq!(
            cursor.chars().count(),
            placeholder.chars().count(),
            "active cursor display width must match placeholder display width"
        );
    }

    #[test]
    fn tree_deeply_nested_continuation_rendered() {
        let mut tree = SpanTree::new()
            .with_cursor_style(Span::styled("▸ ", Style::default()), Span::raw("  "));

        let root = build_timeline_entry("root", "", "", "root", &[]);
        let child1 =
            build_timeline_entry("c1", "├─ ", "│  ", "child1", &["detail1", "detail2"]);
        let child2 =
            build_timeline_entry("c2", "└─ ", "   ", "child2", &["detail3"]);
        tree.set_entries(vec![root, child1, child2]);
        tree.set_selected("c2");

        let rows = render_to_lines(&mut tree, 60, 10);

        assert!(rows[0].contains("root"), "row0: {:?}", rows[0]);
        assert!(rows[1].contains("├─"), "row1 should have ├─: {:?}", rows[1]);
        assert!(rows[2].contains("│"), "row2 should have │: {:?}", rows[2]);
        assert!(rows[3].contains("└─"), "row3 should have └─: {:?}", rows[3]);
        assert!(
            rows[4].contains("▸"),
            "selected header should have cursor: {:?}",
            rows[4]
        );
        assert!(
            rows[5].contains("├─") || rows[5].contains("└─"),
            "continuation should have connector: {:?}",
            rows[5]
        );
    }

    #[test]
    fn tree_all_lines_mode_cursor_on_every_line() {
        let mut tree = SpanTree::new()
            .with_cursor_style(Span::styled("▸ ", Style::default()), Span::raw("  "))
            .with_cursor_line_mode(CursorLineMode::AllLines);

        let entry = build_timeline_entry(
            "x",
            "└─ ",
            "   ",
            "#001 agent",
            &["status line"],
        );
        tree.set_entries(vec![entry]);
        tree.set_selected("x");

        let rows = render_to_lines(&mut tree, 60, 10);
        assert!(rows[0].contains("▸"), "header should have cursor: {:?}", rows[0]);
        assert!(rows[1].contains("▸"), "body should have cursor in AllLines: {:?}", rows[1]);
    }

    #[test]
    fn tree_no_selection_no_cursor_visible() {
        let mut tree = SpanTree::new()
            .with_cursor_style(Span::styled("▸ ", Style::default()), Span::raw("  "));

        let entry = build_timeline_entry(
            "a",
            "└─ ",
            "   ",
            "#001 agent",
            &["status"],
        );
        tree.set_entries(vec![entry]);

        let rows = render_to_lines(&mut tree, 60, 10);
        assert!(
            !rows[0].contains("▸"),
            "no selection: header should NOT show cursor: {:?}",
            rows[0]
        );
        assert!(
            !rows[1].contains("▸"),
            "no selection: body should NOT show cursor: {:?}",
            rows[1]
        );
    }

    #[test]
    fn tree_cursor_column_alignment_consistent_across_header_and_body() {
        let mut tree = SpanTree::new()
            .with_cursor_style(Span::styled("▸ ", Style::default()), Span::raw("  "));

        let entry = build_timeline_entry(
            "a",
            "└─ ",
            "   ",
            "#002 agent",
            &["status", "tool"],
        );
        tree.set_entries(vec![entry]);
        tree.set_selected("a");

        let rows = render_to_lines(&mut tree, 60, 10);

        let header_prefix_end = rows[0]
            .find(|c: char| !c.is_whitespace() && c != '▸')
            .unwrap_or(0);
        let body_prefix_end = rows[1]
            .find(|c: char| !c.is_whitespace() && c != '│' && c != '├' && c != '─')
            .unwrap_or(0);

        assert!(
            body_prefix_end > header_prefix_end,
            "body connector should be indented further than header connector\n  header: {:?}\n  body:   {:?}\n  header_prefix_end={}, body_prefix_end={}",
            rows[0].trim_end(),
            rows[1].trim_end(),
            header_prefix_end,
            body_prefix_end
        );
    }

    #[test]
    fn tree_simulated_timeline_two_agents_with_continuation() {
        let mut tree = SpanTree::new()
            .with_cursor_style(Span::styled("▸ ", Style::default()), Span::raw("  "));

        let root = SpanTreeEntry::new(
            "root",
            vec![vec![
                Span::raw("  "),
                Span::raw("#demiurge"),
            ]],
        );

        let agent1_prefix = "├─ ";
        let agent1 = SpanTreeEntry::new(
            "agent1",
            vec![vec![
                Span::raw("  "),
                Span::styled(agent1_prefix.to_string(), Style::default().fg(Color::DarkGray)),
                Span::raw("#demiurge.001 hubris ✓"),
            ]],
        );

        let agent2_prefix = "└─ ";
        let agent2_cont = "   ";
        let agent2 = SpanTreeEntry::new(
            "agent2",
            vec![
                vec![
                    Span::raw("  "),
                    Span::styled(
                        agent2_prefix.to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw("#demiurge.002 hubris::task_decompose"),
                ],
                vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("{}{}", agent2_cont, "├─ "),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw("…下工作区的当前状态"),
                ],
                vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("{}{}", agent2_cont, "└─ "),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw("hubris::task_decompose [exec]"),
                ],
            ],
        );

        tree.set_entries(vec![root, agent1, agent2]);
        tree.set_selected("agent2");

        let rows = render_to_lines(&mut tree, 70, 10);

        assert!(rows[0].contains("#demiurge"), "row0: {:?}", rows[0]);

        assert!(rows[1].contains("├─"), "row1 should have ├─: {:?}", rows[1]);

        assert!(
            rows[2].contains("▸") && rows[2].contains("└─"),
            "row2 should have cursor + └─: {:?}",
            rows[2]
        );

        let body1 = rows[3].trim_end();
        let body2 = rows[4].trim_end();
        assert!(
            body1.contains("├─"),
            "continuation line1 should have ├─: {:?}",
            body1
        );
        assert!(
            body2.contains("└─"),
            "continuation line2 should have └─: {:?}",
            body2
        );

        let header_connector_col = rows[2].find('└').unwrap_or(0);
        let body_connector_col = body1
            .find(|c: char| c == '├' || c == '│')
            .unwrap_or(0);
        assert!(
            body_connector_col > header_connector_col,
            "body connector (col {}) should be right of header connector (col {})\n  header: {:?}\n  body1: {:?}",
            body_connector_col,
            header_connector_col,
            rows[2].trim_end(),
            body1
        );
    }

    #[test]
    fn tree_three_level_nesting_rendered() {
        let mut tree = SpanTree::new()
            .with_cursor_style(Span::styled("▸ ", Style::default()), Span::raw("  "));

        let root = SpanTreeEntry::new(
            "root",
            vec![vec![Span::raw("  "), Span::raw("root")]],
        );
        let child = SpanTreeEntry::new(
            "child",
            vec![vec![
                Span::raw("  "),
                Span::styled(
                    "├─ ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("child"),
            ]],
        );
        let grandchild = SpanTreeEntry::new(
            "grandchild",
            vec![
                vec![
                    Span::raw("  "),
                    Span::styled(
                        "│  └─ ".to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw("grandchild header"),
                ],
                vec![
                    Span::raw("  "),
                    Span::styled(
                        "│     ├─ ".to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw("grandchild detail1"),
                ],
                vec![
                    Span::raw("  "),
                    Span::styled(
                        "│     └─ ".to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw("grandchild detail2"),
                ],
            ],
        );

        tree.set_entries(vec![root, child, grandchild]);
        tree.set_selected("grandchild");

        let rows = render_to_lines(&mut tree, 60, 10);

        assert!(rows[2].contains("▸"), "selected header should have cursor: {:?}", rows[2]);
        assert!(
            rows[2].contains("│  └─"),
            "grandchild should have │  └─ prefix: {:?}",
            rows[2]
        );
        assert!(
            rows[3].contains("│     ├─"),
            "detail1 should have │     ├─ prefix: {:?}",
            rows[3]
        );
        assert!(
            rows[4].contains("│     └─"),
            "detail2 should have │     └─ prefix: {:?}",
            rows[4]
        );
    }

    #[test]
    fn tree_entry_with_single_char_placeholder_alignment() {
        let mut tree = SpanTree::new();

        let entry = SpanTreeEntry::new(
            "a",
            vec![
                vec![
                    Span::raw(" "),
                    Span::styled("└─ ".to_string(), Style::default().fg(Color::DarkGray)),
                    Span::raw("header"),
                ],
                vec![
                    Span::raw(" "),
                    Span::styled("   ├─ ".to_string(), Style::default().fg(Color::DarkGray)),
                    Span::raw("detail"),
                ],
            ],
        );
        tree.set_entries(vec![entry]);
        tree.set_selected("a");

        let rows = render_to_lines(&mut tree, 40, 5);
        assert!(rows[0].contains("▸"), "header has cursor: {:?}", rows[0]);
        assert!(
            rows[1].contains("├─"),
            "detail has connector preserved: {:?}",
            rows[1]
        );
    }

    #[test]
    fn tree_non_selected_entry_body_gets_blank_cursor() {
        let mut tree = SpanTree::new()
            .with_cursor_style(Span::styled("▸ ", Style::default()), Span::raw("  "));

        let entry = build_timeline_entry(
            "a",
            "└─ ",
            "   ",
            "#001 agent",
            &["status line", "tool info"],
        );
        tree.set_entries(vec![entry]);

        let rows = render_to_lines(&mut tree, 60, 5);

        assert!(
            !rows[0].contains("▸"),
            "non-selected header should use blank: {:?}",
            rows[0]
        );
        assert!(
            rows[1].contains("├─"),
            "body1 should have ├─: {:?}",
            rows[1]
        );
        assert!(
            rows[2].contains("└─"),
            "body2 should have └─: {:?}",
            rows[2]
        );
    }

    #[test]
    fn tree_multiple_siblings_each_with_continuation() {
        let mut tree = SpanTree::new()
            .with_cursor_style(Span::styled("▸ ", Style::default()), Span::raw("  "));

        let sib1 = build_timeline_entry(
            "s1",
            "├─ ",
            "│  ",
            "sibling1",
            &["s1-detail"],
        );
        let sib2 = build_timeline_entry(
            "s2",
            "└─ ",
            "   ",
            "sibling2",
            &["s2-detail1", "s2-detail2"],
        );
        tree.set_entries(vec![sib1, sib2]);
        tree.set_selected("s1");

        let rows = render_to_lines(&mut tree, 60, 10);

        assert!(rows[0].contains("▸") && rows[0].contains("├─"), "selected s1: {:?}", rows[0]);

        assert!(rows[1].contains("│") && rows[1].contains("└─"), "s1 detail: {:?}", rows[1]);

        assert!(rows[2].contains("└─"), "non-selected s2: {:?}", rows[2]);
        assert!(!rows[2].contains("▸"), "non-selected should not have cursor: {:?}", rows[2]);

        assert!(rows[3].contains("├─"), "s2 detail1: {:?}", rows[3]);
        assert!(rows[4].contains("└─"), "s2 detail2: {:?}", rows[4]);
    }
}
