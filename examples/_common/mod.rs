use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind, MouseEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame, Terminal,
};
use ratatui_markdown::theme::{Generation, RichTextTheme};

pub struct Theme;

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

pub struct AppState {
    pub scroll: u16,
    pub doc_h: u16,
    content_h: u16,
}

impl AppState {
    pub fn new(total_lines: usize) -> Self {
        Self {
            scroll: 0,
            doc_h: total_lines as u16,
            content_h: 0,
        }
    }

    pub fn update_content_h(&mut self, h: u16) {
        self.content_h = h;
        self.clamp();
    }

    pub fn clamp(&mut self) {
        let max = self.doc_h.saturating_sub(self.content_h);
        if self.scroll > max {
            self.scroll = max;
        }
    }
}

pub type Term = Terminal<CrosstermBackend<std::io::Stdout>>;

pub fn setup_terminal() -> anyhow::Result<Term> {
    enable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(std::io::stdout());
    Ok(Terminal::new(backend)?)
}

pub fn restore_terminal(terminal: &mut Term) -> anyhow::Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    Ok(())
}

pub fn draw_frame(
    f: &mut Frame,
    title: &str,
    lines: &[Line<'static>],
    state: &mut AppState,
    key_hints: &str,
) {
    let area = f.area();
    let block_area = Rect::new(
        area.x,
        area.y,
        area.width,
        area.height.saturating_sub(1),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .padding(Padding::new(1, 1, 0, 0));

    let inner = block.inner(block_area);
    let content_h = inner.height;
    state.update_content_h(content_h);

    f.render_widget(block, block_area);

    // Every frame: fill inner area with real spaces (skip=false) so the
    // terminal diff engine can never leave stale characters behind.
    let blank = Line::from(Span::raw(" ".repeat(inner.width as usize)));
    let fill: Vec<Line<'static>> = (0..content_h).map(|_| blank.clone()).collect();
    f.render_widget(Paragraph::new(fill), inner);

    let paragraph = Paragraph::new(lines.to_vec())
        .scroll((state.scroll, 0));
    f.render_widget(paragraph, inner);

    if state.doc_h > content_h && content_h > 0 {
        let sb_col = block_area.x + block_area.width.saturating_sub(1);
        let sb_area = Rect::new(sb_col, inner.y, 1, content_h);
        let ratatui_content_len = state
            .doc_h
            .saturating_sub(content_h)
            .saturating_add(1);
        let sb = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .thumb_symbol("█")
            .track_symbol(Some("│"))
            .style(Style::default().fg(Color::DarkGray))
            .thumb_style(Style::default().fg(Color::Cyan));
        let mut sb_state = ScrollbarState::default()
            .content_length(ratatui_content_len as usize)
            .viewport_content_length(content_h as usize)
            .position(state.scroll as usize);
        f.render_stateful_widget(sb, sb_area, &mut sb_state);
    }

    let info_area = Rect::new(area.x, area.height.saturating_sub(1), area.width, 1);
    f.render_widget(
        Paragraph::new(vec![Line::from(Span::styled(
            format!(" {}", key_hints),
            Style::default().fg(Color::DarkGray),
        ))]),
        info_area,
    );
}

pub fn poll_and_handle(state: &mut AppState) -> anyhow::Result<bool> {
    if !event::poll(std::time::Duration::from_millis(50))? {
        return Ok(false);
    }
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Up | KeyCode::Char('k') => {
                state.scroll = state.scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.scroll = state.scroll.saturating_add(1);
                state.clamp();
            }
            KeyCode::PageUp => {
                let step = state.content_h.max(1);
                state.scroll = state.scroll.saturating_sub(step);
            }
            KeyCode::PageDown => {
                let step = state.content_h.max(1);
                state.scroll = state.scroll.saturating_add(step);
                state.clamp();
            }
            KeyCode::Home => state.scroll = 0,
            KeyCode::End => {
                state.scroll = state.doc_h.saturating_sub(state.content_h);
            }
            _ => {}
        },
        Event::Mouse(mouse) => match mouse.kind {
            MouseEventKind::ScrollUp => {
                state.scroll = state.scroll.saturating_sub(3);
            }
            MouseEventKind::ScrollDown => {
                state.scroll = state.scroll.saturating_add(3);
                state.clamp();
            }
            _ => {}
        },
        _ => {}
    }
    Ok(false)
}

pub fn lorem(words: usize) -> String {
    lipsum::lipsum(words)
}
