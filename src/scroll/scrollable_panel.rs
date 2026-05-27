use ratatui::{layout::Rect, text::Line, widgets::Paragraph, Frame};

use crate::{
    scroll::scrollbar::{border_scrollbar_area, ArrowScrollbar},
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

    let need_scrollbar = total > viewport && viewport > 0;
    let content_area = if need_scrollbar && area.width > 1 {
        Rect::new(area.x, area.y, area.width.saturating_sub(1), area.height)
    } else {
        area
    };

    let visible: Vec<Line<'static>> = lines.into_iter().skip(start).take(viewport).collect();

    f.render_widget(Paragraph::new(visible), content_area);

    if need_scrollbar {
        let scrollbar_area = border_scrollbar_area(area, area);
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
