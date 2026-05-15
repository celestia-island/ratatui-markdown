use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Terminal,
};
use ratatui_markdown::{
    markdown::MarkdownRenderer,
    theme::{Generation, RichTextTheme},
};
use ratatui_markdown::markdown::RenderHooks;

struct Theme;

impl RichTextTheme for Theme {
    fn generation(&self) -> Generation { Generation(1) }
    fn get_text_color(&self) -> Color { Color::White }
    fn get_muted_text_color(&self) -> Color { Color::DarkGray }
    fn get_primary_color(&self) -> Color { Color::Cyan }
    fn get_secondary_color(&self) -> Color { Color::Blue }
    fn get_info_color(&self) -> Color { Color::LightBlue }
    fn get_background_color(&self) -> Color { Color::Black }
    fn get_border_color(&self) -> Color { Color::DarkGray }
    fn get_focused_border_color(&self) -> Color { Color::White }
    fn get_popup_selected_background(&self) -> Color { Color::DarkGray }
    fn get_popup_selected_text_color(&self) -> Color { Color::White }
    fn get_json_key_color(&self) -> Color { Color::LightCyan }
    fn get_json_string_color(&self) -> Color { Color::Green }
    fn get_json_number_color(&self) -> Color { Color::Yellow }
    fn get_json_bool_color(&self) -> Color { Color::Magenta }
    fn get_json_null_color(&self) -> Color { Color::DarkGray }
    fn get_accent_yellow(&self) -> Color { Color::Yellow }
}

struct TimelineCodeHooks;

impl RenderHooks for TimelineCodeHooks {
    fn code_block_header(&self, lang: &str) -> Option<Line<'static>> {
        let timestamp = chrono_placeholder();
        Some(Line::from(vec![
            Span::styled(
                format!("\u{256d} [{timestamp}] ", ),
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
            Span::styled(
                "\u{2570} ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                "\u{2191} ",
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                "156 ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                "\u{2193} ",
                Style::default().fg(Color::Red),
            ),
            Span::styled(
                "234 ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                "\u{21c4} ",
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                "1 23.5s",
                Style::default().fg(Color::DarkGray),
            ),
        ]))
    }

    fn code_block_line_prefix(&self, _lang: &str) -> Option<String> {
        Some("\u{2502} ".to_string())
    }
}

fn chrono_placeholder() -> String {
    "12:00:00".to_string()
}

const MARKDOWN: &str = r#"
# Timeline View

This example shows customized code block rendering with extra content
in the rounded border header and footer, inspired by a timeline view.

## Agent Skill Execution

```rust skill::read_file
use std::fs;
let content = fs::read_to_string("PLAN.md")?;
println!("{}", content);
```

The header shows a timestamp and tool name, while the footer
displays token usage statistics: ↑ output ↓ input ⇄ roundtrips duration.

## Another Block

```python skill::analyze
def analyze(data):
    for item in data:
        yield process(item)
```

Press `q` to quit.
"#;

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76)
        .with_render_hooks(Box::new(TimelineCodeHooks));
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
                .block(Block::default().borders(Borders::ALL).title(" Custom Code Block Example "))
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
