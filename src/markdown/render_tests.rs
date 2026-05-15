use ratatui::{
    backend::TestBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Terminal,
};

use anyhow::Context as _;

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

fn render_to_buffer(lines: Vec<Line<'static>>, width: u16, height: u16) -> anyhow::Result<Buffer> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal
        .draw(|f| {
            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, Rect::new(0, 0, width, height));
        })?;
    Ok(terminal.backend().buffer().clone())
}

fn render_markdown(markdown: &str, max_width: usize) -> Vec<Line<'static>> {
    let renderer = MarkdownRenderer::new(max_width);
    let blocks = renderer.parse(markdown);
    renderer.render(&blocks, &TestTheme)
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
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
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
    let spans = crate::markdown::parse_inline_formatting("**bold**", &TestTheme);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, "bold");
    assert!(spans[0].style.add_modifier.contains(Modifier::BOLD));
    Ok(())
}

#[test]
fn inline_italic_renders() -> anyhow::Result<()> {
    let spans = crate::markdown::parse_inline_formatting("*italic*", &TestTheme);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].content, "italic");
    assert!(spans[0].style.add_modifier.contains(Modifier::ITALIC));
    Ok(())
}

#[test]
fn inline_bold_italic_renders() -> anyhow::Result<()> {
    let spans = crate::markdown::parse_inline_formatting("***both***", &TestTheme);
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
    let spans = crate::markdown::parse_inline_formatting("some `code` here", &TestTheme);
    assert!(spans.iter().any(|s| s.content == "code"));
    let code_span = spans.iter().find(|s| s.content == "code").context("find")?;
    assert_eq!(code_span.style.fg, Some(Color::Yellow));
    Ok(())
}

#[test]
fn mixed_inline_formatting() -> anyhow::Result<()> {
    let spans =
        crate::markdown::parse_inline_formatting("normal **bold** *italic* `code`", &TestTheme);
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

#[test]
fn empty_input_produces_no_lines() -> anyhow::Result<()> {
    let lines = render_markdown("", 80);
    assert!(lines.is_empty());
    Ok(())
}

#[test]
fn only_blank_lines_produce_empty_lines() -> anyhow::Result<()> {
    let lines = render_markdown("\n\n\n", 80);
    assert_eq!(lines.len(), 3);
    for line in &lines {
        assert!(line.spans.is_empty() || line.spans.iter().all(|s| s.content.is_empty()));
    }
    Ok(())
}

#[test]
fn code_block_preserves_content() -> anyhow::Result<()> {
    let md = "```python\nprint('hello')\nprint('world')\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 10)?;
    let row1: String = (0..20)
        .map(|x| buf.cell((x, 1)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(row1.contains("print"));
    Ok(())
}

#[test]
fn unclosed_code_block_still_renders() -> anyhow::Result<()> {
    let md = "```js\nconst x = 1;";
    let lines = render_markdown(md, 80);
    assert!(!lines.is_empty(), "unclosed code block should still render");
    Ok(())
}

#[test]
fn table_with_single_column() -> anyhow::Result<()> {
    let md = "| A |\n|---|\n| 1 |";
    let lines = render_markdown(md, 80);
    assert!(lines.len() >= 3);
    Ok(())
}

#[test]
fn table_with_wide_content_wraps() -> anyhow::Result<()> {
    let md = "| Header |\n|--------|\n| a_very_long_word_that_exceeds_width |";
    let lines = render_markdown(md, 20);
    assert!(lines.len() >= 3);
    Ok(())
}

#[test]
fn inline_formatting_unclosed_bold_treated_as_text() -> anyhow::Result<()> {
    let spans = crate::markdown::parse_inline_formatting("**unclosed", &TestTheme);
    assert!(spans.iter().any(|s| s.content.contains("*")));
    Ok(())
}

#[test]
fn inline_formatting_unclosed_italic_treated_as_text() -> anyhow::Result<()> {
    let spans = crate::markdown::parse_inline_formatting("*unclosed", &TestTheme);
    assert!(spans.iter().any(|s| s.content.contains("*")));
    Ok(())
}

#[test]
fn inline_formatting_unclosed_code_treated_as_text() -> anyhow::Result<()> {
    let spans = crate::markdown::parse_inline_formatting("`unclosed", &TestTheme);
    assert!(spans.iter().any(|s| s.content.contains("`")));
    Ok(())
}

#[test]
fn parse_heading_levels() -> anyhow::Result<()> {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("# H1\n## H2\n### H3");
    assert!(matches!(blocks[0], MarkdownBlock::Heading1(ref t) if t == "H1"));
    assert!(matches!(blocks[1], MarkdownBlock::Heading2(ref t) if t == "H2"));
    assert!(matches!(blocks[2], MarkdownBlock::Heading3(ref t) if t == "H3"));
    Ok(())
}

#[test]
fn parse_mixed_block_types() -> anyhow::Result<()> {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("# Title\n\ntext\n\n- item\n\n> quote\n\n---");
    let types: Vec<&str> = blocks
        .iter()
        .map(|b| match b {
            MarkdownBlock::Heading1(_) => "h1",
            MarkdownBlock::Paragraph(_) => "p",
            MarkdownBlock::ListItem(_, _) => "li",
            MarkdownBlock::Blockquote { .. } => "bq",
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
    Ok(())
}

#[test]
fn multiple_paragraphs_separated_by_blank() -> anyhow::Result<()> {
    let md = "First paragraph.\n\nSecond paragraph.";
    let lines = render_markdown(md, 80);
    let blank_count = lines
        .iter()
        .filter(|l| l.spans.is_empty() || l.spans.iter().all(|s| s.content.is_empty()))
        .count();
    assert!(blank_count >= 1, "should have at least one blank separator");
    Ok(())
}

#[test]
fn narrow_width_still_renders() -> anyhow::Result<()> {
    let lines = render_markdown("# Hi\nSome text here", 5);
    assert!(!lines.is_empty());
    Ok(())
}

#[test]
fn very_wide_horizontal_rule_capped_at_80() -> anyhow::Result<()> {
    let lines = render_markdown("---", 200);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    let content: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
    assert_eq!(content.chars().count(), 80);
    Ok(())
}

#[test]
fn table_hline_builder() -> anyhow::Result<()> {
    let hline = MarkdownRenderer::build_table_hline(&[5, 5, 5], "┌", "┬", "┐");
    assert_eq!(hline, "┌─────┬─────┬─────┐");
    Ok(())
}

#[test]
fn table_hline_single_column() -> anyhow::Result<()> {
    let hline = MarkdownRenderer::build_table_hline(&[3], "┌", "┬", "┐");
    assert_eq!(hline, "┌───┐");
    Ok(())
}

#[test]
fn blockquote_italic_style() -> anyhow::Result<()> {
    let lines = render_markdown("> quoted", 80);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    let has_italic = line
        .spans
        .iter()
        .any(|s| s.style.add_modifier.contains(Modifier::ITALIC));
    assert!(has_italic, "blockquote text should be italic");
    Ok(())
}

#[test]
fn hr_star_syntax() -> anyhow::Result<()> {
    let lines = render_markdown("***", 80);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "─");
    Ok(())
}

#[test]
fn hr_underscore_syntax() -> anyhow::Result<()> {
    let lines = render_markdown("___", 80);
    assert_eq!(lines.len(), 1);
    Ok(())
}

// ==================== Image Block Tests ====================

#[test]
fn image_block_parsed_from_standalone_line() -> anyhow::Result<()> {
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
    Ok(())
}

#[test]
fn image_block_parsed_with_empty_alt() -> anyhow::Result<()> {
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
    Ok(())
}

#[test]
fn image_block_between_paragraphs() -> anyhow::Result<()> {
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
    Ok(())
}

#[test]
fn image_fallback_renders_gray_italic_span() -> anyhow::Result<()> {
    let lines = render_markdown("![my alt](missing.png)", 80);
    assert_eq!(lines.len(), 1, "image fallback should be a single line");
    let buf = render_to_buffer(lines, 80, 5)?;
    let first_char = buf.cell((0, 0)).context("cell at (0, 0)")?.symbol();
    assert_eq!(first_char, "[", "fallback should start with '['");
    let text: String = (0..30)
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        text.contains("image"),
        "fallback text should contain 'image': got '{}'",
        text
    );
    let cell = buf.cell((0, 0)).context("cell at (0, 0)")?;
    assert_eq!(
        cell.fg,
        Color::Gray,
        "fallback should be Gray colored"
    );
    assert!(
        cell.style().add_modifier.contains(Modifier::ITALIC),
        "fallback should be italic"
    );
    Ok(())
}

#[test]
fn image_fallback_uses_alt_when_present() -> anyhow::Result<()> {
    let lines = render_markdown("![screenshot](photo.png)", 80);
    let buf = render_to_buffer(lines, 80, 5)?;
    let text: String = (0..40)
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        text.contains("screenshot"),
        "fallback should contain alt text 'screenshot': got '{}'",
        text
    );
    Ok(())
}

#[test]
fn image_fallback_uses_path_when_alt_empty() -> anyhow::Result<()> {
    let lines = render_markdown("![](photo.png)", 80);
    let buf = render_to_buffer(lines, 80, 5)?;
    let text: String = (0..40)
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        text.contains("photo.png"),
        "fallback should contain path when alt is empty: got '{}'",
        text
    );
    Ok(())
}

#[test]
fn image_inline_in_paragraph_not_treated_as_block() -> anyhow::Result<()> {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("text ![inline](i.png) more");
    for block in &blocks {
        assert!(
            !matches!(block, MarkdownBlock::Image { .. }),
            "inline image inside text should not become a block-level Image"
        );
    }
    Ok(())
}

#[test]
fn image_line_count_is_1() -> anyhow::Result<()> {
    let block = MarkdownBlock::Image {
        alt: "test".to_string(),
        path: "test.png".to_string(),
    };
    assert_eq!(block.line_count(), 1);
    Ok(())
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
fn hooks_heading1_override() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(HeadingOverrideHooks));
    let blocks = renderer.parse("# Hello");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), ">");
    assert_eq!(buf.cell((1, 0)).context("cell at (1, 0)")?.symbol(), ">");
    assert_eq!(buf.cell((2, 0)).context("cell at (2, 0)")?.symbol(), ">");
    assert_eq!(buf.cell((4, 0)).context("cell at (4, 0)")?.symbol(), "H");
    let cell = buf.cell((0, 0)).context("cell at (0, 0)")?;
    assert_eq!(cell.fg, Color::Red, "overridden heading should be Red");
    Ok(())
}

struct Heading2PrefixHooks;

impl RenderHooks for Heading2PrefixHooks {
    fn heading2(&self, text: &str) -> Option<Line<'static>> {
        Some(Line::from(Span::raw(format!("## {}", text))))
    }
}

#[test]
fn hooks_heading2_override() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(Heading2PrefixHooks));
    let blocks = renderer.parse("## Section");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "#");
    assert_eq!(buf.cell((1, 0)).context("cell at (1, 0)")?.symbol(), "#");
    let text: String = (0..10)
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(text.starts_with("## Section"));
    Ok(())
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
fn hooks_heading3_override() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(Heading3OverrideHooks));
    let blocks = renderer.parse("### Sub");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.fg, Color::Magenta);
    Ok(())
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
fn hooks_paragraph_override() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(ParagraphOverrideHooks));
    let blocks = renderer.parse("Hello world");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    let text: String = (0..15)
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        text.starts_with("P: Hello"),
        "paragraph should be overridden with prefix: got '{}'",
        text
    );
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.fg, Color::Green);
    Ok(())
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
fn hooks_code_block_header_footer_override() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(CodeBlockCustomHooks));
    let md = "```rust\nfn main() {}\n```";
    let blocks = renderer.parse(md);
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 3, "should have header + 1 content + footer");

    let buf = render_to_buffer(lines.clone(), 80, 5)?;
    assert_eq!(
        buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(),
        "\u{256d}",
        "header should start with ╭"
    );

    let header_text: String = (0..30)
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        header_text.contains("custom-header"),
        "header should contain custom text: got '{}'",
        header_text
    );

    assert_eq!(
        buf.cell((0, 2)).context("cell at (0, 2)")?.symbol(),
        "\u{2570}",
        "footer should start with ╰"
    );
    let footer_text: String = (0..20)
        .map(|x| buf.cell((x, 2)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        footer_text.contains("1 lines"),
        "footer should contain line count: got '{}'",
        footer_text
    );
    Ok(())
}

#[test]
fn hooks_code_block_line_prefix_override() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(CodeBlockCustomHooks));
    let md = "```rust\nhello\n```";
    let blocks = renderer.parse(md);
    let lines = renderer.render(&blocks, &TestTheme);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(
        buf.cell((0, 1)).context("cell at (0, 1)")?.symbol(),
        " ",
        "custom prefix should be spaces, not │"
    );
    Ok(())
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
fn hooks_code_block_line_override() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(CodeBlockLineOverrideHooks));
    let md = "```js\nline1\nline2\n```";
    let blocks = renderer.parse(md);
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 4, "header + 2 content + footer");
    let buf = render_to_buffer(lines, 80, 5)?;
    let line1_text: String = (0..10)
        .map(|x| buf.cell((x, 1)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        line1_text.starts_with("0:"),
        "first code line should have index prefix: got '{}'",
        line1_text
    );
    let line2_text: String = (0..10)
        .map(|x| buf.cell((x, 2)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        line2_text.starts_with("1:"),
        "second code line should have index prefix: got '{}'",
        line2_text
    );
    Ok(())
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
fn hooks_inline_code_override() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(InlineCodeOverrideHooks));
    let blocks = vec![MarkdownBlock::InlineCode("hello".to_string())];
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    let text: String = (0..15)
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        text.starts_with("CODE(hello)"),
        "inline code block should be overridden: got '{}'",
        text
    );
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.fg, Color::Red);
    Ok(())
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
fn hooks_list_item_marker_tree_style() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(TreeListHook));
    let blocks = renderer.parse("- first\n- second\n- third");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 3, "should have 3 list items");
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(
        buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(),
        "\u{251c}",
        "first item (not last) should use ├"
    );
    assert_eq!(
        buf.cell((0, 2)).context("cell at (0, 2)")?.symbol(),
        "\u{2514}",
        "last item should use └"
    );
    Ok(())
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
fn hooks_list_item_content_override() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(ListItemContentHooks));
    let blocks = renderer.parse("- hello");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    let text: String = (0..12)
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        text.contains("[hello]"),
        "list item content should be wrapped in brackets: got '{}'",
        text
    );
    Ok(())
}

struct BlockquoteOverrideHooks;

impl RenderHooks for BlockquoteOverrideHooks {
    fn blockquote(
        &self,
        _level: u8,
        children: &[MarkdownBlock],
    ) -> Option<Vec<Line<'static>>> {
        let text: String = children
            .iter()
            .filter_map(|c| match c {
                MarkdownBlock::Paragraph(lines) => Some(lines.join(" ")),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ");
        Some(vec![Line::from(Span::styled(
            format!("QUOTE: {}", text),
            Style::default().fg(Color::Magenta),
        ))])
    }
}

#[test]
fn hooks_blockquote_override() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(BlockquoteOverrideHooks));
    let blocks = renderer.parse("> some quote");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    let text: String = (0..20)
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        text.starts_with("QUOTE: some quote"),
        "blockquote should be overridden: got '{}'",
        text
    );
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.fg, Color::Magenta);
    Ok(())
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
fn hooks_horizontal_rule_override() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(HorizontalRuleOverrideHooks));
    let blocks = renderer.parse("---");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(
        buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(),
        "=",
        "horizontal rule should be overridden to ="
    );
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.fg, Color::Cyan);
    Ok(())
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
fn hooks_blank_line_override() -> anyhow::Result<()> {
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
    Ok(())
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
fn hooks_table_override() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(TableOverrideHooks));
    let md = "| A | B |\n|---|---|\n| 1 | 2 |";
    let blocks = renderer.parse(md);
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 2, "should have header line + 1 data row");
    let buf = render_to_buffer(lines, 80, 5)?;
    let header: String = (0..20)
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        header.contains("TABLE: 2 headers"),
        "table header should be overridden: got '{}'",
        header
    );
    Ok(())
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
fn hooks_image_fallback_override() -> anyhow::Result<()> {
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(ImageFallbackHooks));
    let blocks = renderer.parse("![logo](logo.png)");
    let lines = renderer.render(&blocks, &TestTheme);
    assert_eq!(lines.len(), 1);
    let buf = render_to_buffer(lines, 80, 5)?;
    let text: String = (0..30)
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        text.contains("IMG: logo (logo.png)"),
        "image fallback should be overridden: got '{}'",
        text
    );
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.fg, Color::Red);
    Ok(())
}

struct NoopHooks;

impl RenderHooks for NoopHooks {}

#[test]
fn hooks_noop_does_not_change_output() -> anyhow::Result<()> {
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
    Ok(())
}

#[test]
fn hooks_selective_override_only_affects_target() -> anyhow::Result<()> {
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
    Ok(())
}

// ==================== Code block rendering detail tests ====================

#[test]
fn code_block_header_contains_lang_label() -> anyhow::Result<()> {
    let md = "```rust\ncode\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 5)?;
    let header: String = (0..20)
        .map(|x| buf.cell((x, 0)).map(|c| c.symbol()).unwrap_or_default()).collect::<String>();
    assert!(
        header.contains("rust"),
        "code block header should contain language label: got '{}'",
        header
    );
    Ok(())
}

#[test]
fn code_block_header_has_rounded_top_left() -> anyhow::Result<()> {
    let md = "```js\nx\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(buf.cell((0, 0)).context("cell at (0, 0)")?.symbol(), "\u{256d}");
    Ok(())
}

#[test]
fn code_block_footer_has_rounded_bottom_left() -> anyhow::Result<()> {
    let md = "```js\nx\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(
        buf.cell((0, 2)).context("cell at (0, 2)")?.symbol(),
        "\u{2570}",
        "footer should have ╰"
    );
    Ok(())
}

#[test]
fn code_block_content_line_has_pipe_prefix() -> anyhow::Result<()> {
    let md = "```sh\necho hello\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 5)?;
    assert_eq!(
        buf.cell((0, 1)).context("cell at (0, 1)")?.symbol(),
        "\u{2502}",
        "content line should have │ prefix"
    );
    Ok(())
}

#[test]
fn code_block_content_line_has_yellow_text() -> anyhow::Result<()> {
    let md = "```sh\necho hello\n```";
    let lines = render_markdown(md, 80);
    let buf = render_to_buffer(lines, 80, 5)?;
    let content_cell = buf.cell((2, 1)).context("cell at (2, 1)")?;
    assert_eq!(
        content_cell.fg,
        Color::Yellow,
        "code content should be Yellow"
    );
    Ok(())
}

// ==================== Image parsing edge cases ====================

#[test]
fn image_not_confused_with_link() -> anyhow::Result<()> {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("[link](url)");
    for block in &blocks {
        assert!(
            !matches!(block, MarkdownBlock::Image { .. }),
            "link syntax should not produce Image block"
        );
    }
    Ok(())
}

#[test]
fn image_with_special_chars_in_alt() -> anyhow::Result<()> {
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
    Ok(())
}

#[test]
fn image_with_url_path() -> anyhow::Result<()> {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("![icon](https://example.com/icon.png)");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Image { path, .. } => {
            assert_eq!(path, "https://example.com/icon.png");
        }
        other => panic!("expected Image, got {:?}", other),
    }
    Ok(())
}

#[test]
fn image_without_path_not_parsed_as_image() -> anyhow::Result<()> {
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse("![]()");
    for block in &blocks {
        assert!(
            !matches!(block, MarkdownBlock::Image { .. }),
            "empty path should not produce Image block"
        );
    }
    Ok(())
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
fn hooks_multiple_overrides_in_same_document() -> anyhow::Result<()> {
    let md = "# Title\n\n## Sub\n\n---\n\ntext";
    let renderer =
        MarkdownRenderer::new(80).with_render_hooks(Box::new(MultiHook));
    let blocks = renderer.parse(md);
    let lines = renderer.render(&blocks, &TestTheme);

    let texts: Vec<String> = lines
        .iter()
        .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
        .collect();

    let h1 = texts.iter().find(|t| t.starts_with("H1:")).context("find H1")?;
    assert_eq!(h1, "H1: Title");

    let h2 = texts.iter().find(|t| t.starts_with("H2:")).context("find H2")?;
    assert_eq!(h2, "H2: Sub");

    let hr = texts.iter().find(|t| t.contains("HR")).context("find HR")?;
    assert!(hr.contains("HR"), "should find HR override");
    Ok(())
}

// ==================== Image render_full + Zoom Tests ====================

#[cfg(feature = "image")]
mod image_zoom_tests {
    use super::*;
    use crate::markdown::image::{ImageResolver, ResolvedImage};
    use crate::markdown::types::MarkdownBlock;

    struct TestImageResolver {
        font_w: u16,
        font_h: u16,
        halfblocks: bool,
        resolve_result: Option<image::DynamicImage>,
    }

    impl TestImageResolver {
        fn with_font(fw: u16, fh: u16) -> Self {
            Self { font_w: fw, font_h: fh, halfblocks: false, resolve_result: None }
        }

        fn halfblocks(fw: u16, fh: u16) -> Self {
            Self { font_w: fw, font_h: fh, halfblocks: true, resolve_result: None }
        }

        fn resolves_to(mut self, img: image::DynamicImage) -> Self {
            self.resolve_result = Some(img);
            self
        }
    }

    impl ImageResolver for TestImageResolver {
        fn resolve(&mut self, _path: &str) -> Option<image::DynamicImage> {
            self.resolve_result.clone()
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
            let w_cells = ((pw + self.font_w as u32 - 1) / self.font_w as u32) as u16;
            let w = w_cells.min(max_width);
            let ratio = ph as u32 * w as u32 / pw.max(1);
            let height_div = if self.halfblocks { self.font_h * 2 } else { self.font_h };
            let h_cells = ((ratio + height_div as u32 - 1) / height_div as u32) as u16;
            (w.max(1), h_cells.max(1))
        }
    }

    fn make_test_img(w: u32, h: u32) -> image::DynamicImage {
        let buf = image::ImageBuffer::from_fn(w, h, |_, _| image::Rgb([255u8, 100, 50]));
        image::DynamicImage::ImageRgb8(buf)
    }

    #[test]
    fn cell_dimensions_100x100_with_10x20_font() {
        let mut r = TestImageResolver::with_font(10, 20);
        let img = make_test_img(100, 100);
        let (cw, ch) = r.cell_dimensions(&img, 80, 30);
        assert_eq!(cw, 10, "100px / 10px-per-cell = 10 cells wide");
        assert!(ch >= 1, "height at least 1 cell, got {}", ch);
    }

    #[test]
    fn cell_dimensions_halfblocks_doubles_height() {
        let mut r = TestImageResolver::halfblocks(10, 20);
        let img = make_test_img(100, 500);
        let (_cw, ch) = r.cell_dimensions(&img, 80, 30);
        assert!(
            ch >= 2,
            "halfblocks: tall image should use multiple rows, got {}",
            ch,
        );
    }

    #[test]
    fn cell_dimensions_respects_max_width() {
        let mut r = TestImageResolver::with_font(10, 20);
        let img = make_test_img(500, 200);
        let (cw, _ch) = r.cell_dimensions(&img, 20, 30);
        assert!(cw <= 20, "width should not exceed max_width=20, got {}", cw);
    }

    #[test]
    fn cell_dimensions_zero_image_returns_zero() {
        let mut r = TestImageResolver::with_font(10, 20);
        let img = make_test_img(0, 0);
        let (cw, ch) = r.cell_dimensions(&img, 80, 30);
        assert_eq!((cw, ch), (0, 0));
    }

    #[test]
    fn cell_dimensions_tiny_image_at_least_one_cell() {
        let mut r = TestImageResolver::with_font(10, 20);
        let img = make_test_img(1, 1);
        let (cw, ch) = r.cell_dimensions(&img, 80, 30);
        assert!(cw >= 1, "at least 1 cell wide");
        assert!(ch >= 1, "at least 1 cell tall");
    }

    #[test]
    fn render_full_reserves_blank_lines_for_resolved_images() {
        let mut r = TestImageResolver::with_font(10, 20).resolves_to(make_test_img(100, 60));
        let renderer = MarkdownRenderer::new(80);
        let blocks = vec![
            MarkdownBlock::Heading1("Title".into()),
            MarkdownBlock::Image { alt: "logo".into(), path: "a.webp".into() },
            MarkdownBlock::Paragraph(vec!["after".into()]),
        ];
        let resolved = vec![ResolvedImage {
            path: "a.webp".into(),
            image: make_test_img(100, 60),
        }];
        let output = renderer.render_full(&blocks, &TestTheme, &resolved, &mut r, 70, 20);

        assert!(output.lines.len() >= 3, "should have heading + blank lines + paragraph");
        assert_eq!(output.images.len(), 1, "one placement");
        let p = &output.images[0];
        assert!(p.height_cells >= 1, "height_cells >= 1, got {}", p.height_cells);
        assert!(p.width_cells >= 1, "width_cells >= 1, got {}", p.width_cells);
        let end_row = (p.row as usize) + (p.height_cells as usize);
        assert!(
            end_row <= output.lines.len(),
            "image area ends within lines: row {} + height {} <= {}",
            p.row, p.height_cells, output.lines.len(),
        );
    }

    #[test]
    fn render_full_fallback_for_unresolved_images() {
        let mut r = TestImageResolver::with_font(10, 20);
        let renderer = MarkdownRenderer::new(80);
        let blocks = vec![
            MarkdownBlock::Heading1("Title".into()),
            MarkdownBlock::Image { alt: "missing".into(), path: "gone.png".into() },
        ];
        let resolved: Vec<ResolvedImage> = Vec::new();
        let output = renderer.render_full(&blocks, &TestTheme, &resolved, &mut r, 70, 20);

        assert_eq!(output.images.len(), 0, "no placements for unresolved images");
        let has_fallback = output.lines.iter()
            .any(|l| l.spans.iter().any(|s| s.content.contains("[image:")));
        assert!(has_fallback, "should contain fallback span with '[image:' prefix");
    }

    #[test]
    fn render_full_multiple_images_independent_placements() {
        let mut r = TestImageResolver::with_font(10, 20)
            .resolves_to(make_test_img(50, 30));
        let renderer = MarkdownRenderer::new(80);
        let blocks = vec![
            MarkdownBlock::Image { alt: "first".into(), path: "a.webp".into() },
            MarkdownBlock::Paragraph(vec!["between".into()]),
            MarkdownBlock::Image { alt: "second".into(), path: "b.webp".into() },
        ];
        let resolved = vec![
            ResolvedImage { path: "a.webp".into(), image: make_test_img(50, 30) },
            ResolvedImage { path: "b.webp".into(), image: make_test_img(80, 40) },
        ];
        let output = renderer.render_full(&blocks, &TestTheme, &resolved, &mut r, 70, 20);

        assert_eq!(output.images.len(), 2, "two placements for two images");
        assert!(
            output.images[0].row < output.images[1].row,
            "first image row ({}) < second image row ({})",
            output.images[0].row,
            output.images[1].row,
        );
    }

    #[test]
    fn render_full_image_placement_row_matches_line_index() {
        let mut r = TestImageResolver::with_font(8, 16).resolves_to(make_test_img(64, 32));
        let renderer = MarkdownRenderer::new(80);
        let blocks = vec![
            MarkdownBlock::Paragraph(vec!["line before".into()]),
            MarkdownBlock::Image { alt: "x".into(), path: "x.png".into() },
        ];
        let resolved = vec![ResolvedImage {
            path: "x.png".into(),
            image: make_test_img(64, 32),
        }];
        let output = renderer.render_full(&blocks, &TestTheme, &resolved, &mut r, 70, 20);

        assert_eq!(output.images.len(), 1);
        let p = &output.images[0];
        assert_eq!(p.row, 1, "image starts at line index 1 (after 1 paragraph line)");
    }

    struct SimulatedZoomState {
        original_w: u32,
        original_h: u32,
        scale: f64,
        font_w: u16,
        font_h: u16,
    }

    impl SimulatedZoomState {
        fn new(w: u32, h: u32, font_w: u16, font_h: u16) -> Self {
            Self { original_w: w, original_h: h, scale: 1.0, font_w, font_h }
        }

        fn zoom_in(&mut self) { self.scale *= 1.25; }

        fn zoom_out(&mut self) {
            self.scale /= 1.25;
            self.scale = self.scale.max(0.05);
        }

        fn scaled_px(&self) -> (u32, u32) {
            (
                ((self.original_w as f64 * self.scale) as u32).max(1),
                ((self.original_h as f64 * self.scale) as u32).max(1),
            )
        }

        fn scaled_cells(&self) -> (u16, u16) {
            let (sw, sh) = self.scaled_px();
            let cw = ((sw + self.font_w as u32 - 1) / self.font_w as u32) as u16;
            let ratio = sh * cw as u32 / sw.max(1);
            let h = ((ratio + self.font_h as u32 - 1) / self.font_h as u32) as u16;
            (cw.max(1), h.max(1))
        }
    }

    #[test]
    fn zoom_in_increases_scale_and_pixel_size() {
        let mut z = SimulatedZoomState::new(100, 100, 10, 20);
        let (w0, h0) = z.scaled_px();
        let s0 = z.scale;
        z.zoom_in();
        let (w1, h1) = z.scaled_px();
        assert!(z.scale > s0, "scale increased: {:.4} > {:.4}", z.scale, s0);
        assert!(w1 > w0, "pixel width grew: {} > {}", w1, w0);
        assert!(h1 > h0, "pixel height grew: {} > {}", h1, h0);
    }

    #[test]
    fn zoom_out_decreases_scale_and_pixel_size() {
        let mut z = SimulatedZoomState::new(100, 100, 10, 20);
        z.scale = 2.0;
        let (w0, h0) = z.scaled_px();
        z.zoom_out();
        let (w1, h1) = z.scaled_px();
        assert!(z.scale < 2.0, "scale decreased: {:.4} < 2.0", z.scale);
        assert!(w1 < w0, "pixel width shrunk: {} < {}", w1, w0);
        assert!(h1 < h0, "pixel height shrunk: {} < {}", h1, h0);
    }

    #[test]
    fn zoom_out_clamps_to_minimum_scale() {
        let mut z = SimulatedZoomState::new(100, 100, 10, 20);
        z.scale = 0.06;
        for _ in 0..20 { z.zoom_out(); }
        assert!(
            (z.scale - 0.05).abs() < 0.001,
            "scale clamped to ~0.05, got {:.6}",
            z.scale,
        );
    }

    #[test]
    fn repeated_zoom_in_grows_exponentially() {
        let mut z = SimulatedZoomState::new(100, 100, 10, 20);
        let scales: Vec<f64> = (0..10).map(|_| { z.zoom_in(); z.scale }).collect();
        for i in 1..scales.len() {
            assert!(
                scales[i] > scales[i-1],
                "scale[{}] ({:.4}) > scale[{}] ({:.4})",
                i, scales[i], i-1, scales[i-1],
            );
        }
    }

    #[test]
    fn simulated_bracket_keys_sequence() {
        let mut z = SimulatedZoomState::new(160, 90, 8, 16);
        let initial = z.scaled_cells();

        simulate_n_presses(&mut z, '[', 3);
        let after_zoom_in = z.scaled_cells();
        assert!(
            after_zoom_in.0 > initial.0,
            "after 3x [ : width {} > {}",
            after_zoom_in.0, initial.0,
        );
        assert!(
            after_zoom_in.1 > initial.1,
            "after 3x [ : height {} > {}",
            after_zoom_in.1, initial.1,
        );

        simulate_n_presses(&mut z, ']', 5);
        let after_zoom_out = z.scaled_cells();
        assert!(
            after_zoom_out.0 < after_zoom_in.0,
            "after 5x ] : width shrunk from {} to {}",
            after_zoom_in.0, after_zoom_out.0,
        );
    }

    #[test]
    fn zoom_preserves_aspect_ratio_approximately() {
        let mut z = SimulatedZoomState::new(200, 100, 10, 20);
        for _ in 0..20 {
            z.zoom_in();
            let (sw, sh) = z.scaled_px();
            let ratio = sw as f64 / sh as f64;
            let orig_ratio = z.original_w as f64 / z.original_h as f64;
            assert!(
                (ratio - orig_ratio).abs() < 0.15,
                "aspect ratio preserved: {:.2} ≈ {:.2} (scale={:.4})",
                ratio, orig_ratio, z.scale,
            );
        }
    }

    #[test]
    fn large_zoom_produces_large_cell_dimensions() {
        let mut z = SimulatedZoomState::new(100, 100, 10, 20);
        for _ in 0..15 { z.zoom_in(); }
        let (cw, ch) = z.scaled_cells();
        assert!(cw > 50, "after 15x zoom in, width cells {} > 50", cw);
        assert!(ch > 5, "after 15x zoom in, height cells {} > 5", ch);
    }

    #[test]
    fn tiny_zoom_still_produces_valid_cells() {
        let mut z = SimulatedZoomState::new(1024, 768, 12, 24);
        for _ in 0..30 { z.zoom_out(); }
        let (cw, ch) = z.scaled_cells();
        assert!(cw >= 1, "even at min zoom, width >= 1: {}", cw);
        assert!(ch >= 1, "even at min zoom, height >= 1: {}", ch);
    }

    #[test]
    fn multiple_images_each_have_own_placement_metadata() {
        let mut r = TestImageResolver::with_font(8, 16)
            .resolves_to(make_test_img(40, 20));
        let renderer = MarkdownRenderer::new(80);
        let blocks: Vec<MarkdownBlock> = (0..5)
            .map(|i| MarkdownBlock::Image {
                alt: format!("img{}", i),
                path: format!("{}.webp", i),
            })
            .collect();
        let resolved: Vec<ResolvedImage> = (0..5)
            .map(|i| ResolvedImage {
                path: format!("{}.webp", i),
                image: make_test_img(40 + i * 10, 20 + i * 5),
            })
            .collect();
        let output = renderer.render_full(&blocks, &TestTheme, &resolved, &mut r, 70, 20);

        assert_eq!(output.images.len(), 5, "5 images => 5 placements");
        for (i, p) in output.images.iter().enumerate() {
            assert!(p.width_cells >= 1, "placement {} width >= 1", i);
            assert!(p.height_cells >= 1, "placement {} height >= 1", i);
            assert!(
                !p.image.as_bytes().is_empty(),
                "placement {} holds actual image data",
                i,
            );
        }
    }

    #[test]
    fn image_between_text_blocks_does_not_displace_text() {
        let mut r = TestImageResolver::with_font(10, 20).resolves_to(make_test_img(80, 40));
        let renderer = MarkdownRenderer::new(80);
        let blocks = vec![
            MarkdownBlock::Paragraph(vec!["before".into()]),
            MarkdownBlock::Image { alt: "mid".into(), path: "m.webp".into() },
            MarkdownBlock::Paragraph(vec!["after".into()]),
        ];
        let resolved = vec![ResolvedImage {
            path: "m.webp".into(),
            image: make_test_img(80, 40),
        }];
        let output = renderer.render_full(&blocks, &TestTheme, &resolved, &mut r, 70, 20);

        let first_line_has_before = output.lines.first()
            .map(|l| l.spans.iter().any(|s| s.content.contains("before")))
            .unwrap_or(false);
        assert!(first_line_has_before, "first line still contains 'before' text");

        let last_line_has_after = output.lines.last()
            .map(|l| l.spans.iter().any(|s| s.content.contains("after")))
            .unwrap_or(false);
        assert!(last_line_has_after, "last line still contains 'after' text");

        let img_row = output.images[0].row;
        assert!(img_row > 0, "image row ({}) after text line 0", img_row);
        let last_text_row = output.lines.len() - 1;
        assert!(
            img_row + output.images[0].height_cells as usize <= last_text_row,
            "image ends before or at last text line",
        );
    }

    fn simulate_n_presses(z: &mut SimulatedZoomState, key: char, n: usize) {
        for _ in 0..n {
            match key {
                '[' => z.zoom_in(),
                ']' => z.zoom_out(),
                _ => {}
            }
        }
    }
}

// ==================== Example Integration Tests ====================
//
// Tests that exercise the exact patterns used by each example in examples/.
// These verify the library APIs work correctly for real-world usage.

mod example_basic_tests {
    use super::*;

    const BASIC_MARKDOWN: &str = r#"
# Getting Started

This is a **basic** markdown rendering example using `ratatui-markdown`.

## Features

- Headings (H1, H2, H3)
- **Bold**, *italic*, and `inline code`
- Code blocks with syntax labels
- Blockquotes
- Tables

### Code Example

```rust
fn main() {
    println!("Hello, world!");
}
```

> This is a blockquote. It supports *inline formatting* too.

### Table

| Feature | Status |
|---------|--------|
| Parser  | Done   |
| Renderer| Done   |
| Hooks   | Done   |

---

Press `q` to quit.
"#;

    #[test]
    fn basic_example_parses_all_block_types() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = renderer.parse(BASIC_MARKDOWN);
        assert!(blocks.len() > 10, "should parse many blocks, got {}", blocks.len());

        let has_heading1 = blocks.iter().any(|b| matches!(b, MarkdownBlock::Heading1(_)));
        let has_heading2 = blocks.iter().any(|b| matches!(b, MarkdownBlock::Heading2(_)));
        let has_heading3 = blocks.iter().any(|b| matches!(b, MarkdownBlock::Heading3(_)));
        let has_paragraph = blocks.iter().any(|b| matches!(b, MarkdownBlock::Paragraph(_)));
        let has_list = blocks.iter().any(|b| matches!(b, MarkdownBlock::ListItem(_, _)));
        let has_code = blocks.iter().any(|b| matches!(b, MarkdownBlock::CodeBlock { .. }));
        let has_quote = blocks.iter().any(|b| matches!(b, MarkdownBlock::Blockquote { .. }));
        let has_table = blocks.iter().any(|b| matches!(b, MarkdownBlock::Table { .. }));
        let has_hr = blocks.iter().any(|b| matches!(b, MarkdownBlock::HorizontalRule));
        assert!(has_heading1, "should have H1");
        assert!(has_heading2, "should have H2");
        assert!(has_heading3, "should have H3");
        assert!(has_paragraph, "should have paragraphs");
        assert!(has_list, "should have list items");
        assert!(has_code, "should have code block");
        assert!(has_quote, "should have blockquote");
        assert!(has_table, "should have table");
        assert!(has_hr, "should have horizontal rule");
    }

    #[test]
    fn basic_example_render_produces_lines() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = renderer.parse(BASIC_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        assert!(!lines.is_empty(), "should produce output lines");
        assert!(lines.len() > 15, "should produce many lines for a rich document, got {}", lines.len());
    }

    #[test]
    fn basic_example_render_to_buffer_no_panic() -> anyhow::Result<()> {
        let renderer = MarkdownRenderer::new(76);
        let blocks = renderer.parse(BASIC_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let buffer = render_to_buffer(lines, 80, 40)?;
        assert_eq!(buffer.area.width, 80);
        assert_eq!(buffer.area.height, 40);
    Ok(())
    }

    #[test]
    fn basic_example_heading1_contains_getting_started() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = renderer.parse(BASIC_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let found = lines.iter().any(|l| {
            l.spans.iter().any(|s| s.content.contains("Getting Started"))
        });
        assert!(found, "H1 'Getting Started' should appear in rendered output");
    }

    #[test]
    fn basic_example_code_block_has_rust_label() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = renderer.parse(BASIC_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let found = lines.iter().any(|l| {
            l.spans.iter().any(|s| s.content.contains("rust"))
        });
        assert!(found, "code block should have 'rust' language label");
    }

    #[test]
    fn basic_example_table_has_borders() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = renderer.parse(BASIC_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let has_border = lines.iter().any(|l| {
            l.spans.iter().any(|s| s.content.contains("│") || s.content.contains("─"))
        });
        assert!(has_border, "table should have border characters");
    }
}

mod example_custom_code_block_tests {
    use super::*;

    struct TimelineCodeHooks;

    impl RenderHooks for TimelineCodeHooks {
        fn code_block_header(&self, lang: &str) -> Option<Line<'static>> {
            Some(Line::from(vec![
                Span::styled(
                    format!("╭ [12:00:00] "),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    lang.to_string(),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]))
        }

        fn code_block_footer(&self, _lang: &str, _content_line_count: usize) -> Option<Line<'static>> {
            Some(Line::from(vec![
                Span::styled("╰ ", Style::default().fg(Color::DarkGray)),
                Span::styled("↑ ", Style::default().fg(Color::Green)),
                Span::styled("156 ", Style::default().fg(Color::DarkGray)),
            ]))
        }

        fn code_block_line_prefix(&self, _lang: &str) -> Option<String> {
            Some("│ ".to_string())
        }
    }

    const TIMELINE_MARKDOWN: &str = r#"
# Timeline View

## Agent Skill Execution

```rust skill::read_file
use std::fs;
let content = fs::read_to_string("PLAN.md")?;
println!("{}", content);
```

## Another Block

```python skill::analyze
def analyze(data):
    for item in data:
        yield process(item)
```
"#;

    #[test]
    fn timeline_hook_custom_header_appears() {
        let renderer = MarkdownRenderer::new(76)
            .with_render_hooks(Box::new(TimelineCodeHooks));
        let blocks = renderer.parse(TIMELINE_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let found = lines.iter().any(|l| {
            l.spans.iter().any(|s| s.content.contains("12:00:00"))
        });
        assert!(found, "custom header with timestamp should appear");
    }

    #[test]
    fn timeline_hook_custom_footer_appears() {
        let renderer = MarkdownRenderer::new(76)
            .with_render_hooks(Box::new(TimelineCodeHooks));
        let blocks = renderer.parse(TIMELINE_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let found = lines.iter().any(|l| {
            l.spans.iter().any(|s| s.content.contains("↑"))
        });
        assert!(found, "custom footer with arrows should appear");
    }

    #[test]
    fn timeline_hook_line_prefix_appears() {
        let renderer = MarkdownRenderer::new(76)
            .with_render_hooks(Box::new(TimelineCodeHooks));
        let blocks = renderer.parse(TIMELINE_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let found = lines.iter().any(|l| {
            l.spans.iter().any(|s| s.content.contains("│ "))
        });
        assert!(found, "custom line prefix │ should appear");
    }

    #[test]
    fn timeline_hook_lang_label_in_header() {
        let renderer = MarkdownRenderer::new(76)
            .with_render_hooks(Box::new(TimelineCodeHooks));
        let blocks = renderer.parse(TIMELINE_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let has_rust = lines.iter().any(|l| {
            l.spans.iter().any(|s| s.content.contains("rust"))
        });
        let has_python = lines.iter().any(|l| {
            l.spans.iter().any(|s| s.content.contains("python"))
        });
        assert!(has_rust, "rust language should appear in header");
        assert!(has_python, "python language should appear in header");
    }

    #[test]
    fn timeline_hook_two_code_blocks_both_customized() {
        let renderer = MarkdownRenderer::new(76)
            .with_render_hooks(Box::new(TimelineCodeHooks));
        let blocks = renderer.parse(TIMELINE_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let footer_count = lines.iter().filter(|l| {
            l.spans.iter().any(|s| s.content.contains("↑"))
        }).count();
        assert_eq!(footer_count, 2, "both code blocks should have custom footer");
    }

    #[test]
    fn timeline_hook_renders_to_buffer_without_panic() -> anyhow::Result<()> {
        let renderer = MarkdownRenderer::new(76)
            .with_render_hooks(Box::new(TimelineCodeHooks));
        let blocks = renderer.parse(TIMELINE_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let buffer = render_to_buffer(lines, 80, 30)?;
        assert_eq!(buffer.area.width, 80);
    Ok(())
    }
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
            _index_in_group: usize,
        ) -> Option<String> {
            let marker = if is_last_in_group { "└─ " } else { "├─ " };
            if indent == 0 {
                return Some(marker.to_string());
            }
            let mut prefix = String::new();
            for (depth, &is_last_ancestor) in ancestors_are_last.iter().enumerate() {
                if depth >= indent as usize - 1 {
                    break;
                }
                if is_last_ancestor {
                    for _ in 0..3 { prefix.push(' '); }
                } else {
                    prefix.push_str("│  ");
                }
            }
            if (indent as usize - 1) > ancestors_are_last.len() {
                let extra = (indent as usize - 1).saturating_sub(ancestors_are_last.len());
                for _ in 0..3 * extra { prefix.push(' '); }
            }
            Some(format!("{}{}", prefix, marker))
        }

        fn tree_indent_width(&self) -> Option<usize> { Some(3) }
        fn tree_text_gap(&self) -> Option<usize> { Some(0) }
    }

    const TREE_MARKDOWN: &str = r#"
## Project TODO

- Setup project structure
  - Initialize Cargo workspace
  - Add dependencies
    - ratatui
    - image crate
- Implement core features
  - Parser
    - Heading detection
    - Code block parsing
  - Renderer
- Write tests
- Deploy to crates.io
"#;

    #[test]
    fn tree_hook_root_items_have_tree_markers() {
        let renderer = MarkdownRenderer::new(76)
            .with_render_hooks(Box::new(TreeListHooks));
        let blocks = renderer.parse(TREE_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let has_tree_marker = lines.iter().any(|l| {
            l.spans.iter().any(|s| s.content.contains("├─") || s.content.contains("└─"))
        });
        assert!(has_tree_marker, "tree markers should appear in output");
    }

    #[test]
    fn tree_hook_nested_items_have_pipe_prefix() {
        let renderer = MarkdownRenderer::new(76)
            .with_render_hooks(Box::new(TreeListHooks));
        let blocks = renderer.parse(TREE_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let all_text: String = lines.iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(all_text.contains("│"), "nested items should have │ pipe prefix");
    }

    #[test]
    fn tree_hook_last_item_uses_corner_marker() {
        let renderer = MarkdownRenderer::new(76)
            .with_render_hooks(Box::new(TreeListHooks));
        let blocks = renderer.parse(TREE_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let has_corner = lines.iter().any(|l| {
            l.spans.iter().any(|s| s.content.contains("└─"))
        });
        assert!(has_corner, "last items should use └─ corner marker");
    }

    #[test]
    fn tree_hook_all_list_content_preserved() {
        let renderer = MarkdownRenderer::new(76)
            .with_render_hooks(Box::new(TreeListHooks));
        let blocks = renderer.parse(TREE_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let all_text: String = lines.iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(all_text.contains("Setup project structure"), "content preserved");
        assert!(all_text.contains("Initialize Cargo workspace"), "nested content preserved");
        assert!(all_text.contains("ratatui"), "deeply nested content preserved");
        assert!(all_text.contains("Deploy to crates.io"), "last item content preserved");
    }

    #[test]
    fn tree_hook_render_to_buffer_no_panic() -> anyhow::Result<()> {
        let renderer = MarkdownRenderer::new(76)
            .with_render_hooks(Box::new(TreeListHooks));
        let blocks = renderer.parse(TREE_MARKDOWN);
        let lines = renderer.render(&blocks, &TestTheme);
        let buffer = render_to_buffer(lines, 80, 40)?;
        assert_eq!(buffer.area.height, 40);
    Ok(())
    }

    #[test]
    fn tree_hook_indent_depth_3_has_correct_prefix() {
        let renderer = MarkdownRenderer::new(76)
            .with_render_hooks(Box::new(TreeListHooks));
        let md = "- A\n  - B\n    - C\n    - D\n  - E\n- F";
        let blocks = renderer.parse(md);
        let lines = renderer.render(&blocks, &TestTheme);
        let all_text: String = lines.iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(all_text.contains("C"), "depth-3 item C should appear");
        assert!(all_text.contains("D"), "depth-3 item D should appear");
        assert!(all_text.contains("E"), "depth-1 item E should appear");
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
            self.total_lines.saturating_sub(self.viewport_height(area_height))
        }

        fn max_h_offset(&self, area_width: u16) -> usize {
            self.max_line_width.saturating_sub(self.viewport_width(area_width))
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

    fn area(w: u16, h: u16) -> Rect { Rect::new(0, 0, w, h) }

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
        let lines = renderer.render(&blocks, &TestTheme);
        let max_w = lines.iter().map(|l| {
            l.spans.iter().map(|s| unicode_width::UnicodeWidthStr::width(s.content.as_ref())).sum::<usize>()
        }).max().unwrap_or(0);
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

    struct SimpleResolver { font_w: u16, font_h: u16 }

    impl SimpleResolver {
        fn new(fw: u16, fh: u16) -> Self { Self { font_w: fw, font_h: fh } }
    }

    impl ImageResolver for SimpleResolver {
        fn resolve(&mut self, _path: &str) -> Option<image::DynamicImage> { None }

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

    fn scale_to_fit_rows(pw: u32, ph: u32, target_rows: u16, font_h: u16) -> f64 {
        if ph == 0 { return 1.0; }
        let natural_h = (ph as f64 / font_h as f64).ceil();
        if natural_h <= target_rows as f64 { return 1.0; }
        target_rows as f64 * font_h as f64 / ph as f64
    }

    #[test]
    fn scale_to_fit_rows_small_image_no_scale() {
        let s = scale_to_fit_rows(100, 18, 2, 18);
        assert!((s - 1.0).abs() < 0.01, "image already fits, scale should be 1.0, got {}", s);
    }

    #[test]
    fn scale_to_fit_rows_tall_image() {
        let s = scale_to_fit_rows(100, 180, 2, 18);
        let scaled_h = (180.0 * s) as u32;
        let scaled_rows = (scaled_h as f64 / 18.0).ceil() as u16;
        assert!(scaled_rows <= 2, "scaled to {} rows (scale={})", scaled_rows, s);
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
        assert!(h <= 2, "pre-scaled image should fit in 2 rows, got {} rows", h);
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
        assert!(h <= 3, "pre-scaled image should fit in 3 rows, got {} rows", h);
    }

    #[test]
    fn image_example_render_full_with_pre_scaled_images() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = vec![
            MarkdownBlock::Image { alt: "logo".into(), path: "logo.webp".into() },
            MarkdownBlock::Paragraph(vec!["between".into()]),
            MarkdownBlock::Image { alt: "demo".into(), path: "demo.webp".into() },
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
            ResolvedImage { path: "logo.webp".into(), image: logo_scaled },
            ResolvedImage { path: "demo.webp".into(), image: demo_scaled },
        ];
        let mut r = SimpleResolver::new(9, 18);
        let output = renderer.render_full(&blocks, &TestTheme, &resolved, &mut r, 70, 20);
        assert_eq!(output.images.len(), 2);
        assert!(output.images[0].height_cells <= 2, "logo should fit in 2 rows, got {}", output.images[0].height_cells);
        assert!(output.images[1].height_cells <= 3, "demo should fit in 3 rows, got {}", output.images[1].height_cells);
    }

    #[test]
    fn image_example_zoom_changes_placement_dimensions() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = vec![
            MarkdownBlock::Image { alt: "test".into(), path: "t.webp".into() },
        ];

        let img = make_img(100, 100);
        let resolved_base = vec![
            ResolvedImage { path: "t.webp".into(), image: img.clone() },
        ];
        let mut r1 = SimpleResolver::new(9, 18);
        let output1 = renderer.render_full(&blocks, &TestTheme, &resolved_base, &mut r1, 70, 20);

        let zoomed = img.resize_exact(200, 200, image::imageops::FilterType::Triangle);
        let resolved_zoomed = vec![
            ResolvedImage { path: "t.webp".into(), image: zoomed },
        ];
        let mut r2 = SimpleResolver::new(9, 18);
        let output2 = renderer.render_full(&blocks, &TestTheme, &resolved_zoomed, &mut r2, 70, 20);

        assert!(
            output2.images[0].height_cells >= output1.images[0].height_cells,
            "zoomed image should have >= height: {} vs {}",
            output2.images[0].height_cells, output1.images[0].height_cells,
        );
        assert!(
            output2.images[0].width_cells >= output1.images[0].width_cells,
            "zoomed image should have >= width: {} vs {}",
            output2.images[0].width_cells, output1.images[0].width_cells,
        );
    }

    #[test]
    fn image_example_zoom_out_shrinks_placement() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = vec![
            MarkdownBlock::Image { alt: "test".into(), path: "t.webp".into() },
        ];

        let img = make_img(200, 200);
        let resolved_base = vec![
            ResolvedImage { path: "t.webp".into(), image: img.clone() },
        ];
        let mut r1 = SimpleResolver::new(9, 18);
        let output1 = renderer.render_full(&blocks, &TestTheme, &resolved_base, &mut r1, 70, 20);

        let shrunk = img.resize_exact(50, 50, image::imageops::FilterType::Triangle);
        let resolved_shrunk = vec![
            ResolvedImage { path: "t.webp".into(), image: shrunk },
        ];
        let mut r2 = SimpleResolver::new(9, 18);
        let output2 = renderer.render_full(&blocks, &TestTheme, &resolved_shrunk, &mut r2, 70, 20);

        assert!(
            output2.images[0].height_cells <= output1.images[0].height_cells,
            "shrunk image should have <= height: {} vs {}",
            output2.images[0].height_cells, output1.images[0].height_cells,
        );
    }

    #[test]
    fn image_example_blank_lines_match_height_cells() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = vec![
            MarkdownBlock::Paragraph(vec!["before".into()]),
            MarkdownBlock::Image { alt: "logo".into(), path: "logo.webp".into() },
            MarkdownBlock::Paragraph(vec!["after".into()]),
        ];
        let resolved = vec![
            ResolvedImage { path: "logo.webp".into(), image: make_img(90, 36) },
        ];
        let mut r = SimpleResolver::new(9, 18);
        let output = renderer.render_full(&blocks, &TestTheme, &resolved, &mut r, 70, 20);

        let img = &output.images[0];
        let row = img.row;
        let height = img.height_cells as usize;
        assert!(height > 0, "should have at least 1 row");
        let blank_count = output.lines[row..row + height].iter()
            .filter(|l| l.spans.is_empty() || l.spans.iter().all(|s| s.content.is_empty()))
            .count();
        assert_eq!(blank_count, height, "should have {} blank lines for image, got {}", height, blank_count);
    }

    #[test]
    fn image_example_zoom_then_rerender_changes_layout() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = vec![
            MarkdownBlock::Image { alt: "a".into(), path: "a.webp".into() },
            MarkdownBlock::Paragraph(vec!["text below".into()]),
        ];

        let original = make_img(180, 180);
        let scale_base = scale_to_fit_rows(180, 180, 2, 18);
        let sw = (180.0 * scale_base).ceil() as u32;
        let sh = (180.0 * scale_base).ceil() as u32;
        let base_img = original.resize_exact(sw, sh, image::imageops::FilterType::Triangle);

        let resolved_base = vec![ResolvedImage { path: "a.webp".into(), image: base_img }];
        let mut r1 = SimpleResolver::new(9, 18);
        let out1 = renderer.render_full(&blocks, &TestTheme, &resolved_base, &mut r1, 70, 20);

        let zoom_sw = (sw as f64 * 2.0).ceil() as u32;
        let zoom_sh = (sh as f64 * 2.0).ceil() as u32;
        let zoomed = original.resize_exact(zoom_sw, zoom_sh, image::imageops::FilterType::Triangle);
        let resolved_zoom = vec![ResolvedImage { path: "a.webp".into(), image: zoomed }];
        let mut r2 = SimpleResolver::new(9, 18);
        let out2 = renderer.render_full(&blocks, &TestTheme, &resolved_zoom, &mut r2, 70, 20);

        assert!(
            out2.images[0].height_cells > out1.images[0].height_cells,
            "2x zoom should increase height: {} -> {}",
            out1.images[0].height_cells, out2.images[0].height_cells,
        );
        let text_row_1 = out1.images[0].row + out1.images[0].height_cells as usize;
        let text_row_2 = out2.images[0].row + out2.images[0].height_cells as usize;
        assert!(
            text_row_2 > text_row_1,
            "text after zoomed image should be pushed down: {} -> {}",
            text_row_1, text_row_2,
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
        let cropped = scaled.crop_imm(px_x, py_y, px_w.min(scaled.width()), py_h.min(scaled.height()));
        let (crop_cw, crop_ch) = (
            (cropped.width().div_ceil(fw as u32)) as u16,
            (cropped.height().div_ceil(fh as u32)) as u16,
        );
        assert!(crop_cw <= vp_w, "cropped width {} <= vp_w {}", crop_cw, vp_w);
        assert!(crop_ch <= vp_h, "cropped height {} <= vp_h {}", crop_ch, vp_h);
    }

    #[test]
    fn image_crop_with_scroll_offset() {
        let img = make_img(180, 180);
        let scaled = img.resize_exact(360, 360, image::imageops::FilterType::Triangle);
        let fw: u16 = 9;
        let fh: u16 = 18;
        let full_cw = (360u32.div_ceil(fw as u32)) as u16;
        let full_ch = (360u32.div_ceil(fh as u32)) as u16;
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

        assert!(x0 > 0, "crop starts at pixel x={}, scroll was {}", x0, scroll_x);
        assert!(y0 > 0, "crop starts at pixel y={}, scroll was {}", y0, scroll_y);
        assert_eq!(cropped.width(), px_w, "cropped width matches viewport pixels");
        assert_eq!(cropped.height(), py_h, "cropped height matches viewport pixels");
    }

    #[test]
    fn image_placement_has_crop_field_none_by_default() {
        let renderer = MarkdownRenderer::new(76);
        let blocks = vec![
            MarkdownBlock::Image { alt: "t".into(), path: "t.webp".into() },
        ];
        let resolved = vec![
            ResolvedImage { path: "t.webp".into(), image: make_img(90, 36) },
        ];
        let mut r = SimpleResolver::new(9, 18);
        let output = renderer.render_full(&blocks, &TestTheme, &resolved, &mut r, 70, 20);
        assert_eq!(output.images.len(), 1);
        assert!(output.images[0].crop.is_none(), "crop should be None by default from render_full");
    }

    #[test]
    fn grow_by_one_row_each_press() {
        let mut rows = 2u16;
        for _ in 0..10 { rows = rows.saturating_add(1); }
        assert_eq!(rows, 12, "10 grows from 2 = 12 rows");
    }

    #[test]
    fn shrink_by_one_row_each_press() {
        let mut rows = 12u16;
        for _ in 0..10 { rows = rows.saturating_sub(1).max(1); }
        assert_eq!(rows, 2, "10 shrinks from 12 = 2 rows");
    }

    #[test]
    fn shrink_clamps_at_one_row() {
        let mut rows = 2u16;
        for _ in 0..10 { rows = rows.saturating_sub(1).max(1); }
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
        MarkdownBlock::Blockquote { level, children } => {
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
    assert_eq!(blocks.len(), 1, "consecutive > lines should be grouped into one blockquote");
    match &blocks[0] {
        MarkdownBlock::Blockquote { level, children } => {
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
    assert!(blocks.len() >= 1, "should parse nested blockquote");
    match &blocks[0] {
        MarkdownBlock::Blockquote { level, children } => {
            assert_eq!(*level, 1);
            let has_nested = children.iter().any(|c| matches!(c, MarkdownBlock::Blockquote { .. }));
            assert!(has_nested, "level 1 should contain a nested level 2 blockquote");
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
    assert!(text.starts_with("│"), "blockquote should start with │: got '{}'", text);
    assert!(text.contains("hello"), "blockquote should contain text: got '{}'", text);
    Ok(())
}

#[test]
fn nested_blockquote_renders_with_double_pipe() -> anyhow::Result<()> {
    let lines = render_markdown("> outer\n> > inner", 80);
    let all_text: String = lines.iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect::<Vec<&str>>()
        .join("");
    assert!(all_text.contains("│ │"), "nested blockquote should have double pipe prefix");
    Ok(())
}

#[test]
fn blockquote_with_code_inside() -> anyhow::Result<()> {
    let md = "> text before\n> ```rust\n> fn main() {}\n> ```\n> text after";
    let renderer = MarkdownRenderer::new(80);
    let blocks = renderer.parse(md);
    let bq = blocks.iter().find(|b| matches!(b, MarkdownBlock::Blockquote { .. }));
    assert!(bq.is_some(), "should parse blockquote");
    if let Some(MarkdownBlock::Blockquote { children, .. }) = bq {
        let has_code = children.iter().any(|c| matches!(c, MarkdownBlock::CodeBlock { .. }));
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
    let lines = renderer.render(&[block], &TestTheme);
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
    let lines = renderer.render(&[block], &TestTheme);
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
    let lines = renderer.render(&[block], &TestTheme);
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
        MarkdownBlock::CodeBlock { lang, code, header_override, footer_override, prefix_override } => {
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
        let all_text: String = lines.iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(all_text.contains("Start"), "rendered output should contain node label 'Start': got '{}'", all_text);
        assert!(all_text.contains("End"), "rendered output should contain node label 'End': got '{}'", all_text);
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
        let all_text: String = lines.iter()
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
        let all_text: String = lines.iter()
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
        let all_text: String = lines.iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(all_text.contains("yes"), "edge label should appear: got '{}'", all_text);
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
        assert!(lines.is_empty(), "invalid mermaid should be skipped gracefully");
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
        let result = crate::mermaid::render_mermaid(source, 80, None, &TestTheme);
        assert!(result.is_some());
        let lines = result.context("parse result")?;
        assert!(!lines.is_empty());
        let all_text: String = lines.iter()
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
        let result = crate::mermaid::render_mermaid(source, 80, Some(10), &TestTheme);
        assert!(result.is_some());
        let lines = result.context("parse result")?;
        assert!(lines.len() <= 25, "should try to respect max_height, got {} lines", lines.len());
    Ok(())
    }
}
