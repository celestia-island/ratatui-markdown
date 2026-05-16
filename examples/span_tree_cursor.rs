#[path = "utils/mod.rs"]
mod common;

use common::{restore_terminal, setup_terminal, Theme};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame,
};
use ratatui_markdown::scroll::{CursorLineMode, SpanTree, SpanTreeEntry};

struct App {
    trees: [SpanTree; 2],
    active: usize,
}

fn make_entry(id: &str, name: &str, details: Vec<&str>) -> SpanTreeEntry {
    let mut lines = Vec::new();
    lines.push(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            name.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    for d in &details {
        lines.push(vec![
            Span::styled("    ", Style::default()),
            Span::styled(d.to_string(), Style::default().fg(Color::White)),
        ]);
    }
    SpanTreeEntry::new(id, lines)
}

fn build_tree(mode: CursorLineMode) -> SpanTree {
    let mut tree = SpanTree::new().with_cursor_line_mode(mode);
    let entries = vec![
        make_entry(
            "a",
            "Alpha",
            vec!["Task: Write docs", "Status: In progress", "Priority: High"],
        ),
        make_entry("b", "Beta", vec!["Task: Fix bugs", "Status: Done"]),
        make_entry(
            "c",
            "Gamma",
            vec!["Task: Add tests", "Status: Pending", "ETA: 2 days"],
        ),
        make_entry("d", "Delta", vec!["Task: Review PR", "Assignee: Alice"]),
    ];
    tree.set_entries(entries);
    tree.set_selected_index(0);
    tree
}

fn run(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> anyhow::Result<()> {
    let mut app = App {
        trees: [
            build_tree(CursorLineMode::HeaderOnly),
            build_tree(CursorLineMode::AllLines),
        ],
        active: 0,
    };

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Tab => app.active = 1 - app.active,
                KeyCode::Up => app.trees[app.active].navigate_up(),
                KeyCode::Down => app.trees[app.active].navigate_down(),
                KeyCode::Home => app.trees[app.active].navigate_to_first(),
                KeyCode::End => app.trees[app.active].navigate_to_last(),
                _ => {}
            }
        }
    }
}

fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let full = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1));

    let half_w = full.width / 2;
    let left_area = Rect::new(full.x, full.y, half_w, full.height);
    let right_area = Rect::new(full.x + half_w, full.y, full.width - half_w, full.height);

    draw_tree_panel(f, app, 0, "HeaderOnly (default)", left_area);
    draw_tree_panel(f, app, 1, "AllLines", right_area);

    let status =
        " Tab:switch panel \u{00b7} \u{2191}\u{2193}:navigate \u{00b7} Home/End \u{00b7} q:quit ";
    let status_area = Rect::new(area.x, area.height.saturating_sub(1), area.width, 1);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            status,
            Style::default().fg(Color::DarkGray),
        ))),
        status_area,
    );
}

fn draw_tree_panel(f: &mut Frame, app: &mut App, idx: usize, title: &str, area: Rect) {
    let focused = app.active == idx;
    let border_color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(border_color))
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    f.render_widget(block, area);
    app.trees[idx].render(f, inner, area, &Theme);
}

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}
