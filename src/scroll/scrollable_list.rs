use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::{
    scroll::{anchored_panel_scrollbar_area, render_arrow_scrollbar},
    theme::RichTextTheme,
};

pub struct RenderParams<'a, 'f, R: RichTextTheme> {
    pub frame: &'a mut Frame<'f>,
    pub panel_area: Rect,
    pub inner_area: Rect,
    pub theme: &'a R,
    pub is_focused: bool,
    pub empty_text: &'a str,
}

pub trait ListItemRenderer {
    fn is_separator(&self) -> bool {
        false
    }

    fn render_line(&self, theme: &impl RichTextTheme, is_selected: bool) -> Line<'_>;
}

#[derive(Debug)]
pub struct ScrollableList<T: ListItemRenderer> {
    items: Vec<T>,
    pub selected_index: usize,
    scroll_offset: usize,
    title: Option<String>,
}

impl<T: ListItemRenderer> Default for ScrollableList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ListItemRenderer> ScrollableList<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            title: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn with_items(mut self, items: Vec<T>) -> Self {
        self.set_items(items);
        self
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn total_lines(&self) -> usize {
        self.items.len()
    }

    fn selectable_count(&self) -> usize {
        self.items.iter().filter(|i| !i.is_separator()).count()
    }

    fn selectable_index_to_item_index(&self, selectable_idx: usize) -> usize {
        let mut count = 0;
        for (idx, item) in self.items.iter().enumerate() {
            if !item.is_separator() {
                if count == selectable_idx {
                    return idx;
                }
                count += 1;
            }
        }
        0
    }

    fn ensure_visible(&mut self, visible_height: usize) {
        if visible_height == 0 || self.items.is_empty() {
            return;
        }

        let total_lines = self.total_lines();
        let max_offset = total_lines.saturating_sub(visible_height);

        let selectable_count = self.selectable_count();
        if selectable_count > 0 {
            self.selected_index = self.selected_index.min(selectable_count - 1);
        } else {
            self.selected_index = 0;
        }

        let item_idx = self.selectable_index_to_item_index(self.selected_index);

        let margin = visible_height / 3;
        if item_idx < self.scroll_offset + margin {
            self.scroll_offset = item_idx.saturating_sub(margin);
        } else if item_idx >= self.scroll_offset + visible_height - margin {
            self.scroll_offset = (item_idx + margin)
                .saturating_sub(visible_height)
                .min(max_offset);
        }

        self.scroll_offset = self.scroll_offset.min(max_offset);
    }

    pub fn navigate_up(&mut self) {
        let selectable_count = self.selectable_count();
        if selectable_count > 0 && self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        let selectable_count = self.selectable_count();
        if selectable_count > 0 {
            self.selected_index = (self.selected_index + 1).min(selectable_count - 1);
        }
    }

    pub fn navigate_to_top(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn navigate_to_bottom(&mut self) {
        let selectable_count = self.selectable_count();
        if selectable_count > 0 {
            self.selected_index = selectable_count - 1;
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        let selectable_count = self.selectable_count();
        if selectable_count > 0 {
            self.selected_index = self.selected_index.saturating_sub(page_size);
        }
    }

    pub fn page_down(&mut self, page_size: usize) {
        let selectable_count = self.selectable_count();
        if selectable_count > 0 {
            self.selected_index = (self.selected_index + page_size).min(selectable_count - 1);
        }
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
        self.selected_index = 0;
    }

    pub fn selected_item(&self) -> Option<&T> {
        if self.items.is_empty() {
            return None;
        }
        let item_idx = self.selectable_index_to_item_index(self.selected_index);
        self.items.get(item_idx)
    }

    pub fn render_bordered(
        &mut self,
        f: &mut Frame,
        area: Rect,
        theme: &impl RichTextTheme,
        is_focused: bool,
        empty_text: &str,
    ) {
        let border_color = if is_focused {
            theme.get_focused_border_color()
        } else {
            theme.get_border_color()
        };

        let block = if let Some(title) = &self.title {
            Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
        };

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        self.render_inner(f, area, inner_area, theme, is_focused, empty_text);
    }

    pub fn render_inner(
        &mut self,
        f: &mut Frame,
        panel_area: Rect,
        inner_area: Rect,
        theme: &impl RichTextTheme,
        is_focused: bool,
        empty_text: &str,
    ) {
        let visible_height = inner_area.height as usize;

        if self.items.is_empty() {
            let placeholder = ratatui::widgets::Paragraph::new(Line::from(Span::styled(
                format!("     {}", empty_text),
                Style::default()
                    .fg(theme.get_muted_text_color())
                    .add_modifier(Modifier::ITALIC),
            )))
            .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(placeholder, inner_area);
            return;
        }

        self.ensure_visible(visible_height);

        let total_lines = self.total_lines();
        let start_idx = self.scroll_offset;
        let end_idx = (start_idx + visible_height).min(total_lines);

        let mut list_items: Vec<ListItem> = Vec::new();
        let mut selectable_count = 0;

        for (idx, item) in self.items.iter().enumerate() {
            if idx < start_idx {
                if !item.is_separator() {
                    selectable_count += 1;
                }
                continue;
            }
            if idx >= end_idx {
                break;
            }

            let is_selectable = !item.is_separator();
            let is_selected =
                is_focused && is_selectable && selectable_count == self.selected_index;

            let line = item.render_line(theme, is_selected);
            list_items.push(ListItem::new(line));

            if is_selectable {
                selectable_count += 1;
            }
        }

        let list = List::new(list_items);
        f.render_widget(list, inner_area);

        if total_lines > visible_height {
            let scrollbar_area = anchored_panel_scrollbar_area(panel_area, inner_area);
            render_arrow_scrollbar(
                f,
                scrollbar_area,
                total_lines,
                visible_height,
                self.scroll_offset,
                theme,
            );
        }
    }

    pub fn render_with<'a, 'f, F, R>(&mut self, params: RenderParams<'a, 'f, R>, renderer: F)
    where
        R: RichTextTheme,
        F: Fn(&T, &R, bool) -> Line<'a>,
    {
        let visible_height = params.inner_area.height as usize;

        if self.items.is_empty() {
            let placeholder = ratatui::widgets::Paragraph::new(Line::from(Span::styled(
                format!("     {}", params.empty_text),
                Style::default()
                    .fg(params.theme.get_muted_text_color())
                    .add_modifier(Modifier::ITALIC),
            )))
            .alignment(ratatui::layout::Alignment::Center);
            params.frame.render_widget(placeholder, params.inner_area);
            return;
        }

        self.ensure_visible(visible_height);

        let total_lines = self.total_lines();
        let start_idx = self.scroll_offset;
        let end_idx = (start_idx + visible_height).min(total_lines);

        let mut list_items: Vec<ListItem> = Vec::new();
        let mut selectable_count = 0;

        for (idx, item) in self.items.iter().enumerate() {
            if idx < start_idx {
                if !item.is_separator() {
                    selectable_count += 1;
                }
                continue;
            }
            if idx >= end_idx {
                break;
            }

            let is_selectable = !item.is_separator();
            let is_selected =
                params.is_focused && is_selectable && selectable_count == self.selected_index;

            let line = renderer(item, params.theme, is_selected);
            list_items.push(ListItem::new(line));

            if is_selectable {
                selectable_count += 1;
            }
        }

        let list = List::new(list_items);
        params.frame.render_widget(list, params.inner_area);

        if total_lines > visible_height {
            let scrollbar_area =
                anchored_panel_scrollbar_area(params.panel_area, params.inner_area);
            render_arrow_scrollbar(
                params.frame,
                scrollbar_area,
                total_lines,
                visible_height,
                self.scroll_offset,
                params.theme,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestItem {
        name: String,
        is_sep: bool,
    }

    impl ListItemRenderer for TestItem {
        fn is_separator(&self) -> bool {
            self.is_sep
        }

        fn render_line(&self, _theme: &impl RichTextTheme, is_selected: bool) -> Line<'_> {
            if is_selected {
                Line::raw(format!("> {}", self.name))
            } else {
                Line::raw(format!("  {}", self.name))
            }
        }
    }

    #[test]
    fn test_selectable_count() {
        let mut list = ScrollableList::new();
        list.set_items(vec![
            TestItem {
                name: "Item 1".to_string(),
                is_sep: false,
            },
            TestItem {
                name: "Separator".to_string(),
                is_sep: true,
            },
            TestItem {
                name: "Item 2".to_string(),
                is_sep: false,
            },
        ]);

        assert_eq!(list.selectable_count(), 2);
        assert_eq!(list.total_lines(), 3);
    }

    #[test]
    fn test_navigation() {
        let mut list = ScrollableList::new();
        list.set_items(vec![
            TestItem {
                name: "Item 1".to_string(),
                is_sep: false,
            },
            TestItem {
                name: "Item 2".to_string(),
                is_sep: false,
            },
            TestItem {
                name: "Item 3".to_string(),
                is_sep: false,
            },
        ]);

        assert_eq!(list.selected_index, 0);

        list.navigate_down();
        assert_eq!(list.selected_index, 1);

        list.navigate_up();
        assert_eq!(list.selected_index, 0);

        list.navigate_to_bottom();
        assert_eq!(list.selected_index, 2);
    }
}
