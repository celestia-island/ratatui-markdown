use std::sync::Arc;

use ratatui::text::Line;

use super::CodeHighlighter;

pub struct HighlightHooks {
    highlighter: Arc<dyn CodeHighlighter>,
    max_width: usize,
    prefix: String,
}

impl HighlightHooks {
    pub fn new(highlighter: Arc<dyn CodeHighlighter>, max_width: usize) -> Self {
        Self {
            highlighter,
            max_width,
            prefix: String::new(),
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
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
        Some(super::segment::segments_to_lines(
            content,
            &segments,
            &self.prefix,
            self.max_width,
        ))
    }
}
