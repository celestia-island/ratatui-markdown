use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthChar;

use super::{inline::parse_inline_formatting, types::TextToken, MarkdownRenderer};
use crate::theme::RichTextTheme;

impl MarkdownRenderer {
    pub fn wrap_text_with_inline_formatting(
        &self,
        text: &str,
        theme: &impl RichTextTheme,
    ) -> Vec<Line<'static>> {
        let spans = parse_inline_formatting(text, theme);
        self.wrap_styled_spans_to_lines(spans)
    }

    fn wrap_styled_spans_to_lines(&self, spans: Vec<Span<'static>>) -> Vec<Line<'static>> {
        if self.max_width == 0 {
            return if spans.is_empty() {
                vec![Line::raw("")]
            } else {
                vec![Line::from(spans)]
            };
        }

        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut current_line: Vec<Span<'static>> = Vec::new();
        let mut current_width: usize = 0;
        let mut pending_space = false;

        for span in spans {
            let style = span.style;
            let text = span.content.to_string();
            let tokens = Self::tokenize(&text);

            for token in tokens {
                match token {
                    TextToken::Newline => {
                        lines.push(Line::from(std::mem::take(&mut current_line)));
                        current_width = 0;
                        pending_space = false;
                    }
                    TextToken::Space => {
                        pending_space = true;
                    }
                    TextToken::Word(word) => {
                        let word_w = Self::string_width(&word);
                        let space_w: usize = if pending_space && current_width > 0 {
                            1
                        } else {
                            0
                        };

                        let needs_wrap = if current_width == 0 {
                            false
                        } else if space_w > 0 && current_width + space_w >= self.max_width {
                            true
                        } else {
                            current_width + space_w + word_w > self.max_width
                        };

                        if needs_wrap {
                            lines.push(Line::from(std::mem::take(&mut current_line)));
                            current_width = 0;
                            pending_space = false;
                        }

                        if pending_space && current_width > 0 {
                            current_line.push(Span::styled(" ", style));
                            current_width += 1;
                        }
                        pending_space = false;

                        if word_w > self.max_width {
                            let mut buf = String::new();
                            let mut buf_w = 0usize;
                            for ch in word.chars() {
                                let cw = Self::display_width(ch);
                                if current_width + buf_w + cw > self.max_width
                                    && (current_width > 0 || buf_w > 0)
                                {
                                    if !buf.is_empty() {
                                        current_line.push(Span::styled(buf.clone(), style));
                                        buf.clear();
                                    }
                                    lines.push(Line::from(std::mem::take(&mut current_line)));
                                    current_width = 0;
                                    buf_w = 0;
                                }
                                buf.push(ch);
                                buf_w += cw;
                            }
                            if !buf.is_empty() {
                                current_line.push(Span::styled(buf, style));
                                current_width += buf_w;
                            }
                        } else {
                            current_line.push(Span::styled(word, style));
                            current_width += word_w;
                        }
                    }
                }
            }
        }

        if !current_line.is_empty() {
            lines.push(Line::from(current_line));
        }

        if lines.is_empty() {
            lines.push(Line::raw(""));
        }

        lines
    }

    pub fn wrap_text_simple(&self, text: &str) -> Vec<String> {
        if self.max_width == 0 {
            return vec![text.to_string()];
        }

        let tokens = Self::tokenize(text);
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;
        let mut pending_space = false;

        for token in tokens {
            match token {
                TextToken::Newline => {
                    lines.push(Self::trim_line_end(&current_line));
                    current_line.clear();
                    current_width = 0;
                    pending_space = false;
                }
                TextToken::Space => {
                    pending_space = true;
                }
                TextToken::Word(word) => {
                    let word_width = Self::string_width(&word);
                    let space_width = if pending_space { 1 } else { 0 };
                    let total_width = word_width + space_width;

                    let needs_wrap = if current_width == 0 {
                        false
                    } else if current_width + space_width >= self.max_width {
                        true
                    } else {
                        current_width + total_width > self.max_width
                    };

                    if needs_wrap && !current_line.is_empty() {
                        lines.push(Self::trim_line_end(&current_line));
                        current_line.clear();
                        current_width = 0;
                        pending_space = false;
                    }

                    if pending_space && current_width > 0 {
                        current_line.push(' ');
                        current_width += 1;
                    }
                    pending_space = false;

                    if word_width > self.max_width {
                        for ch in word.chars() {
                            let ch_width = Self::display_width(ch);
                            if current_width + ch_width > self.max_width && !current_line.is_empty()
                            {
                                lines.push(Self::trim_line_end(&current_line));
                                current_line.clear();
                                current_width = 0;
                            }
                            current_line.push(ch);
                            current_width += ch_width;
                        }
                    } else {
                        current_line.push_str(&word);
                        current_width += word_width;
                    }
                }
            }
        }

        if !current_line.is_empty() {
            lines.push(Self::trim_line_end(&current_line));
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    pub(crate) fn tokenize(text: &str) -> Vec<TextToken> {
        let text = text.replace('\t', "    ");
        let mut tokens = Vec::new();
        let mut current_word = String::new();

        for c in text.chars() {
            if c == '\n' {
                if !current_word.is_empty() {
                    tokens.push(TextToken::Word(current_word));
                    current_word = String::new();
                }
                tokens.push(TextToken::Newline);
                continue;
            }

            if c == ' ' {
                if !current_word.is_empty() {
                    tokens.push(TextToken::Word(current_word));
                    current_word = String::new();
                }
                tokens.push(TextToken::Space);
                continue;
            }

            let is_cjk = Self::is_cjk(c);

            if is_cjk {
                if !current_word.is_empty() {
                    tokens.push(TextToken::Word(current_word));
                    current_word = String::new();
                }
                tokens.push(TextToken::Word(c.to_string()));
            } else {
                current_word.push(c);
            }
        }

        if !current_word.is_empty() {
            tokens.push(TextToken::Word(current_word));
        }

        tokens
    }

    pub(crate) fn display_width(c: char) -> usize {
        UnicodeWidthChar::width(c).unwrap_or(0)
    }

    pub(crate) fn string_width(s: &str) -> usize {
        s.chars().map(Self::display_width).sum()
    }

    fn trim_line_end(line: &str) -> String {
        line.trim_end_matches(char::is_whitespace).to_string()
    }

    fn is_cjk(c: char) -> bool {
        let cp = c as u32;
        (0x4E00..=0x9FFF).contains(&cp)
            || (0x3400..=0x4DBF).contains(&cp)
            || (0x20000..=0x2CEAF).contains(&cp)
            || (0x3040..=0x309F).contains(&cp)
            || (0x30A0..=0x30FF).contains(&cp)
            || (0xAC00..=0xD7AF).contains(&cp)
            || (0x1100..=0x11FF).contains(&cp)
    }
}
