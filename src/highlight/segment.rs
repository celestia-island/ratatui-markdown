use ratatui::{
    style::Style,
    text::{Line, Span},
};

use super::StyleSegment;

pub fn segments_to_lines(
    source: &str,
    segments: &[StyleSegment],
    prefix: &str,
    max_width: usize,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut current_line_len: usize = 0;
    let prefix_width = unicode_width::UnicodeWidthStr::width(prefix);

    if !prefix.is_empty() {
        current_spans.push(Span::raw(prefix.to_string()));
        current_line_len += prefix_width;
    }

    let sorted = sort_and_merge(segments);
    let mut seg_idx = 0;
    let chars: Vec<char> = source.chars().collect();
    let mut byte_pos: usize = 0;

    macro_rules! flush_line {
        () => {
            lines.push(Line::from(std::mem::take(&mut current_spans)));
            current_line_len = 0;
            if !prefix.is_empty() {
                current_spans.push(Span::raw(prefix.to_string()));
                current_line_len += prefix_width;
            }
        };
    }

    for (_, ch) in chars.iter().enumerate() {
        let char_byte_start = byte_pos;
        byte_pos += ch.len_utf8();

        let cw = unicode_width::UnicodeWidthChar::width(*ch).unwrap_or(0);
        if current_line_len + cw > max_width && current_line_len > prefix_width {
            flush_line!();
        }

        let style = style_at_byte(&sorted, &mut seg_idx, char_byte_start);

        if !current_spans.is_empty() {
            if let Some(last) = current_spans.last_mut() {
                if last.style == style {
                    last.content = format!("{}{}", last.content, ch).into();
                    current_line_len += cw;
                    continue;
                }
            }
        }

        current_spans.push(Span::styled(ch.to_string(), style));
        current_line_len += cw;
    }

    if !current_spans.is_empty() {
        lines.push(Line::from(std::mem::take(&mut current_spans)));
    }

    lines
}

fn style_at_byte(segments: &[(usize, usize, Style)], seg_idx: &mut usize, byte_pos: usize) -> Style {
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
