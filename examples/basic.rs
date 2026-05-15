use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Terminal,
};
use ratatui_markdown::{
    markdown::MarkdownRenderer,
    theme::{Generation, RichTextTheme},
};

struct Theme;

impl RichTextTheme for Theme {
    fn generation(&self) -> Generation {
        Generation(1)
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
    fn get_secondary_color(&self) -> Color {
        Color::Blue
    }
    fn get_info_color(&self) -> Color {
        Color::LightBlue
    }
    fn get_background_color(&self) -> Color {
        Color::Black
    }
    fn get_border_color(&self) -> Color {
        Color::DarkGray
    }
    fn get_focused_border_color(&self) -> Color {
        Color::White
    }
    fn get_popup_selected_background(&self) -> Color {
        Color::DarkGray
    }
    fn get_popup_selected_text_color(&self) -> Color {
        Color::White
    }
    fn get_json_key_color(&self) -> Color {
        Color::LightCyan
    }
    fn get_json_string_color(&self) -> Color {
        Color::Green
    }
    fn get_json_number_color(&self) -> Color {
        Color::Yellow
    }
    fn get_json_bool_color(&self) -> Color {
        Color::Magenta
    }
    fn get_json_null_color(&self) -> Color {
        Color::DarkGray
    }
    fn get_accent_yellow(&self) -> Color {
        Color::Yellow
    }
}

const MARKDOWN: &str = r#"
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

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76);
    let blocks = renderer.parse(MARKDOWN);
    let lines = renderer.render(&blocks, &theme);

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let inner = Rect::new(
                area.x + 1, area.y + 1,
                area.width.saturating_sub(2), area.height.saturating_sub(2),
            );
            let paragraph = Paragraph::new(lines.clone())
                .block(Block::default().borders(Borders::ALL).title(" Basic Markdown Example "))
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, inner);

            let content_h = inner.height.saturating_sub(2);
            let total = lines.len();
            if total > content_h as usize && content_h > 0 {
                let sb_area = Rect::new(
                    inner.x + inner.width.saturating_sub(1),
                    inner.y + 1,
                    1,
                    content_h,
                );
                let sb = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .thumb_symbol("█")
                    .track_symbol(Some("│"))
                    .style(Style::default().fg(Color::DarkGray))
                    .thumb_style(Style::default().fg(Color::Cyan));
                let mut sb_state = ScrollbarState::default()
                    .content_length(total)
                    .viewport_content_length(content_h as usize);
                f.render_stateful_widget(sb, sb_area, &mut sb_state);
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
