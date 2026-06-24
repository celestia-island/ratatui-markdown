use ratatui::{
    style::{Modifier, Style},
    text::Span,
};

use crate::theme::RichTextTheme;

pub fn parse_inline_formatting(text: &str, theme: &impl RichTextTheme) -> Vec<Span<'static>> {
    let expanded = text.replace('\t', "    ");
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = expanded.chars().collect();
    let len = chars.len();
    let mut i = 0;

    macro_rules! flush_current {
        () => {
            if !current.is_empty() {
                spans.push(Span::styled(
                    current.clone(),
                    Style::default().fg(theme.get_text_color()),
                ));
                current.clear();
            }
        };
    }

    while i < len {
        if chars[i] == '*' && i + 2 < len && chars[i + 1] == '*' && chars[i + 2] == '*' {
            flush_current!();
            let start = i + 3;
            let mut found = false;
            let mut end = start;
            while end + 2 < len {
                if chars[end] == '*' && chars[end + 1] == '*' && chars[end + 2] == '*' {
                    let t: String = chars[start..end].iter().collect();
                    spans.push(Span::styled(
                        t,
                        Style::default()
                            .fg(theme.get_text_color())
                            .add_modifier(Modifier::BOLD | Modifier::ITALIC),
                    ));
                    i = end + 3;
                    found = true;
                    break;
                }
                end += 1;
            }
            if !found {
                current.push('*');
                current.push('*');
                current.push('*');
                i += 3;
            }
            continue;
        }

        if (chars[i] == '*' || chars[i] == '_') && i + 1 < len && chars[i + 1] == chars[i] {
            flush_current!();
            let delimiter = chars[i];
            let start = i + 2;
            let mut end = start;
            let mut found = false;
            while end + 1 < len {
                if chars[end] == delimiter && chars[end + 1] == delimiter {
                    let t: String = chars[start..end].iter().collect();
                    spans.push(Span::styled(
                        t,
                        Style::default()
                            .fg(theme.get_text_color())
                            .add_modifier(Modifier::BOLD),
                    ));
                    i = end + 2;
                    found = true;
                    break;
                }
                end += 1;
            }
            if !found {
                current.push(chars[i]);
                current.push(chars[i]);
                i += 2;
            }
            continue;
        }

        if chars[i] == '*' || chars[i] == '_' {
            let is_left_flanking = i == 0
                || chars[i - 1] == ' '
                || chars[i - 1] == '\t'
                || chars[i - 1] == '\n'
                || chars[i - 1] == '('
                || chars[i - 1] == '[';
            if !is_left_flanking {
                current.push(chars[i]);
                i += 1;
                continue;
            }
            flush_current!();
            let delimiter = chars[i];
            let start = i + 1;
            let mut end = start;
            let mut found = false;
            while end < len {
                if chars[end] == delimiter {
                    let t: String = chars[start..end].iter().collect();
                    spans.push(Span::styled(
                        t,
                        Style::default()
                            .fg(theme.get_text_color())
                            .add_modifier(Modifier::ITALIC),
                    ));
                    i = end + 1;
                    found = true;
                    break;
                }
                end += 1;
            }
            if !found {
                current.push(chars[i]);
                i += 1;
            }
            continue;
        }

        if chars[i] == '`' {
            let start = i + 1;
            let mut end = start;
            let mut found = false;
            while end < len {
                if chars[end] == '`' {
                    found = true;
                    break;
                }
                end += 1;
            }
            if !found {
                current.push('`');
                i += 1;
                continue;
            }
            let need_before = if !current.is_empty() {
                !current.ends_with(' ')
            } else {
                spans.last().is_some_and(|s| !s.content.ends_with(' '))
            };
            if need_before {
                current.push(' ');
            }
            flush_current!();
            let t: String = chars[start..end].iter().collect();
            spans.push(Span::styled(
                t,
                Style::default().fg(theme.get_accent_yellow()),
            ));
            i = end + 1;
            if i < len && chars[i] != ' ' && chars[i] != '\n' {
                current.push(' ');
            }
            continue;
        }

        if chars[i] == '~' && i + 1 < len && chars[i + 1] == '~' {
            let start = i + 2;
            let mut end = start;
            let mut found = false;
            while end + 1 < len {
                if chars[end] == '~' && chars[end + 1] == '~' {
                    let t: String = chars[start..end].iter().collect();
                    flush_current!();
                    spans.push(Span::styled(
                        t,
                        Style::default()
                            .fg(theme.get_text_color())
                            .add_modifier(Modifier::CROSSED_OUT),
                    ));
                    i = end + 2;
                    found = true;
                    break;
                }
                end += 1;
            }
            if !found {
                current.push('~');
                current.push('~');
                i += 2;
            }
            continue;
        }

        if chars[i] == '\\' && i + 1 < len {
            let next = chars[i + 1];
            if next.is_ascii_punctuation() {
                current.push(next);
                i += 2;
                continue;
            }
        }

        if chars[i] == '[' {
            let mut end_bracket = i + 1;
            let mut found_link = false;
            let mut bracket_depth: usize = 0;
            while end_bracket < len {
                if chars[end_bracket] == '[' {
                    bracket_depth += 1;
                } else if chars[end_bracket] == ']' {
                    if bracket_depth > 0 {
                        bracket_depth -= 1;
                    } else if end_bracket + 1 < len && chars[end_bracket + 1] == '(' {
                        let url_start = end_bracket + 2;
                        let mut url_end = url_start;
                        let mut paren_depth: usize = 1;
                        while url_end < len {
                            if chars[url_end] == '(' {
                                paren_depth += 1;
                            } else if chars[url_end] == ')' {
                                paren_depth -= 1;
                                if paren_depth == 0 {
                                    break;
                                }
                            }
                            url_end += 1;
                        }
                        let link_text: String = chars[i + 1..end_bracket].iter().collect();
                        let _url: String = chars[url_start..url_end].iter().collect();
                        flush_current!();
                        let link_style = Style::default()
                            .fg(theme.get_primary_color())
                            .add_modifier(Modifier::UNDERLINED);
                        if link_text.is_empty() {
                            spans.push(Span::styled(_url, link_style));
                        } else {
                            spans.push(Span::styled(link_text, link_style));
                        }
                        i = url_end + 1;
                        found_link = true;
                        break;
                    } else {
                        break;
                    }
                }
                end_bracket += 1;
            }
            if found_link {
                continue;
            }
        }

        current.push(chars[i]);
        i += 1;
    }

    if !current.is_empty() {
        spans.push(Span::styled(
            current,
            Style::default().fg(theme.get_text_color()),
        ));
    }

    spans
}
