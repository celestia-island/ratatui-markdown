use ratatui::{
    style::Style,
    text::{Line, Span},
};

use super::StyleSegment;

pub fn segments_to_lines(
    source: &str,
    segments: &[StyleSegment],
    prefix: &str,
    border_style: Style,
    max_width: usize,
) -> Vec<Line<'static>> {
    let prefix_width = unicode_width::UnicodeWidthStr::width(prefix);
    let mut lines: Vec<Line<'static>> = Vec::new();

    let mut line_start: usize = 0;
    for raw_line in source.split('\n') {
        let line_end = line_start + raw_line.len();

        let line_segs: Vec<StyleSegment> = segments
            .iter()
            .filter(|s| s.start < line_end && s.end > line_start)
            .map(|s| StyleSegment {
                start: s.start.saturating_sub(line_start),
                end: s.end.min(line_end).saturating_sub(line_start),
                style: s.style,
            })
            .filter(|s| s.start < s.end)
            .collect();

        let mut wrapped = wrap_line(
            raw_line.replace('\t', "    ").as_str(),
            &line_segs,
            prefix,
            prefix_width,
            border_style,
            max_width,
        );
        lines.append(&mut wrapped);

        line_start = line_end + 1;
    }

    lines
}

fn wrap_line(
    text: &str,
    segments: &[StyleSegment],
    prefix: &str,
    prefix_width: usize,
    border_style: Style,
    max_width: usize,
) -> Vec<Line<'static>> {
    let mut result = Vec::new();
    if text.is_empty() {
        let mut spans: Vec<Span<'static>> = Vec::new();
        if !prefix.is_empty() {
            spans.push(Span::styled(prefix.to_string(), border_style));
        }
        result.push(Line::from(spans));
        return result;
    }

    let sorted = sort_and_merge(segments);
    let mut seg_idx = 0;
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    if !prefix.is_empty() {
        current_spans.push(Span::styled(prefix.to_string(), border_style));
    }
    let mut current_len = prefix_width;
    let mut byte_pos: usize = 0;

    for ch in text.chars() {
        let char_byte_start = byte_pos;
        byte_pos += ch.len_utf8();

        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_len + cw > max_width && current_len > prefix_width {
            result.push(Line::from(std::mem::take(&mut current_spans)));
            current_spans = Vec::new();
            if !prefix.is_empty() {
                current_spans.push(Span::styled(prefix.to_string(), border_style));
            }
            current_len = prefix_width;
        }

        let style = style_at_byte(&sorted, &mut seg_idx, char_byte_start);

        if let Some(last) = current_spans.last_mut() {
            if last.style == style {
                last.content = format!("{}{}", last.content, ch).into();
                current_len += cw;
                continue;
            }
        }

        current_spans.push(Span::styled(ch.to_string(), style));
        current_len += cw;
    }

    if !current_spans.is_empty() {
        result.push(Line::from(current_spans));
    }

    result
}

fn style_at_byte(
    segments: &[(usize, usize, Style)],
    seg_idx: &mut usize,
    byte_pos: usize,
) -> Style {
    while *seg_idx < segments.len() && segments[*seg_idx].1 <= byte_pos {
        *seg_idx += 1;
    }
    if *seg_idx < segments.len() && segments[*seg_idx].0 <= byte_pos {
        segments[*seg_idx].2
    } else {
        Style::default()
    }
}

fn sort_and_merge(segments: &[StyleSegment]) -> Vec<(usize, usize, Style)> {
    if segments.is_empty() {
        return Vec::new();
    }
    let mut sorted: Vec<(usize, usize, Style)> =
        segments.iter().map(|s| (s.start, s.end, s.style)).collect();
    sorted.sort_by_key(|s| s.0);
    sorted
}
