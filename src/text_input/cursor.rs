use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use std::borrow::Cow;

use super::types::{CursorPosition, CursorShape, CursorStyle, Selection, SelectionStyle};
use crate::theme::RichTextTheme;

struct CursorRenderContext<'a> {
    cursor_char_idx: usize,
    cursor_style: &'a CursorStyle,
    selection: Option<&'a Selection>,
    selection_style: &'a SelectionStyle,
    blink_visible: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_cursor_and_selection(
    line: &mut Line<'static>,
    cursor_char_idx: usize,
    _horizontal_scroll: usize,
    cursor_style: &CursorStyle,
    selection: Option<&Selection>,
    selection_style: &SelectionStyle,
    blink_visible: bool,
    theme: &impl RichTextTheme,
) {
    let ctx = CursorRenderContext {
        cursor_char_idx,
        cursor_style,
        selection,
        selection_style,
        blink_visible,
    };

    apply_selection(line, &ctx, theme);

    if !ctx.blink_visible {
        return;
    }

    let resolved_cursor_fg = ctx.cursor_style.fg.unwrap_or_else(|| theme.get_primary_color());
    let resolved_cursor_bg = ctx.cursor_style.bg.unwrap_or_else(|| theme.get_background_color());

    match ctx.cursor_style.position {
        CursorPosition::OnChar => {
            let mut char_pos = 0usize;
            for span in &mut line.spans {
                let span_len = span.content.chars().count();
                if ctx.cursor_char_idx >= char_pos && ctx.cursor_char_idx < char_pos + span_len {
                    let char_in_span = ctx.cursor_char_idx - char_pos;
                    apply_cursor_on_char(span, char_in_span, ctx.cursor_style.shape, resolved_cursor_fg, resolved_cursor_bg);
                    return;
                }
                char_pos += span_len;
            }
        }
        CursorPosition::BeforeChar => {
            let mut char_pos = 0usize;
            for span_idx in 0..line.spans.len() {
                let span_len = line.spans[span_idx].content.chars().count();
                if ctx.cursor_char_idx == char_pos {
                    let cursor_span = make_before_cursor_span(ctx.cursor_style.shape, resolved_cursor_fg, resolved_cursor_bg);
                    line.spans.insert(span_idx, cursor_span);
                    return;
                }
                if ctx.cursor_char_idx > char_pos && ctx.cursor_char_idx < char_pos + span_len {
                    let char_in_span = ctx.cursor_char_idx - char_pos;
                    split_and_insert_cursor_before(
                        &mut line.spans, span_idx, char_in_span,
                        ctx.cursor_style.shape, resolved_cursor_fg, resolved_cursor_bg,
                    );
                    return;
                }
                char_pos += span_len;
            }
            if ctx.cursor_char_idx == char_pos {
                let cursor_span = make_before_cursor_span(ctx.cursor_style.shape, resolved_cursor_fg, resolved_cursor_bg);
                line.spans.push(cursor_span);
            }
        }
    }
}

fn apply_selection(line: &mut Line<'static>, ctx: &CursorRenderContext<'_>, theme: &impl RichTextTheme) {
    let selection_bg = ctx.selection_style
        .bg
        .unwrap_or_else(|| theme.get_popup_selected_background());
    let selection_fg = ctx.selection_style.fg;

    let mut span_char_start = 0usize;

    for span_idx in 0..line.spans.len() {
        let span_char_count = line.spans[span_idx].content.chars().count();
        let span_char_end = span_char_start + span_char_count;

        if let Some(sel) = ctx.selection {
            let (sel_start, sel_end) = sel.ordered();
            if span_char_end > sel_start && span_char_start < sel_end {
                let mut new_style = line.spans[span_idx].style;
                new_style = new_style.bg(selection_bg);
                if let Some(fg) = selection_fg {
                    new_style = new_style.fg(fg);
                }
                line.spans[span_idx].style = new_style;
            }
        }

        span_char_start = span_char_end;
    }
}

fn apply_cursor_on_char(
    span: &mut Span<'static>,
    char_in_span: usize,
    shape: CursorShape,
    fg: Color,
    bg: Color,
) {
    let chars: Vec<char> = span.content.chars().collect();
    if char_in_span >= chars.len() {
        return;
    }
    match shape {
        CursorShape::Block => {
            if chars.len() == 1 {
                span.style = Style::default().fg(bg).bg(fg);
            } else {
                let before: String = chars[..char_in_span].iter().collect();
                let target = chars[char_in_span];
                let after: String = chars[char_in_span + 1..].iter().collect();
                span.content = Cow::Owned(format!("{}{}{}", before, target, after));
            }
        }
        CursorShape::Underline => {
            span.style = span.style.add_modifier(Modifier::UNDERLINED);
        }
        CursorShape::Bar => {
            let before: String = chars[..char_in_span].iter().collect();
            let target = chars[char_in_span];
            let after: String = chars[char_in_span + 1..].iter().collect();
            let base_style = span.style;
            span.content = Cow::Owned(before);
            span.style = base_style;
            let _target_span = Span::styled(target.to_string(), base_style);
            let _after_span = if after.is_empty() {
                None
            } else {
                Some(Span::styled(after, base_style))
            };
        }
        CursorShape::HollowBlock => {
            span.style = span.style.fg(fg);
        }
    }
}

fn make_before_cursor_span(shape: CursorShape, fg: Color, bg: Color) -> Span<'static> {
    match shape {
        CursorShape::Block => Span::styled(" ", Style::default().fg(bg).bg(fg)),
        CursorShape::Bar => Span::styled("│", Style::default().fg(fg)),
        CursorShape::Underline => Span::styled(" ", Style::default().fg(fg).add_modifier(Modifier::UNDERLINED)),
        CursorShape::HollowBlock => Span::styled(" ", Style::default().fg(fg)),
    }
}

fn split_and_insert_cursor_before(
    spans: &mut Vec<Span<'static>>,
    span_idx: usize,
    char_in_span: usize,
    shape: CursorShape,
    fg: Color,
    bg: Color,
) {
    let original = spans.remove(span_idx);
    let chars: Vec<char> = original.content.chars().collect();
    let before: String = chars[..char_in_span].iter().collect();
    let after: String = chars[char_in_span..].iter().collect();
    let base_style = original.style;

    let cursor_span = make_before_cursor_span(shape, fg, bg);

    let mut new_spans = Vec::new();
    if !before.is_empty() {
        new_spans.push(Span::styled(before, base_style));
    }
    new_spans.push(cursor_span);
    if !after.is_empty() {
        new_spans.push(Span::styled(after, base_style));
    }

    for (i, new_span) in new_spans.into_iter().enumerate() {
        spans.insert(span_idx + i, new_span);
    }
}
