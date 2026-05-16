use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::{CursorLineMode, SpanTree};
use crate::{scroll::render_arrow_scrollbar, theme::RichTextTheme};

pub(super) fn render(
    tree: &mut SpanTree,
    f: &mut Frame,
    inner_area: Rect,
    outer_area: Rect,
    theme: &impl RichTextTheme,
) {
    let visible_height = inner_area.height as usize;
    tree.viewport_height = visible_height.max(1);

    tree.clamp_scroll_offset();

    if tree.entries.is_empty() {
        return;
    }

    let highlight_bg = theme.get_popup_selected_background();
    let selected_id = tree.selected_id.as_deref();

    let mut visible_lines: Vec<Line<'static>> = Vec::new();
    let mut global_line = 0usize;
    let start = tree.scroll_offset;
    let end = start + visible_height;

    for entry in &tree.entries {
        let is_selected = selected_id == Some(entry.id.as_str());

        for (line_idx, entry_spans) in entry.lines.iter().enumerate() {
            if global_line >= end {
                break;
            }
            if global_line >= start {
                let mut spans: Vec<Span<'static>> = entry_spans.clone();
                let should_render_cursor = tree.cursor_column < spans.len()
                    && match tree.cursor_line_mode {
                        CursorLineMode::HeaderOnly => line_idx == 0,
                        CursorLineMode::AllLines => true,
                    };

                if is_selected {
                    if should_render_cursor {
                        spans[tree.cursor_column] = tree.cursor_span.clone();
                    }
                    for span in &mut spans {
                        span.style = span.style.bg(highlight_bg);
                    }
                } else if should_render_cursor {
                    spans[tree.cursor_column] = tree.blank_cursor_span.clone();
                }

                visible_lines.push(Line::from(spans));
            }
            global_line += 1;
        }

        if entry.lines.is_empty() {
            if global_line >= start && global_line < end {
                let mut spans: Vec<Span<'static>> = Vec::new();
                if is_selected {
                    spans.push(tree.cursor_span.clone());
                    for span in &mut spans {
                        span.style = span.style.bg(highlight_bg);
                    }
                } else {
                    spans.push(tree.blank_cursor_span.clone());
                }
                visible_lines.push(Line::from(spans));
            }
            global_line += 1;
        }

        if global_line >= end {
            break;
        }
    }

    let paragraph = Paragraph::new(visible_lines);
    f.render_widget(paragraph, inner_area);

    let total = tree.total_lines();
    if total > visible_height {
        render_arrow_scrollbar(
            f,
            outer_area,
            total,
            visible_height,
            tree.scroll_offset,
            theme,
        );
    }
}
