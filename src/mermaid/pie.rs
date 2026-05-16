use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::types::PieChart;
use crate::theme::RichTextTheme;

const BLOCK: char = '█';
const LIGHT_BLOCK: char = '░';

pub fn parse_pie(source: &str) -> Option<PieChart> {
    let mut title: Option<String> = None;
    let mut slices: Vec<(String, f64)> = Vec::new();

    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("pie") {
            let rest = line.strip_prefix("pie").unwrap().trim();
            if let Some(t) = rest.strip_prefix("title ") {
                title = Some(t.trim().to_string());
            }
            continue;
        }

        if line.starts_with("title ") {
            title = Some(line.strip_prefix("title ").unwrap().trim().to_string());
            continue;
        }

        if let Some(slice) = parse_slice(line) {
            slices.push(slice);
        }
    }

    if slices.is_empty() {
        return None;
    }

    Some(PieChart { title, slices })
}

fn parse_slice(line: &str) -> Option<(String, f64)> {
    let line = line.trim();
    let (label_part, value_part) = if let Some(stripped) = line.strip_prefix('"') {
        let end_quote = stripped.find('"')?;
        let label = stripped[..end_quote].to_string();
        let rest = stripped[end_quote + 1..].trim();
        let value_str = rest.strip_prefix(':').unwrap_or(rest).trim();
        (label, value_str.to_string())
    } else {
        let colon_pos = line.rfind(':')?;
        (
            line[..colon_pos].trim().to_string(),
            line[colon_pos + 1..].trim().to_string(),
        )
    };

    let value: f64 = value_part.parse().ok()?;
    if value <= 0.0 {
        return None;
    }

    Some((label_part, value))
}

pub fn render_pie(
    diagram: &PieChart,
    max_width: usize,
    theme: &impl RichTextTheme,
) -> Vec<Line<'static>> {
    if diagram.slices.is_empty() {
        return vec![Line::from(Span::styled(
            "(empty pie chart)",
            Style::default().fg(theme.get_muted_text_color()),
        ))];
    }

    let title_style = Style::default()
        .fg(theme.get_primary_color())
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(theme.get_text_color());
    let bar_style = Style::default()
        .fg(theme.get_primary_color())
        .add_modifier(Modifier::BOLD);
    let _bar_bg_style = Style::default().fg(theme.get_muted_text_color());
    let pct_style = Style::default()
        .fg(theme.get_secondary_color())
        .add_modifier(Modifier::BOLD);

    let inner_w = max_width.saturating_sub(4);
    let label_col = 14usize;
    let pct_col = 6usize;
    let bar_max = inner_w
        .saturating_sub(label_col)
        .saturating_sub(pct_col)
        .min(30);
    let bar_max = bar_max.max(10);

    let total: f64 = diagram.slices.iter().map(|(_, v)| v).sum();
    let max_val = diagram
        .slices
        .iter()
        .map(|(_, v)| *v)
        .fold(f64::NEG_INFINITY, f64::max);

    let mut lines: Vec<Line<'static>> = Vec::new();

    if let Some(ref title) = diagram.title {
        let tw = unicode_width::UnicodeWidthStr::width(title.as_str());
        let pad = inner_w.saturating_sub(tw);
        let left_pad = pad / 2;
        let right_pad = pad - left_pad;
        lines.push(Line::from(vec![
            Span::styled(" ".repeat(left_pad), title_style),
            Span::styled(title.clone(), title_style),
            Span::styled(" ".repeat(right_pad), title_style),
        ]));
        lines.push(Line::from(vec![Span::styled(
            " ".repeat(inner_w),
            Style::default(),
        )]));
    }

    for (label, value) in &diagram.slices {
        let pct = if total > 0.0 {
            value / total * 100.0
        } else {
            0.0
        };
        let bar_len = if max_val > 0.0 {
            ((value / max_val) * bar_max as f64).round() as usize
        } else {
            0
        };
        let bar_len = bar_len.min(bar_max);

        let label_display = if label.len() > label_col {
            let mut end = label_col;
            while end > 0 && !label.is_char_boundary(end) {
                end -= 1;
            }
            label[..end].to_string()
        } else {
            label.clone()
        };
        let label_w = unicode_width::UnicodeWidthStr::width(label_display.as_str());
        let label_pad = label_col.saturating_sub(label_w);

        let pct_str = format!("{:.0}%", pct);
        let pct_w = pct_str.len();

        let mut bar_str = BLOCK.to_string().repeat(bar_len);
        let bg_len = bar_max.saturating_sub(bar_len);
        if bg_len > 0 {
            bar_str.push_str(&LIGHT_BLOCK.to_string().repeat(bg_len));
        }

        lines.push(Line::from(vec![
            Span::styled(" ".to_string(), label_style),
            Span::styled(label_display, label_style),
            Span::styled(" ".repeat(label_pad), label_style),
            Span::styled(" ".to_string(), label_style),
            Span::styled(bar_str, bar_style),
            Span::styled(" ".to_string(), label_style),
            Span::styled(pct_str, pct_style),
            Span::styled(" ".repeat(pct_col.saturating_sub(pct_w)), pct_style),
        ]));
    }

    lines
}
