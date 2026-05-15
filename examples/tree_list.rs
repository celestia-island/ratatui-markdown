use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind, MouseEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Terminal,
};
use ratatui_markdown::{
    constants::{
        BRANCH_END_SP, BRANCH_MID_SP, BRANCH_VERT_PAD, VLINE,
    },
    markdown::{MarkdownRenderer, RenderHooks},
    theme::{Generation, RichTextTheme},
};

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
        let base: usize = 3;
        let offset: usize = Self::tree_indent_offset(self).unwrap_or(0);
        let unit = base + offset;
        let connector = if is_last_in_group {
            BRANCH_END_SP
        } else {
            BRANCH_MID_SP
        };
        if indent == 0 {
            return Some(connector.to_string());
        }
        let mut prefix = String::new();
        for (i, &is_last_anc) in ancestors_are_last.iter().enumerate() {
            if i >= indent as usize {
                break;
            }
            if is_last_anc {
                for _ in 0..unit {
                    prefix.push(' ');
                }
            } else {
                prefix.push_str(BRANCH_VERT_PAD);
                for _ in 0..offset {
                    prefix.push(' ');
                }
            }
        }
        if indent as usize > ancestors_are_last.len() {
            let extra = indent as usize - ancestors_are_last.len();
            for _ in 0..unit * extra {
                prefix.push(' ');
            }
        }
        Some(format!("{prefix}{connector}"))
    }

    fn tree_indent_width(&self) -> Option<usize> {
        Some(3 + Self::tree_indent_offset(self).unwrap_or(0))
    }

    fn tree_text_gap(&self) -> Option<usize> {
        Some(1)
    }

    fn tree_indent_offset(&self) -> Option<usize> {
        Some(1)
    }
}

const MARKDOWN: &str = r#"
## Project TODO

- Setup project structure
  - Initialize Cargo workspace with members for core and crates
  - Add dependencies
    - ratatui for terminal UI rendering and display management
    - image crate for image support and protocol handling
    - crossterm for crossplatform terminal event handling
- Implement core features
  - Parser
    - Heading detection with nested component extraction logic
    - Code block parsing with language-aware fence matching rules
    - Image syntax for embedded visual content rendering pipeline
  - Renderer
    - Inline formatting with bold italic and code spans
    - Code block borders using rounded box drawing characters
    - Text wrapping engine that respects word boundaries and CJK width
  - Hooks system
    - RenderHooks trait for customizable list item markers and styling
    - Theme hooks for dynamic color palette switching at runtime
- Write tests
  - Unit tests for parser edge cases and corner conditions
  - Integration tests for full markdown document rendering pipeline
  - Visual regression tests for tree layout and wrap behavior verification
- Deploy to crates.io
  - Write comprehensive documentation with usage examples
  - Publish stable release with semver version bump
  - Create GitHub Actions CI pipeline for automated testing across platforms

Press `q` to quit.
"#;

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76)
        .with_render_hooks(Box::new(TreeListHooks));
    let blocks = renderer.parse(MARKDOWN);
    let lines = renderer.render(&blocks, &theme);

    let mut scroll: u16 = 0;

    loop {
        let doc_h = lines.len() as u16;
        terminal.draw(|f| {
            let area = f.area();
            let inner = Rect::new(
                area.x + 1,
                area.y + 1,
                area.width.saturating_sub(2),
                area.height.saturating_sub(2),
            );
            let text_top = inner.y + 1;
            let text_bot = inner.y + inner.height.saturating_sub(2);
            let sb_col = inner.x + inner.width.saturating_sub(1);
            let content_h = text_bot.saturating_sub(text_top).saturating_add(1);

            let max_scroll = doc_h.saturating_sub(content_h);
            if scroll > max_scroll {
                scroll = max_scroll;
            }

            let paragraph = Paragraph::new(lines.clone())
                .block(Block::default().borders(Borders::ALL).title(" Tree-Style List Example "))
                .scroll((scroll, 0));
            f.render_widget(paragraph, inner);

            if doc_h > content_h && content_h > 0 {
                let sb_area = Rect::new(sb_col, text_top, 1, content_h);
                let sb = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .thumb_symbol("█")
                    .track_symbol(Some(VLINE))
                    .style(Style::default().fg(Color::DarkGray))
                    .thumb_style(Style::default().fg(Color::Cyan));
                let ratatui_content_len = doc_h
                    .saturating_sub(content_h)
                    .saturating_add(1);
                let mut sb_state = ScrollbarState::default()
                    .content_length(ratatui_content_len as usize)
                    .viewport_content_length(content_h as usize)
                    .position(scroll as usize);
                f.render_stateful_widget(sb, sb_area, &mut sb_state);
            }
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up | KeyCode::Char('k') => {
                        scroll = scroll.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        scroll = scroll.saturating_add(1);
                    }
                    _ => {}
                },
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        scroll = scroll.saturating_sub(3);
                    }
                    MouseEventKind::ScrollDown => {
                        scroll = scroll.saturating_add(3);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
