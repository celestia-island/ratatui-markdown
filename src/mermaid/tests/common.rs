use ratatui::{
    backend::TestBackend, buffer::Buffer, layout::Rect, style::Color, text::Line,
    widgets::Paragraph, Terminal,
};

use crate::theme::ThemeConfig;

pub fn test_theme() -> ThemeConfig {
    ThemeConfig::default()
        .with_info_color(Color::Blue)
        .with_focused_border_color(Color::Cyan)
        .with_secondary_color(Color::Yellow)
        .with_json_key_color(Color::Cyan)
        .with_json_bool_color(Color::Yellow)
        .with_json_number_color(Color::Magenta)
}

pub fn render_lines(source: &str, width: usize) -> Vec<Line<'static>> {
    crate::mermaid::render_mermaid(source, width, None, &test_theme()).unwrap_or_default()
}

pub fn render_to_buffer(source: &str, width: u16, height: u16) -> Buffer {
    let lines = render_lines(source, width as usize);
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("create terminal");
    terminal
        .draw(|f| {
            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, Rect::new(0, 0, width, height));
        })
        .expect("draw");
    terminal.backend().buffer().clone()
}

pub fn buffer_to_string(buf: &Buffer) -> String {
    let width = buf.area.width as usize;
    let height = buf.area.height as usize;
    let mut rows = Vec::with_capacity(height);
    for y in 0..height {
        let row: String = (0..width)
            .map(|x| {
                buf.cell((x as u16, y as u16))
                    .map(|c| c.symbol())
                    .unwrap_or(" ")
            })
            .collect();
        rows.push(row.trim_end().to_string());
    }
    while rows.last().is_some_and(|r| r.is_empty()) {
        rows.pop();
    }
    rows.join("\n")
}

pub fn assert_buffer_eq(buf: &Buffer, expected: &str) {
    let actual = buffer_to_string(buf);
    let expected = expected.strip_prefix('\n').unwrap_or(expected);
    let expected = expected.strip_suffix('\n').unwrap_or(expected);
    if actual != expected {
        let actual_lines: Vec<&str> = actual.split('\n').collect();
        let expected_lines: Vec<&str> = expected.split('\n').collect();
        let max = actual_lines.len().max(expected_lines.len());
        let mut diffs = Vec::new();
        for i in 0..max {
            let a = actual_lines.get(i).copied().unwrap_or("");
            let e = expected_lines.get(i).copied().unwrap_or("");
            if a != e {
                diffs.push(format!(
                    "  line {i}: expected |{e}|\n           actual   |{a}|"
                ));
            }
        }
        panic!(
            "buffer mismatch ({} diff(s)):\n{}\n\n--- expected ---\n{expected}\n\n--- actual ---\n{actual}",
            diffs.len(),
            diffs.join("\n")
        );
    }
}
