use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::types::{SeqArrowKind, SequenceDiagram, SequenceMessage};
use crate::theme::RichTextTheme;

const HLINE: char = '─';
const VLINE: char = '│';

pub fn parse_sequence(source: &str) -> Option<SequenceDiagram> {
    let mut participants: Vec<String> = Vec::new();
    let mut messages: Vec<SequenceMessage> = Vec::new();
    let mut part_set = std::collections::HashSet::new();

    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line == "sequenceDiagram" {
            continue;
        }

        if let Some(name) = line.strip_prefix("participant ") {
            let name = name.trim().to_string();
            if !part_set.contains(&name) {
                part_set.insert(name.clone());
                participants.push(name);
            }
            continue;
        }

        if let Some(msg) = parse_message(line) {
            for name in &[&msg.from, &msg.to] {
                if !part_set.contains(*name) {
                    part_set.insert((*name).clone());
                    participants.push((*name).clone());
                }
            }
            messages.push(msg);
        }
    }

    if participants.is_empty() {
        return None;
    }
    Some(SequenceDiagram {
        participants,
        messages,
    })
}

fn parse_message(line: &str) -> Option<SequenceMessage> {
    let line = line.trim();

    let (arrow_str, arrow_start, arrow_end) = find_arrow_pos(line)?;

    let arrow_kind = match arrow_str {
        "->" => SeqArrowKind::Solid,
        "-->" => SeqArrowKind::Dotted,
        "->>" => SeqArrowKind::SolidOpen,
        "-->>" => SeqArrowKind::DottedOpen,
        _ => return None,
    };

    let from = line[..arrow_start].trim().to_string();
    let to_and_text = line[arrow_end..].trim();

    let (to, text) = if let Some(colon_pos) = to_and_text.find(':') {
        (
            to_and_text[..colon_pos].trim().to_string(),
            to_and_text[colon_pos + 1..].trim().to_string(),
        )
    } else {
        (to_and_text.trim().to_string(), String::new())
    };

    if from.is_empty() || to.is_empty() {
        return None;
    }

    Some(SequenceMessage {
        from,
        to,
        text,
        arrow_kind,
    })
}

fn find_arrow_pos(s: &str) -> Option<(&str, usize, usize)> {
    for arrow in &["-->>", "->>", "-->", "->"] {
        if let Some(idx) = s.find(arrow) {
            return Some((*arrow, idx, idx + arrow.len()));
        }
    }
    None
}

pub fn render_sequence(
    diagram: &SequenceDiagram,
    max_width: usize,
    theme: &impl RichTextTheme,
) -> Vec<Line<'static>> {
    if diagram.participants.is_empty() {
        return vec![Line::from(Span::styled(
            "(empty sequence diagram)",
            Style::default().fg(theme.get_muted_text_color()),
        ))];
    }

    let n_part = diagram.participants.len();
    let col_width = ((max_width - 2) / n_part).clamp(6, 20);
    let _total_w = col_width * n_part + 2;

    let mut lines: Vec<Line<'static>> = Vec::new();

    let header_style = Style::default()
        .fg(theme.get_primary_color())
        .add_modifier(Modifier::BOLD);
    let line_style = Style::default().fg(theme.get_muted_text_color());
    let msg_style = Style::default().fg(theme.get_text_color());
    let arrow_style = Style::default()
        .fg(theme.get_primary_color())
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default()
        .fg(theme.get_info_color())
        .add_modifier(Modifier::ITALIC);

    {
        let mut spans: Vec<Span<'static>> = Vec::new();
        for (i, name) in diagram.participants.iter().enumerate() {
            let tw = unicode_width::UnicodeWidthStr::width(name.as_str());
            let pad = col_width.saturating_sub(tw);
            let left_pad = pad / 2;
            let right_pad = pad - left_pad;
            for _ in 0..left_pad {
                spans.push(Span::styled(" ".to_string(), header_style));
            }
            spans.push(Span::styled(name.clone(), header_style));
            for _ in 0..right_pad {
                spans.push(Span::styled(" ".to_string(), header_style));
            }
            if i < n_part - 1 {
                spans.push(Span::styled(" ".to_string(), line_style));
            }
        }
        lines.push(Line::from(spans));
    }

    for msg in &diagram.messages {
        let from_idx = diagram
            .participants
            .iter()
            .position(|p| p == &msg.from)
            .unwrap_or(0);
        let to_idx = diagram
            .participants
            .iter()
            .position(|p| p == &msg.to)
            .unwrap_or(0);

        let lifeline_spans = |style: Style| -> Vec<Span<'static>> {
            let mut spans: Vec<Span<'static>> = Vec::new();
            for i in 0..n_part {
                let center = col_width / 2;
                for c in 0..col_width {
                    if c == center {
                        spans.push(Span::styled(VLINE.to_string(), style));
                    } else {
                        spans.push(Span::styled(" ".to_string(), style));
                    }
                }
                if i < n_part - 1 {
                    spans.push(Span::styled(" ".to_string(), style));
                }
            }
            spans
        };

        lines.push(Line::from(lifeline_spans(line_style)));

        if !msg.text.is_empty() {
            let text_w = unicode_width::UnicodeWidthStr::width(msg.text.as_str());
            let mut label_spans: Vec<Span<'static>> = Vec::new();
            for i in 0..n_part {
                let center = col_width / 2;
                for c in 0..col_width {
                    if c == center {
                        label_spans.push(Span::styled(VLINE.to_string(), line_style));
                    } else {
                        label_spans.push(Span::styled(" ".to_string(), line_style));
                    }
                }
                if i < n_part - 1 {
                    label_spans.push(Span::styled(" ".to_string(), line_style));
                }
            }

            let min_idx = from_idx.min(to_idx);
            let max_idx = from_idx.max(to_idx);
            let label_center_x = if min_idx == max_idx {
                min_idx * (col_width + 1) + col_width / 2
            } else {
                let left_x = min_idx * (col_width + 1) + col_width / 2 + 1;
                let right_x = max_idx * (col_width + 1) + col_width / 2;
                (left_x + right_x) / 2
            };
            let label_start = label_center_x.saturating_sub(text_w / 2);

            let chars: Vec<(usize, char)> = {
                let mut v = Vec::new();
                let mut cx = label_start;
                for ch in msg.text.chars() {
                    let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
                    v.push((cx, ch));
                    cx += cw;
                }
                v
            };

            for (pos, ch) in chars {
                if pos < label_spans.len() {
                    let existing = &label_spans[pos];
                    if existing.content == " " {
                        label_spans[pos] = Span::styled(ch.to_string(), label_style);
                    }
                }
            }

            lines.push(Line::from(label_spans));
        }

        {
            let mut arrow_spans: Vec<Span<'static>> = Vec::new();
            for i in 0..n_part {
                let center = col_width / 2;
                for c in 0..col_width {
                    if c == center {
                        arrow_spans.push(Span::styled(VLINE.to_string(), line_style));
                    } else {
                        arrow_spans.push(Span::styled(" ".to_string(), msg_style));
                    }
                }
                if i < n_part - 1 {
                    arrow_spans.push(Span::styled(" ".to_string(), msg_style));
                }
            }

            let go_right = to_idx > from_idx;
            let left_idx = from_idx.min(to_idx);
            let right_idx = from_idx.max(to_idx);

            let x_left = left_idx * (col_width + 1) + col_width / 2 + 1;
            let x_right = right_idx * (col_width + 1) + col_width / 2 - 1;

            let line_ch = match msg.arrow_kind {
                SeqArrowKind::Solid | SeqArrowKind::SolidOpen => HLINE,
                SeqArrowKind::Dotted | SeqArrowKind::DottedOpen => '╌',
            };

            for x in x_left..=x_right {
                if x < arrow_spans.len() {
                    let existing = &arrow_spans[x];
                    if existing.content == " " {
                        arrow_spans[x] = Span::styled(line_ch.to_string(), arrow_style);
                    }
                }
            }

            let head_x = if go_right {
                right_idx * (col_width + 1) + col_width / 2 - 1
            } else {
                left_idx * (col_width + 1) + col_width / 2 + 1
            };

            let head_ch = match msg.arrow_kind {
                SeqArrowKind::Solid | SeqArrowKind::Dotted => {
                    if go_right {
                        '►'
                    } else {
                        '◄'
                    }
                }
                SeqArrowKind::SolidOpen | SeqArrowKind::DottedOpen => {
                    if go_right {
                        '▶'
                    } else {
                        '◀'
                    }
                }
            };

            let tail_x = if go_right {
                left_idx * (col_width + 1) + col_width / 2 + 1
            } else {
                right_idx * (col_width + 1) + col_width / 2 - 1
            };

            if head_x < arrow_spans.len() {
                arrow_spans[head_x] = Span::styled(head_ch.to_string(), arrow_style);
            }

            let tail_ch = if go_right { '>' } else { '<' };
            if tail_x < arrow_spans.len() && arrow_spans[tail_x].content == " " {
                arrow_spans[tail_x] = Span::styled(tail_ch.to_string(), arrow_style);
            }

            lines.push(Line::from(arrow_spans));
        }
    }

    let lifeline_style = Style::default().fg(theme.get_muted_text_color());
    lines.push(Line::from({
        let mut spans: Vec<Span<'static>> = Vec::new();
        for i in 0..n_part {
            let center = col_width / 2;
            for c in 0..col_width {
                if c == center {
                    spans.push(Span::styled(VLINE.to_string(), lifeline_style));
                } else {
                    spans.push(Span::styled(" ".to_string(), lifeline_style));
                }
            }
            if i < n_part - 1 {
                spans.push(Span::styled(" ".to_string(), lifeline_style));
            }
        }
        spans
    }));

    lines
}
