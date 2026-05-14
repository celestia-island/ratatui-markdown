use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::HybridScrollView;
use crate::{scroll::render_arrow_scrollbar, theme::RichTextTheme};

pub(super) fn render(
    view: &mut HybridScrollView,
    f: &mut Frame,
    inner_area: Rect,
    outer_area: Rect,
    theme: &impl RichTextTheme,
) {
    let visible_height = inner_area.height as usize;
    view.viewport_height = visible_height.max(1);

    if view.scroll_offset > view.max_offset() {
        view.scroll_offset = view.max_offset();
    }

    let total = view.lines.len();
    if total == 0 {
        return;
    }

    let start = view.scroll_offset;
    let end = (start + visible_height).min(total);

    let highlight_bg = theme.get_popup_selected_background();

    let visible_lines: Vec<Line> = (start..end)
        .map(|line_idx| {
            let mut line = view.lines[line_idx].clone();

            if let Some(region_idx) = view.engaged_region {
                let item = &view.regions[region_idx].items[view.cursor_item];
                if line_idx >= item.start_line && line_idx < item.end_line {
                    for span in &mut line.spans {
                        span.style = span.style.bg(highlight_bg).add_modifier(Modifier::BOLD);
                    }
                }
            }

            if view.show_cursor_indicator {
                let is_engaged_line = view.engaged_region.is_some_and(|region_idx| {
                    let item = &view.regions[region_idx].items[view.cursor_item];
                    line_idx >= item.start_line && line_idx < item.end_line
                });
                let prefix_str = if is_engaged_line { "> " } else { "  " };
                line.spans.insert(
                    0,
                    Span::styled(prefix_str, Style::default().fg(theme.get_primary_color())),
                );
            } else if view.left_padding {
                line.spans.insert(0, Span::raw(" "));
            }

            line
        })
        .collect();

    let paragraph = Paragraph::new(visible_lines);
    f.render_widget(paragraph, inner_area);

    if total > visible_height {
        render_arrow_scrollbar(
            f,
            outer_area,
            total,
            visible_height,
            view.scroll_offset,
            theme,
        );
    }
}
