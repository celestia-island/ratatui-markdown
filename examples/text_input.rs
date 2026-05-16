#[path = "utils/mod.rs"]
mod common;

use std::{cell::Cell, rc::Rc};

use common::{restore_terminal, setup_terminal, Theme};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame, Terminal,
};
use ratatui_markdown::text_input::{
    CursorBlinkController, CursorPosition, CursorShape, CursorStyle, InputMode, TextInput,
};

const INITIAL_TEXT: &str = "\
# Hello World

This is a **markdown** document with *italic*, `inline code`, and [links](https://example.com).

## Features

- **Bold** and *italic* text
- ~~Strikethrough~~ support
- `Code spans` highlighted

> A blockquote with *formatting*";

const CURSOR_SHAPES: [CursorShape; 4] = [
    CursorShape::Block,
    CursorShape::Bar,
    CursorShape::Underline,
    CursorShape::HollowBlock,
];

struct SimpleBlink {
    visible: Cell<bool>,
}

impl CursorBlinkController for SimpleBlink {
    fn is_visible(&self) -> bool {
        self.visible.get()
    }
}

struct App {
    input: TextInput,
    blink: Rc<SimpleBlink>,
    blink_tick: u8,
    cursor_shape_idx: usize,
}

impl App {
    fn new() -> Self {
        let blink = Rc::new(SimpleBlink {
            visible: Cell::new(true),
        });
        let mut input = TextInput::new()
            .with_mode(InputMode::Edit)
            .with_cursor_style(
                CursorStyle::new()
                    .with_shape(CursorShape::Block)
                    .with_position(CursorPosition::OnChar),
            )
            .with_blink_controller(blink.clone())
            .with_placeholder("Type markdown here...");
        input.set_text(INITIAL_TEXT);

        Self {
            input,
            blink,
            blink_tick: 0,
            cursor_shape_idx: 0,
        }
    }

    fn cycle_cursor_shape(&mut self) {
        self.cursor_shape_idx = (self.cursor_shape_idx + 1) % CURSOR_SHAPES.len();
        self.input = {
            let mut new_input = TextInput::new()
                .with_mode(self.input.mode())
                .with_cursor_style(
                    CursorStyle::new()
                        .with_shape(CURSOR_SHAPES[self.cursor_shape_idx])
                        .with_position(CursorPosition::OnChar),
                )
                .with_blink_controller(self.blink.clone())
                .with_placeholder("Type markdown here...");
            let text = self.input.text().to_string();
            let cursor = self.input.cursor_char_idx();
            new_input.set_text(text);
            new_input.set_cursor_char_idx(cursor);
            new_input
        };
    }

    fn shape_name(&self) -> &'static str {
        match CURSOR_SHAPES[self.cursor_shape_idx] {
            CursorShape::Block => "Block",
            CursorShape::Bar => "Bar",
            CursorShape::Underline => "ULine",
            CursorShape::HollowBlock => "Hollow",
        }
    }
}

fn run(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> anyhow::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? && handle_event(&mut app)? {
            return Ok(());
        }

        app.blink_tick = (app.blink_tick + 1) % 8;
        app.blink.visible.set(app.blink_tick < 5);
    }
}

fn handle_event(app: &mut App) -> anyhow::Result<bool> {
    let Event::Key(key) = event::read()? else {
        return Ok(false);
    };
    if key.kind != KeyEventKind::Press {
        return Ok(false);
    }

    match key.code {
        KeyCode::Esc => return Ok(true),

        KeyCode::F(2) => match app.input.mode() {
            InputMode::Edit => app.input.set_mode(InputMode::Read),
            InputMode::Read => app.input.set_mode(InputMode::Edit),
        },

        KeyCode::F(3) => {
            app.cycle_cursor_shape();
        }

        KeyCode::Left => app.input.move_cursor_left(),
        KeyCode::Right => app.input.move_cursor_right(),
        KeyCode::Up => match app.input.mode() {
            InputMode::Edit => app.input.move_cursor_up(),
            InputMode::Read => {
                let off = app.input.scroll_offset().saturating_sub(1);
                app.input.set_scroll_offset(off);
            }
        },
        KeyCode::Down => match app.input.mode() {
            InputMode::Edit => app.input.move_cursor_down(),
            InputMode::Read => {
                let off = app.input.scroll_offset() + 1;
                app.input.set_scroll_offset(off);
            }
        },
        KeyCode::Home => app.input.move_cursor_to_start(),
        KeyCode::End => app.input.move_cursor_to_end(),

        KeyCode::Backspace => app.input.delete_char_backward(),
        KeyCode::Delete => app.input.delete_char_forward(),

        KeyCode::Enter if app.input.mode() == InputMode::Edit => app.input.insert_char('\n'),

        KeyCode::Char(ch) if app.input.mode() == InputMode::Edit => app.input.insert_char(ch),

        _ => {}
    }

    Ok(false)
}

fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).split(area);

    let mode_label = match app.input.mode() {
        InputMode::Edit => "EDIT",
        InputMode::Read => "READ",
    };
    let shape = app.shape_name();
    let title = format!(" TextInput [{mode_label}] cursor:{shape} ");

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(chunks[0]);
    f.render_widget(block, chunks[0]);

    app.input.render(f, inner, &Theme);

    let (mode_hint, quit_hint) = match app.input.mode() {
        InputMode::Edit => (
            "type to insert \u{00b7} Enter:newline \u{00b7} \u{2190}\u{2191}\u{2192}\u{2193}/Home/End \u{00b7} Bksp/Del",
            "Esc",
        ),
        InputMode::Read => (
            "\u{2191}\u{2193} scroll",
            "Esc",
        ),
    };
    let status = format!(
        " {mode_hint} \u{00b7} F2:toggle mode \u{00b7} F3:cycle cursor \u{00b7} {quit_hint}:quit "
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            status,
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[1],
    );
}

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}
