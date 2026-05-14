use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::{
    constants::list_prefix::{ARROW_DOWN, ARROW_UP, BLOCK_FULL, VLINE},
    theme::RichTextTheme,
};

pub struct ArrowScrollbar {
    content_length: usize,
    viewport_length: usize,
    position: usize,
}

impl ArrowScrollbar {
    pub fn new(content_length: usize, viewport_length: usize) -> Self {
        Self {
            content_length,
            viewport_length,
            position: 0,
        }
    }

    pub fn position(mut self, position: usize) -> Self {
        self.position = position;
        self
    }

    pub fn render(self, f: &mut Frame, area: ratatui::layout::Rect, theme: &impl RichTextTheme) {
        if self.content_length == 0 || self.viewport_length == 0 {
            return;
        }

        if self.content_length <= self.viewport_length {
            return;
        }

        let max_offset = self.content_length.saturating_sub(self.viewport_length);
        let max_scrollbar_pos = self.content_length.saturating_sub(1);
        let remapped_position = if max_offset > 0 && max_scrollbar_pos > 0 {
            (self.position as u64 * max_scrollbar_pos as u64 / max_offset as u64) as usize
        } else {
            0
        };

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some(ARROW_UP))
            .end_symbol(Some(ARROW_DOWN))
            .track_symbol(Some(VLINE))
            .thumb_symbol(BLOCK_FULL)
            .begin_style(Style::default().fg(theme.get_muted_text_color()))
            .end_style(Style::default().fg(theme.get_muted_text_color()))
            .track_style(Style::default().fg(theme.get_muted_text_color()))
            .thumb_style(Style::default().fg(theme.get_text_color()));

        let mut state = ScrollbarState::default()
            .content_length(self.content_length)
            .viewport_content_length(self.viewport_length)
            .position(remapped_position);

        f.render_stateful_widget(scrollbar, area, &mut state);
    }
}

pub fn render_arrow_scrollbar(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    content_length: usize,
    viewport_length: usize,
    position: usize,
    theme: &impl RichTextTheme,
) {
    ArrowScrollbar::new(content_length, viewport_length)
        .position(position)
        .render(f, area, theme);
}

pub fn anchored_panel_scrollbar_area(panel_area: Rect, content_area: Rect) -> Rect {
    Rect {
        x: panel_area.x + panel_area.width.saturating_sub(2),
        y: content_area.y,
        width: 2,
        height: content_area.height,
    }
}

pub fn border_scrollbar_area(panel_area: Rect, content_area: Rect) -> Rect {
    Rect {
        x: panel_area.x + panel_area.width.saturating_sub(1),
        y: content_area.y,
        width: 1,
        height: content_area.height,
    }
}
