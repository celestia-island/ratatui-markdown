use ratatui::{
    style::Style,
    text::{Line, Span},
};

use super::{CollapsibleTree, EntryKind, KeyStyle};
use crate::{constants::*, scroll::FocusableItemRange, theme::RichTextTheme};

impl CollapsibleTree {
    pub fn render_lines(&self, width: usize, theme: &impl RichTextTheme) -> Vec<Line<'static>> {
        let entries = self.flatten();
        let base = "  ".repeat(self.base_indent);
        let key_color = theme.get_json_key_color();
        let muted = theme.get_muted_text_color();

        entries
            .iter()
            .map(|entry| {
                let indent = build_indent(&entry.is_last_stack, entry.depth);
                let prefix = if self.base_indent > 0 {
                    Span::styled(base.clone(), Style::default())
                } else {
                    Span::raw("")
                };
                let indent_span = Span::styled(indent, Style::default().fg(muted));

                match &entry.kind {
                    EntryKind::Collapsed { label, count_str } => Line::from(vec![
                        prefix,
                        indent_span,
                        Span::styled("▶ ", Style::default().fg(key_color)),
                        Span::styled(label.replace('\t', "    "), Style::default().fg(key_color)),
                        Span::styled(format!(" {}", count_str), Style::default().fg(muted)),
                    ]),
                    EntryKind::Expanded { label, count_str } => Line::from(vec![
                        prefix,
                        indent_span,
                        Span::styled(
                            format!("{} ", TRIANGLE_DOWN),
                            Style::default().fg(key_color),
                        ),
                        Span::styled(label.replace('\t', "    "), Style::default().fg(key_color)),
                        Span::styled(format!(" {}", count_str), Style::default().fg(muted)),
                    ]),
                    EntryKind::Leaf {
                        key,
                        value,
                        value_type,
                    } => {
                        let val_color = match value_type {
                            super::ValueType::String => theme.get_json_string_color(),
                            super::ValueType::Number => theme.get_json_number_color(),
                            super::ValueType::Boolean => theme.get_json_bool_color(),
                            super::ValueType::Null => theme.get_json_null_color(),
                        };
                        let (key_prefix, separator) = match self.key_style {
                            KeyStyle::Json => ("\"", "\": "),
                            KeyStyle::Toml => ("", " = "),
                        };
                        let truncated =
                            truncate_value(value, width, entry.depth + self.base_indent);
                        Line::from(vec![
                            prefix,
                            indent_span,
                            Span::styled(
                                format!("{}{}{}", key_prefix, key.replace('\t', "    "), separator),
                                Style::default().fg(key_color),
                            ),
                            Span::styled(truncated, Style::default().fg(val_color)),
                        ])
                    }
                }
            })
            .collect()
    }

    pub fn total_lines(&self) -> usize {
        self.flatten().len()
    }

    pub fn build_focusable_items(&self) -> Vec<FocusableItemRange> {
        self.flatten()
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                matches!(
                    e.kind,
                    EntryKind::Collapsed { .. } | EntryKind::Expanded { .. }
                )
            })
            .map(|(i, e)| FocusableItemRange {
                start_line: i,
                end_line: i + 1,
                id: e.path.clone(),
            })
            .collect()
    }

    pub fn handle_toggle(&mut self, node_id: &str) -> bool {
        let is_collapsible = self
            .flatten()
            .iter()
            .find(|e| e.path == node_id)
            .map(|e| {
                matches!(
                    e.kind,
                    EntryKind::Collapsed { .. } | EntryKind::Expanded { .. }
                )
            })
            .unwrap_or(false);
        if is_collapsible {
            self.toggle(node_id);
            true
        } else {
            false
        }
    }

    pub fn count_expanded_lines(value: &serde_json::Value) -> usize {
        let mut tree = Self::from_value(value.clone());
        tree.expand_all();
        tree.total_lines()
    }
}

fn build_indent(is_last_stack: &[bool], depth: usize) -> String {
    if depth == 0 {
        return String::new();
    }
    let effective = is_last_stack.len().saturating_sub(1);
    let mut s = String::new();
    for &is_last in is_last_stack.iter().take(effective) {
        s.push_str(if is_last { "   " } else { BRANCH_VERT_PAD });
    }
    if let Some(&is_last) = is_last_stack.last() {
        s.push_str(if is_last {
            BRANCH_END_SP
        } else {
            BRANCH_MID_SP
        });
    }
    s
}

fn truncate_value(value: &str, total_width: usize, depth: usize) -> String {
    let value = value.replace('\t', "    ");
    let indent_len = depth * 3 + 4;
    let max_len = total_width.saturating_sub(indent_len + 4);
    let chars: Vec<char> = value.chars().collect();
    if chars.len() > max_len {
        let truncated: String = chars.into_iter().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    } else {
        value
    }
}
