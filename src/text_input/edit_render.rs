use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::theme::RichTextTheme;

pub fn render_edit_mode(
    text: &str,
    cursor_char_idx: usize,
    horizontal_scroll: usize,
    max_width: usize,
    password: bool,
    placeholder: Option<&str>,
    theme: &impl RichTextTheme,
) -> Vec<Line<'static>> {
    if text.is_empty() {
        let display = placeholder.unwrap_or("");
        return vec![Line::from(vec![Span::styled(
            display.to_string(),
            Style::default().fg(theme.get_muted_text_color()),
        )])];
    }

    if password {
        let masked = "*".repeat(text.chars().count());
        return vec![Line::from(Span::styled(
            masked,
            Style::default().fg(theme.get_text_color()),
        ))];
    }

    let cursor_line_idx = char_offset_to_line(text, cursor_char_idx);
    let mut lines: Vec<Line<'static>> = Vec::new();

    for (line_idx, raw_line) in text.split('\n').enumerate() {
        let spans = style_source_spans(raw_line, theme);
        let line = Line::from(spans);
        if line_idx == cursor_line_idx {
            lines.push(apply_horizontal_scroll(&line, horizontal_scroll, max_width));
        } else {
            lines.push(line);
        }
    }

    lines
}

pub(in crate::text_input) fn char_offset_to_line(text: &str, char_idx: usize) -> usize {
    let mut offset = 0usize;
    for (i, line) in text.split('\n').enumerate() {
        let line_len = line.chars().count();
        if offset + line_len >= char_idx {
            return i;
        }
        offset += line_len + 1;
    }
    text.split('\n').count().saturating_sub(1)
}

pub(in crate::text_input) fn char_offset_to_line_col(
    text: &str,
    char_idx: usize,
) -> (usize, usize) {
    let mut offset = 0usize;
    for (i, line) in text.split('\n').enumerate() {
        let line_len = line.chars().count();
        if offset + line_len >= char_idx {
            return (i, char_idx - offset);
        }
        offset += line_len + 1;
    }
    let last_line_len = text
        .split('\n')
        .next_back()
        .map(|l| l.chars().count())
        .unwrap_or(0);
    let num_lines = text.split('\n').count().saturating_sub(1);
    (num_lines, last_line_len)
}

pub(in crate::text_input) fn expanded_display_col(raw_line: &str, raw_col: usize) -> usize {
    let mut display_col = 0usize;
    for (i, ch) in raw_line.chars().enumerate() {
        if i >= raw_col {
            break;
        }
        if ch == '\t' {
            display_col += 4;
        } else {
            display_col += 1;
        }
    }
    display_col
}

pub(in crate::text_input) fn line_col_to_char_offset(
    text: &str,
    line_idx: usize,
    col: usize,
) -> usize {
    let mut offset = 0usize;
    for (i, line) in text.split('\n').enumerate() {
        if i == line_idx {
            return offset + col.min(line.chars().count());
        }
        offset += line.chars().count() + 1;
    }
    text.chars().count()
}

fn style_source_spans(text: &str, theme: &impl RichTextTheme) -> Vec<Span<'static>> {
    let expanded = text.replace('\t', "    ");
    let mut spans: Vec<Span<'static>> = Vec::new();
    let chars: Vec<char> = expanded.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut current = String::new();
    let text_color = theme.get_text_color();
    let muted_color = theme.get_muted_text_color();

    macro_rules! flush_current {
        () => {
            if !current.is_empty() {
                spans.push(Span::styled(
                    current.clone(),
                    Style::default().fg(text_color),
                ));
                current.clear();
            }
        };
    }

    while i < len {
        if chars[i] == '#' && (i == 0 || chars[i - 1] == '\n') {
            flush_current!();
            let mut hash_count = 0;
            let start = i;
            while i < len && chars[i] == '#' {
                hash_count += 1;
                i += 1;
            }
            if i < len && chars[i] == ' ' {
                i += 1;
                let hashes: String = chars[start..start + hash_count].iter().collect();
                let space = " ".to_string();
                spans.push(Span::styled(
                    hashes,
                    Style::default()
                        .fg(theme.get_primary_color())
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(
                    space,
                    Style::default().fg(theme.get_primary_color()),
                ));
                let rest: String = chars[i..].iter().take_while(|c| **c != '\n').collect();
                if !rest.is_empty() {
                    spans.push(Span::styled(
                        rest.clone(),
                        Style::default().fg(text_color).add_modifier(Modifier::BOLD),
                    ));
                    i += rest.chars().count();
                }
                continue;
            } else {
                let hashes: String = chars[start..i].iter().collect();
                current.push_str(&hashes);
                continue;
            }
        }

        if chars[i] == '*' && i + 2 < len && chars[i + 1] == '*' && chars[i + 2] == '*' {
            flush_current!();
            let start = i + 3;
            let mut found = false;
            let mut end = start;
            while end + 2 < len {
                if chars[end] == '*' && chars[end + 1] == '*' && chars[end + 2] == '*' {
                    let delim: String = chars[i..i + 3].iter().collect();
                    let content: String = chars[start..end].iter().collect();
                    spans.push(Span::styled(delim, Style::default().fg(muted_color)));
                    spans.push(Span::styled(
                        content.clone(),
                        Style::default()
                            .fg(text_color)
                            .add_modifier(Modifier::BOLD | Modifier::ITALIC),
                    ));
                    let delim_end: String = chars[end..end + 3].iter().collect();
                    spans.push(Span::styled(delim_end, Style::default().fg(muted_color)));
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
                    let delim: String = chars[i..i + 2].iter().collect();
                    let content: String = chars[start..end].iter().collect();
                    spans.push(Span::styled(delim, Style::default().fg(muted_color)));
                    spans.push(Span::styled(
                        content.clone(),
                        Style::default().fg(text_color).add_modifier(Modifier::BOLD),
                    ));
                    let delim_end: String = chars[end..end + 2].iter().collect();
                    spans.push(Span::styled(delim_end, Style::default().fg(muted_color)));
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
                    let delim = delimiter.to_string();
                    let content: String = chars[start..end].iter().collect();
                    spans.push(Span::styled(delim, Style::default().fg(muted_color)));
                    spans.push(Span::styled(
                        content.clone(),
                        Style::default()
                            .fg(text_color)
                            .add_modifier(Modifier::ITALIC),
                    ));
                    let delim_end = delimiter.to_string();
                    spans.push(Span::styled(delim_end, Style::default().fg(muted_color)));
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
            flush_current!();
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
            if found {
                let backtick = "`".to_string();
                let content: String = chars[start..end].iter().collect();
                let backtick_end = "`".to_string();
                spans.push(Span::styled(backtick, Style::default().fg(muted_color)));
                spans.push(Span::styled(
                    content.clone(),
                    Style::default().fg(theme.get_accent_yellow()),
                ));
                spans.push(Span::styled(backtick_end, Style::default().fg(muted_color)));
                i = end + 1;
            } else {
                current.push('`');
                i += 1;
            }
            continue;
        }

        if chars[i] == '~' && i + 1 < len && chars[i + 1] == '~' {
            flush_current!();
            let start = i + 2;
            let mut end = start;
            let mut found = false;
            while end + 1 < len {
                if chars[end] == '~' && chars[end + 1] == '~' {
                    let delim = "~~".to_string();
                    let content: String = chars[start..end].iter().collect();
                    spans.push(Span::styled(delim, Style::default().fg(muted_color)));
                    spans.push(Span::styled(
                        content.clone(),
                        Style::default()
                            .fg(text_color)
                            .add_modifier(Modifier::CROSSED_OUT),
                    ));
                    let delim_end = "~~".to_string();
                    spans.push(Span::styled(delim_end, Style::default().fg(muted_color)));
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

        if chars[i] == '[' {
            let mut end_bracket = i + 1;
            let mut found_link = false;
            while end_bracket < len {
                if chars[end_bracket] == ']' {
                    if end_bracket + 1 < len && chars[end_bracket + 1] == '(' {
                        let url_start = end_bracket + 2;
                        let mut url_end = url_start;
                        while url_end < len {
                            if chars[url_end] == ')' {
                                let bracket_open = "[".to_string();
                                let link_text: String = chars[i + 1..end_bracket].iter().collect();
                                let bracket_close_paren_open = "](".to_string();
                                let url_text: String = chars[url_start..url_end].iter().collect();
                                let paren_close = ")".to_string();

                                flush_current!();
                                spans.push(Span::styled(
                                    bracket_open,
                                    Style::default().fg(muted_color),
                                ));
                                spans.push(Span::styled(
                                    link_text,
                                    Style::default()
                                        .fg(theme.get_primary_color())
                                        .add_modifier(Modifier::UNDERLINED),
                                ));
                                spans.push(Span::styled(
                                    bracket_close_paren_open,
                                    Style::default().fg(muted_color),
                                ));
                                spans
                                    .push(Span::styled(url_text, Style::default().fg(muted_color)));
                                spans.push(Span::styled(
                                    paren_close,
                                    Style::default().fg(muted_color),
                                ));
                                i = url_end + 1;
                                found_link = true;
                                break;
                            }
                            url_end += 1;
                        }
                    }
                    break;
                }
                end_bracket += 1;
            }
            if found_link {
                continue;
            }
        }

        if chars[i] == '>' && (i == 0 || chars[i - 1] == '\n') {
            flush_current!();
            spans.push(Span::styled(
                ">",
                Style::default().fg(theme.get_info_color()),
            ));
            i += 1;
            if i < len && chars[i] == ' ' {
                spans.push(Span::styled(
                    " ",
                    Style::default().fg(theme.get_info_color()),
                ));
                i += 1;
            }
            let rest: String = chars[i..].iter().take_while(|c| **c != '\n').collect();
            if !rest.is_empty() {
                spans.push(Span::styled(
                    rest.clone(),
                    Style::default()
                        .fg(text_color)
                        .add_modifier(Modifier::ITALIC),
                ));
                i += rest.chars().count();
            }
            continue;
        }

        if (chars[i] == '-' || chars[i] == '*')
            && i + 1 < len
            && chars[i + 1] == ' '
            && (i == 0 || chars[i - 1] == '\n')
        {
            flush_current!();
            let marker = chars[i].to_string();
            let space = " ".to_string();
            spans.push(Span::styled(marker, Style::default().fg(muted_color)));
            spans.push(Span::styled(space, Style::default().fg(muted_color)));
            i += 2;
            continue;
        }

        if chars[i] == '`' && i + 2 < len && chars[i + 1] == '`' && chars[i + 2] == '`' {
            flush_current!();
            let fence_start = i;
            let mut fence_end = fence_start + 3;
            while fence_end + 2 < len {
                if chars[fence_end] == '`'
                    && chars[fence_end + 1] == '`'
                    && chars[fence_end + 2] == '`'
                {
                    break;
                }
                fence_end += 1;
            }
            let fence_line: String = if fence_end + 2 < len {
                chars[fence_start..=fence_end + 2].iter().collect()
            } else {
                chars[fence_start..].iter().collect()
            };
            spans.push(Span::styled(
                fence_line,
                Style::default().fg(theme.get_secondary_color()),
            ));
            i = if fence_end + 2 < len {
                fence_end + 3
            } else {
                len
            };
            continue;
        }

        current.push(chars[i]);
        i += 1;
    }

    flush_current!();
    spans
}

fn apply_horizontal_scroll(line: &Line<'_>, scroll: usize, max_width: usize) -> Line<'static> {
    let mut skip = scroll;
    let mut result: Vec<Span<'static>> = Vec::new();
    let mut collected = 0usize;

    for span in &line.spans {
        let span_text: String = span.content.clone().into();
        let w = unicode_width::UnicodeWidthStr::width(span_text.as_str());
        if skip > 0 {
            if w <= skip {
                skip -= w;
                continue;
            }
            let chars: Vec<char> = span_text.chars().collect();
            let mut ci = 0;
            while ci < chars.len() && skip > 0 {
                let cw = unicode_width::UnicodeWidthChar::width(chars[ci]).unwrap_or(0);
                skip = skip.saturating_sub(cw);
                ci += 1;
            }
            let remaining: String = chars[ci..].iter().collect();
            let rw = unicode_width::UnicodeWidthStr::width(remaining.as_str());
            if collected + rw <= max_width {
                result.push(Span::styled(remaining, span.style));
                collected += rw;
            } else {
                let trunc = truncate_to_width(&remaining, max_width - collected);
                let tw = unicode_width::UnicodeWidthStr::width(trunc.as_str());
                result.push(Span::styled(trunc, span.style));
                collected += tw;
            }
        } else if collected + w <= max_width {
            result.push(Span::styled(span_text, span.style));
            collected += w;
        } else {
            let trunc = truncate_to_width(&span_text, max_width - collected);
            let tw = unicode_width::UnicodeWidthStr::width(trunc.as_str());
            result.push(Span::styled(trunc, span.style));
            collected += tw;
        }

        if collected >= max_width {
            break;
        }
    }

    Line::from(result)
}

fn truncate_to_width(s: &str, max_w: usize) -> String {
    let mut result = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw > max_w {
            break;
        }
        result.push(ch);
        w += cw;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeConfig;

    fn theme() -> ThemeConfig {
        ThemeConfig::default()
    }

    #[test]
    fn empty_text_shows_nothing() {
        let lines = render_edit_mode("", 0, 0, 80, false, None, &theme());
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn plain_text_renders() {
        let lines = render_edit_mode("hello world", 0, 0, 80, false, None, &theme());
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn multiline_text_renders_multiple_lines() {
        let lines = render_edit_mode("hello\nworld", 5, 0, 80, false, None, &theme());
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn password_mode_masks_text() {
        let lines = render_edit_mode("secret", 0, 0, 80, true, None, &theme());
        assert_eq!(lines.len(), 1);
        let line = &lines[0];
        let content: String = line.spans.iter().map(|s| s.content.clone()).collect();
        assert_eq!(content, "******");
    }

    #[test]
    fn placeholder_shown_when_empty() {
        let lines = render_edit_mode("", 0, 0, 80, false, Some("type here"), &theme());
        let content: String = lines[0].spans.iter().map(|s| s.content.clone()).collect();
        assert_eq!(content, "type here");
    }

    #[test]
    fn bold_delimiters_styled() {
        let spans = style_source_spans("**bold**", &theme());
        assert!(spans.len() >= 3);
    }

    #[test]
    fn italic_delimiters_styled() {
        let spans = style_source_spans("*italic*", &theme());
        assert!(spans.len() >= 3);
    }

    #[test]
    fn inline_code_styled() {
        let spans = style_source_spans("`code`", &theme());
        assert!(spans.len() >= 3);
    }

    #[test]
    fn link_styled() {
        let spans = style_source_spans("[text](url)", &theme());
        assert!(spans.len() >= 5);
    }

    #[test]
    fn tab_expanded_to_spaces_in_style_source_spans() {
        let spans = style_source_spans("\thello", &theme());
        let content: String = spans.iter().map(|s| s.content.clone()).collect();
        assert!(
            !content.contains('\t'),
            "tab should be expanded to spaces: {:?}",
            content
        );
        assert!(
            content.starts_with("    "),
            "tab should expand to 4 spaces: {:?}",
            content
        );
    }

    #[test]
    fn expanded_display_col_with_tabs() {
        assert_eq!(expanded_display_col("abc", 3), 3);
        assert_eq!(expanded_display_col("\tbc", 1), 4);
        assert_eq!(expanded_display_col("\t\tc", 2), 8);
        assert_eq!(expanded_display_col("a\tc", 2), 5);
        assert_eq!(expanded_display_col("", 0), 0);
    }
}
