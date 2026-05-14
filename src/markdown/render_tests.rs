use ratatui::{
    backend::TestBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Terminal,
};

use crate::{
    markdown::{MarkdownBlock, MarkdownRenderer, RenderHooks},
    theme::RichTextTheme,
};

struct TestTheme;

impl RichTextTheme for TestTheme {
    fn generation(&self) -> crate::theme::Generation {
        crate::theme::Generation::default()
    }
    fn get_text_color(&self) -> Color {
        Color::White
    }
    fn get_muted_text_color(&self) -> Color {
        Color::DarkGray
    }
    fn get_primary_color(&self) -> Color {
        Color::Cyan
    }
    fn get_info_color(&self) -> Color {
        Color::Blue
    }
    fn get_popup_selected_background(&self) -> Color {
        Color::DarkGray
    }
    fn get_popup_selected_text_color(&self) -> Color {
        Color::White
    }
    fn get_border_color(&self) -> Color {
        Color::DarkGray
    }
    fn get_focused_border_color(&self) -> Color {
        Color::Cyan
    }
    fn get_secondary_color(&self) -> Color {
        Color::Yellow
    }
    fn get_background_color(&self) -> Color {
        Color::Black
    }
    fn get_json_key_color(&self) -> Color {
        Color::Cyan
    }
    fn get_json_string_color(&self) -> Color {
        Color::Green
    }
    fn get_json_number_color(&self) -> Color {
        Color::Magenta
    }
    fn get_json_bool_color(&self) -> Color {
        Color::Yellow
    }
    fn get_json_null_color(&self) -> Color {
        Color::DarkGray
    }
    fn get_accent_yellow(&self) -> Color {
        Color::Yellow
    }
}

fn render_to_buffer(lines: Vec<Line<'static>>, width: u16, height: u16) -> Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, Rect::new(0, 0, width, height));
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

fn render_markdown(markdown: &str, max_width: usize) -> Vec<Line<'static>> {
    let renderer = MarkdownRenderer::new(max_width);
    let blocks = renderer.parse(markdown);
    renderer.render(&blocks, &TestTheme)
}

#[test]
fn heading1_renders_bold_underlined() {
    let lines = render_markdown("# Hello World", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    let cell = buf.cell((0, 0)).unwrap();
    assert_eq!(cell.symbol(), "H");
}

#[test]
fn heading2_renders_bold() {
    let lines = render_markdown("## Section", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    let cell = buf.cell((0, 0)).unwrap();
    assert_eq!(cell.symbol(), "S");
}

#[test]
fn heading3_renders_bold_secondary() {
    let lines = render_markdown("### Subsection", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "S");
}

#[test]
fn paragraph_renders_text() {
    let lines = render_markdown("Hello, world!", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    let text: String = (0..13)
        .map(|x| buf.cell((x, 0)).unwrap().symbol())
        .collect();
    assert_eq!(text, "Hello, world!");
}

#[test]
fn paragraph_wraps_at_max_width() {
    let lines = render_markdown("abcdefghij klmnopqrst uvwxyz", 15);
    assert!(
        lines.len() >= 2,
        "expected wrapping, got {} lines",
        lines.len()
    );
}

#[test]
fn blank_line_produces_empty_line() {
    let lines = render_markdown("Hello\n\nWorld", 80);
    let blank_idx = lines
        .iter()
        .position(|l| l.spans.is_empty() || l.spans.iter().all(|s| s.content.is_empty()));
    assert!(
        blank_idx.is_some(),
        "expected a blank line between two paragraphs"
    );
}

#[test]
fn horizontal_rule_renders_dashes() {
    let lines = render_markdown("---", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "─");
}

#[test]
fn code_block_with_lang_renders_bordered_box() {
    let md = "```rust\nfn main() {}\n```";
    let lines = render_markdown(md, 80);
    assert!(
        lines.len() >= 3,
        "expected header, content, footer; got {} lines",
        lines.len()
    );
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "╭");
}

#[test]
fn code_block_without_lang_renders_minimal_header() {
    let md = "```\nsome code\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 5);
    let header_sym = buf.cell((0, 0)).unwrap().symbol();
    assert_eq!(header_sym, "╭");
}

#[test]
fn mermaid_code_block_is_skipped() {
    let md = "```mermaid\ngraph TD\nA-->B\n```";
    let lines = render_markdown(md, 80);
    assert!(
        lines.is_empty(),
        "mermaid blocks should produce zero output lines"
    );
}

#[test]
fn unordered_list_dash_renders_bullet() {
    let lines = render_markdown("- item one", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "•");
}

#[test]
fn unordered_list_star_renders_bullet() {
    let lines = render_markdown("* item one", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "•");
}

#[test]
fn unordered_list_plus_renders_bullet() {
    let lines = render_markdown("+ item one", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "•");
}

#[test]
fn ordered_list_renders_items() {
    let lines = render_markdown("1. first\n2. second\n3. third", 80);
    assert_eq!(lines.len(), 3);
    let buf = render_to_buffer(lines, 80, 10);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "•");
    assert_eq!(buf.cell((0, 1)).unwrap().symbol(), "•");
    assert_eq!(buf.cell((0, 2)).unwrap().symbol(), "•");
}

#[test]
fn nested_list_indents() {
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
}

#[test]
fn blockquote_renders_with_prefix() {
    let lines = render_markdown("> quoted text", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), ">");
}

#[test]
fn table_renders_with_borders() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |";
    let lines = render_markdown(md, 80);
    assert!(
        lines.len() >= 4,
        "expected top border, header, separator, row, bottom; got {}",
        lines.len()
    );
    let buf = render_to_buffer(lines, 80, 10);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "┌");
}

#[test]
fn table_bottom_border_uses_bl_br_corners() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |";
    let lines = render_markdown(md, 80);
    let last = &lines[lines.len() - 1];
    let buf = render_to_buffer(vec![last.clone()], 80, 1);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "└");
}

#[test]
fn inline_bold_renders() {
    let spans = crate::markdown::parse_inline_formatting("**bold**", &TestTheme);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, "bold");
    assert!(spans[0].style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn inline_italic_renders() {
    let spans = crate::markdown::parse_inline_formatting("*italic*", &TestTheme);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, "italic");
    assert!(spans[0].style.add_modifier.contains(Modifier::ITALIC));
}

#[test]
fn inline_bold_italic_renders() {
    let spans = crate::markdown::parse_inline_formatting("***both***", &TestTheme);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, "both");
    assert!(spans[0]
        .style
        .add_modifier
        .contains(Modifier::BOLD | Modifier::ITALIC));
}

#[test]
fn inline_code_renders() {
    let spans = crate::markdown::parse_inline_formatting("some `code` here", &TestTheme);
    assert!(spans.iter().any(|s| s.content == "code"));
    let code_span = spans.iter().find(|s| s.content == "code").unwrap();
    assert_eq!(code_span.style.fg, Some(Color::Yellow));
}

#[test]
fn mixed_inline_formatting() {
    let spans =
        crate::markdown::parse_inline_formatting("normal **bold** *italic* `code`", &TestTheme);
    assert!(
        spans.len() >= 4,
        "expected at least 4 spans for mixed formatting"
    );
}

#[test]
fn complex_document_renders() {
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
    let buf = render_to_buffer(lines, 80, 40);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "T");
}

#[test]
fn empty_input_produces_no_lines() {
    let lines = render_markdown("", 80);
    assert!(lines.is_empty());
}

#[test]
fn only_blank_lines_produce_empty_lines() {
    let lines = render_markdown("\n\n\n", 80);
    assert_eq!(lines.len(), 3);
    for line in &lines {
        assert!(line.spans.is_empty() || line.spans.iter().all(|s| s.content.is_empty()));
    }
}

#[test]
fn code_block_preserves_content() {
    let md = "```python\nprint('hello')\nprint('world')\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 10);
    let row1: String = (0..20)
        .map(|x| buf.cell((x, 1)).unwrap().symbol())
        .collect();
    assert!(row1.contains("print"));
}

#[test]
fn unclosed_code_block_still_renders() {
    let md = "```js\nconst x = 1;";
    let lines = render_markdown(md, 80);
    assert!(!lines.is_empty(), "unclosed code block should still render");
}

#[test]
fn table_with_single_column() {
    let md = "| A |\n|---|\n| 1 |";
    let lines = render_markdown(md, 80);
    assert!(lines.len() >= 3);
}

#[test]
fn table_with_wide_content_wraps() {
    let md = "| Header |\n|--------|\n| a_very_long_word_that_exceeds_width |";
    let lines = render_markdown(md, 20);
    assert!(lines.len() >= 3);
}

#[test]
fn inline_formatting_unclosed_bold_treated_as_text() {
    let spans = crate::markdown::parse_inline_formatting("**unclosed", &TestTheme);
    assert!(spans.iter().any(|s| s.content.contains("*")));
}

#[test]
fn inline_formatting_unclosed_italic_treated_as_text() {
    let spans = crate::markdown::parse_inline_formatting("*unclosed", &TestTheme);
    assert!(spans.iter().any(|s| s.content.contains("*")));
}

#[test]
fn inline_formatting_unclosed_code_treated_as_text() {
    let spans = crate::markdown::parse_inline_formatting("`unclosed", &TestTheme);
    assert!(spans.iter().any(|s| s.content.contains("`")));
}

#[test]
fn parse_heading_levels() {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("# H1\n## H2\n### H3");
    assert!(matches!(blocks[0], MarkdownBlock::Heading1(ref t) if t == "H1"));
    assert!(matches!(blocks[1], MarkdownBlock::Heading2(ref t) if t == "H2"));
    assert!(matches!(blocks[2], MarkdownBlock::Heading3(ref t) if t == "H3"));
}

#[test]
fn parse_mixed_block_types() {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("# Title\n\ntext\n\n- item\n\n> quote\n\n---");
    let types: Vec<&str> = blocks
        .iter()
        .map(|b| match b {
            MarkdownBlock::Heading1(_) => "h1",
            MarkdownBlock::Paragraph(_) => "p",
            MarkdownBlock::ListItem(_, _) => "li",
            MarkdownBlock::Blockquote(_) => "bq",
            MarkdownBlock::HorizontalRule => "hr",
            MarkdownBlock::BlankLine => "blank",
            _ => "other",
        })
        .collect();
    assert!(types.contains(&"h1"));
    assert!(types.contains(&"p"));
    assert!(types.contains(&"li"));
    assert!(types.contains(&"bq"));
    assert!(types.contains(&"hr"));
}

#[test]
fn multiple_paragraphs_separated_by_blank() {
    let md = "First paragraph.\n\nSecond paragraph.";
    let lines = render_markdown(md, 80);
    let blank_count = lines
        .iter()
        .filter(|l| l.spans.is_empty() || l.spans.iter().all(|s| s.content.is_empty()))
        .count();
    assert!(blank_count >= 1, "should have at least one blank separator");
}

#[test]
fn narrow_width_still_renders() {
    let lines = render_markdown("# Hi\nSome text here", 5);
    assert!(!lines.is_empty());
}

#[test]
fn very_wide_horizontal_rule_capped_at_80() {
    let lines = render_markdown("---", 200);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    let content: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
    assert_eq!(content.chars().count(), 80);
}

#[test]
fn table_hline_builder() {
    let hline = MarkdownRenderer::build_table_hline(&[5, 5, 5], "┌", "┬", "┐");
    assert_eq!(hline, "┌─────┬─────┬─────┐");
}

#[test]
fn table_hline_single_column() {
    let hline = MarkdownRenderer::build_table_hline(&[3], "┌", "┬", "┐");
    assert_eq!(hline, "┌───┐");
}

#[test]
fn blockquote_italic_style() {
    let lines = render_markdown("> quoted", 80);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    let has_italic = line
        .spans
        .iter()
        .any(|s| s.style.add_modifier.contains(Modifier::ITALIC));
    assert!(has_italic, "blockquote text should be italic");
}

#[test]
fn hr_star_syntax() {
    let lines = render_markdown("***", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "─");
}

#[test]
fn hr_underscore_syntax() {
    let lines = render_markdown("___", 80);
    assert_eq!(lines.len(), 1);
}

// ==================== Image Block Tests ====================

#[test]
fn image_block_parsed_from_standalone_line() {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("![alt text](image.png)");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Image { alt, path } => {
            assert_eq!(alt, "alt text");
            assert_eq!(path, "image.png");
        }
        other => panic!("expected Image block, got {:?}", other),
    }
}

#[test]
fn image_block_parsed_with_empty_alt() {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("![](photo.jpg)");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Image { alt, path } => {
            assert_eq!(alt, "");
            assert_eq!(path, "photo.jpg");
        }
        other => panic!("expected Image block, got {:?}", other),
    }
}

#[test]
fn image_block_between_paragraphs() {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("text before\n\n![img](a.png)\n\ntext after");
    let types: Vec<&str> = blocks
        .iter()
        .map(|b| match b {
            MarkdownBlock::Paragraph(_) => "p",
            MarkdownBlock::Image { .. } => "img",
            MarkdownBlock::BlankLine => "blank",
            _ => "other",
        })
        .collect();
    assert!(types.contains(&"p"), "should have paragraph blocks");
    assert!(types.contains(&"img"), "should have image block");
}

#[test]
fn image_fallback_renders_gray_italic_span() {
    let lines = render_markdown("![my alt](missing.png)", 80);
    assert_eq!(lines.len(), 1, "image fallback should be a single line");
    let buf = render_to_buffer(lines, 80, 5);
    let first_char = buf.cell((0, 0)).unwrap().symbol();
    assert_eq!(first_char, "[", "fallback should start with '['");
    let text: String = (0..30)
        .map(|x| buf.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(
        text.contains("image"),
        "fallback text should contain 'image': got '{}'",
        text
    );
    let cell = buf.cell((0, 0)).unwrap();
    assert_eq!(
        cell.fg,
        Color::Gray,
        "fallback should be Gray colored"
    );
    assert!(
        cell.style().add_modifier.contains(Modifier::ITALIC),
        "fallback should be italic"
    );
}

#[test]
fn image_fallback_uses_alt_when_present() {
    let lines = render_markdown("![screenshot](photo.png)", 80);
    let buf = render_to_buffer(lines, 80, 5);
    let text: String = (0..40)
        .map(|x| buf.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(
        text.contains("screenshot"),
        "fallback should contain alt text 'screenshot': got '{}'",
        text
    );
}

#[test]
fn image_fallback_uses_path_when_alt_empty() {
    let lines = render_markdown("![](photo.png)", 80);
    let buf = render_to_buffer(lines, 80, 5);
    let text: String = (0..40)
        .map(|x| buf.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(
        text.contains("photo.png"),
        "fallback should contain path when alt is empty: got '{}'",
        text
    );
}

#[test]
fn image_inline_in_paragraph_not_treated_as_block() {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("text ![inline](i.png) more");
    for block in &blocks {
        assert!(
            !matches!(block, MarkdownBlock::Image { .. }),
            "inline image inside text should not become a block-level Image"
        );
    }
}

#[test]
fn image_line_count_is_1() {
    let block = MarkdownBlock::Image {
        alt: "test".to_string(),
        path: "test.png".to_string(),
    };
    assert_eq!(block.line_count(), 1);
}

// ==================== RenderHooks Tests ====================

struct HeadingOverrideHooks;

impl RenderHooks for HeadingOverrideHooks {
    fn heading1(&self, text: &str) -> Option<Line<'static>> {
        Some(Line::from(Span::styled(
            format!(">>> {}", text),
            Style::default().fg(Color::Red),
        )))
    }
}

#[test]
fn hooks_heading1_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(HeadingOverrideHooks));
    let blocks = renderer.parse("# Hello");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), ">");
    assert_eq!(buf.cell((1, 0)).unwrap().symbol(), ">");
    assert_eq!(buf.cell((2, 0)).unwrap().symbol(), ">");
    assert_eq!(buf.cell((4, 0)).unwrap().symbol(), "H");
    let cell = buf.cell((0, 0)).unwrap();
    assert_eq!(cell.fg, Color::Red, "overridden heading should be Red");
}

struct Heading2PrefixHooks;

impl RenderHooks for Heading2PrefixHooks {
    fn heading2(&self, text: &str) -> Option<Line<'static>> {
        Some(Line::from(Span::raw(format!("## {}", text))))
    }
}

#[test]
fn hooks_heading2_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(Heading2PrefixHooks));
    let blocks = renderer.parse("## Section");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "#");
    assert_eq!(buf.cell((1, 0)).unwrap().symbol(), "#");
    let text: String = (0..10)
        .map(|x| buf.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(text.starts_with("## Section"));
}

struct Heading3OverrideHooks;

impl RenderHooks for Heading3OverrideHooks {
    fn heading3(&self, text: &str) -> Option<Line<'static>> {
        Some(Line::from(Span::styled(
            format!("### {}", text),
            Style::default().fg(Color::Magenta),
        )))
    }
}

#[test]
fn hooks_heading3_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(Heading3OverrideHooks));
    let blocks = renderer.parse("### Sub");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(buf.cell((0, 0)).unwrap().fg, Color::Magenta);
}

struct ParagraphOverrideHooks;

impl RenderHooks for ParagraphOverrideHooks {
    fn paragraph(&self, lines: &[String]) -> Option<Vec<Line<'static>>> {
        let text = lines.join(" | ");
        Some(vec![Line::from(Span::styled(
            format!("P: {}", text),
            Style::default().fg(Color::Green),
        ))])
    }
}

#[test]
fn hooks_paragraph_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(ParagraphOverrideHooks));
    let blocks = renderer.parse("Hello world");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    let text: String = (0..15)
        .map(|x| buf.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(
        text.starts_with("P: Hello"),
        "paragraph should be overridden with prefix: got '{}'",
        text
    );
    assert_eq!(buf.cell((0, 0)).unwrap().fg, Color::Green);
}

struct CodeBlockCustomHooks;

impl RenderHooks for CodeBlockCustomHooks {
    fn code_block_header(&self, lang: &str) -> Option<Line<'static>> {
        Some(Line::from(Span::styled(
            format!("\u{256d} [{}] custom-header", lang),
            Style::default().fg(Color::Cyan),
        )))
    }

    fn code_block_footer(&self, _lang: &str, line_count: usize) -> Option<Line<'static>> {
        Some(Line::from(Span::styled(
            format!("\u{2570} {} lines", line_count),
            Style::default().fg(Color::Cyan),
        )))
    }

    fn code_block_line_prefix(&self, _lang: &str) -> Option<String> {
        Some("  ".to_string())
    }
}

#[test]
fn hooks_code_block_header_footer_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(CodeBlockCustomHooks));
    let md = "```rust\nfn main() {}\n```";
    let blocks = renderer.parse(md);
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 3, "should have header + 1 content + footer");

    let buf = render_to_buffer(lines.clone(), 80, 5);
    assert_eq!(
        buf.cell((0, 0)).unwrap().symbol(),
        "\u{256d}",
        "header should start with ╭"
    );

    let header_text: String = (0..30)
        .map(|x| buf.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(
        header_text.contains("custom-header"),
        "header should contain custom text: got '{}'",
        header_text
    );

    assert_eq!(
        buf.cell((0, 2)).unwrap().symbol(),
        "\u{2570}",
        "footer should start with ╰"
    );
    let footer_text: String = (0..20)
        .map(|x| buf.cell((x, 2)).unwrap().symbol())
        .collect();
    assert!(
        footer_text.contains("1 lines"),
        "footer should contain line count: got '{}'",
        footer_text
    );
}

#[test]
fn hooks_code_block_line_prefix_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(CodeBlockCustomHooks));
    let md = "```rust\nhello\n```";
    let blocks = renderer.parse(md);
    let lines = renderer.render(&blocks, &TestTheme);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(
        buf.cell((0, 1)).unwrap().symbol(),
        " ",
        "custom prefix should be spaces, not │"
    );
}

struct CodeBlockLineOverrideHooks;

impl RenderHooks for CodeBlockLineOverrideHooks {
    fn code_block_line(&self, line: &str, idx: usize, _total: usize) -> Option<Line<'static>> {
        Some(Line::from(Span::styled(
            format!("{}: {}", idx, line),
            Style::default().fg(Color::Yellow),
        )))
    }
}

#[test]
fn hooks_code_block_line_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(CodeBlockLineOverrideHooks));
    let md = "```js\nline1\nline2\n```";
    let blocks = renderer.parse(md);
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 4, "header + 2 content + footer");
    let buf = render_to_buffer(lines, 80, 5);
    let line1_text: String = (0..10)
        .map(|x| buf.cell((x, 1)).unwrap().symbol())
        .collect();
    assert!(
        line1_text.starts_with("0:"),
        "first code line should have index prefix: got '{}'",
        line1_text
    );
    let line2_text: String = (0..10)
        .map(|x| buf.cell((x, 2)).unwrap().symbol())
        .collect();
    assert!(
        line2_text.starts_with("1:"),
        "second code line should have index prefix: got '{}'",
        line2_text
    );
}

struct InlineCodeOverrideHooks;

impl RenderHooks for InlineCodeOverrideHooks {
    fn inline_code(&self, code: &str) -> Option<Line<'static>> {
        Some(Line::from(Span::styled(
            format!("CODE({})", code),
            Style::default().fg(Color::Red),
        )))
    }
}

#[test]
fn hooks_inline_code_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(InlineCodeOverrideHooks));
    let blocks = vec![MarkdownBlock::InlineCode("hello".to_string())];
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    let text: String = (0..15)
        .map(|x| buf.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(
        text.starts_with("CODE(hello)"),
        "inline code block should be overridden: got '{}'",
        text
    );
    assert_eq!(buf.cell((0, 0)).unwrap().fg, Color::Red);
}

struct TreeListHook;

impl RenderHooks for TreeListHook {
    fn list_item_marker(
        &self,
        _indent: u8,
        is_last_in_group: bool,
        _ancestors_are_last: &[bool],
        _index_in_group: usize,
    ) -> Option<String> {
        let marker = if is_last_in_group {
            "\u{2514}\u{2500} "
        } else {
            "\u{251c}\u{2500} "
        };
        Some(marker.to_string())
    }
}

#[test]
fn hooks_list_item_marker_tree_style() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(TreeListHook));
    let blocks = renderer.parse("- first\n- second\n- third");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 3, "should have 3 list items");
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(
        buf.cell((0, 0)).unwrap().symbol(),
        "\u{251c}",
        "first item (not last) should use ├"
    );
    assert_eq!(
        buf.cell((0, 2)).unwrap().symbol(),
        "\u{2514}",
        "last item should use └"
    );
}

struct ListItemContentHooks;

impl RenderHooks for ListItemContentHooks {
    fn list_item_content(&self, text: &str, _indent: u8) -> Option<Vec<Line<'static>>> {
        Some(vec![Line::from(Span::styled(
            format!("[{}]", text),
            Style::default().fg(Color::Blue),
        ))])
    }

    fn list_item_marker(
        &self,
        _indent: u8,
        _is_last_in_group: bool,
        _ancestors_are_last: &[bool],
        _index_in_group: usize,
    ) -> Option<String> {
        Some("* ".to_string())
    }
}

#[test]
fn hooks_list_item_content_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(ListItemContentHooks));
    let blocks = renderer.parse("- hello");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    let text: String = (0..12)
        .map(|x| buf.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(
        text.contains("[hello]"),
        "list item content should be wrapped in brackets: got '{}'",
        text
    );
}

struct BlockquoteOverrideHooks;

impl RenderHooks for BlockquoteOverrideHooks {
    fn blockquote(&self, text: &str) -> Option<Vec<Line<'static>>> {
        Some(vec![Line::from(Span::styled(
            format!("QUOTE: {}", text),
            Style::default().fg(Color::Magenta),
        ))])
    }
}

#[test]
fn hooks_blockquote_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(BlockquoteOverrideHooks));
    let blocks = renderer.parse("> some quote");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    let text: String = (0..20)
        .map(|x| buf.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(
        text.starts_with("QUOTE: some quote"),
        "blockquote should be overridden: got '{}'",
        text
    );
    assert_eq!(buf.cell((0, 0)).unwrap().fg, Color::Magenta);
}

struct HorizontalRuleOverrideHooks;

impl RenderHooks for HorizontalRuleOverrideHooks {
    fn horizontal_rule(&self) -> Option<Line<'static>> {
        Some(Line::from(Span::styled(
            "=".repeat(40),
            Style::default().fg(Color::Cyan),
        )))
    }
}

#[test]
fn hooks_horizontal_rule_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(HorizontalRuleOverrideHooks));
    let blocks = renderer.parse("---");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(
        buf.cell((0, 0)).unwrap().symbol(),
        "=",
        "horizontal rule should be overridden to ="
    );
    assert_eq!(buf.cell((0, 0)).unwrap().fg, Color::Cyan);
}

struct BlankLineOverrideHooks;

impl RenderHooks for BlankLineOverrideHooks {
    fn blank_line(&self) -> Option<Line<'static>> {
        Some(Line::from(Span::styled(
            "\u{00b7}",
            Style::default().fg(Color::DarkGray),
        )))
    }
}

#[test]
fn hooks_blank_line_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(BlankLineOverrideHooks));
    let blocks = renderer.parse("a\n\nb");
    let lines = renderer.render(&blocks, &TestTheme);
    let blank = lines.iter().find(|l| {
        l.spans.len() == 1 && l.spans[0].content == "\u{00b7}"
    });
    assert!(
        blank.is_some(),
        "blank line should be overridden to middle dot"
    );
}

struct TableOverrideHooks;

impl RenderHooks for TableOverrideHooks {
    fn table(
        &self,
        headers: &[String],
        rows: &[Vec<String>],
    ) -> Option<Vec<Line<'static>>> {
        let mut out = Vec::new();
        out.push(Line::from(Span::styled(
            format!("TABLE: {} headers", headers.len()),
            Style::default().fg(Color::Cyan),
        )));
        for row in rows {
            out.push(Line::from(Span::styled(
                row.join(", "),
                Style::default().fg(Color::White),
            )));
        }
        Some(out)
    }
}

#[test]
fn hooks_table_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(TableOverrideHooks));
    let md = "| A | B |\n|---|---|\n| 1 | 2 |";
    let blocks = renderer.parse(md);
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 2, "should have header line + 1 data row");
    let buf = render_to_buffer(lines, 80, 5);
    let header: String = (0..20)
        .map(|x| buf.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(
        header.contains("TABLE: 2 headers"),
        "table header should be overridden: got '{}'",
        header
    );
}

struct ImageFallbackHooks;

impl RenderHooks for ImageFallbackHooks {
    fn image_fallback(&self, alt: &str, path: &str) -> Option<Vec<Line<'static>>> {
        Some(vec![Line::from(Span::styled(
            format!("[IMG: {} ({})]", alt, path),
            Style::default().fg(Color::Red),
        ))])
    }
}

#[test]
fn hooks_image_fallback_override() {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(ImageFallbackHooks));
    let blocks = renderer.parse("![logo](logo.png)");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5);
    let text: String = (0..30)
        .map(|x| buf.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(
        text.contains("IMG: logo (logo.png)"),
        "image fallback should be overridden: got '{}'",
        text
    );
    assert_eq!(buf.cell((0, 0)).unwrap().fg, Color::Red);
}

struct NoopHooks;

impl RenderHooks for NoopHooks {}

#[test]
fn hooks_noop_does_not_change_output() {
    let md = "# Title\n\nParagraph **bold**.\n\n- item\n\n> quote\n\n---";
    let renderer_no_hooks = MarkdownRenderer::new(80);
    let blocks = renderer_no_hooks.parse(md);
    let lines_no_hooks = renderer_no_hooks.render(&blocks, &TestTheme);

    let renderer_with_noop =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(NoopHooks));
    let blocks2 = renderer_with_noop.parse(md);
    let lines_with_noop = renderer_with_noop.render(&blocks2, &TestTheme);

    assert_eq!(
        lines_no_hooks.len(),
        lines_with_noop.len(),
        "noop hooks should produce same number of lines"
    );
    for (i, (a, b)) in lines_no_hooks.iter().zip(lines_with_noop.iter()).enumerate() {
        let a_text: String = a.spans.iter().map(|s| s.content.as_ref()).collect();
        let b_text: String = b.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(
            a_text, b_text,
            "line {} text should match with noop hooks",
            i
        );
    }
}

#[test]
fn hooks_selective_override_only_affects_target() {
    let md = "# Title\n\nParagraph text.\n\n- item";
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(HeadingOverrideHooks));
    let blocks = renderer.parse(md);
    let lines = renderer.render(&blocks, &TestTheme);

    let heading_line = &lines[0];
    let heading_text: String = heading_line.spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(
        heading_text.starts_with(">>> "),
        "heading should be overridden"
    );

    let paragraph_line = &lines[2];
    let para_text: String = paragraph_line.spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(
        para_text.contains("Paragraph text"),
        "paragraph should NOT be overridden: got '{}'",
        para_text
    );
    assert!(
        !para_text.starts_with("P:"),
        "paragraph should use default rendering, not hook override"
    );
}

// ==================== Code block rendering detail tests ====================

#[test]
fn code_block_header_contains_lang_label() {
    let md = "```rust\ncode\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 5);
    let header: String = (0..20)
        .map(|x| buf.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(
        header.contains("rust"),
        "code block header should contain language label: got '{}'",
        header
    );
}

#[test]
fn code_block_header_has_rounded_top_left() {
    let md = "```js\nx\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "\u{256d}");
}

#[test]
fn code_block_footer_has_rounded_bottom_left() {
    let md = "```js\nx\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(
        buf.cell((0, 2)).unwrap().symbol(),
        "\u{2570}",
        "footer should have ╰"
    );
}

#[test]
fn code_block_content_line_has_pipe_prefix() {
    let md = "```sh\necho hello\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 5);
    assert_eq!(
        buf.cell((0, 1)).unwrap().symbol(),
        "\u{2502}",
        "content line should have │ prefix"
    );
}

#[test]
fn code_block_content_line_has_yellow_text() {
    let md = "```sh\necho hello\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 5);
    let content_cell = buf.cell((2, 1)).unwrap();
    assert_eq!(
        content_cell.fg,
        Color::Yellow,
        "code content should be Yellow"
    );
}

// ==================== Image parsing edge cases ====================

#[test]
fn image_not_confused_with_link() {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("[link](url)");
    for block in &blocks {
        assert!(
            !matches!(block, MarkdownBlock::Image { .. }),
            "link syntax should not produce Image block"
        );
    }
}

#[test]
fn image_with_special_chars_in_alt() {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("![alt-text_here](path.png)");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Image { alt, path } => {
            assert_eq!(alt, "alt-text_here");
            assert_eq!(path, "path.png");
        }
        other => panic!("expected Image, got {:?}", other),
    }
}

#[test]
fn image_with_url_path() {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("![icon](https://example.com/icon.png)");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Image { path, .. } => {
            assert_eq!(path, "https://example.com/icon.png");
        }
        other => panic!("expected Image, got {:?}", other),
    }
}

#[test]
fn image_without_path_not_parsed_as_image() {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("![]()");
    for block in &blocks {
        assert!(
            !matches!(block, MarkdownBlock::Image { .. }),
            "empty path should not produce Image block"
        );
    }
}

// ==================== Hook with complex markdown ====================

struct MultiHook;

impl RenderHooks for MultiHook {
    fn heading1(&self, text: &str) -> Option<Line<'static>> {
        Some(Line::from(Span::raw(format!("H1: {}", text))))
    }

    fn heading2(&self, text: &str) -> Option<Line<'static>> {
        Some(Line::from(Span::raw(format!("H2: {}", text))))
    }

    fn horizontal_rule(&self) -> Option<Line<'static>> {
        Some(Line::from(Span::raw("---HR---")))
    }
}

#[test]
fn hooks_multiple_overrides_in_same_document() {
    let md = "# Title\n\n## Sub\n\n---\n\ntext";
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(MultiHook));
    let blocks = renderer.parse(md);
    let lines = renderer.render(&blocks, &TestTheme);

    let texts: Vec<String> = lines
        .iter()
        .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
        .collect();

    let h1 = texts.iter().find(|t| t.starts_with("H1:"));
    assert!(h1.is_some(), "should find H1 override");
    assert_eq!(h1.unwrap(), "H1: Title");

    let h2 = texts.iter().find(|t| t.starts_with("H2:"));
    assert!(h2.is_some(), "should find H2 override");
    assert_eq!(h2.unwrap(), "H2: Sub");

    let hr = texts.iter().find(|t| t.contains("HR"));
    assert!(hr.is_some(), "should find HR override");
}
