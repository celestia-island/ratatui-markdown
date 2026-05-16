use ratatui::{layout::Rect, text::Line, Frame};

#[cfg(feature = "markdown")]
use crate::markdown::MarkdownRenderer;
use crate::theme::RichTextTheme;

#[cfg(feature = "markdown")]
pub(super) fn render_read_mode(
    text: &str,
    f: &mut Frame,
    area: Rect,
    scroll_offset: usize,
    theme: &impl RichTextTheme,
) {
    let renderer = MarkdownRenderer::new(area.width as usize);
    let blocks = renderer.parse(text);
    let all_lines = renderer.render(&blocks, theme);
    let visible_height = area.height as usize;
    let skipped: Vec<Line<'static>> = all_lines
        .into_iter()
        .skip(scroll_offset)
        .take(visible_height)
        .collect();
    let paragraph = ratatui::widgets::Paragraph::new(skipped);
    f.render_widget(paragraph, area);
}

#[cfg(feature = "markdown")]
pub(super) fn rendered_height(text: &str, width: usize, theme: &impl RichTextTheme) -> u16 {
    let renderer = MarkdownRenderer::new(width);
    let blocks = renderer.parse(text);
    let lines = renderer.render(&blocks, theme);
    lines.len().min(u16::MAX as usize) as u16
}
