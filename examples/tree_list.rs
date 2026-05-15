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

struct TreeListHooks;

impl RenderHooks for TreeListHooks {
    fn list_item_marker(
        &self,
        indent: u8,
        is_last_in_group: bool,
        ancestors_are_last: &[bool],
        _index_in_group: usize,
    ) -> Option<String> {
        let marker = if is_last_in_group { "\u{2514}\u{2500} " } else { "\u{251c}\u{2500} " };
        if indent == 0 {
            return Some(marker.to_string());
        }
        let mut prefix = String::new();
        for (depth, &is_last_ancestor) in ancestors_are_last.iter().enumerate() {
            if depth >= indent as usize - 1 {
                break;
            }
            if is_last_ancestor {
                push_repeat(&mut prefix, ' ', 3);
            } else {
                prefix.push_str("\u{2502}  ");
            }
        }
        if (indent as usize - 1) > ancestors_are_last.len() {
            let extra = (indent as usize - 1).saturating_sub(ancestors_are_last.len());
            push_repeat(&mut prefix, ' ', 3 * extra);
        }
        Some(format!("{}{}", prefix, marker))
    }

    fn tree_indent_width(&self) -> Option<usize> {
        Some(3)
    }

    fn tree_text_gap(&self) -> Option<usize> {
        Some(0)
    }
}

fn push_repeat(s: &mut String, ch: char, n: usize) {
    for _ in 0..n {
        s.push(ch);
    }
}

const MARKDOWN: &str = r#"
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
    - Image syntax
  - Renderer
    - Inline formatting
    - Code block borders
  - Hooks system
- Write tests
- Deploy to crates.io

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
        .with_render_hooks(Box::new(TreeListHooks));
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
                .block(Block::default().borders(Borders::ALL).title(" Tree-Style List Example "))
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
