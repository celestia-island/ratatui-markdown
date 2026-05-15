use std::sync::Arc;

use ratatui::{
    style::Style,
    text::{Line, Span},
};

use super::CodeHighlighter;

use crate::constants::list_prefix::{HLINE, ROUNDED_BL, ROUNDED_TL, VLINE};

pub struct HighlightHooks {
    highlighter: Arc<dyn CodeHighlighter>,
    max_width: usize,
    border_color: ratatui::style::Color,
}

impl HighlightHooks {
    pub fn new(highlighter: Arc<dyn CodeHighlighter>, max_width: usize) -> Self {
        Self {
            highlighter,
            max_width,
            border_color: ratatui::style::Color::DarkGray,
        }
    }

    pub fn with_border_color(mut self, color: ratatui::style::Color) -> Self {
        self.border_color = color;
        self
    }
}

#[cfg(feature = "markdown")]
impl crate::markdown::RenderHooks for HighlightHooks {
    fn render_code_block(
        &self,
        lang: &str,
        content: &str,
    ) -> Option<Vec<Line<'static>>> {
        let segments = self.highlighter.highlight(lang, content);
        if segments.is_empty() {
            return None;
        }

        let border_style = Style::default().fg(self.border_color);
        let display_lang = if lang.is_empty() { "code" } else { lang };

        let mut lines = Vec::new();

        lines.push(Line::from(Span::styled(
            format!("{ROUNDED_TL}{HLINE} {display_lang}"),
            border_style,
        )));

        let prefix = format!("{VLINE} ");
        let content_width = self.max_width.saturating_sub(2);

        let code_lines = super::segment::segments_to_lines(
            content,
            &segments,
            &prefix,
            border_style,
            content_width,
        );
        lines.extend(code_lines);

        lines.push(Line::from(Span::styled(
            format!("{ROUNDED_BL}{HLINE}"),
            border_style,
        )));

        Some(lines)
    }
}
