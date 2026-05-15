use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind,
    },
    execute, queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, size,
    },
};
use ratatui::text::Line;
use ratatui_markdown::theme::{Generation, RichTextTheme};

pub struct Theme;

impl RichTextTheme for Theme {
    fn generation(&self) -> Generation {
        Generation(1)
    }
    fn get_text_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::White
    }
    fn get_muted_text_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::DarkGray
    }
    fn get_primary_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::Cyan
    }
    fn get_secondary_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::Blue
    }
    fn get_info_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::LightBlue
    }
    fn get_background_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::Black
    }
    fn get_border_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::DarkGray
    }
    fn get_focused_border_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::White
    }
    fn get_popup_selected_background(&self) -> ratatui::style::Color {
        ratatui::style::Color::DarkGray
    }
    fn get_popup_selected_text_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::White
    }
    fn get_json_key_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::LightCyan
    }
    fn get_json_string_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::Green
    }
    fn get_json_number_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::Yellow
    }
    fn get_json_bool_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::Magenta
    }
    fn get_json_null_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::DarkGray
    }
    fn get_accent_yellow(&self) -> ratatui::style::Color {
        ratatui::style::Color::Yellow
    }
}

fn convert_color(c: ratatui::style::Color) -> Color {
    use ratatui::style::Color as RC;
    match c {
        RC::Black => Color::Black,
        RC::Red => Color::DarkRed,
        RC::Green => Color::DarkGreen,
        RC::Yellow => Color::DarkYellow,
        RC::Blue => Color::DarkBlue,
        RC::Magenta => Color::DarkMagenta,
        RC::Cyan => Color::DarkCyan,
        RC::Gray => Color::DarkGrey,
        RC::DarkGray => Color::Grey,
        RC::LightRed => Color::Red,
        RC::LightGreen => Color::Green,
        RC::LightYellow => Color::Yellow,
        RC::LightBlue => Color::Blue,
        RC::LightMagenta => Color::Magenta,
        RC::LightCyan => Color::Cyan,
        RC::White => Color::White,
        RC::Rgb(r, g, b) => Color::Rgb { r, g, b },
        RC::Reset | _ => Color::Reset,
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

pub fn setup_terminal() -> anyhow::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

pub fn restore_terminal() -> anyhow::Result<()> {
    execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn draw_frame(
    title: &str,
    lines: &[Line<'static>],
    state: &mut AppState,
    key_hints: &str,
) -> io::Result<()> {
    let mut stdout = io::stdout();
    let (w, h) = size()?;
    if w < 6 || h < 4 {
        return Ok(());
    }

    let bw = w;
    let bh = h.saturating_sub(1);
    let content_h = bh.saturating_sub(2);
    state.update_content_h(content_h);

    queue!(stdout, MoveTo(0, 0))?;

    let border_color = Color::DarkGrey;
    let border_fg = SetForegroundColor(border_color);
    let reset = ResetColor;

    queue!(stdout, border_fg, Print("╭─"))?;
    queue!(stdout, SetForegroundColor(Color::White), Print(format!(" {title} ")))?;
    let used = 4 + title.len() as u16 + 2;
    if used < bw.saturating_sub(1) {
        let dash_count = bw.saturating_sub(used) - 1;
        queue!(stdout, border_fg, Print("─".repeat(dash_count as usize)))?;
    }
    queue!(stdout, border_fg, Print("╮"), reset)?;

    let scroll = state.scroll as usize;
    let visible = content_h as usize;
    let inner_w = bw.saturating_sub(4) as usize;

    for y in 0..visible {
        let row = y as u16 + 1;
        queue!(stdout, MoveTo(0, row), border_fg, Print("│ "))?;

        let line_idx = scroll + y;
        if line_idx < lines.len() {
            let mut written = 0usize;
            for span in &lines[line_idx].spans {
                if written >= inner_w {
                    break;
                }
                if let Some(fg) = span.style.fg {
                    queue!(stdout, SetForegroundColor(convert_color(fg)))?;
                }
                if let Some(bg) = span.style.bg {
                    queue!(stdout, SetBackgroundColor(convert_color(bg)))?;
                }
                let add = span.style.add_modifier;
                if add.contains(ratatui::style::Modifier::BOLD) {
                    queue!(stdout, SetAttribute(Attribute::Bold))?;
                }
                if add.contains(ratatui::style::Modifier::ITALIC) {
                    queue!(stdout, SetAttribute(Attribute::Italic))?;
                }
                if add.contains(ratatui::style::Modifier::UNDERLINED) {
                    queue!(stdout, SetAttribute(Attribute::Underlined))?;
                }

                let remaining = inner_w.saturating_sub(written);
                let truncated: String = span.content.chars().take(remaining).collect();
                let cw = unicode_width::UnicodeWidthStr::width(truncated.as_str());
                written += cw;
                queue!(stdout, Print(&truncated), ResetColor)?;
            }
        }

        queue!(stdout, Clear(ClearType::UntilNewLine))?;
        queue!(stdout, MoveTo(bw.saturating_sub(1), row), border_fg, Print("│"), reset)?;
    }

    queue!(
        stdout,
        MoveTo(0, bh.saturating_sub(1)),
        border_fg,
        Print("╰"),
        Print("─".repeat(bw.saturating_sub(2) as usize)),
        Print("╯"),
        reset
    )?;

    if state.doc_h > content_h && content_h > 0 {
        let col = bw.saturating_sub(1);
        let ratio = state.scroll as f64 / (state.doc_h - content_h).max(1) as f64;
        let thumb_y = 1 + (ratio * (content_h as f64 - 1.0)).round() as u16;
        for y in 1..=content_h {
            queue!(stdout, MoveTo(col - 1, y))?;
            if y == thumb_y {
                queue!(stdout, SetForegroundColor(Color::Cyan), Print("█"), ResetColor)?;
            } else {
                queue!(stdout, SetForegroundColor(Color::DarkGrey), Print("│"), ResetColor)?;
            }
        }
    }

    queue!(
        stdout,
        MoveTo(0, h.saturating_sub(1)),
        SetForegroundColor(Color::DarkGrey),
        Print(format!(" {key_hints}")),
        Clear(ClearType::UntilNewLine),
        ResetColor
    )?;

    stdout.flush()?;
    Ok(())
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
