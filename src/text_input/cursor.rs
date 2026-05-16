use std::borrow::Cow;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::types::{CursorShape, CursorStyle, Selection, SelectionStyle};
use crate::theme::RichTextTheme;

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_cursor_and_selection(
    line: &mut Line<'static>,
    cursor_col: usize,
    _horizontal_scroll: usize,
    cursor_style: &CursorStyle,
    selection: Option<&Selection>,
    selection_style: &SelectionStyle,
    blink_visible: bool,
    theme: &impl RichTextTheme,
) {
    apply_selection(line, selection, selection_style, theme);

    if !blink_visible {
        return;
    }

    let total_chars: usize = line.spans.iter().map(|s| s.content.chars().count()).sum();
    let fg = cursor_style.fg.unwrap_or_else(|| theme.get_primary_color());
    let bg = cursor_style
        .bg
        .unwrap_or_else(|| theme.get_background_color());

    if total_chars == 0 || cursor_col >= total_chars {
        line.spans
            .push(make_end_cursor_span(cursor_style.shape, fg, bg));
        return;
    }

    let is_insert = matches!(cursor_style.shape, CursorShape::Bar);

    let mut char_pos = 0usize;
    let mut idx = 0;

    while idx < line.spans.len() {
        let span_len = line.spans[idx].content.chars().count();
        if span_len == 0 {
            idx += 1;
            continue;
        }
        if cursor_col >= char_pos && cursor_col < char_pos + span_len {
            let offset_in_span = cursor_col - char_pos;
            if is_insert {
                insert_cursor_before(&mut line.spans, idx, offset_in_span, fg);
            } else {
                split_and_apply_on_char(
                    &mut line.spans,
                    idx,
                    offset_in_span,
                    cursor_style.shape,
                    fg,
                    bg,
                );
            }
            return;
        }
        char_pos += span_len;
        idx += 1;
    }

    line.spans
        .push(make_end_cursor_span(cursor_style.shape, fg, bg));
}

fn insert_cursor_before(spans: &mut Vec<Span<'static>>, span_idx: usize, offset: usize, fg: Color) {
    let original = spans.remove(span_idx);
    let chars: Vec<char> = original.content.chars().collect();
    let base_style = original.style;

    let before: String = chars[..offset].iter().collect();
    let after: String = chars[offset..].iter().collect();

    let mut parts = Vec::new();
    if !before.is_empty() {
        parts.push(Span::styled(before, base_style));
    }
    parts.push(Span::styled(Cow::Borrowed("│"), Style::default().fg(fg)));
    if !after.is_empty() {
        parts.push(Span::styled(after, base_style));
    }

    for (i, part) in parts.into_iter().enumerate() {
        spans.insert(span_idx + i, part);
    }
}

fn split_and_apply_on_char(
    spans: &mut Vec<Span<'static>>,
    span_idx: usize,
    offset: usize,
    shape: CursorShape,
    fg: Color,
    bg: Color,
) {
    let original = spans.remove(span_idx);
    let chars: Vec<char> = original.content.chars().collect();
    let base_style = original.style;

    let before: String = chars[..offset].iter().collect();
    let target = chars[offset];
    let after: String = chars[offset + 1..].iter().collect();

    let on_char_style = match shape {
        CursorShape::Block => Style::default().fg(bg).bg(fg),
        CursorShape::Underline => base_style.add_modifier(Modifier::UNDERLINED).fg(fg),
        CursorShape::HollowBlock => base_style.fg(fg),
        CursorShape::Bar => unreachable!(),
    };

    let mut parts = Vec::new();
    if !before.is_empty() {
        parts.push(Span::styled(before, base_style));
    }
    parts.push(Span::styled(target.to_string(), on_char_style));
    if !after.is_empty() {
        parts.push(Span::styled(after, base_style));
    }

    for (i, part) in parts.into_iter().enumerate() {
        spans.insert(span_idx + i, part);
    }
}

fn make_end_cursor_span(shape: CursorShape, fg: Color, bg: Color) -> Span<'static> {
    match shape {
        CursorShape::Bar => Span::styled(Cow::Borrowed("│"), Style::default().fg(fg)),
        CursorShape::Block => Span::styled(Cow::Borrowed(" "), Style::default().fg(bg).bg(fg)),
        CursorShape::Underline => Span::styled(
            Cow::Borrowed(" "),
            Style::default().fg(fg).add_modifier(Modifier::UNDERLINED),
        ),
        CursorShape::HollowBlock => Span::styled(Cow::Borrowed(" "), Style::default().fg(fg)),
    }
}

fn apply_selection(
    line: &mut Line<'static>,
    selection: Option<&Selection>,
    selection_style: &SelectionStyle,
    theme: &impl RichTextTheme,
) {
    let sel = match selection {
        Some(s) => s,
        None => return,
    };
    let sel_bg = selection_style
        .bg
        .unwrap_or_else(|| theme.get_popup_selected_background());
    let (sel_start, sel_end) = sel.ordered();
    let mut char_start = 0usize;

    for span in &mut line.spans {
        let span_len = span.content.chars().count();
        let span_end = char_start + span_len;
        if span_end > sel_start && char_start < sel_end {
            span.style = span.style.bg(sel_bg);
            if let Some(fg) = selection_style.fg {
                span.style = span.style.fg(fg);
            }
        }
        char_start = span_end;
    }
}
