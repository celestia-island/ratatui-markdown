#[path = "utils/mod.rs"]
mod common;

use std::rc::Rc;

use common::{restore_terminal, setup_terminal, Theme};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame, Terminal,
};
use ratatui_markdown::{
    scroll::{CursorLineMode, SpanTree, SpanTreeEntry},
    text_input::{
        CursorBlinkController, CursorPosition, CursorShape, CursorStyle, InputMode, Selection,
        TextInput,
    },
};

const SAMPLE_TEXT: &str = r#"# Hello World

This is a **markdown** document with *italic*, `inline code`, and [links](https://example.com).

## Features

- Dual-mode rendering
- **Bold** and *italic* text
- ~~Strikethrough~~ support
- `Code spans` highlighted

### Code Block

```rust
fn main() {
    println!("Hello, ratatui-markdown!");
}
```

> A blockquote with *formatting*

| Col A | Col B |
|-------|-------|
| 1     | 2     |
"#;

struct SimpleBlink {
    visible: bool,
}

impl CursorBlinkController for SimpleBlink {
    fn is_visible(&self) -> bool {
        self.visible
    }
}

struct App {
    input: TextInput,
    blink: Rc<SimpleBlink>,
    blink_tick: u8,
    panel: Panel,
    cursor_shape_idx: usize,
    read_scroll: usize,
    tree: SpanTree,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Panel {
    Input,
    SpanTree,
}

const CURSOR_SHAPES: [CursorShape; 4] = [
    CursorShape::Block,
    CursorShape::Bar,
    CursorShape::Underline,
    CursorShape::HollowBlock,
];

impl App {
    fn new() -> Self {
        let blink = Rc::new(SimpleBlink { visible: true });
        let input = TextInput::new()
            .with_mode(InputMode::Edit)
            .with_blink_controller(blink.clone())
            .with_placeholder("Type markdown here...");

        let mut tree = SpanTree::new().with_cursor_line_mode(CursorLineMode::AllLines);
        let entries = vec![
            make_entry("agent-1", "Alpha", vec!["Task: Write docs", "Status: In progress", "Priority: High"]),
            make_entry("agent-2", "Beta", vec!["Task: Fix bugs", "Status: Done"]),
            make_entry("agent-3", "Gamma", vec!["Task: Add tests", "Status: Pending", "Priority: Medium", "ETA: 2 days"]),
            make_entry("agent-4", "Delta", vec!["Task: Review PR"]),
        ];
        tree.set_entries(entries);
        tree.set_selected_index(0);

        Self {
            input,
            blink,
            blink_tick: 0,
            panel: Panel::Input,
            cursor_shape_idx: 0,
            read_scroll: 0,
            tree,
        }
    }

    fn current_shape(&self) -> CursorShape {
        CURSOR_SHAPES[self.cursor_shape_idx]
    }

    fn shape_name(&self) -> &'static str {
        match self.current_shape() {
            CursorShape::Block => "Block",
            CursorShape::Bar => "Bar",
            CursorShape::Underline => "Underline",
            CursorShape::HollowBlock => "HollowBlock",
        }
    }
}

fn make_entry(id: &str, name: &str, details: Vec<&str>) -> SpanTreeEntry {
    let mut lines = Vec::new();
    lines.push(vec![
        Span::styled("  ", Style::default()),
        Span::styled(name.to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]);
    for d in &details {
        lines.push(vec![
            Span::styled("    ", Style::default()),
            Span::styled(d.to_string(), Style::default().fg(Color::White)),
        ]);
    }
    SpanTreeEntry::new(id, lines)
}

fn run(terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>) -> anyhow::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if handle_event(&mut app)? {
                return Ok(());
            }
        }

        app.blink_tick = (app.blink_tick + 1) % 8;
        Rc::get_mut(&mut app.blink).unwrap().visible = app.blink_tick < 5;
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
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Tab => {
            app.panel = match app.panel {
                Panel::Input => Panel::SpanTree,
                Panel::SpanTree => Panel::Input,
            };
        }
        KeyCode::Char('m') if app.panel == Panel::Input => {
            match app.input.mode() {
                InputMode::Edit => {
                    app.input.set_mode(InputMode::Read);
                    app.read_scroll = 0;
                }
                InputMode::Read => {
                    app.input.set_mode(InputMode::Edit);
                }
            }
        }
        KeyCode::Char('s') if app.panel == Panel::Input => {
            app.cursor_shape_idx = (app.cursor_shape_idx + 1) % CURSOR_SHAPES.len();
            app.input = TextInput::new()
                .with_mode(app.input.mode())
                .with_cursor_style(
                    CursorStyle::new()
                        .with_shape(app.current_shape())
                        .with_position(CursorPosition::OnChar),
                )
                .with_blink_controller(app.blink.clone())
                .with_placeholder("Type markdown here...");
            app.input.set_text(SAMPLE_TEXT);
            app.input.set_cursor_char_idx(SAMPLE_TEXT.len());
        }
        KeyCode::Char('l') if app.panel == Panel::Input => {
            let pos = match app.input.cursor_char_idx() {
                idx if idx < SAMPLE_TEXT.len() => idx + 1,
                _ => app.input.cursor_char_idx(),
            };
            app.input.set_cursor_char_idx(pos);
        }
        KeyCode::Char('h') if app.panel == Panel::Input => {
            if app.input.cursor_char_idx() > 0 {
                app.input.set_cursor_char_idx(app.input.cursor_char_idx() - 1);
            }
        }
        KeyCode::Char('p') if app.panel == Panel::Input => {
            let idx = app.input.cursor_char_idx();
            if idx > 5 && idx < SAMPLE_TEXT.len() {
                app.input.set_selection(Some(Selection::new(idx - 5, idx)));
            }
        }
        KeyCode::Up if app.panel == Panel::SpanTree => {
            app.tree.navigate_up();
        }
        KeyCode::Down if app.panel == Panel::SpanTree => {
            app.tree.navigate_down();
        }
        KeyCode::Up if app.panel == Panel::Input && app.input.mode() == InputMode::Read => {
            app.read_scroll = app.read_scroll.saturating_sub(1);
            app.input.set_scroll_offset(app.read_scroll);
        }
        KeyCode::Down if app.panel == Panel::Input && app.input.mode() == InputMode::Read => {
            app.read_scroll += 1;
            app.input.set_scroll_offset(app.read_scroll);
        }
        _ => {}
    }

    Ok(false)
}

fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let outer = Layout::vertical([
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .split(area);

    let main = Layout::horizontal([
        Constraint::Percentage(60),
        Constraint::Percentage(40),
    ])
    .split(outer[0]);

    draw_input_panel(f, app, main[0]);
    draw_spantree_panel(f, app, main[1]);
    draw_status_bar(f, app, outer[1]);
}

fn draw_input_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.panel == Panel::Input;
    let border_color = if focused { Color::Cyan } else { Color::DarkGray };
    let mode_label = match app.input.mode() {
        InputMode::Edit => "EDIT",
        InputMode::Read => "READ",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" TextInput [{mode_label}] "))
        .border_style(Style::default().fg(border_color))
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.input.text().is_empty() {
        app.input.set_text(SAMPLE_TEXT);
        app.input.set_cursor_char_idx(0);
    }

    app.input.render(f, inner, &Theme);
}

fn draw_spantree_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.panel == Panel::SpanTree;
    let border_color = if focused { Color::Cyan } else { Color::DarkGray };

    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(" SpanTree [AllLines] ")
        .border_style(Style::default().fg(border_color));
    let inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    app.tree.render(f, inner, area, &Theme);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let panel_name = match app.panel {
        Panel::Input => "Input",
        Panel::SpanTree => "SpanTree",
    };
    let shape = app.shape_name();
    let mode = match app.input.mode() {
        InputMode::Edit => "Edit",
        InputMode::Read => "Read",
    };

    let hints = format!(
        " Tab:switch \u{00b7} m:mode({mode}) \u{00b7} s:cursor({shape}) \u{00b7} \u{2191}\u{2193}:nav \u{00b7} h/l:move cursor \u{00b7} p:select \u{00b7} panel:{panel_name} \u{00b7} q:quit "
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(hints, Style::default().fg(Color::DarkGray)))),
        area,
    );
}

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}
