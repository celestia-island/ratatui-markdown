use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::{CursorLineMode, SpanTree};
use crate::{scroll::render_arrow_scrollbar, theme::RichTextTheme};

fn width_preserving_replacement(original: &Span<'_>, replacement: Span<'static>) -> Span<'static> {
    let original_w = original.width();
    let replacement_w = replacement.width();
    if replacement_w < original_w {
        let padding = " ".repeat(original_w - replacement_w);
        Span::styled(
            format!("{}{}", replacement.content, padding),
            replacement.style,
        )
    } else {
        replacement
    }
}

fn apply_cursor(
    spans: &mut Vec<Span<'static>>,
    tree: &SpanTree,
    is_selected: bool,
    line_idx: usize,
    highlight_bg: ratatui::style::Color,
) {
    let col = tree.cursor_column;
    if col >= spans.len() {
        return;
    }

    let is_cursor_line = match tree.cursor_line_mode {
        CursorLineMode::HeaderOnly => line_idx == 0,
        CursorLineMode::AllLines => true,
    };

    let original = spans[col].clone();

    if is_selected {
        if is_cursor_line {
            spans[col] = width_preserving_replacement(&original, tree.cursor_span.clone());
        }
        for span in spans {
            span.style = span.style.bg(highlight_bg);
        }
    } else {
        spans[col] = width_preserving_replacement(&original, tree.blank_cursor_span.clone());
    }
}

pub(super) fn render(
    tree: &mut SpanTree,
    f: &mut Frame,
    inner_area: Rect,
    _outer_area: Rect,
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
                apply_cursor(&mut spans, tree, is_selected, line_idx, highlight_bg);
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

    let total = tree.total_lines();
    let need_scrollbar = total > visible_height;
    let content_area = if need_scrollbar && inner_area.width > 1 {
        Rect::new(
            inner_area.x,
            inner_area.y,
            inner_area.width.saturating_sub(1),
            inner_area.height,
        )
    } else {
        inner_area
    };

    let paragraph = Paragraph::new(visible_lines);
    f.render_widget(paragraph, content_area);

    if need_scrollbar {
        let scrollbar_area = Rect::new(
            inner_area.x + inner_area.width.saturating_sub(1),
            inner_area.y,
            1,
            inner_area.height,
        );
        render_arrow_scrollbar(
            f,
            scrollbar_area,
            total,
            visible_height,
            tree.scroll_offset,
            theme,
        );
    }
}
