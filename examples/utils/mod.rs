use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind, MouseEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame, Terminal,
};
use ratatui_markdown::theme::{CodeColors, ThemeConfig};

pub struct Theme;

// Keep Theme struct backward compatible by delegating to ThemeConfig
impl std::ops::Deref for Theme {
    type Target = ThemeConfig;
    fn deref(&self) -> &ThemeConfig {
        &THEME
    }
}

static THEME: ThemeConfig = ThemeConfig {
    gen: ratatui_markdown::theme::Generation(1),
    text_color: Color::White,
    muted_text_color: Color::DarkGray,
    primary_color: Color::Cyan,
    secondary_color: Color::Blue,
    info_color: Color::LightBlue,
    border_color: Color::DarkGray,
    focused_border_color: Color::White,
    popup_selected_background: Color::DarkGray,
    json_key_color: Color::LightCyan,
    json_string_color: Color::Green,
    json_number_color: Color::Yellow,
    json_bool_color: Color::Magenta,
    json_null_color: Color::DarkGray,
    accent_yellow: Color::Yellow,
    code_colors: CodeColors::DEFAULT,
};

impl ratatui_markdown::theme::RichTextTheme for Theme {
    fn generation(&self) -> ratatui_markdown::theme::Generation {
        THEME.gen
    }
    fn get_text_color(&self) -> Color {
        THEME.text_color
    }
    fn get_muted_text_color(&self) -> Color {
        THEME.muted_text_color
    }
    fn get_primary_color(&self) -> Color {
        THEME.primary_color
    }
    fn get_secondary_color(&self) -> Color {
        THEME.secondary_color
    }
    fn get_info_color(&self) -> Color {
        THEME.info_color
    }
    fn get_border_color(&self) -> Color {
        THEME.border_color
    }
    fn get_focused_border_color(&self) -> Color {
        THEME.focused_border_color
    }
    fn get_popup_selected_background(&self) -> Color {
        THEME.popup_selected_background
    }
    fn get_json_key_color(&self) -> Color {
        THEME.json_key_color
    }
    fn get_json_string_color(&self) -> Color {
        THEME.json_string_color
    }
    fn get_json_number_color(&self) -> Color {
        THEME.json_number_color
    }
    fn get_json_bool_color(&self) -> Color {
        THEME.json_bool_color
    }
    fn get_json_null_color(&self) -> Color {
        THEME.json_null_color
    }
    fn get_accent_yellow(&self) -> Color {
        THEME.accent_yellow
    }
}

#[allow(dead_code)]
pub struct AppState {
    pub scroll: u16,
    pub doc_h: u16,
    content_h: u16,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub type Term = Terminal<CrosstermBackend<std::io::Stdout>>;

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn restore_terminal(terminal: &mut Term) -> anyhow::Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    Ok(())
}

#[allow(dead_code)]
pub fn draw_frame(
    f: &mut Frame,
    title: &str,
    lines: &[Line<'static>],
    state: &mut AppState,
    key_hints: &str,
) {
    let area = f.area();
    let block_area = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .padding(Padding::new(1, 1, 0, 0));

    let inner = block.inner(block_area);
    let content_h = inner.height;
    let inner_w = inner.width as usize;
    state.update_content_h(content_h);

    // Build a fully-padded view of the visible content so every cell in the
    // inner area is written to by Paragraph (skip=false).  The root cause of
    // scroll artifacts is that Paragraph clips lines wider than the area and
    // leaves trailing cells unwritten; stuffing trailing spaces into each line
    // guarantees all cells are touched.
    let scroll = state.scroll as usize;
    let visible = content_h as usize;
    let blank = Line::from(Span::raw(" ".repeat(inner_w)));
    let mut padded: Vec<Line<'static>> = Vec::with_capacity(visible);

    for line in lines.iter().skip(scroll).take(visible) {
        let spans = line.spans.clone();
        let used: usize = spans.iter().map(|s| s.width()).sum();
        if used < inner_w {
            let mut s = spans;
            s.push(Span::raw(" ".repeat(inner_w - used)));
            padded.push(Line::from(s));
        } else if used > inner_w {
            // Truncate to inner width — the cut-off content will still be
            // completely covered by trailing spaces in the next iteration.
            let mut taken = 0usize;
            let mut short: Vec<Span<'static>> = Vec::new();
            for sp in spans {
                let sp_w = sp.width();
                if taken + sp_w > inner_w {
                    let keep = inner_w - taken;
                    let chop: String = sp.content.chars().take(keep).collect();
                    short.push(Span::styled(chop, sp.style));
                    break;
                }
                taken += sp_w;
                short.push(sp);
            }
            while taken < inner_w {
                short.push(Span::raw(" "));
                taken += 1;
            }
            padded.push(Line::from(short));
        } else {
            padded.push(Line::from(spans));
        }
    }
    while padded.len() < visible {
        padded.push(blank.clone());
    }

    f.render_widget(block, block_area);

    let paragraph = Paragraph::new(padded);
    f.render_widget(paragraph, inner);

    if state.doc_h > content_h && content_h > 0 {
        let sb_col = block_area.x + block_area.width.saturating_sub(1);
        let sb_area = Rect::new(sb_col, inner.y, 1, content_h);
        let ratatui_content_len = state.doc_h.saturating_sub(content_h).saturating_add(1);
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

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn lorem(words: usize) -> String {
    lipsum::lipsum(words)
}
