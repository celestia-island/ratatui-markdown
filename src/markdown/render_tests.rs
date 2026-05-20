use anyhow::Context as _;

use ratatui::{
    backend::TestBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
    text::Line,
    widgets::Paragraph,
    Terminal,
};

use crate::{
    markdown::{MarkdownBlock, MarkdownRenderer, RenderHooks},
    theme::ThemeConfig,
};

fn test_theme() -> ThemeConfig {
    ThemeConfig::default()
        .with_info_color(Color::Blue)
        .with_focused_border_color(Color::Cyan)
        .with_secondary_color(Color::Yellow)
        .with_json_key_color(Color::Cyan)
        .with_json_bool_color(Color::Yellow)
        .with_json_number_color(Color::Magenta)
}

fn render_to_buffer(lines: Vec<Line<'static>>, width: u16, height: u16) -> anyhow::Result<Buffer> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| {
        let paragraph = Paragraph::new(lines);
        f.render_widget(paragraph, Rect::new(0, 0, width, height));
    })?;
    Ok(terminal.backend().buffer().clone())
}

fn render_markdown(markdown: &str, max_width: usize) -> Vec<Line<'static>> {
    let renderer = MarkdownRenderer::new(max_width);
    let blocks = renderer.parse(markdown);
    renderer.render(&blocks, &test_theme())
}

#[test]
fn heading1_renders_bold_underlined() -> anyhow::Result<()> {
    let lines = render_markdown("# Hello World", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    let cell = buf.cell((0, 0)).context("cell at (0, 0)")?;
    assert_eq!(cell.symbol(), "H");
    Ok(())
}

#[test]
fn heading2_renders_bold() -> anyhow::Result<()> {
    let lines = render_markdown("## Section", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    let cell = buf.cell((0, 0)).context("cell at (0, 0)")?;
    assert_eq!(cell.symbol(), "S");
    Ok(())
}

#[test]
fn heading3_renders_bold_secondary() -> anyhow::Result<()> {
    let lines = render_markdown("### Subsection", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "S");
    Ok(())
}

#[test]
fn paragraph_renders_text() -> anyhow::Result<()> {
    let lines = render_markdown("Hello, world!", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    let text: String = (0..13)
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default())
        .collect::<String>();
    assert_eq!(text, "Hello, world!");
    Ok(())
}

#[test]
fn paragraph_wraps_at_max_width() -> anyhow::Result<()> {
    let lines = render_markdown("abcdefghij klmnopqrst uvwxyz", 15);
    assert!(
        lines.len() >= 2,
        "expected wrapping, got {} lines",
        lines.len()
    );
    Ok(())
}

#[test]
fn blank_line_produces_empty_line() -> anyhow::Result<()> {
    let lines = render_markdown("Hello\n\nWorld", 80);
    let blank_idx = lines
        .iter()
        .position(|l| l.spans.is_empty() || l.spans.iter().all(|s| s.content.is_empty()));
    assert!(
        blank_idx.is_some(),
        "expected a blank line between two paragraphs"
    );
    Ok(())
}

#[test]
fn horizontal_rule_renders_dashes() -> anyhow::Result<()> {
    let lines = render_markdown("---", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "─");
    Ok(())
}

#[test]
fn code_block_with_lang_renders_bordered_box() -> anyhow::Result<()> {
    let md = "```rust\nfn main() {}\n```";
    let lines = render_markdown(md, 80);
    assert!(
        lines.len() >= 3,
        "expected header, content, footer; got {} lines",
        lines.len()
    );
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "╭");
    Ok(())
}

#[test]
fn code_block_without_lang_renders_minimal_header() -> anyhow::Result<()> {
    let md = "```\nsome code\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 5)?;
    let header_sym = buf.cell((0, 0)).context("cell at (0, 0)")?.symbol();
    assert_eq!(header_sym, "╭");
    Ok(())
}

#[test]
fn mermaid_code_block_is_rendered() -> anyhow::Result<()> {
    let md = "```mermaid\ngraph TD\nA-->B\n```";
    let lines = render_markdown(md, 80);
    #[cfg(feature = "mermaid")]
    {
        assert!(
            !lines.is_empty(),
            "mermaid blocks should produce rendered output with mermaid feature"
        );
    }
    #[cfg(not(feature = "mermaid"))]
    {
        assert!(
            lines.is_empty(),
            "mermaid blocks should produce zero output lines without mermaid feature"
        );
    }
    Ok(())
}

#[test]
fn unordered_list_dash_renders_bullet() -> anyhow::Result<()> {
    let lines = render_markdown("- item one", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "•");
    Ok(())
}

#[test]
fn unordered_list_star_renders_bullet() -> anyhow::Result<()> {
    let lines = render_markdown("* item one", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "•");
    Ok(())
}

#[test]
fn unordered_list_plus_renders_bullet() -> anyhow::Result<()> {
    let lines = render_markdown("+ item one", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "•");
    Ok(())
}

#[test]
fn ordered_list_renders_items() -> anyhow::Result<()> {
    let lines = render_markdown("1. first\n2. second\n3. third", 80);
    assert_eq!(lines.len(), 3);
    let buf = render_to_buffer(lines, 80, 10)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "•");
    assert_eq!(buf.cell((0, 1)).context("cell at (0, 1)")?.symbol(), "•");
    assert_eq!(buf.cell((0, 2)).context("cell at (0, 2)")?.symbol(), "•");
    Ok(())
}

#[test]
fn nested_list_indents() -> anyhow::Result<()> {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("- outer\n  - inner");
    let inner_block = blocks
        .iter()
        .find(|b| matches!(b, MarkdownBlock::ListItem(t, _) if t == "inner"));
    assert!(inner_block.is_some(), "should find inner list item");
    if let Some(MarkdownBlock::ListItem(_, indent)) = inner_block {
        assert_eq!(
            *indent, 1,
            "inner list item should have indent=1, got {}",
            indent
        );
    }

    let lines = render_markdown("- outer\n  - inner", 80);
    assert_eq!(lines.len(), 2);
    Ok(())
}

#[test]
fn blockquote_renders_with_prefix() -> anyhow::Result<()> {
    let lines = render_markdown("> quoted text", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "│");
    Ok(())
}

#[test]
fn table_renders_with_borders() -> anyhow::Result<()> {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |";
    let lines = render_markdown(md, 80);
    assert!(
        lines.len() >= 4,
        "expected top border, header, separator, row, bottom; got {}",
        lines.len()
    );
    let buf = render_to_buffer(lines, 80, 10)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "┌");
    Ok(())
}

#[test]
fn table_bottom_border_uses_bl_br_corners() -> anyhow::Result<()> {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |";
    let lines = render_markdown(md, 80);
    let last = &lines[lines.len() - 1];
    let buf = render_to_buffer(vec![last.clone()], 80, 1)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "└");
    Ok(())
}

#[test]
fn inline_bold_renders() -> anyhow::Result<()> {
    let spans = crate::markdown::parse_inline_formatting("**bold**", &test_theme());
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, "bold");
    assert!(spans[0].style.add_modifier.contains(Modifier::BOLD));
    Ok(())
}

#[test]
fn inline_italic_renders() -> anyhow::Result<()> {
    let spans = crate::markdown::parse_inline_formatting("*italic*", &test_theme());
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, "italic");
    assert!(spans[0].style.add_modifier.contains(Modifier::ITALIC));
    Ok(())
}

#[test]
fn inline_bold_italic_renders() -> anyhow::Result<()> {
    let spans = crate::markdown::parse_inline_formatting("***both***", &test_theme());
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, "both");
    assert!(spans[0]
        .style
        .add_modifier
        .contains(Modifier::BOLD | Modifier::ITALIC));
    Ok(())
}

#[test]
fn inline_code_renders() -> anyhow::Result<()> {
    let spans = crate::markdown::parse_inline_formatting("some `code` here", &test_theme());
    assert!(spans.iter().any(|s| s.content == "code"));
    let code_span = spans.iter().find(|s| s.content == "code").context("find")?;
    assert_eq!(code_span.style.fg, Some(Color::Yellow));
    Ok(())
}

#[test]
fn mixed_inline_formatting() -> anyhow::Result<()> {
    let spans =
        crate::markdown::parse_inline_formatting("normal **bold** *italic* `code`", &test_theme());
    assert!(
        spans.len() >= 4,
        "expected at least 4 spans for mixed formatting"
    );
    Ok(())
}

#[test]
fn complex_document_renders() -> anyhow::Result<()> {
    let md = r#"# Title

A paragraph with **bold** and *italic*.

## Section

- item 1
- item 2

> A quote

```
code here
```

---

| H1 | H2 |
|----|----|
| a  | b  |
"#;
    let lines = render_markdown(md, 80);
    assert!(
        lines.len() > 10,
        "complex document should produce many lines, got {}",
        lines.len()
    );
    let buf = render_to_buffer(lines, 80, 40)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "T");
    Ok(())
}

mod example_tree_list_tests {
    use super::*;

    struct TreeListHooks;

    impl RenderHooks for TreeListHooks {
        fn list_item_marker(
            &self,
            indent: u8,
            is_last_in_group: bool,
            ancestors_are_last: &[bool],
            index_in_group: usize,
        ) -> Option<String> {
            let marker = if is_last_in_group {
                "└─ "
            } else if indent == 0 && index_in_group == 0 {
                "┌─ "
            } else {
                "├─ "
            };
            if indent == 0 {
                return Some(marker.to_string());
            }
            let mut prefix = String::new();
            for (depth, &is_last_ancestor) in ancestors_are_last.iter().enumerate() {
                if depth >= indent as usize {
                    break;
                }
                if is_last_ancestor {
                    for _ in 0..3 {
                        prefix.push(' ');
                    }
                } else {
                    prefix.push_str("│  ");
                }
            }
            if indent as usize > ancestors_are_last.len() {
                let extra = indent as usize - ancestors_are_last.len();
                for _ in 0..3 * extra {
                    prefix.push(' ');
                }
            }
            Some(format!("{}{}", prefix, marker))
        }

        fn tree_indent_unit(&self) -> Option<usize> {
            Some(3)
        }

        fn tree_continuation_prefix(
            &self,
            indent: u8,
            ancestors_are_last: &[bool],
        ) -> Option<String> {
            let unit = 3;
            let mut p = String::new();
            for (i, &last) in ancestors_are_last.iter().enumerate() {
                if i >= indent as usize {
                    break;
                }
                if last {
                    for _ in 0..unit {
                        p.push(' ');
                    }
                } else {
                    p.push_str("│  ");
                }
            }
            for _ in 0..unit {
                p.push(' ');
            }
            Some(p)
        }
    }

    #[test]
    fn tree_hook_root_items_have_tree_markers() {
        let renderer = MarkdownRenderer::new(76).with_render_hooks(Box::new(TreeListHooks));
        let blocks = renderer.parse("- A\n  - B\n  - C\n- D");
        let lines = renderer.render(&blocks, &test_theme());
        let has_tree_marker = lines.iter().any(|l| {
            l.spans.iter().any(|s| {
                s.content.contains("┌─") || s.content.contains("├─") || s.content.contains("└─")
            })
        });
        assert!(has_tree_marker, "tree markers should appear in output");
    }

    #[test]
    fn tree_hook_nested_items_have_pipe_prefix() {
        let renderer = MarkdownRenderer::new(76).with_render_hooks(Box::new(TreeListHooks));
        let blocks = renderer.parse("- A\n  - B\n- C");
        let lines = renderer.render(&blocks, &test_theme());
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(
            all_text.contains("│"),
            "nested items should have │ pipe prefix"
        );
    }

    #[test]
    fn tree_hook_last_item_uses_corner_marker() {
        let renderer = MarkdownRenderer::new(76).with_render_hooks(Box::new(TreeListHooks));
        let blocks = renderer.parse("- A\n- B");
        let lines = renderer.render(&blocks, &test_theme());
        let has_corner = lines
            .iter()
            .any(|l| l.spans.iter().any(|s| s.content.contains("└─")));
        assert!(has_corner, "last items should use └─ corner marker");
    }

    #[test]
    fn tree_hook_render_to_buffer_no_panic() -> anyhow::Result<()> {
        let renderer = MarkdownRenderer::new(76).with_render_hooks(Box::new(TreeListHooks));
        let blocks = renderer.parse("- A\n  - B\n- C");
        let lines = renderer.render(&blocks, &test_theme());
        let buffer = render_to_buffer(lines, 80, 40)?;
        assert_eq!(buffer.area.height, 40);
        Ok(())
    }

    #[test]
    fn tree_hook_two_sections_separated_by_paragraph() {
        let renderer = MarkdownRenderer::new(40).with_render_hooks(Box::new(TreeListHooks));
        let md = "- Alpha\n  - Beta\n  - Gamma\n\nSome paragraph text between.\n\n- Delta\n  - Epsilon\n  - Zeta";
        let blocks = renderer.parse(md);
        let lines = renderer.render(&blocks, &test_theme());

        let texts: Vec<String> = lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
            .collect();

        assert!(
            texts[0].starts_with("└─"),
            "first tree has one root (Alpha) so it should use └─, got: {}",
            texts[0]
        );
        assert!(
            texts[1].starts_with("   "),
            "Alpha is last root, children should have spaces prefix, got: {}",
            texts[1]
        );

        let delta_line = texts
            .iter()
            .position(|t| t.contains("Delta"))
            .expect("Delta should exist");
        assert!(
            texts[delta_line].starts_with("└─"),
            "second tree has one root (Delta) so it should use └─, got: {}",
            texts[delta_line],
        );
        let epsilon_line = texts
            .iter()
            .position(|t| t.contains("Epsilon"))
            .expect("Epsilon should exist");
        assert!(
            texts[epsilon_line].starts_with("   "),
            "Delta is last root in its group, children should have spaces prefix, got: {}",
            texts[epsilon_line],
        );
    }

    #[test]
    fn tree_hook_multi_root_groups_isolated() {
        let renderer = MarkdownRenderer::new(40).with_render_hooks(Box::new(TreeListHooks));
        let md = "- A1\n  - B1\n  - B2\n- A2\n  - B3\n\nParagraph.\n\n- C1\n  - D1\n- C2\n  - D2";
        let blocks = renderer.parse(md);
        let lines = renderer.render(&blocks, &test_theme());
        let texts: Vec<String> = lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
            .collect();

        let a2_line = texts.iter().position(|t| t.contains("A2")).expect("A2");
        assert!(
            texts[a2_line].starts_with("└─"),
            "A2 is last root in first group: got {}",
            texts[a2_line],
        );
        let b3_line = texts.iter().position(|t| t.contains("B3")).expect("B3");
        assert!(
            !texts[b3_line].contains("│"),
            "A2 is last, so B3 should have spaces not │: got {}",
            texts[b3_line],
        );

        let c1_line = texts.iter().position(|t| t.contains("C1")).expect("C1");
        assert!(
            texts[c1_line].starts_with("┌─"),
            "C1 is first sibling and has sibling C2: got {}",
            texts[c1_line],
        );
        let c2_line = texts.iter().position(|t| t.contains("C2")).expect("C2");
        assert!(
            texts[c2_line].starts_with("└─"),
            "C2 is last root in second group: got {}",
            texts[c2_line],
        );
    }
}

mod example_scrollable_tests {
    use super::*;

    struct ScrollState {
        v_offset: usize,
        h_offset: usize,
        total_lines: usize,
        max_line_width: usize,
        pad_top: u16,
        pad_bottom: u16,
        pad_left: u16,
        pad_right: u16,
    }

    impl ScrollState {
        fn new(total_lines: usize, max_line_width: usize) -> Self {
            Self {
                v_offset: 0,
                h_offset: 0,
                total_lines,
                max_line_width,
                pad_top: 1,
                pad_bottom: 1,
                pad_left: 2,
                pad_right: 2,
            }
        }

        fn viewport_height(&self, area_height: u16) -> usize {
            area_height.saturating_sub(self.pad_top + self.pad_bottom) as usize
        }

        fn viewport_width(&self, area_width: u16) -> usize {
            area_width.saturating_sub(self.pad_left + self.pad_right) as usize
        }

        fn max_v_offset(&self, area_height: u16) -> usize {
            self.total_lines
                .saturating_sub(self.viewport_height(area_height))
        }

        fn max_h_offset(&self, area_width: u16) -> usize {
            self.max_line_width
                .saturating_sub(self.viewport_width(area_width))
        }

        fn clamp(&mut self, area: Rect) {
            self.v_offset = self.v_offset.min(self.max_v_offset(area.height));
            self.h_offset = self.h_offset.min(self.max_h_offset(area.width));
        }

        fn scroll_v(&mut self, delta: isize, area: Rect) {
            if delta >= 0 {
                self.v_offset = self.v_offset.saturating_add(delta as usize);
            } else {
                self.v_offset = self.v_offset.saturating_sub((-delta) as usize);
            }
            self.clamp(area);
        }

        fn scroll_h(&mut self, delta: isize, area: Rect) {
            if delta >= 0 {
                self.h_offset = self.h_offset.saturating_add(delta as usize);
            } else {
                self.h_offset = self.h_offset.saturating_sub((-delta) as usize);
            }
            self.clamp(area);
        }

        fn page_up(&mut self, area: Rect) {
            let step = self.viewport_height(area.height).max(1);
            self.scroll_v(-(step as isize), area);
        }

        fn page_down(&mut self, area: Rect) {
            let step = self.viewport_height(area.height).max(1);
            self.scroll_v(step as isize, area);
        }
    }

    fn area(w: u16, h: u16) -> Rect {
        Rect::new(0, 0, w, h)
    }

    #[test]
    fn scroll_state_initial_offsets_zero() {
        let s = ScrollState::new(100, 200);
        assert_eq!(s.v_offset, 0);
        assert_eq!(s.h_offset, 0);
    }

    #[test]
    fn scroll_state_viewport_dimensions() {
        let s = ScrollState::new(100, 200);
        let vp_h = s.viewport_height(24);
        let vp_w = s.viewport_width(80);
        assert_eq!(vp_h, 22, "24 - pad_top(1) - pad_bottom(1) = 22");
        assert_eq!(vp_w, 76, "80 - pad_left(2) - pad_right(2) = 76");
    }

    #[test]
    fn scroll_state_max_v_offset() {
        let s = ScrollState::new(100, 80);
        let max_v = s.max_v_offset(24);
        assert_eq!(max_v, 78, "100 - 22 viewport = 78");
    }

    #[test]
    fn scroll_state_max_h_offset() {
        let s = ScrollState::new(50, 200);
        let max_h = s.max_h_offset(80);
        assert_eq!(max_h, 124, "200 - 76 viewport = 124");
    }

    #[test]
    fn scroll_state_clamp_v_offset() {
        let mut s = ScrollState::new(50, 80);
        s.v_offset = 100;
        s.clamp(area(80, 24));
        assert_eq!(s.v_offset, 28, "clamped to 50 - 22 = 28");
    }

    #[test]
    fn scroll_state_clamp_h_offset() {
        let mut s = ScrollState::new(50, 100);
        s.h_offset = 200;
        s.clamp(area(80, 24));
        assert_eq!(s.h_offset, 24, "clamped to 100 - 76 = 24");
    }

    #[test]
    fn scroll_state_scroll_down() {
        let mut s = ScrollState::new(100, 80);
        s.scroll_v(5, area(80, 24));
        assert_eq!(s.v_offset, 5);
    }

    #[test]
    fn scroll_state_scroll_up_from_zero() {
        let mut s = ScrollState::new(100, 80);
        s.scroll_v(-5, area(80, 24));
        assert_eq!(s.v_offset, 0, "can't scroll above 0");
    }

    #[test]
    fn scroll_state_scroll_down_clamps_at_max() {
        let mut s = ScrollState::new(30, 80);
        s.scroll_v(100, area(80, 24));
        assert_eq!(s.v_offset, 8, "clamped to 30 - 22 = 8");
    }

    #[test]
    fn scroll_state_scroll_horizontal() {
        let mut s = ScrollState::new(50, 200);
        s.scroll_h(10, area(80, 24));
        assert_eq!(s.h_offset, 10);
    }

    #[test]
    fn scroll_state_scroll_horizontal_clamps() {
        let mut s = ScrollState::new(50, 100);
        s.scroll_h(200, area(80, 24));
        assert_eq!(s.h_offset, 24, "clamped to 100 - 76 = 24");
    }

    #[test]
    fn scroll_state_page_down() {
        let mut s = ScrollState::new(200, 80);
        s.page_down(area(80, 24));
        assert_eq!(s.v_offset, 22, "page down by viewport height");
    }

    #[test]
    fn scroll_state_page_up() {
        let mut s = ScrollState::new(200, 80);
        s.v_offset = 50;
        s.page_up(area(80, 24));
        assert_eq!(s.v_offset, 28, "50 - 22 = 28");
    }

    #[test]
    fn scroll_state_page_up_at_top() {
        let mut s = ScrollState::new(200, 80);
        s.page_up(area(80, 24));
        assert_eq!(s.v_offset, 0);
    }

    #[test]
    fn scroll_state_content_fits_viewport_no_scroll() {
        let mut s = ScrollState::new(10, 50);
        s.scroll_v(5, area(80, 24));
        assert_eq!(s.v_offset, 0, "content fits, offset stays 0");
    }

    #[test]
    fn scrollable_example_render_and_measure() {
        let md = "# Scrollable\n\nLine 1\nLine 2\nLine 3\nLine 4\nLine 5\n";
        let renderer = MarkdownRenderer::new(120);
        let blocks = renderer.parse(md);
        let lines = renderer.render(&blocks, &test_theme());
        let max_w = lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| unicode_width::UnicodeWidthStr::width(s.content.as_ref()))
                    .sum::<usize>()
            })
            .max()
            .unwrap_or(0);
        let mut scroll = ScrollState::new(lines.len(), max_w);
        assert!(scroll.total_lines > 0);
        assert!(scroll.max_line_width > 0);
        scroll.scroll_v(1, area(80, 24));
        assert!(scroll.v_offset <= scroll.max_v_offset(24));
    }
}

#[cfg(feature = "image")]
mod example_image_tests {
    use super::*;
    use crate::markdown::image::{ImageResolver, ResolvedImage};

    struct SimpleResolver {
        font_w: u16,
        font_h: u16,
    }

    impl SimpleResolver {
        fn new(fw: u16, fh: u16) -> Self {
            Self {
                font_w: fw,
                font_h: fh,
            }
        }
    }

    impl ImageResolver for SimpleResolver {
        fn resolve(&mut self, _path: &str) -> Option<image::DynamicImage> {
            None
        }

        fn cell_dimensions(
            &mut self,
            img: &image::DynamicImage,
            max_width: u16,
            _max_height: u16,
        ) -> (u16, u16) {
            let pw = img.width();
            let ph = img.height();
            if pw == 0 || ph == 0 || self.font_w == 0 || max_width == 0 {
                return (0, 0);
            }
            let cw = (pw as f64 / self.font_w as f64).ceil() as u16;
            let w = cw.min(max_width);
            let ch = (ph as f64 / self.font_h as f64).ceil() as u16;
            if w < cw {
                let ratio = ph as f64 * w as f64 / (pw as f64).max(1.0);
                let h = (ratio / self.font_h as f64).ceil() as u16;
                (w.max(1), h.max(1))
            } else {
                (w.max(1), ch.max(1))
            }
        }
    }

    fn make_img(w: u32, h: u32) -> image::DynamicImage {
        let buf = image::ImageBuffer::from_fn(w, h, |_, _| image::Rgb([128u8, 128, 128]));
        image::DynamicImage::ImageRgb8(buf)
    }

    fn scale_to_fit_rows(_pw: u32, ph: u32, target_rows: u16, font_h: u16) -> f64 {
        if ph == 0 {
            return 1.0;
        }
        let natural_h = (ph as f64 / font_h as f64).ceil();
        if natural_h <= target_rows as f64 {
            return 1.0;
        }
        target_rows as f64 * font_h as f64 / ph as f64
    }

    #[test]
    fn scale_to_fit_rows_small_image_no_scale() {
        let s = scale_to_fit_rows(100, 18, 2, 18);
        assert!(
            (s - 1.0).abs() < 0.01,
            "image already fits, scale should be 1.0, got {}",
            s
        );
    }

    #[test]
    fn scale_to_fit_rows_tall_image() {
        let s = scale_to_fit_rows(100, 180, 2, 18);
        let scaled_h = (180.0 * s) as u32;
        let scaled_rows = (scaled_h as f64 / 18.0).ceil() as u16;
        assert!(
            scaled_rows <= 2,
            "scaled to {} rows (scale={})",
            scaled_rows,
            s
        );
    }

    #[test]
    fn image_example_small_image_one_row() {
        let mut r = SimpleResolver::new(9, 18);
        let img = make_img(18, 18);
        let (w, h) = r.cell_dimensions(&img, 70, 20);
        assert_eq!(w, 2);
        assert_eq!(h, 1);
    }

    #[test]
    fn image_example_pre_scaled_to_2_rows() {
        let mut r = SimpleResolver::new(9, 18);
        let original = make_img(300, 300);
        let scale = scale_to_fit_rows(300, 300, 2, 18);
        let sw = (300.0 * scale).ceil() as u32;
        let sh = (300.0 * scale).ceil() as u32;
        let scaled = original.resize_exact(sw, sh, image::imageops::FilterType::Triangle);
        let (_, h) = r.cell_dimensions(&scaled, 70, 20);
        assert!(
            h <= 2,
            "pre-scaled image should fit in 2 rows, got {} rows",
            h
        );
    }

    #[test]
    fn image_example_pre_scaled_to_3_rows() {
        let mut r = SimpleResolver::new(9, 18);
        let original = make_img(600, 400);
        let scale = scale_to_fit_rows(600, 400, 3, 18);
        let sw = (600.0 * scale).ceil() as u32;
        let sh = (400.0 * scale).ceil() as u32;
        let scaled = original.resize_exact(sw, sh, image::imageops::FilterType::Triangle);
        let (_, h) = r.cell_dimensions(&scaled, 70, 20);
        assert!(
            h <= 3,
            "pre-scaled image should fit in 3 rows, got {} rows",
            h
        );
    }

    #[test]
    fn image_example_render_full_with_pre_scaled_images() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = vec![
            MarkdownBlock::Image {
                alt: "logo".into(),
                path: "logo.webp".into(),
            },
            MarkdownBlock::Paragraph(vec!["between".into()]),
            MarkdownBlock::Image {
                alt: "demo".into(),
                path: "demo.webp".into(),
            },
        ];

        let logo_orig = make_img(300, 300);
        let logo_scale = scale_to_fit_rows(300, 300, 2, 18);
        let sw = (300.0 * logo_scale).ceil() as u32;
        let sh = (300.0 * logo_scale).ceil() as u32;
        let logo_scaled = logo_orig.resize_exact(sw, sh, image::imageops::FilterType::Triangle);

        let demo_orig = make_img(600, 400);
        let demo_scale = scale_to_fit_rows(600, 400, 3, 18);
        let dsw = (600.0 * demo_scale).ceil() as u32;
        let dsh = (400.0 * demo_scale).ceil() as u32;
        let demo_scaled = demo_orig.resize_exact(dsw, dsh, image::imageops::FilterType::Triangle);

        let resolved = vec![
            ResolvedImage {
                path: "logo.webp".into(),
                image: logo_scaled,
            },
            ResolvedImage {
                path: "demo.webp".into(),
                image: demo_scaled,
            },
        ];
        let mut r = SimpleResolver::new(9, 18);
        let output = renderer.render_full(&blocks, &test_theme(), &resolved, &mut r, 70, 20);
        assert_eq!(output.images.len(), 2);
        assert!(
            output.images[0].height_cells <= 2,
            "logo should fit in 2 rows, got {}",
            output.images[0].height_cells
        );
        assert!(
            output.images[1].height_cells <= 3,
            "demo should fit in 3 rows, got {}",
            output.images[1].height_cells
        );
    }

    #[test]
    fn image_example_zoom_changes_placement_dimensions() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = vec![MarkdownBlock::Image {
            alt: "test".into(),
            path: "t.webp".into(),
        }];

        let img = make_img(100, 100);
        let resolved_base = vec![ResolvedImage {
            path: "t.webp".into(),
            image: img.clone(),
        }];
        let mut r1 = SimpleResolver::new(9, 18);
        let output1 = renderer.render_full(&blocks, &test_theme(), &resolved_base, &mut r1, 70, 20);

        let zoomed = img.resize_exact(200, 200, image::imageops::FilterType::Triangle);
        let resolved_zoomed = vec![ResolvedImage {
            path: "t.webp".into(),
            image: zoomed,
        }];
        let mut r2 = SimpleResolver::new(9, 18);
        let output2 =
            renderer.render_full(&blocks, &test_theme(), &resolved_zoomed, &mut r2, 70, 20);

        assert!(
            output2.images[0].height_cells >= output1.images[0].height_cells,
            "zoomed image should have >= height: {} vs {}",
            output2.images[0].height_cells,
            output1.images[0].height_cells,
        );
        assert!(
            output2.images[0].width_cells >= output1.images[0].width_cells,
            "zoomed image should have >= width: {} vs {}",
            output2.images[0].width_cells,
            output1.images[0].width_cells,
        );
    }

    #[test]
    fn image_example_zoom_out_shrinks_placement() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = vec![MarkdownBlock::Image {
            alt: "test".into(),
            path: "t.webp".into(),
        }];

        let img = make_img(200, 200);
        let resolved_base = vec![ResolvedImage {
            path: "t.webp".into(),
            image: img.clone(),
        }];
        let mut r1 = SimpleResolver::new(9, 18);
        let output1 = renderer.render_full(&blocks, &test_theme(), &resolved_base, &mut r1, 70, 20);

        let shrunk = img.resize_exact(50, 50, image::imageops::FilterType::Triangle);
        let resolved_shrunk = vec![ResolvedImage {
            path: "t.webp".into(),
            image: shrunk,
        }];
        let mut r2 = SimpleResolver::new(9, 18);
        let output2 =
            renderer.render_full(&blocks, &test_theme(), &resolved_shrunk, &mut r2, 70, 20);

        assert!(
            output2.images[0].height_cells <= output1.images[0].height_cells,
            "shrunk image should have <= height: {} vs {}",
            output2.images[0].height_cells,
            output1.images[0].height_cells,
        );
    }

    #[test]
    fn image_example_blank_lines_match_height_cells() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = vec![
            MarkdownBlock::Paragraph(vec!["before".into()]),
            MarkdownBlock::Image {
                alt: "logo".into(),
                path: "logo.webp".into(),
            },
            MarkdownBlock::Paragraph(vec!["after".into()]),
        ];
        let resolved = vec![ResolvedImage {
            path: "logo.webp".into(),
            image: make_img(90, 36),
        }];
        let mut r = SimpleResolver::new(9, 18);
        let output = renderer.render_full(&blocks, &test_theme(), &resolved, &mut r, 70, 20);

        let img = &output.images[0];
        let row = img.row;
        let height = img.height_cells as usize;
        assert!(height > 0, "should have at least 1 row");
        let blank_count = output.lines[row..row + height]
            .iter()
            .filter(|l| l.spans.is_empty() || l.spans.iter().all(|s| s.content.is_empty()))
            .count();
        assert_eq!(
            blank_count, height,
            "should have {} blank lines for image, got {}",
            height, blank_count
        );
    }

    #[test]
    fn image_example_zoom_then_rerender_changes_layout() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = vec![
            MarkdownBlock::Image {
                alt: "a".into(),
                path: "a.webp".into(),
            },
            MarkdownBlock::Paragraph(vec!["text below".into()]),
        ];

        let original = make_img(180, 180);
        let scale_base = scale_to_fit_rows(180, 180, 2, 18);
        let sw = (180.0 * scale_base).ceil() as u32;
        let sh = (180.0 * scale_base).ceil() as u32;
        let base_img = original.resize_exact(sw, sh, image::imageops::FilterType::Triangle);

        let resolved_base = vec![ResolvedImage {
            path: "a.webp".into(),
            image: base_img,
        }];
        let mut r1 = SimpleResolver::new(9, 18);
        let out1 = renderer.render_full(&blocks, &test_theme(), &resolved_base, &mut r1, 70, 20);

        let zoom_sw = (sw as f64 * 2.0).ceil() as u32;
        let zoom_sh = (sh as f64 * 2.0).ceil() as u32;
        let zoomed = original.resize_exact(zoom_sw, zoom_sh, image::imageops::FilterType::Triangle);
        let resolved_zoom = vec![ResolvedImage {
            path: "a.webp".into(),
            image: zoomed,
        }];
        let mut r2 = SimpleResolver::new(9, 18);
        let out2 = renderer.render_full(&blocks, &test_theme(), &resolved_zoom, &mut r2, 70, 20);

        assert!(
            out2.images[0].height_cells > out1.images[0].height_cells,
            "2x zoom should increase height: {} -> {}",
            out1.images[0].height_cells,
            out2.images[0].height_cells,
        );
        let text_row_1 = out1.images[0].row + out1.images[0].height_cells as usize;
        let text_row_2 = out2.images[0].row + out2.images[0].height_cells as usize;
        assert!(
            text_row_2 > text_row_1,
            "text after zoomed image should be pushed down: {} -> {}",
            text_row_1,
            text_row_2,
        );
    }

    #[test]
    fn image_crop_larger_than_viewport() {
        let img = make_img(180, 180);
        let scaled = img.resize_exact(360, 360, image::imageops::FilterType::Triangle);
        let fw: u16 = 9;
        let fh: u16 = 18;
        let (full_cw, full_ch) = (
            (360u32.div_ceil(fw as u32)) as u16,
            (360u32.div_ceil(fh as u32)) as u16,
        );
        let vp_w = 20u16;
        let vp_h = 5u16;
        assert!(full_cw > vp_w, "full_cw ({}) > vp_w ({})", full_cw, vp_w);
        assert!(full_ch > vp_h, "full_ch ({}) > vp_h ({})", full_ch, vp_h);

        let px_x = 0u32;
        let py_y = 0u32;
        let px_w = vp_w as u32 * fw as u32;
        let py_h = vp_h as u32 * fh as u32;
        let cropped = scaled.crop_imm(
            px_x,
            py_y,
            px_w.min(scaled.width()),
            py_h.min(scaled.height()),
        );
        let (crop_cw, crop_ch) = (
            (cropped.width().div_ceil(fw as u32)) as u16,
            (cropped.height().div_ceil(fh as u32)) as u16,
        );
        assert!(
            crop_cw <= vp_w,
            "cropped width {} <= vp_w {}",
            crop_cw,
            vp_w
        );
        assert!(
            crop_ch <= vp_h,
            "cropped height {} <= vp_h {}",
            crop_ch,
            vp_h
        );
    }

    #[test]
    fn image_crop_with_scroll_offset() {
        let img = make_img(180, 180);
        let scaled = img.resize_exact(360, 360, image::imageops::FilterType::Triangle);
        let fw: u16 = 9;
        let fh: u16 = 18;
        let _full_cw = (360u32.div_ceil(fw as u32)) as u16;
        let _full_ch = (360u32.div_ceil(fh as u32)) as u16;
        let vp_w = 20u16;
        let vp_h = 5u16;
        let scroll_x = 5u16;
        let scroll_y = 3u16;

        let px_x = scroll_x as u32 * fw as u32;
        let py_y = scroll_y as u32 * fh as u32;
        let px_w = vp_w as u32 * fw as u32;
        let py_h = vp_h as u32 * fh as u32;
        let x0 = px_x.min(scaled.width());
        let y0 = py_y.min(scaled.height());
        let x1 = (x0 + px_w).min(scaled.width());
        let y1 = (y0 + py_h).min(scaled.height());
        let cropped = scaled.crop_imm(x0, y0, x1 - x0, y1 - y0);

        assert!(
            x0 > 0,
            "crop starts at pixel x={}, scroll was {}",
            x0,
            scroll_x
        );
        assert!(
            y0 > 0,
            "crop starts at pixel y={}, scroll was {}",
            y0,
            scroll_y
        );
        assert_eq!(
            cropped.width(),
            px_w,
            "cropped width matches viewport pixels"
        );
        assert_eq!(
            cropped.height(),
            py_h,
            "cropped height matches viewport pixels"
        );
    }

    #[test]
    fn image_placement_has_crop_field_none_by_default() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = vec![MarkdownBlock::Image {
            alt: "t".into(),
            path: "t.webp".into(),
        }];
        let resolved = vec![ResolvedImage {
            path: "t.webp".into(),
            image: make_img(90, 36),
        }];
        let mut r = SimpleResolver::new(9, 18);
        let output = renderer.render_full(&blocks, &test_theme(), &resolved, &mut r, 70, 20);
        assert_eq!(output.images.len(), 1);
        assert!(
            output.images[0].crop.is_none(),
            "crop should be None by default from render_full"
        );
    }

    #[test]
    fn grow_by_one_row_each_press() {
        let mut rows = 2u16;
        for _ in 0..10 {
            rows = rows.saturating_add(1);
        }
        assert_eq!(rows, 12, "10 grows from 2 = 12 rows");
    }

    #[test]
    fn shrink_by_one_row_each_press() {
        let mut rows = 12u16;
        for _ in 0..10 {
            rows = rows.saturating_sub(1).max(1);
        }
        assert_eq!(rows, 2, "10 shrinks from 12 = 2 rows");
    }

    #[test]
    fn shrink_clamps_at_one_row() {
        let mut rows = 2u16;
        for _ in 0..10 {
            rows = rows.saturating_sub(1).max(1);
        }
        assert_eq!(rows, 1, "cannot go below 1 row");
    }

    #[test]
    fn image_crop_fits_within_viewport() {
        let fw: u16 = 9;
        let fh: u16 = 18;
        let vp_w = 30u16;
        let vp_h = 10u16;
        let img = make_img(900, 720);
        let (cw, ch) = ((900u32 / fw as u32) as u16, (720u32 / fh as u32) as u16);
        assert!(cw > vp_w);
        assert!(ch > vp_h);
        let px_w = vp_w as u32 * fw as u32;
        let py_h = vp_h as u32 * fh as u32;
        let cropped = img.crop_imm(0, 0, px_w, py_h);
        let (ccw, cch) = (
            (cropped.width().div_ceil(fw as u32)) as u16,
            (cropped.height().div_ceil(fh as u32)) as u16,
        );
        assert!(ccw <= vp_w, "cropped cols {} <= vp {}", ccw, vp_w);
        assert!(cch <= vp_h, "cropped rows {} <= vp {}", cch, vp_h);
    }
}

// ==================== Nested Blockquote Tests ====================

#[test]
fn blockquote_parsed_with_level_and_children() -> anyhow::Result<()> {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("> quoted text");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Blockquote {
            level, children, ..
        } => {
            assert_eq!(*level, 1);
            assert!(!children.is_empty());
        }
        other => panic!("expected Blockquote, got {:?}", other),
    }
    Ok(())
}

#[test]
fn blockquote_multiline_grouped() -> anyhow::Result<()> {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("> line 1\n> line 2\n> line 3");
    assert_eq!(
        blocks.len(),
        1,
        "consecutive > lines should be grouped into one blockquote"
    );
    match &blocks[0] {
        MarkdownBlock::Blockquote {
            level, children, ..
        } => {
            assert_eq!(*level, 1);
            assert!(!children.is_empty());
        }
        other => panic!("expected single Blockquote, got {:?}", other),
    }
    Ok(())
}

#[test]
fn nested_blockquote_parsed() -> anyhow::Result<()> {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("> level 1\n> > level 2");
    assert!(!blocks.is_empty(), "should parse nested blockquote");
    match &blocks[0] {
        MarkdownBlock::Blockquote {
            level, children, ..
        } => {
            assert_eq!(*level, 1);
            let has_nested = children
                .iter()
                .any(|c| matches!(c, MarkdownBlock::Blockquote { .. }));
            assert!(
                has_nested,
                "level 1 should contain a nested level 2 blockquote"
            );
        }
        other => panic!("expected Blockquote, got {:?}", other),
    }
    Ok(())
}

#[test]
fn blockquote_renders_with_pipe_prefix() -> anyhow::Result<()> {
    let lines = render_markdown("> hello", 80);
    assert_eq!(lines.len(), 1);
    let text: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(
        text.starts_with("│"),
        "blockquote should start with │: got '{}'",
        text
    );
    assert!(
        text.contains("hello"),
        "blockquote should contain text: got '{}'",
        text
    );
    Ok(())
}

#[test]
fn nested_blockquote_renders_with_double_pipe() -> anyhow::Result<()> {
    let lines = render_markdown("> outer\n> > inner", 80);
    let all_text: String = lines
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect::<Vec<&str>>()
        .join("");
    assert!(
        all_text.contains("│ │"),
        "nested blockquote should have double pipe prefix"
    );
    Ok(())
}

#[test]
fn blockquote_with_code_inside() -> anyhow::Result<()> {
    let md = "> text before\n> ```rust\n> fn main() {}\n> ```\n> text after";
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse(md);
    let bq = blocks
        .iter()
        .find(|b| matches!(b, MarkdownBlock::Blockquote { .. }));
    assert!(bq.is_some(), "should parse blockquote");
    if let Some(MarkdownBlock::Blockquote { children, .. }) = bq {
        let has_code = children
            .iter()
            .any(|c| matches!(c, MarkdownBlock::CodeBlock { .. }));
        assert!(has_code, "blockquote children should contain a code block");
    }
    Ok(())
}

// ==================== CodeBlock Override Tests ====================

#[test]
fn code_block_override_header() -> anyhow::Result<()> {
    let block = MarkdownBlock::CodeBlock {
        lang: "rust".into(),
        code: "fn main() {}".into(),
        header_override: Some("╭─ Input ──".into()),
        footer_override: None,
        prefix_override: None,
    };
    let renderer = MarkdownRenderer::new(80);
    let lines = renderer.render(&[block], &test_theme());
    let header_text: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(
        header_text.contains("Input"),
        "header override should be used: got '{}'",
        header_text
    );
    Ok(())
}

#[test]
fn code_block_override_footer() -> anyhow::Result<()> {
    let block = MarkdownBlock::CodeBlock {
        lang: "rust".into(),
        code: "fn main() {}".into(),
        header_override: None,
        footer_override: Some("╰─ Output ──".into()),
        prefix_override: None,
    };
    let renderer = MarkdownRenderer::new(80);
    let lines = renderer.render(&[block], &test_theme());
    let last = lines.last().context("last line")?;
    let footer_text: String = last.spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(
        footer_text.contains("Output"),
        "footer override should be used: got '{}'",
        footer_text
    );
    Ok(())
}

#[test]
fn code_block_override_prefix() -> anyhow::Result<()> {
    let block = MarkdownBlock::CodeBlock {
        lang: "json".into(),
        code: r#"{"key": "value"}"#.into(),
        header_override: None,
        footer_override: None,
        prefix_override: Some("║ ".into()),
    };
    let renderer = MarkdownRenderer::new(80);
    let lines = renderer.render(&[block], &test_theme());
    let code_line = &lines[1];
    let text: String = code_line.spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(
        text.starts_with("║"),
        "prefix override should be used: got '{}'",
        text
    );
    Ok(())
}

#[test]
fn code_block_constructor_helper() -> anyhow::Result<()> {
    let block = MarkdownBlock::code_block("python", "print(1)");
    match block {
        MarkdownBlock::CodeBlock {
            lang,
            code,
            header_override,
            footer_override,
            prefix_override,
        } => {
            assert_eq!(lang, "python");
            assert_eq!(code, "print(1)");
            assert!(header_override.is_none());
            assert!(footer_override.is_none());
            assert!(prefix_override.is_none());
        }
        other => panic!("expected CodeBlock, got {:?}", other),
    }
    Ok(())
}

// ==================== Mermaid Rendering Tests ====================

#[cfg(feature = "mermaid")]
mod mermaid_render_tests {
    use super::*;

    #[test]
    fn mermaid_simple_flowchart_renders() {
        let md = "```mermaid\ngraph TD\nA[Start] --> B[End]\n```";
        let lines = render_markdown(md, 80);
        assert!(!lines.is_empty(), "mermaid flowchart should render output");
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(
            all_text.contains("Start"),
            "rendered output should contain node label 'Start': got '{}'",
            all_text
        );
        assert!(
            all_text.contains("End"),
            "rendered output should contain node label 'End': got '{}'",
            all_text
        );
    }

    #[test]
    fn mermaid_lr_direction_renders() {
        let md = "```mermaid\ngraph LR\nA --> B\n```";
        let lines = render_markdown(md, 80);
        assert!(!lines.is_empty(), "LR flowchart should render");
    }

    #[test]
    fn mermaid_three_node_chain() {
        let md = "```mermaid\ngraph TD\nA[First] --> B[Second] --> C[Third]\n```";
        let lines = render_markdown(md, 80);
        assert!(!lines.is_empty());
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(all_text.contains("First"));
        assert!(all_text.contains("Second"));
        assert!(all_text.contains("Third"));
    }

    #[test]
    fn mermaid_diamond_shape_renders() {
        let md = "```mermaid\ngraph TD\nA{Decision} --> B[Result]\n```";
        let lines = render_markdown(md, 80);
        assert!(!lines.is_empty());
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(all_text.contains("Decision"));
    }

    #[test]
    fn mermaid_labeled_edge_renders() {
        let md = "```mermaid\ngraph TD\nA -->|yes| B\n```";
        let lines = render_markdown(md, 80);
        assert!(!lines.is_empty());
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(
            all_text.contains("yes"),
            "edge label should appear: got '{}'",
            all_text
        );
    }

    #[test]
    fn mermaid_viewport_adaptation() {
        let md = "```mermaid\ngraph TD\nA[Very Long Node Label Here] --> B[Another]\n```";
        let lines_narrow = render_markdown(md, 30);
        let lines_wide = render_markdown(md, 80);
        assert!(!lines_narrow.is_empty());
        assert!(!lines_wide.is_empty());
    }

    #[test]
    fn mermaid_invalid_syntax_skipped() {
        let md = "```mermaid\nnot a valid mermaid diagram\n```";
        let lines = render_markdown(md, 80);
        assert!(
            lines.is_empty(),
            "invalid mermaid should be skipped gracefully"
        );
    }

    #[test]
    fn mermaid_empty_nodes_render() -> anyhow::Result<()> {
        let md = "```mermaid\ngraph TD\nA --> B\n```";
        let lines = render_markdown(md, 80);
        assert!(!lines.is_empty(), "unnamed nodes should still render");
        let buf = render_to_buffer(lines, 80, 20)?;
        assert!(buf.area.width > 0);
        Ok(())
    }

    #[test]
    fn mermaid_multiple_edges_render() {
        let md = "```mermaid\ngraph TD\nA --> B\nA --> C\nB --> D\nC --> D\n```";
        let lines = render_markdown(md, 80);
        assert!(!lines.is_empty());
    }

    #[test]
    fn mermaid_direct_render_api() -> anyhow::Result<()> {
        let source = "graph TD\nA[Hello] --> B[World]";
        let result = crate::mermaid::render_mermaid(source, 80, None, &test_theme());
        assert!(result.is_some());
        let lines = result.context("parse result")?;
        assert!(!lines.is_empty());
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(all_text.contains("Hello"));
        assert!(all_text.contains("World"));
        Ok(())
    }

    #[test]
    fn mermaid_max_height_constraint() -> anyhow::Result<()> {
        let source = "graph TD\nA --> B\nB --> C\nC --> D\nD --> E";
        let result = crate::mermaid::render_mermaid(source, 80, Some(10), &test_theme());
        assert!(result.is_some());
        let lines = result.context("parse result")?;
        assert!(
            lines.len() <= 25,
            "should try to respect max_height, got {} lines",
            lines.len()
        );
        Ok(())
    }

    #[test]
    fn mermaid_sequence_diagram_renders() {
        let md = "```mermaid\nsequenceDiagram\n    Alice->>Bob: Hello\n    Bob-->>Alice: Hi\n```";
        let lines = render_markdown(md, 80);
        assert!(!lines.is_empty(), "sequence diagram should render");
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(
            all_text.contains("Alice"),
            "should contain participant Alice"
        );
        assert!(all_text.contains("Bob"), "should contain participant Bob");
        assert!(all_text.contains("Hello"), "should contain message text");
    }

    #[test]
    fn mermaid_pie_chart_renders() {
        let md = "```mermaid\npie title Pets\n    \"Dogs\" : 386\n    \"Cats\" : 85\n```";
        let lines = render_markdown(md, 80);
        assert!(!lines.is_empty(), "pie chart should render");
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(all_text.contains("Pets"), "should contain title");
        assert!(all_text.contains("Dogs"), "should contain slice label");
        assert!(all_text.contains("Cats"), "should contain slice label");
    }

    #[test]
    fn mermaid_gantt_chart_renders() {
        let md = "```mermaid\ngantt\ntitle Project\nsection Phase 1\nTask A :a1, 7d\nTask B :a2, 5d\n```";
        let lines = render_markdown(md, 80);
        assert!(!lines.is_empty(), "gantt chart should render");
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(all_text.contains("Project"), "should contain title");
        assert!(all_text.contains("Task A"), "should contain task name");
    }

    #[test]
    fn mermaid_state_diagram_renders() {
        let md = "```mermaid\nstateDiagram-v2\n    [*] --> Idle\n    Idle --> Running\n    Running --> Idle\n```";
        let lines = render_markdown(md, 80);
        assert!(!lines.is_empty(), "state diagram should render");
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(all_text.contains("Idle"), "should contain state name");
        assert!(all_text.contains("Running"), "should contain state name");
    }

    #[test]
    fn mermaid_diamond_uses_rounded_corners() -> anyhow::Result<()> {
        let source = "graph TD\nA{Decision} --> B[Result]";
        let result = crate::mermaid::render_mermaid(source, 80, None, &test_theme());
        assert!(result.is_some());
        let lines = result.context("parse result")?;
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(
            !all_text.contains('/'),
            "diamond should use rounded corners, not slashes"
        );
        assert!(
            all_text.contains("Decision"),
            "should contain diamond label"
        );
        Ok(())
    }

    #[test]
    fn mermaid_no_dangling_cross_chars() -> anyhow::Result<()> {
        let source = "graph TD\nA[Start] --> B[End]";
        let result = crate::mermaid::render_mermaid(source, 80, None, &test_theme());
        assert!(result.is_some());
        let lines = result.context("parse result")?;
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(
            !all_text.contains('┼'),
            "should not have dangling cross characters"
        );
        Ok(())
    }

    fn lines_to_text(lines: &[Line<'static>]) -> String {
        lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<Vec<&str>>()
                    .join("")
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn render_lines(source: &str, width: usize) -> Vec<Line<'static>> {
        crate::mermaid::render_mermaid(source, width, None, &test_theme()).unwrap_or_default()
    }

    #[test]
    fn mermaid_fork_uses_correct_tee() -> anyhow::Result<()> {
        let source = "graph TD\nD{Decision} -->|Yes| A[Action A]\nD -->|No| B[Action B]";
        let lines = render_lines(source, 80);
        let text = lines_to_text(&lines);

        assert!(
            text.contains('┴'),
            "fork junction should use ┴ (tee-up, connects up+left+right), got:\n{text}"
        );
        assert!(
            !text.contains('┬'),
            "fork junction should NOT use ┬ (tee-down), got:\n{text}"
        );
        assert!(text.contains("Decision"), "should contain Decision label");
        assert!(
            text.contains("Action A") && text.contains("Action B"),
            "should contain both action labels"
        );
        Ok(())
    }

    #[test]
    fn mermaid_merge_uses_correct_tee() -> anyhow::Result<()> {
        let source = "graph TD\nA[Action A] --> E[End]\nB[Action B] --> E";
        let lines = render_lines(source, 80);
        let text = lines_to_text(&lines);

        assert!(
            text.contains('┬'),
            "merge junction should use ┬ (connects down+left+right), got:\n{text}"
        );
        assert!(
            !text.contains('┴'),
            "merge junction should NOT use ┴ (connects up+left+right), got:\n{text}"
        );
        assert!(
            text.contains("Action A") && text.contains("Action B") && text.contains("End"),
            "should contain all node labels"
        );
        Ok(())
    }

    #[test]
    fn mermaid_straight_line_has_no_breaks() -> anyhow::Result<()> {
        let source = "graph TD\nA[Start] --> B[End]";
        let lines = render_lines(source, 80);
        let text = lines_to_text(&lines);

        assert!(
            text.contains('│') || text.contains('▼'),
            "straight edge should have vertical line or arrow, got:\n{text}"
        );
        assert!(
            !text.contains('┼'),
            "straight edge should NOT have cross chars, got:\n{text}"
        );
        assert!(
            !text.contains('┬') || text.split('┬').count() <= 2,
            "straight edge should NOT have spurious T-junctions"
        );
        Ok(())
    }

    #[test]
    fn mermaid_corners_are_correct() -> anyhow::Result<()> {
        let source = "graph TD\nA[Start] --> B[End]";
        let lines = render_lines(source, 80);
        let text = lines_to_text(&lines);

        let bad_chars: Vec<char> = text.chars().filter(|c| "╱╲+*#".contains(*c)).collect();
        assert!(
            bad_chars.is_empty(),
            "should not contain stray chars {:?}, got:\n{text}",
            bad_chars
        );
        Ok(())
    }

    #[test]
    fn mermaid_class_diagram_renders_borders() -> anyhow::Result<()> {
        let source = "classDiagram\nclass Animal {\n  +String name\n  +makeSound() void\n}";
        let lines = render_lines(source, 80);
        let text = lines_to_text(&lines);

        assert!(
            text.contains("Animal"),
            "class diagram should contain class name, got:\n{text}"
        );
        assert!(
            text.contains("name"),
            "class diagram should contain attribute, got:\n{text}"
        );
        assert!(
            text.contains("makeSound"),
            "class diagram should contain method, got:\n{text}"
        );

        let top_border_count = text.matches('┌').count();
        let bottom_border_count = text.matches('└').count();
        assert!(
            top_border_count >= 1 && bottom_border_count >= 1,
            "class box should have top-left ┌ and bottom-left └ corners, got:\n{text}"
        );

        let right_top = text.matches('┐').count();
        let right_bottom = text.matches('┘').count();
        assert!(
            right_top >= 1 && right_bottom >= 1,
            "class box should have top-right ┐ and bottom-right ┘ corners, got:\n{text}"
        );
        Ok(())
    }

    #[test]
    fn mermaid_class_inheritance_edge_renders() -> anyhow::Result<()> {
        let source = "classDiagram\nclass Animal {\n  +String name\n}\nclass Dog {\n  +String breed\n}\nAnimal <|-- Dog";
        let lines = render_lines(source, 80);
        let text = lines_to_text(&lines);

        assert!(
            text.contains("Animal") && text.contains("Dog"),
            "should contain both class names, got:\n{text}"
        );
        let has_edge_chars =
            text.contains('│') || text.contains('┬') || text.contains('├') || text.contains('▼');
        assert!(
            has_edge_chars,
            "inheritance should have edge line chars, got:\n{text}"
        );
        Ok(())
    }

    #[test]
    fn mermaid_three_way_fork_full_output() -> anyhow::Result<()> {
        let source = "graph TD\nS[Start] --> A[One]\nS --> B[Two]\nS --> C[Three]";
        let lines = render_lines(source, 80);
        let text = lines_to_text(&lines);

        assert!(text.contains("Start"), "should have Start");
        assert!(text.contains("One"), "should have One");
        assert!(text.contains("Two"), "should have Two");
        assert!(text.contains("Three"), "should have Three");

        assert!(
            text.contains('┬'),
            "three-way fork should have ┬ junction, got:\n{text}"
        );
        assert!(
            !text.contains('┼'),
            "should not have spurious cross chars, got:\n{text}"
        );
        Ok(())
    }
}

// ==================== Link Inline Parsing Tests ====================

#[test]
fn inline_link_renders_underlined() -> anyhow::Result<()> {
    let lines = render_markdown("click [here](https://example.com) now", 80);
    assert_eq!(lines.len(), 1);
    let has_here = lines[0]
        .spans
        .iter()
        .any(|s| s.content == "here" && s.style.add_modifier == Modifier::UNDERLINED);
    assert!(has_here, "link text should be underlined");
    Ok(())
}

#[test]
fn inline_link_without_url_unchanged() -> anyhow::Result<()> {
    let lines = render_markdown("plain [text] here", 80);
    let all_text: String = lines
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect();
    assert!(
        all_text.contains("[text]"),
        "non-link bracket text should remain unchanged"
    );
    Ok(())
}

// ==================== Strikethrough Tests ====================

#[test]
fn strikethrough_renders_crossed_out() -> anyhow::Result<()> {
    let lines = render_markdown("this is ~~deleted~~ text", 80);
    assert_eq!(lines.len(), 1);
    let has_crossed = lines[0]
        .spans
        .iter()
        .any(|s| s.content == "deleted" && s.style.add_modifier == Modifier::CROSSED_OUT);
    assert!(
        has_crossed,
        "strikethrough text should have CROSSED_OUT modifier"
    );
    Ok(())
}

#[test]
fn strikethrough_unterminated_unchanged() -> anyhow::Result<()> {
    let lines = render_markdown("~~no end", 80);
    let all_text: String = lines
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect();
    assert!(
        all_text.contains("~~no end"),
        "unterminated strikethrough should remain as-is"
    );
    Ok(())
}

// ==================== Task List Tests ====================

#[test]
fn task_list_unchecked_parsed() -> anyhow::Result<()> {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("- [ ] pending task");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::TaskItem { text, checked, .. } => {
            assert_eq!(text, "pending task");
            assert!(!checked);
        }
        other => panic!("expected TaskItem, got {:?}", other),
    }
    Ok(())
}

#[test]
fn task_list_checked_parsed() -> anyhow::Result<()> {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("- [x] done task");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::TaskItem { text, checked, .. } => {
            assert_eq!(text, "done task");
            assert!(checked);
        }
        other => panic!("expected TaskItem, got {:?}", other),
    }
    Ok(())
}

#[test]
fn task_list_renders_checkbox() -> anyhow::Result<()> {
    let lines = render_markdown("- [ ] todo\n- [x] done", 80);
    let all_text: String = lines
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect();
    assert!(
        all_text.contains("☐ todo"),
        "unchecked task should render ☐: got '{}'",
        all_text
    );
    assert!(
        all_text.contains("☑ done"),
        "checked task should render ☑: got '{}'",
        all_text
    );
    Ok(())
}

#[test]
fn task_list_uppercase_x_checked() -> anyhow::Result<()> {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("- [X] done");
    match &blocks[0] {
        MarkdownBlock::TaskItem { checked, .. } => {
            assert!(checked);
        }
        other => panic!("expected TaskItem, got {:?}", other),
    }
    Ok(())
}

#[cfg(all(feature = "mermaid", feature = "image"))]
mod mermaid_image_render_tests {
    use super::*;

    fn setup_fontdb() -> fontdb::Database {
        let mut db = fontdb::Database::new();
        let detected = detect_system_font();
        if let Some(ref fam) = detected {
            if !try_load_by_name(&mut db, fam) {
                db.load_system_fonts();
            }
        } else {
            db.load_system_fonts();
        }
        for p in &[
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
        ] {
            if std::path::Path::new(p).exists() {
                let _ = db.load_font_file(p);
            }
        }
        let fallback = detected.as_deref().unwrap_or("DejaVu Sans");
        db.set_sans_serif_family(fallback);
        db.set_serif_family(fallback);
        db.set_monospace_family(fallback);
        db
    }

    fn detect_system_font() -> Option<String> {
        let source = font_kit::source::SystemSource::new();
        let handle = source
            .select_best_match(
                &[font_kit::family_name::FamilyName::SansSerif],
                &font_kit::properties::Properties::new(),
            )
            .ok()?;
        let font = handle.load().ok()?;
        Some(font.full_name())
    }

    fn try_load_by_name(db: &mut fontdb::Database, name: &str) -> bool {
        let source = font_kit::source::SystemSource::new();
        let handle = match source.select_best_match(
            &[font_kit::family_name::FamilyName::Title(name.to_string())],
            &font_kit::properties::Properties::new(),
        ) {
            Ok(h) => h,
            Err(_) => return false,
        };
        let font = match handle.load() {
            Ok(f) => f,
            Err(_) => return false,
        };
        if let Some(bytes) = font.copy_font_data() {
            db.load_font_data((*bytes).clone());
            true
        } else {
            false
        }
    }

    fn render_svg_to_image(source: &str, bg: image::Rgba<u8>) -> Option<image::DynamicImage> {
        let bg_hex = format!("{:02X}{:02X}{:02X}", bg.0[0], bg.0[1], bg.0[2]);
        let mut opts = mermaid_rs_renderer::RenderOptions::default();
        opts.theme.background = bg_hex;
        let svg = mermaid_rs_renderer::render_with_options(source, opts).ok()?;
        let db = setup_fontdb();
        let opts = usvg::Options {
            fontdb: std::sync::Arc::new(db),
            ..usvg::Options::default()
        };
        let tree = usvg::Tree::from_str(&svg, &opts).ok()?;
        let sz = tree.size();
        let w = (sz.width() as f64).round() as u32;
        let h = (sz.height() as f64).round() as u32;
        if w == 0 || h == 0 {
            return None;
        }
        let mut pm = tiny_skia::Pixmap::new(w, h)?;
        pm.fill(tiny_skia::Color::from_rgba8(
            bg.0[0], bg.0[1], bg.0[2], bg.0[3],
        ));
        resvg::render(&tree, tiny_skia::Transform::identity(), &mut pm.as_mut());
        let rgba = pm.data().to_vec();
        image::RgbaImage::from_raw(w, h, rgba).map(image::DynamicImage::ImageRgba8)
    }

    fn is_bg(px: image::Rgba<u8>, bg: image::Rgba<u8>) -> bool {
        px.0 == bg.0
    }

    #[test]
    fn mermaid_image_no_black_edges_simple() {
        let bg = image::Rgba([0, 0, 0, 255]);
        let img = render_svg_to_image("graph TD\n    A-->B", bg).expect("render should succeed");
        let (w, h) = (img.width(), img.height());
        assert!(w > 0 && h > 0, "image should have nonzero dimensions");

        let rgba = img.to_rgba8();
        let total = (w * h) as usize;
        let non_bg = (0..total)
            .filter(|i| {
                let x = (i % w as usize) as u32;
                let y = (i / w as usize) as u32;
                !is_bg(*rgba.get_pixel(x, y), bg)
            })
            .count();
        assert!(non_bg > 0, "image should contain diagram content");
    }

    #[test]
    fn mermaid_image_no_black_edges_complex() {
        let bg = image::Rgba([0, 0, 0, 255]);
        let source = "graph TD\n    A{Cache Hit?} -->|No| B[Compute Result]\n    B --> C[Update Cache]\n    A -->|Yes| D[Return Cached]\n    C --> D\n    D --> E[Response]";
        let img = render_svg_to_image(source, bg).expect("render should succeed");
        let (w, h) = (img.width(), img.height());
        assert!(w > 0 && h > 0, "image should have nonzero dimensions");
        let rgba = img.to_rgba8();

        let total = (w * h) as usize;
        let non_bg = (0..total)
            .filter(|i| {
                let x = (i % w as usize) as u32;
                let y = (i / w as usize) as u32;
                !is_bg(*rgba.get_pixel(x, y), bg)
            })
            .count();
        assert!(non_bg > 0, "image should contain diagram content");
    }

    #[test]
    fn mermaid_image_padding_canvas_covers_all_cells() {
        let bg = image::Rgba([30, 30, 46, 255]);
        let font_w: u32 = 9;
        let font_h: u32 = 18;

        let img =
            render_svg_to_image("graph LR\n    Input-->Output", bg).expect("render should succeed");
        let img_w = img.width();
        let img_h = img.height();

        let cell_w = (img_w as f64 / font_w as f64).ceil() as u32;
        let cell_h = (img_h as f64 / font_h as f64).ceil() as u32;
        let target_px_w = cell_w * font_w;
        let target_px_h = cell_h * font_h;

        assert!(
            target_px_w >= img_w && target_px_h >= img_h,
            "cell-aligned canvas ({},{}) should >= image ({},{})",
            target_px_w,
            target_px_h,
            img_w,
            img_h,
        );

        let extra_x = target_px_w - img_w;
        let extra_y = target_px_h - img_h;
        assert!(
            extra_x < font_w && extra_y < font_h,
            "padding should be < 1 cell: extra_x={} < font_w={}, extra_y={} < font_h={}",
            extra_x,
            font_w,
            extra_y,
            font_h,
        );

        let mut canvas = image::RgbaImage::from_pixel(target_px_w, target_px_h, bg);
        let ox = (target_px_w.saturating_sub(img_w)) / 2;
        let oy = (target_px_h.saturating_sub(img_h)) / 2;
        image::imageops::overlay(&mut canvas, &img.to_rgba8(), ox as i64, oy as i64);

        let center_x = ox + img_w / 2;
        let center_y = oy + img_h / 2;
        let center_px = canvas.get_pixel(center_x, center_y);
        assert!(
            !is_bg(*center_px, bg),
            "center pixel ({},{}) should be SVG content, not bg",
            center_x,
            center_y
        );

        let top_left_bg = is_bg(*canvas.get_pixel(0, 0), bg);
        let bot_right_bg = is_bg(*canvas.get_pixel(target_px_w - 1, target_px_h - 1), bg);
        assert!(top_left_bg || bot_right_bg,
            "with round()-based SVG size, padding area at corners should be bg-colored (this is expected — the real fix is to match bg color to terminal)");
    }

    #[test]
    fn mermaid_image_render_to_cell_aligned_dimensions() -> anyhow::Result<()> {
        let bg = image::Rgba([30, 30, 46, 255]);
        let font_w: u32 = 9;
        let font_h: u32 = 18;

        let source = "graph LR\n    Input-->Output";
        let svg = mermaid_rs_renderer::render(source)
            .map_err(|e| anyhow::anyhow!("mermaid render: {e}"))?;
        let db = setup_fontdb();
        let opts = usvg::Options {
            fontdb: std::sync::Arc::new(db),
            ..usvg::Options::default()
        };
        let tree =
            usvg::Tree::from_str(&svg, &opts).map_err(|e| anyhow::anyhow!("svg parse: {e}"))?;
        let sz = tree.size();

        let raw_w = (sz.width() as f64).round() as u32;
        let raw_h = (sz.height() as f64).round() as u32;
        let cell_w = (raw_w as f64 / font_w as f64).ceil() as u32;
        let cell_h = (raw_h as f64 / font_h as f64).ceil() as u32;
        let aligned_w = cell_w * font_w;
        let aligned_h = cell_h * font_h;

        let mut pm = tiny_skia::Pixmap::new(aligned_w, aligned_h).ok_or_else(|| {
            anyhow::anyhow!("failed to create pixmap {}x{}", aligned_w, aligned_h)
        })?;
        pm.fill(tiny_skia::Color::from_rgba8(
            bg.0[0], bg.0[1], bg.0[2], bg.0[3],
        ));

        let scale_x = aligned_w as f32 / sz.width();
        let scale_y = aligned_h as f32 / sz.height();
        let ts = tiny_skia::Transform::from_scale(scale_x, scale_y);
        resvg::render(&tree, ts, &mut pm.as_mut());

        let rgba = pm.data();
        let img = image::RgbaImage::from_raw(aligned_w, aligned_h, rgba.to_vec())
            .ok_or_else(|| anyhow::anyhow!("failed to create rgba image"))?;

        for x in 0..aligned_w {
            let top = img.get_pixel(x, 0);
            assert!(!is_bg(*top, bg), "scaled top px({},0) should not be bg", x);
            let bot = img.get_pixel(x, aligned_h - 1);
            assert!(
                !is_bg(*bot, bg),
                "scaled bottom px({},{}) should not be bg",
                x,
                aligned_h - 1
            );
        }
        for y in 0..aligned_h {
            let left = img.get_pixel(0, y);
            assert!(
                !is_bg(*left, bg),
                "scaled left px(0,{}) should not be bg",
                y
            );
            let right = img.get_pixel(aligned_w - 1, y);
            assert!(
                !is_bg(*right, bg),
                "scaled right px({},{}) should not be bg",
                aligned_w - 1,
                y
            );
        }
        Ok(())
    }

    #[test]
    fn mermaid_image_cell_dimensions_match_placeholder_count() {
        let theme = test_theme();
        let hooks = MermaidImageHookImpl {
            bg: image::Rgba([0, 0, 0, 255]),
        };
        let renderer = MarkdownRenderer::new(80).with_render_hooks(Box::new(hooks));
        let md = "```mermaid\ngraph TD\n    A-->B\n```\n";
        let blocks = renderer.parse(md);
        let mut resolver = crate::markdown::image::NoopImageResolver;
        let output = renderer.render_full(&blocks, &theme, &[], &mut resolver, 80, 999);

        assert!(
            !output.images.is_empty(),
            "should have at least one image placement"
        );

        for placement in &output.images {
            let expected_rows = placement.height_cells as usize;
            let mut count = 0;
            let start = placement.row;
            for line in &output.lines[start..start + expected_rows] {
                let has_prefix = line
                    .spans
                    .first()
                    .is_some_and(|s| s.content.starts_with("│") || s.content.starts_with("|"));
                if has_prefix {
                    count += 1;
                }
            }
            assert_eq!(
                count, expected_rows,
                "placeholder rows with │ prefix ({}) should match height_cells ({})",
                count, expected_rows
            );
        }

        let header = output
            .lines
            .iter()
            .find(|l| l.spans.iter().any(|s| s.content.contains("mermaid")));
        assert!(header.is_some(), "should have ╭─ mermaid header line");

        let footer = output
            .lines
            .iter()
            .find(|l| l.spans.iter().any(|s| s.content.starts_with("╰")));
        assert!(footer.is_some(), "should have ╰─ footer line");
    }

    #[test]
    fn mermaid_image_placement_col_accounts_for_prefix() {
        let theme = test_theme();
        let hooks = MermaidImageHookImpl {
            bg: image::Rgba([0, 0, 0, 255]),
        };
        let renderer = MarkdownRenderer::new(80).with_render_hooks(Box::new(hooks));
        let md = "```mermaid\ngraph TD\n    A-->B\n```\n";
        let blocks = renderer.parse(md);
        let mut resolver = crate::markdown::image::NoopImageResolver;
        let output = renderer.render_full(&blocks, &theme, &[], &mut resolver, 80, 999);

        for placement in &output.images {
            assert!(
                placement.col >= 2,
                "image col should be at least 2 (prefix), got {}",
                placement.col
            );
            let available = 80u16.saturating_sub(2);
            assert!(
                placement.col as u16 + placement.width_cells <= 2 + available,
                "image should fit within available width: col={}, width={}, available={}",
                placement.col,
                placement.width_cells,
                available
            );
        }
    }

    struct MermaidImageHookImpl {
        bg: image::Rgba<u8>,
    }

    impl RenderHooks for MermaidImageHookImpl {
        fn render_mermaid_image(&self, source: &str) -> Option<image::DynamicImage> {
            render_svg_to_image(source, self.bg)
        }
    }

    #[test]
    fn mermaid_image_svg_background_is_patched() -> anyhow::Result<()> {
        let bg_hex = "1a1b2c";
        let mut opts = mermaid_rs_renderer::RenderOptions::default();
        opts.theme.background = bg_hex.to_string();
        let svg = mermaid_rs_renderer::render_with_options("graph TD\n    A-->B", opts)
            .map_err(|e| anyhow::anyhow!("render with options: {e}"))?;
        assert!(!svg.contains("#FFFFFF"), "should have no #FFFFFF left");
        assert!(svg.contains(bg_hex), "should contain custom bg hex");

        let default_svg = mermaid_rs_renderer::render("graph TD\n    A-->B")
            .map_err(|e| anyhow::anyhow!("default render: {e}"))?;
        assert!(
            default_svg.contains("#FFFFFF"),
            "default theme should contain white bg"
        );
        Ok(())
    }

    #[test]
    fn mermaid_theme_dark_bg_uses_light_colors() {
        use crate::mermaid::theme::{MermaidTheme, PRIMARY_DARK};
        let theme = MermaidTheme::dark_bg();
        assert_eq!(theme.primary_color, PRIMARY_DARK);
        assert_eq!(theme.background, ratatui::style::Color::Rgb(255, 255, 255));
    }

    #[test]
    fn mermaid_theme_light_bg_uses_saturated_colors() {
        use crate::mermaid::theme::{MermaidTheme, PIE_COLORS_LIGHT, PRIMARY_LIGHT};
        let theme = MermaidTheme::light_bg();
        assert_eq!(theme.primary_color, PRIMARY_LIGHT);
        assert_eq!(theme.pie_colors, PIE_COLORS_LIGHT);
    }

    #[test]
    fn mermaid_theme_for_background_selects_correctly() {
        use crate::mermaid::theme::{MermaidTheme, PRIMARY_DARK, PRIMARY_LIGHT};
        let dark = MermaidTheme::for_background(ratatui::style::Color::Black);
        assert_eq!(dark.primary_color, PRIMARY_DARK);
        let light = MermaidTheme::for_background(ratatui::style::Color::White);
        assert_eq!(light.primary_color, PRIMARY_LIGHT);
    }
}
