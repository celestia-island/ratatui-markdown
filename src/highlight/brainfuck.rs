use ratatui::style::{Color, Modifier, Style};

use super::{CodeHighlighter, StyleSegment};

pub struct BrainfuckHighlighter;

impl CodeHighlighter for BrainfuckHighlighter {
    fn highlight(&self, lang: &str, code: &str) -> Vec<StyleSegment> {
        if lang != "brainfuck" && lang != "bf" {
            return Vec::new();
        }

        let mut segments = Vec::new();
        let mut run_start: usize = 0;
        let mut prev: Option<Style> = None;

        for (i, ch) in code.char_indices() {
            let style = char_style(ch);
            if prev != Some(style) {
                if let Some(ps) = prev {
                    segments.push(StyleSegment {
                        start: run_start,
                        end: i,
                        style: ps,
                    });
                }
                run_start = i;
                prev = Some(style);
            }
        }

        if let Some(style) = prev {
            segments.push(StyleSegment {
                start: run_start,
                end: code.len(),
                style,
            });
        }

        segments
    }
}

fn char_style(ch: char) -> Style {
    match ch {
        '>' | '<' => Style::default().fg(Color::Cyan),
        '+' | '-' => Style::default().fg(Color::Green),
        '.' | ',' => Style::default().fg(Color::Yellow),
        '[' | ']' => Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
        _ => Style::default().fg(Color::DarkGray),
    }
}
