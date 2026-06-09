use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use unicode_width::UnicodeWidthChar;

use crate::{markdown::MarkdownRenderer, theme::RichTextTheme};

pub struct MarkdownViewer {
    content: String,
    lines: Vec<Line<'static>>,
    scroll: u16,
    doc_h: u16,
    content_h: u16,
    cached_width: usize,
    title: String,
    key_hints: String,
    max_width: usize,
}

impl Default for MarkdownViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownViewer {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            lines: Vec::new(),
            scroll: 0,
            doc_h: 0,
            content_h: 0,
            cached_width: 0,
            title: String::new(),
            key_hints: String::new(),
            max_width: 0,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_key_hints(mut self, hints: impl Into<String>) -> Self {
        self.key_hints = hints.into();
        self
    }

    pub fn with_max_width(mut self, width: usize) -> Self {
        self.max_width = width;
        self
    }

    pub fn set_content(&mut self, content: &str, theme: &impl RichTextTheme) {
        if self.content != content {
            self.content = content.to_string();
            self.lines.clear();
            self.cached_width = 0;
        }
        self.ensure_rendered(theme);
    }

    pub fn scroll_up(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    pub fn scroll_down(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_add(n);
        self.clamp_scroll();
    }

    pub fn page_up(&mut self) {
        let step = self.content_h.max(1);
        self.scroll = self.scroll.saturating_sub(step);
    }

    pub fn page_down(&mut self) {
        let step = self.content_h.max(1);
        self.scroll = self.scroll.saturating_add(step);
        self.clamp_scroll();
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll = self.doc_h.saturating_sub(self.content_h);
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &impl RichTextTheme) {
        self.ensure_rendered(theme);

        let block_area = Rect::new(
            area.x,
            area.y,
            area.width,
            area.height
                .saturating_sub(if self.key_hints.is_empty() { 0 } else { 1 }),
        );

        let mut block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::new(1, 1, 0, 0));

        if !self.title.is_empty() {
            block = block.title(format!(" {} ", self.title));
        }

        let inner = block.inner(block_area);
        self.content_h = inner.height;
        let inner_w = inner.width as usize;
        self.clamp_scroll();

        let need_scrollbar = self.doc_h > self.content_h && self.content_h > 0;
        let content_w = if need_scrollbar {
            inner_w.saturating_sub(1)
        } else {
            inner_w
        };

        let scroll = self.scroll as usize;
        let visible = self.content_h as usize;
        let blank = Line::from(Span::raw(" ".repeat(content_w)));
        let mut padded: Vec<Line<'static>> = Vec::with_capacity(visible);

        for i in scroll..scroll.saturating_add(visible).min(self.lines.len()) {
            let spans = self.lines[i].spans.clone();
            let used: usize = spans.iter().map(|s| s.width()).sum();
            if used < content_w {
                let mut s = spans;
                s.push(Span::raw(" ".repeat(content_w - used)));
                padded.push(Line::from(s));
            } else if used > content_w {
                let mut taken = 0usize;
                let mut short: Vec<Span<'static>> = Vec::new();
                for sp in spans {
                    let sp_w = sp.width();
                    if taken + sp_w > content_w {
                        let keep = content_w.saturating_sub(taken);
                        let mut chop = String::new();
                        let mut chop_w = 0usize;
                        for ch in sp.content.chars() {
                            let cw = UnicodeWidthChar::width(ch).unwrap_or(0);
                            if chop_w + cw > keep {
                                break;
                            }
                            chop.push(ch);
                            chop_w += cw;
                        }
                        short.push(Span::styled(chop, sp.style));
                        taken += chop_w;
                        break;
                    }
                    taken += sp_w;
                    short.push(sp);
                }
                while taken < content_w {
                    short.push(Span::raw(" "));
                    taken += 1;
                }
                padded.push(Line::from(short));
            } else {
                padded.push(Line::from(spans));
            }
        }
        while padded.len() < visible {
            padded.push(blank.clone());
        }

        f.render_widget(block, block_area);
        f.render_widget(Paragraph::new(padded), inner);

        if need_scrollbar {
            let sb_col = inner.x + inner.width.saturating_sub(1);
            let sb_area = Rect::new(sb_col, inner.y, 1, self.content_h);
            let content_len = self.doc_h.saturating_sub(self.content_h).saturating_add(1);
            let sb = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .thumb_symbol("\u{2588}")
                .track_symbol(Some("\u{2502}"))
                .style(Style::default().fg(Color::DarkGray))
                .thumb_style(Style::default().fg(Color::Cyan));
            let mut sb_state = ScrollbarState::default()
                .content_length(content_len as usize)
                .viewport_content_length(self.content_h as usize)
                .position(self.scroll as usize);
            f.render_stateful_widget(sb, sb_area, &mut sb_state);
        }

        if !self.key_hints.is_empty() {
            let info_area = Rect::new(area.x, area.y + area.height.saturating_sub(1), area.width, 1);
            f.render_widget(
                Paragraph::new(vec![Line::from(Span::styled(
                    format!(" {}", self.key_hints),
                    Style::default().fg(Color::DarkGray),
                ))]),
                info_area,
            );
        }
    }

    fn ensure_rendered(&mut self, theme: &impl RichTextTheme) {
        if self.lines.is_empty() && !self.content.is_empty() {
            let w = if self.max_width > 0 {
                self.max_width
            } else {
                76
            };
            let renderer = MarkdownRenderer::new(w);
            let blocks = renderer.parse(&self.content);
            self.lines = renderer.render(&blocks, theme);
            self.doc_h = self.lines.len() as u16;
            self.clamp_scroll();
        }
    }

    fn clamp_scroll(&mut self) {
        let max = self.doc_h.saturating_sub(self.content_h);
        if self.scroll > max {
            self.scroll = max;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThemeConfig;

    fn test_theme() -> ThemeConfig {
        ThemeConfig::default()
    }

    #[test]
    fn new_viewer_is_empty() {
        let v = MarkdownViewer::new();
        assert!(v.content.is_empty());
        assert!(v.lines.is_empty());
        assert_eq!(v.scroll, 0);
    }

    #[test]
    fn with_title_and_hints() {
        let v = MarkdownViewer::new()
            .with_title("Test")
            .with_key_hints("j/k: scroll");
        assert_eq!(v.title, "Test");
        assert_eq!(v.key_hints, "j/k: scroll");
    }

    #[test]
    fn set_content_renders() {
        let mut v = MarkdownViewer::new();
        let theme = test_theme();
        v.set_content("# Hello\n\nWorld", &theme);
        assert!(!v.lines.is_empty());
        assert!(v.doc_h > 0);
    }

    #[test]
    fn scroll_clamping() {
        let mut v = MarkdownViewer::new();
        let theme = test_theme();
        v.set_content("# Hello\n\nLine1\n\nLine2\n\nLine3", &theme);
        v.scroll_down(100);
        assert!(v.scroll > 0);
        v.scroll_up(1);
        assert!(v.scroll > 0);
    }

    #[test]
    fn page_scroll() {
        let mut v = MarkdownViewer::new();
        let theme = test_theme();
        v.set_content("# Title\n\nLine1\n\nLine2\n\nLine3\n\nLine4", &theme);
        v.scroll_down(2);
        assert_eq!(v.scroll, 2);
        v.page_up();
        assert!(v.scroll <= 2);
    }

    #[test]
    fn scroll_to_top_and_bottom() {
        let mut v = MarkdownViewer::new();
        let theme = test_theme();
        v.set_content("# Title\n\nLine1\n\nLine2\n\nLine3", &theme);
        v.scroll_down(10);
        assert!(v.scroll > 0);
        v.scroll_to_top();
        assert_eq!(v.scroll, 0);
    }

    #[test]
    fn duplicate_content_does_not_rerender() {
        let mut v = MarkdownViewer::new();
        let theme = test_theme();
        v.set_content("# Hello", &theme);
        let line_count = v.lines.len();
        v.set_content("# Hello", &theme);
        assert_eq!(v.lines.len(), line_count);
    }

    #[test]
    fn with_max_width() {
        let v = MarkdownViewer::new().with_max_width(40);
        assert_eq!(v.max_width, 40);
    }
}
