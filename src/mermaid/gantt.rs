use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::types::{GanttChart, GanttSection, GanttTask};
use crate::theme::RichTextTheme;

const BLOCK: char = '█';
const LIGHT_BLOCK: char = '░';

pub fn parse_gantt(source: &str) -> Option<GanttChart> {
    let mut title: Option<String> = None;
    let mut sections: Vec<GanttSection> = Vec::new();
    let mut current_section: Option<GanttSection> = None;

    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line == "gantt" {
            continue;
        }

        if let Some(rest) = line.strip_prefix("title ") {
            title = Some(rest.trim().to_string());
            continue;
        }

        if line.starts_with("dateFormat") || line.starts_with("axisFormat") {
            continue;
        }

        if let Some(rest) = line.strip_prefix("section ") {
            if let Some(sec) = current_section.take() {
                if !sec.tasks.is_empty() {
                    sections.push(sec);
                }
            }
            current_section = Some(GanttSection {
                name: rest.trim().to_string(),
                tasks: Vec::new(),
            });
            continue;
        }

        if let Some(task) = parse_task(line) {
            if let Some(ref mut sec) = current_section {
                sec.tasks.push(task);
            } else {
                current_section = Some(GanttSection {
                    name: String::new(),
                    tasks: vec![task],
                });
            }
        }
    }

    if let Some(sec) = current_section.take() {
        if !sec.tasks.is_empty() {
            sections.push(sec);
        }
    }

    if sections.iter().all(|s| s.tasks.is_empty()) {
        return None;
    }

    Some(GanttChart { title, sections })
}

fn parse_task(line: &str) -> Option<GanttTask> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let colon_pos = line.find(':')?;
    let name = line[..colon_pos].trim().to_string();
    let rest = line[colon_pos + 1..].trim();

    let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();

    let mut id: Option<String> = None;
    let mut deps: Option<Vec<String>> = None;
    let mut duration: Option<String> = None;

    if !parts.is_empty() {
        id = Some(parts[0].to_string());
    }

    for part in parts.iter().skip(1) {
        if let Some(dur_str) = part.strip_suffix('d') {
            if dur_str.parse::<usize>().is_ok() || dur_str.parse::<f64>().is_ok() {
                duration = Some(part.to_string());
                continue;
            }
        }
        if part.starts_with("after ") {
            deps = Some(
                part.strip_prefix("after ")
                    .unwrap()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            );
            continue;
        }
        if duration.is_none() {
            duration = Some(part.to_string());
        }
    }

    Some(GanttTask {
        name,
        id,
        deps: deps.unwrap_or_default(),
        duration,
    })
}

pub fn render_gantt(
    chart: &GanttChart,
    max_width: usize,
    theme: &impl RichTextTheme,
) -> Vec<Line<'static>> {
    let title_style = Style::default()
        .fg(theme.get_primary_color())
        .add_modifier(Modifier::BOLD);
    let section_style = Style::default()
        .fg(theme.get_secondary_color())
        .add_modifier(Modifier::BOLD);
    let task_style = Style::default().fg(theme.get_text_color());
    let bar_style = Style::default()
        .fg(theme.get_primary_color())
        .add_modifier(Modifier::BOLD);
    let _bar_bg_style = Style::default().fg(theme.get_muted_text_color());
    let dur_style = Style::default().fg(theme.get_info_color());

    let all_tasks: Vec<&GanttTask> = chart.sections.iter().flat_map(|s| &s.tasks).collect();

    let label_col = 16usize;
    let dur_col = 8usize;
    let inner_w = max_width.saturating_sub(4);
    let bar_max = inner_w
        .saturating_sub(label_col)
        .saturating_sub(dur_col)
        .saturating_sub(4);
    let bar_max = bar_max.clamp(8, 40);

    let mut task_end: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut timeline_pos: usize = 0;
    let mut resolved_tasks: Vec<(String, usize, usize)> = Vec::new();

    for task in &all_tasks {
        let start = if task.deps.is_empty() {
            0usize
        } else {
            task.deps
                .iter()
                .filter_map(|d| task_end.get(d).copied())
                .max()
                .unwrap_or(0)
        };
        let dur: usize = task
            .duration
            .as_deref()
            .and_then(|d| d.strip_suffix('d'))
            .and_then(|d| d.parse().ok())
            .unwrap_or(5);
        let end = start + dur;
        if let Some(ref id) = task.id {
            task_end.insert(id.clone(), end);
        }
        resolved_tasks.push((task.name.clone(), start, dur));
        timeline_pos = timeline_pos.max(end);
    }
    let timeline_pos = timeline_pos.max(1);

    let mut lines: Vec<Line<'static>> = Vec::new();

    if let Some(ref title) = chart.title {
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

    let mut task_idx: usize = 0;

    for section in &chart.sections {
        if !section.name.is_empty() {
            let sw = unicode_width::UnicodeWidthStr::width(section.name.as_str());
            let pad = inner_w.saturating_sub(sw).saturating_sub(2);
            lines.push(Line::from(vec![
                Span::styled(" ".to_string(), section_style),
                Span::styled(section.name.clone(), section_style),
                Span::styled(" ".repeat(pad + 1), section_style),
            ]));
        }

        for task in &section.tasks {
            let name_display = truncate_str(&task.name, label_col);
            let nw = unicode_width::UnicodeWidthStr::width(name_display.as_str());
            let name_pad = label_col.saturating_sub(nw);

            let (.., start, dur) = resolved_tasks[task_idx];
            let bar_offset = (start as f64 / timeline_pos as f64 * bar_max as f64).round() as usize;
            let bar_len = (dur as f64 / timeline_pos as f64 * bar_max as f64).round() as usize;
            let bar_len = bar_len.max(4).min(bar_max.saturating_sub(bar_offset));

            let mut bar_str = " ".repeat(bar_offset);
            bar_str.push_str(&BLOCK.to_string().repeat(bar_len));
            let bg_len = bar_max.saturating_sub(bar_offset).saturating_sub(bar_len);
            if bg_len > 0 {
                bar_str.push_str(&LIGHT_BLOCK.to_string().repeat(bg_len));
            }

            let dur_text = task.duration.as_deref().unwrap_or("");

            lines.push(Line::from(vec![
                Span::styled("  ".to_string(), task_style),
                Span::styled(name_display, task_style),
                Span::styled(" ".repeat(name_pad), task_style),
                Span::styled(" ".to_string(), task_style),
                Span::styled(bar_str, bar_style),
                Span::styled(" ".to_string(), task_style),
                Span::styled(dur_text.to_string(), dur_style),
            ]));

            task_idx += 1;
        }
    }

    lines
}

fn truncate_str(s: &str, max_width: usize) -> String {
    let w = unicode_width::UnicodeWidthStr::width(s);
    if w <= max_width {
        return s.to_string();
    }
    let mut result = String::new();
    let mut current_w = 0;
    for ch in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_w + cw > max_width.saturating_sub(1) {
            break;
        }
        result.push(ch);
        current_w += cw;
    }
    if !result.is_empty() {
        result.push('…');
    }
    result
}
