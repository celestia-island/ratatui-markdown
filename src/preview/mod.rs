use std::boxed::Box;

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    Frame,
};

use crate::{
    markdown::{MarkdownRenderer, RenderHooks},
    scroll::{FocusableItemRange, FocusableRegion, HybridScrollView},
    theme::RichTextTheme,
    tree::CollapsibleTree,
};

const FRONTMATTER_DELIMITER: &str = "+++";

pub struct ActionItem {
    pub id: String,
    pub label: String,
}

pub struct MarkdownPreview {
    content: String,
    scroll_view: HybridScrollView,
    cached_width: usize,
    cached_generation: crate::theme::Generation,
    strip_frontmatter: bool,
    tree: Option<CollapsibleTree>,
    tree_dirty: bool,
    action_items: Vec<ActionItem>,
    actions_dirty: bool,
    hooks: Option<Box<dyn RenderHooks>>,
}

impl Default for MarkdownPreview {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownPreview {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            scroll_view: HybridScrollView::new()
                .with_left_padding(false)
                .with_cursor_indicator(true),
            cached_width: 0,
            cached_generation: crate::theme::Generation::default(),
            strip_frontmatter: true,
            tree: None,
            tree_dirty: false,
            action_items: Vec::new(),
            actions_dirty: false,
            hooks: None,
        }
    }

    pub fn with_strip_frontmatter(mut self, strip: bool) -> Self {
        self.strip_frontmatter = strip;
        self
    }

    pub fn with_render_hooks(mut self, hooks: Box<dyn RenderHooks>) -> Self {
        self.hooks = Some(hooks);
        self
    }

    pub fn with_left_padding(mut self, padding: bool) -> Self {
        self.scroll_view = self.scroll_view.with_left_padding(padding);
        self
    }

    pub fn set_tree(&mut self, tree: Option<CollapsibleTree>) {
        self.tree = tree;
        self.tree_dirty = true;
        self.scroll_view.clear();
    }

    pub fn tree_mut(&mut self) -> Option<&mut CollapsibleTree> {
        self.tree.as_mut()
    }

    pub fn has_tree(&self) -> bool {
        self.tree.is_some()
    }

    pub fn set_action_items(&mut self, items: Vec<ActionItem>) {
        self.action_items = items;
        self.actions_dirty = true;
        self.scroll_view.clear();
    }

    pub fn selected_action_id(&self) -> Option<&str> {
        let id = self.scroll_view.selected_item_id()?;
        if let Some(action_id) = id.strip_prefix("action:") {
            Some(action_id)
        } else {
            None
        }
    }

    pub fn set_content(&mut self, content: &str) {
        if self.content != content {
            self.content = content.to_string();
            self.scroll_view.clear();
            self.cached_width = 0;
        }
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.tree = None;
        self.tree_dirty = false;
        self.action_items.clear();
        self.actions_dirty = false;
        self.scroll_view.clear();
        self.cached_width = 0;
        self.cached_generation = crate::theme::Generation::default();
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty() && self.tree.is_none() && self.action_items.is_empty()
    }

    pub fn scroll_up(&mut self) {
        self.scroll_view.scroll_up();
    }

    pub fn scroll_down(&mut self) {
        self.scroll_view.scroll_down();
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_view.scroll_to_top();
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_view.scroll_to_bottom();
    }

    pub fn page_up(&mut self, lines: usize) {
        self.scroll_view.page_up(lines);
    }

    pub fn page_down(&mut self, lines: usize) {
        self.scroll_view.page_down(lines);
    }

    pub fn total_lines(&self) -> usize {
        self.scroll_view.total_lines()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_view.get_scroll_offset()
    }

    pub fn visible_height(&self) -> usize {
        self.scroll_view.get_viewport_height()
    }

    pub fn is_engaged(&self) -> bool {
        self.scroll_view.is_engaged()
    }

    pub fn engaged_cursor(&self) -> Option<(usize, usize)> {
        self.scroll_view.engaged_cursor()
    }

    pub fn selected_item_id(&self) -> Option<&str> {
        self.scroll_view.selected_item_id()
    }

    pub fn toggle_tree_node(&mut self) -> bool {
        if let Some(id) = self.scroll_view.selected_item_id().map(|s| s.to_string()) {
            if let Some(tree) = &mut self.tree {
                if tree.handle_toggle(&id) {
                    self.tree_dirty = true;
                    return true;
                }
            }
        }
        false
    }

    pub fn render(
        &mut self,
        f: &mut Frame,
        inner_area: Rect,
        outer_area: Rect,
        theme: &impl RichTextTheme,
    ) {
        let render_width = self.content_width(inner_area.width);
        let generation = theme.generation();

        if (self.scroll_view.is_empty() && !self.content.is_empty())
            || self.cached_width != render_width
            || self.cached_generation != generation
            || self.tree_dirty
            || self.actions_dirty
        {
            self.rebuild_lines(render_width, theme);
            self.tree_dirty = false;
            self.actions_dirty = false;
            self.cached_generation = generation;
        }

        self.scroll_view.render(f, inner_area, outer_area, theme);
    }

    fn rebuild_lines(&mut self, width: usize, theme: &impl RichTextTheme) {
        let saved_id = self.scroll_view.selected_item_id().map(|s| s.to_string());
        let saved_offset = self.scroll_view.get_scroll_offset();

        let mut all_lines: Vec<Line<'static>> = Vec::new();
        let mut regions: Vec<FocusableRegion> = Vec::new();

        if let Some(tree) = &self.tree {
            let tree_lines = tree.render_lines(width, theme);
            let tree_items = tree.build_focusable_items();
            all_lines.extend(tree_lines);
            if !tree_items.is_empty() {
                regions.push(FocusableRegion { items: tree_items });
            }
        }

        let content = if self.strip_frontmatter {
            Self::strip_toml_frontmatter(&self.content)
        } else {
            self.content.clone()
        };

        if !content.trim().is_empty() {
            let mut renderer = MarkdownRenderer::new(width);
            if let Some(hooks) = self.hooks.take() {
                renderer = renderer.with_render_hooks(hooks);
            }
            let blocks = renderer.parse(&content);
            let md_lines = renderer.render(&blocks, theme);
            self.hooks = renderer.hooks.take();
            all_lines.extend(md_lines);
        }

        if !self.action_items.is_empty() {
            if !all_lines.is_empty() {
                all_lines.push(Line::raw(""));
            }
            let mut action_ranges: Vec<FocusableItemRange> = Vec::new();
            let bracket_color = theme.get_primary_color();
            let text_color = theme.get_text_color();
            for item in &self.action_items {
                let line_idx = all_lines.len();
                all_lines.push(Line::from(vec![
                    Span::styled("[", Style::default().fg(bracket_color)),
                    Span::styled(
                        item.label.clone(),
                        Style::default().fg(text_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("]", Style::default().fg(bracket_color)),
                ]));
                action_ranges.push(FocusableItemRange {
                    start_line: line_idx,
                    end_line: line_idx + 1,
                    id: format!("action:{}", item.id),
                });
            }
            if !action_ranges.is_empty() {
                regions.push(FocusableRegion {
                    items: action_ranges,
                });
            }
        }

        self.cached_width = width;
        let has_focusable = regions.iter().any(|r| !r.items.is_empty());
        self.scroll_view.set_content(all_lines, regions);

        if let Some(ref id) = saved_id {
            let found = self.scroll_view.engage_by_id(id);
            if found {
                let max_off = self
                    .scroll_view
                    .total_lines()
                    .saturating_sub(self.scroll_view.get_viewport_height());
                self.scroll_view
                    .set_scroll_offset(saved_offset.min(max_off));
            } else {
                self.scroll_view.engage_first();
            }
        } else if has_focusable {
            self.scroll_view.engage_first();
        }
    }

    fn content_width(&self, inner_width: u16) -> usize {
        usize::from(inner_width).saturating_sub(self.scroll_view.effective_padding())
    }

    fn strip_toml_frontmatter(content: &str) -> String {
        let trimmed = content.trim_start_matches('\n');
        if !trimmed.starts_with(FRONTMATTER_DELIMITER) {
            return content.to_string();
        }

        let mut in_frontmatter = false;
        let mut content_lines = Vec::new();

        for line in trimmed.lines() {
            if line.trim() == FRONTMATTER_DELIMITER {
                if in_frontmatter {
                    in_frontmatter = false;
                    continue;
                } else {
                    in_frontmatter = true;
                    continue;
                }
            }

            if !in_frontmatter {
                content_lines.push(line);
            }
        }

        content_lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::MarkdownPreview;

    #[test]
    fn content_width_reserves_cursor_indicator_columns() {
        let preview = MarkdownPreview::new();
        assert_eq!(preview.content_width(10), 8);
    }

    #[test]
    fn content_width_never_underflows_when_padding_enabled() {
        let preview = MarkdownPreview::new();
        assert_eq!(preview.content_width(0), 0);
        assert_eq!(preview.content_width(1), 0);
    }
}
