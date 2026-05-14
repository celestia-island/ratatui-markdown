mod input;
mod render;

use ratatui::text::Line;

use crate::theme::RichTextTheme;

#[derive(Debug, Clone)]
pub struct FocusableItemRange {
    pub start_line: usize,
    pub end_line: usize,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct FocusableRegion {
    pub items: Vec<FocusableItemRange>,
}

impl FocusableRegion {
    pub(super) fn item_center(&self, item_idx: usize) -> usize {
        let item = &self.items[item_idx];
        (item.start_line + item.end_line.saturating_sub(1)) / 2
    }
}

pub struct HybridScrollView {
    pub(super) lines: Vec<Line<'static>>,
    pub(super) regions: Vec<FocusableRegion>,

    pub(super) scroll_offset: usize,
    pub(super) viewport_height: usize,

    pub(super) engaged_region: Option<usize>,
    pub(super) cursor_item: usize,

    pub(super) last_center: usize,

    pub(super) left_padding: bool,
    pub(super) show_cursor_indicator: bool,
}

impl Default for HybridScrollView {
    fn default() -> Self {
        Self::new()
    }
}

impl HybridScrollView {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            regions: Vec::new(),
            scroll_offset: 0,
            viewport_height: 10,
            engaged_region: None,
            cursor_item: 0,
            last_center: 0,
            left_padding: false,
            show_cursor_indicator: false,
        }
    }

    pub fn with_left_padding(mut self, padding: bool) -> Self {
        self.left_padding = padding;
        self
    }

    pub fn with_cursor_indicator(mut self, show: bool) -> Self {
        self.show_cursor_indicator = show;
        self
    }

    pub fn has_left_padding(&self) -> bool {
        self.left_padding
    }

    pub fn effective_padding(&self) -> usize {
        if self.show_cursor_indicator {
            2
        } else if self.left_padding {
            1
        } else {
            0
        }
    }

    pub fn set_content(&mut self, lines: Vec<Line<'static>>, regions: Vec<FocusableRegion>) {
        self.lines = lines;
        self.regions = regions;
        self.scroll_offset = 0;
        self.engaged_region = None;
        self.cursor_item = 0;
        self.last_center = self.viewport_center();
    }

    pub fn set_lines(&mut self, lines: Vec<Line<'static>>) {
        self.set_content(lines, Vec::new());
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.regions.clear();
        self.scroll_offset = 0;
        self.engaged_region = None;
        self.cursor_item = 0;
        self.last_center = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn get_scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn get_viewport_height(&self) -> usize {
        self.viewport_height
    }

    pub fn is_engaged(&self) -> bool {
        self.engaged_region.is_some()
    }

    pub fn engaged_cursor(&self) -> Option<(usize, usize)> {
        self.engaged_region.map(|r| (r, self.cursor_item))
    }

    pub fn selected_item_id(&self) -> Option<&str> {
        self.engaged_region.and_then(|r| {
            self.regions[r]
                .items
                .get(self.cursor_item)
                .map(|item| item.id.as_str())
        })
    }

    pub(super) fn max_offset(&self) -> usize {
        self.lines.len().saturating_sub(self.viewport_height)
    }

    pub(super) fn viewport_center(&self) -> usize {
        self.scroll_offset + self.viewport_height / 2
    }

    pub(super) fn center_on_item(&mut self, region_idx: usize, item_idx: usize) {
        let item_center = self.regions[region_idx].item_center(item_idx);
        let target = item_center.saturating_sub(self.viewport_height / 2);
        self.scroll_offset = target.min(self.max_offset());
        self.last_center = self.viewport_center();
    }

    pub fn render(
        &mut self,
        f: &mut ratatui::Frame,
        inner_area: ratatui::layout::Rect,
        outer_area: ratatui::layout::Rect,
        theme: &impl RichTextTheme,
    ) {
        render::render(self, f, inner_area, outer_area, theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lines(count: usize) -> Vec<Line<'static>> {
        (0..count)
            .map(|i| Line::raw(format!("line {}", i)))
            .collect()
    }

    fn make_region(items: Vec<(usize, usize, &str)>) -> FocusableRegion {
        FocusableRegion {
            items: items
                .into_iter()
                .map(|(s, e, id)| FocusableItemRange {
                    start_line: s,
                    end_line: e,
                    id: id.to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn free_scroll_no_regions() {
        let mut view = HybridScrollView::new();
        view.set_lines(make_lines(100));
        view.viewport_height = 20;

        assert_eq!(view.get_scroll_offset(), 0);
        assert!(!view.is_engaged());

        view.scroll_down();
        assert_eq!(view.get_scroll_offset(), 1);
        assert!(!view.is_engaged());

        for _ in 0..200 {
            view.scroll_down();
        }
        assert_eq!(view.get_scroll_offset(), 80);
    }

    #[test]
    fn free_scroll_up() {
        let mut view = HybridScrollView::new();
        view.set_lines(make_lines(100));
        view.viewport_height = 20;

        view.scroll_to_bottom();
        assert_eq!(view.get_scroll_offset(), 80);

        view.scroll_up();
        assert_eq!(view.get_scroll_offset(), 79);

        for _ in 0..200 {
            view.scroll_up();
        }
        assert_eq!(view.get_scroll_offset(), 0);
    }

    #[test]
    fn engage_on_scroll_down() {
        let mut view = HybridScrollView::new();
        let region = make_region(vec![(30, 31, "a"), (31, 32, "b"), (32, 33, "c")]);
        view.set_content(make_lines(100), vec![region]);
        view.viewport_height = 20;

        let mut engaged = false;
        for _ in 0..50 {
            view.scroll_down();
            if view.is_engaged() {
                engaged = true;
                break;
            }
        }
        assert!(engaged);
        assert_eq!(view.selected_item_id(), Some("a"));
    }

    #[test]
    fn navigate_through_region_down() {
        let mut view = HybridScrollView::new();
        let region = make_region(vec![(30, 31, "a"), (31, 32, "b"), (32, 33, "c")]);
        view.set_content(make_lines(100), vec![region]);
        view.viewport_height = 20;

        view.engaged_region = Some(0);
        view.cursor_item = 0;
        view.center_on_item(0, 0);

        view.scroll_down();
        assert!(view.is_engaged());
        assert_eq!(view.selected_item_id(), Some("b"));

        view.scroll_down();
        assert!(view.is_engaged());
        assert_eq!(view.selected_item_id(), Some("c"));

        view.scroll_down();
        assert!(view.is_engaged());
    }

    #[test]
    fn engage_on_scroll_up() {
        let mut view = HybridScrollView::new();
        let region = make_region(vec![(30, 31, "a"), (31, 32, "b"), (32, 33, "c")]);
        view.set_content(make_lines(100), vec![region]);
        view.viewport_height = 20;

        view.scroll_to_bottom();

        let mut engaged = false;
        for _ in 0..100 {
            view.scroll_up();
            if view.is_engaged() {
                engaged = true;
                break;
            }
        }
        assert!(engaged);
        assert_eq!(view.selected_item_id(), Some("c"));
    }

    #[test]
    fn navigate_through_region_up() {
        let mut view = HybridScrollView::new();
        let region = make_region(vec![(30, 31, "a"), (31, 32, "b"), (32, 33, "c")]);
        view.set_content(make_lines(100), vec![region]);
        view.viewport_height = 20;

        view.engaged_region = Some(0);
        view.cursor_item = 2;
        view.center_on_item(0, 2);

        view.scroll_up();
        assert!(view.is_engaged());
        assert_eq!(view.selected_item_id(), Some("b"));

        view.scroll_up();
        assert!(view.is_engaged());
        assert_eq!(view.selected_item_id(), Some("a"));

        view.scroll_up();
        assert!(view.is_engaged());
    }

    #[test]
    fn scroll_to_top_disengages() {
        let mut view = HybridScrollView::new();
        let region = make_region(vec![(30, 31, "a")]);
        view.set_content(make_lines(100), vec![region]);
        view.viewport_height = 20;

        view.engaged_region = Some(0);
        view.cursor_item = 0;

        view.scroll_to_top();
        assert!(!view.is_engaged());
        assert_eq!(view.get_scroll_offset(), 0);
    }

    #[test]
    fn scroll_to_bottom_disengages() {
        let mut view = HybridScrollView::new();
        let region = make_region(vec![(30, 31, "a")]);
        view.set_content(make_lines(100), vec![region]);
        view.viewport_height = 20;

        view.engaged_region = Some(0);
        view.cursor_item = 0;

        view.scroll_to_bottom();
        assert!(!view.is_engaged());
        assert_eq!(view.get_scroll_offset(), 80);
    }

    #[test]
    fn empty_content() {
        let mut view = HybridScrollView::new();
        view.scroll_down();
        view.scroll_up();
        assert_eq!(view.get_scroll_offset(), 0);
    }

    #[test]
    fn multi_line_items() {
        let mut view = HybridScrollView::new();
        let region = make_region(vec![(20, 23, "x"), (23, 26, "y"), (26, 29, "z")]);
        view.set_content(make_lines(60), vec![region]);
        view.viewport_height = 20;

        for _ in 0..50 {
            view.scroll_down();
            if view.is_engaged() {
                break;
            }
        }
        assert!(view.is_engaged());
        assert_eq!(view.selected_item_id(), Some("x"));

        view.scroll_down();
        assert_eq!(view.selected_item_id(), Some("y"));

        view.scroll_down();
        assert_eq!(view.selected_item_id(), Some("z"));

        view.scroll_down();
        assert!(view.is_engaged());
    }

    #[test]
    fn multiple_regions() {
        let mut view = HybridScrollView::new();
        let r1 = make_region(vec![(20, 21, "r1a"), (21, 22, "r1b")]);
        let r2 = make_region(vec![(60, 61, "r2a"), (61, 62, "r2b")]);
        view.set_content(make_lines(100), vec![r1, r2]);
        view.viewport_height = 10;

        for _ in 0..100 {
            view.scroll_down();
            if view.is_engaged() {
                break;
            }
        }
        assert!(view.is_engaged());
        assert_eq!(view.selected_item_id(), Some("r1a"));

        view.scroll_down();
        view.scroll_down();

        assert!(!view.is_engaged());

        for _ in 0..100 {
            view.scroll_down();
            if view.is_engaged() {
                break;
            }
        }
        assert!(view.is_engaged());
        assert_eq!(view.selected_item_id(), Some("r2a"));
    }

    #[test]
    fn page_operations() {
        let mut view = HybridScrollView::new();
        view.set_lines(make_lines(100));
        view.viewport_height = 20;

        view.page_down(10);
        assert_eq!(view.get_scroll_offset(), 10);

        view.page_up(5);
        assert_eq!(view.get_scroll_offset(), 5);
    }
}
