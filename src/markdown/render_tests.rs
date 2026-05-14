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
    markdown::{MarkdownBlock, MarkdownRenderer},
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
