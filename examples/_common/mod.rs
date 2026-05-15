use ratatui::{
    backend::CrosstermBackend,
    buffer::Cell,
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
use unicode_width::UnicodeWidthChar;

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

fn render_lines_to_buf(
    lines: &[Line<'static>],
    scroll: usize,
    area: Rect,
    buf: &mut ratatui::buffer::Buffer,
) {
    let max_x = area.x + area.width;
    for y in 0..area.height {
        let row = area.y + y;
        let line_idx = scroll + y as usize;
        let mut col = area.x;
        if line_idx < lines.len() {
            for span in &lines[line_idx].spans {
                for ch in span.content.chars() {
                    if col >= max_x {
                        break;
                    }
                    let w = UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
                    if w == 0 {
                        continue;
                    }
                    if col + w > max_x {
                        break;
                    }
                    let mut cell = Cell::new(" ");
                    cell.set_symbol(&ch.to_string());
                    cell.set_style(span.style);
                    buf[(col, row)] = cell;
                    col += w;
                }
            }
        }
        while col < max_x {
            buf[(col, row)] = Cell::new(" ");
            col += 1;
        }
    }
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

    // Bypass Paragraph entirely — write every cell in the inner area directly
    // to the buffer with skip=false.  This is the only way to guarantee no
    // stale characters survive ratatui's diff engine, because code-block lines
    // can exceed the inner width and Paragraph silently clips (leaving skip=true
    // holes that retain previous-frame content).
    render_lines_to_buf(lines, state.scroll as usize, inner, f.buffer_mut());

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
