use ratatui::{layout::Rect, text::Line, widgets::Paragraph, Frame};

use crate::{
    scroll::{anchored_panel_scrollbar_area, ArrowScrollbar},
    theme::RichTextTheme,
};

pub struct ScrollableRenderResult {
    pub start: usize,
    pub total: usize,
    pub viewport: usize,
}

pub fn render_scrollable(
    f: &mut Frame,
    area: Rect,
    lines: Vec<Line<'static>>,
    scroll_offset: usize,
    theme: &impl RichTextTheme,
) -> ScrollableRenderResult {
    let total = lines.len();
    let viewport = area.height as usize;
    let max = total.saturating_sub(viewport);
    let start = scroll_offset.min(max);
    let visible: Vec<Line<'static>> = lines.into_iter().skip(start).take(viewport).collect();

    f.render_widget(Paragraph::new(visible), area);

    if total > viewport && viewport > 0 {
        let scrollbar_area = anchored_panel_scrollbar_area(area, area);
        ArrowScrollbar::new(total, viewport)
            .position(start)
            .render(f, scrollbar_area, theme);
    }

    ScrollableRenderResult {
        start,
        total,
        viewport,
    }
}
