use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};
use unicode_width::UnicodeWidthChar;

use crate::theme::RichTextTheme;

#[derive(Debug, Clone, PartialEq)]
pub struct QuadrantChart {
    pub title: Option<String>,
    pub x_axis_left: String,
    pub x_axis_right: String,
    pub y_axis_bottom: String,
    pub y_axis_top: String,
    pub quadrants: Vec<Option<String>>,
    pub points: Vec<QuadrantPoint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuadrantPoint {
    pub label: String,
    pub x: f64,
    pub y: f64,
}

pub fn parse_quadrant(source: &str) -> Option<QuadrantChart> {
    let mut title: Option<String> = None;
    let mut x_left = String::new();
    let mut x_right = String::new();
    let mut y_bottom = String::new();
    let mut y_top = String::new();
    let mut quadrants: Vec<Option<String>> = vec![None; 4];
    let mut points: Vec<QuadrantPoint> = Vec::new();

    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('%') {
            continue;
        }

        if let Some(rest) = line.strip_prefix("quadrantChart") {
            let part = rest.trim();
            if !part.is_empty() {
                title = Some(part.to_string());
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("title ") {
            title = Some(rest.trim().to_string());
            continue;
        }

        if let Some(rest) = line.strip_prefix("x-axis ") {
            let parts: Vec<&str> = rest.split("-->").collect();
            if parts.len() == 2 {
                x_left = parts[0].trim().to_string();
                x_right = parts[1].trim().to_string();
            }
            continue;
        }
        if line == "x-axis" {
            continue;
        }

        if let Some(rest) = line.strip_prefix("y-axis ") {
            let parts: Vec<&str> = rest.split("-->").collect();
            if parts.len() == 2 {
                y_bottom = parts[0].trim().to_string();
                y_top = parts[1].trim().to_string();
            }
            continue;
        }
        if line == "y-axis" {
            continue;
        }

        for (i, q) in quadrants.iter_mut().enumerate() {
            let prefix = format!("quadrant-{} ", i + 1);
            let alt_prefix = format!("quadrant-{}", i + 1);
            if let Some(rest) = line.strip_prefix(&prefix) {
                *q = Some(rest.trim().to_string());
            } else if line == alt_prefix {
                *q = Some(format!("Q{}", i + 1));
            }
        }

        if let Some(colon) = line.find(':') {
            let label = line[..colon].trim();
            let rest = &line[colon + 1..].trim();
            if let (Some(open), Some(close)) = (rest.find('['), rest.find(']')) {
                let coords = &rest[open + 1..close].trim();
                let parts: Vec<&str> = coords.split(',').collect();
                if parts.len() == 2 {
                    if let (Ok(x), Ok(y)) = (
                        parts[0].trim().parse::<f64>(),
                        parts[1].trim().parse::<f64>(),
                    ) {
                        points.push(QuadrantPoint {
                            label: label.to_string(),
                            x,
                            y,
                        });
                    }
                }
            }
        }
    }

    if points.is_empty() {
        return None;
    }

    Some(QuadrantChart {
        title,
        x_axis_left: if x_left.is_empty() {
            "Low".to_string()
        } else {
            x_left
        },
        x_axis_right: if x_right.is_empty() {
            "High".to_string()
        } else {
            x_right
        },
        y_axis_bottom: if y_bottom.is_empty() {
            "Low".to_string()
        } else {
            y_bottom
        },
        y_axis_top: if y_top.is_empty() {
            "High".to_string()
        } else {
            y_top
        },
        quadrants,
        points,
    })
}

pub fn render_quadrant(
    chart: &QuadrantChart,
    max_width: usize,
    theme: &impl RichTextTheme,
) -> Vec<Line<'static>> {
    let plot_w = (max_width.saturating_sub(4)).clamp(20, 50);
    let plot_h = (plot_w / 2).clamp(8, 20);

    let mut lines: Vec<Line<'static>> = Vec::new();

    if let Some(ref t) = chart.title {
        let t_w: usize = t.chars().map(|c| c.width().unwrap_or(1)).sum();
        let pad = (max_width.saturating_sub(t_w)) / 2;
        lines.push(Line::from(Span::styled(
            format!("{}{}", " ".repeat(pad), t),
            Style::default()
                .fg(theme.get_primary_color())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::raw(""));
    }

    let q_top_left = chart.quadrants.first().and_then(|s| s.clone());

    let left_label_w = 14;

    for row in 0..=plot_h {
        let mut spans: Vec<Span<'static>> = Vec::new();

        if row == 0 {
            let label = if let Some(ref q) = q_top_left {
                format!("{:>width$} ", q, width = left_label_w)
            } else {
                " ".repeat(left_label_w + 1)
            };
            spans.push(Span::styled(
                label,
                Style::default().fg(theme.get_muted_text_color()),
            ));
        } else {
            spans.push(Span::raw(" ".repeat(left_label_w + 1)));
        }

        spans.push(Span::styled(
            "\u{2502}",
            Style::default().fg(theme.get_muted_text_color()),
        ));

        let mut grid_row: Vec<char> = vec![' '; plot_w];
        let mid = plot_w / 2;

        if row == plot_h {
            for (x, cell) in grid_row.iter_mut().enumerate().take(plot_w) {
                if x == mid {
                    *cell = '\u{2534}';
                } else {
                    *cell = '\u{2500}';
                }
            }
        } else {
            grid_row[mid] = '\u{2502}';
            if plot_h > 0 && row == plot_h / 2 {
                for (x, cell) in grid_row.iter_mut().enumerate().take(plot_w) {
                    if x == mid {
                        *cell = '\u{253C}';
                    } else {
                        *cell = '\u{2500}';
                    }
                }
            }
        }

        for point in &chart.points {
            let px = ((point.x * (plot_w - 1) as f64) as usize).min(plot_w - 1);
            let py = (((1.0 - point.y) * (plot_h as f64)) as usize).min(plot_h);
            if py == row && px < plot_w {
                grid_row[px] = '\u{25CF}';
            }
        }

        let row_str: String = grid_row.iter().collect();
        spans.push(Span::styled(
            row_str,
            Style::default().fg(theme.get_secondary_color()),
        ));

        lines.push(Line::from(spans));
    }

    let mut axis_labels: Vec<Span<'static>> = Vec::new();
    axis_labels.push(Span::raw(" ".repeat(left_label_w + 1)));
    axis_labels.push(Span::styled(
        format!(
            "{} -- {} -- {}",
            chart.x_axis_left, "\u{25B2}", chart.x_axis_right
        ),
        Style::default().fg(theme.get_muted_text_color()),
    ));
    lines.push(Line::from(axis_labels));

    lines.push(Line::raw(""));

    let mut legend_lines: Vec<Line<'static>> = Vec::new();
    for point in &chart.points {
        legend_lines.push(Line::from(vec![
            Span::styled(" \u{25CF} ", Style::default().fg(theme.get_primary_color())),
            Span::styled(
                format!("{} ({:.2}, {:.2})", point.label, point.x, point.y),
                Style::default().fg(theme.get_text_color()),
            ),
        ]));
    }
    lines.extend(legend_lines);

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quadrant_chart() {
        let source = "quadrantChart\n    x-axis Low --> High\n    y-axis Low --> High\n    A: [0.3, 0.6]\n    B: [0.45, 0.23]\n";
        let chart = parse_quadrant(source).unwrap();
        assert_eq!(chart.points.len(), 2);
        assert_eq!(chart.x_axis_left, "Low");
        assert_eq!(chart.x_axis_right, "High");
    }

    #[test]
    fn test_parse_quadrant_with_quadrants() {
        let source = "quadrantChart\n    quadrant-1 We should expand\n    A: [0.3, 0.6]\n";
        let chart = parse_quadrant(source).unwrap();
        assert_eq!(chart.quadrants[0].as_deref(), Some("We should expand"));
    }

    #[test]
    fn test_parse_empty_returns_none() {
        let chart = parse_quadrant("quadrantChart\n");
        assert!(chart.is_none());
    }
}
