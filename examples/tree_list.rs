#[path = "utils/mod.rs"]
mod common;

use common::{restore_terminal, setup_terminal, Theme};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Padding, Paragraph},
    Frame,
};
use ratatui_markdown::{
    constants::{BRANCH_END_SP, BRANCH_MID_SP, VLINE},
    scroll::{CursorLineMode, SpanTree, SpanTreeEntry},
};

struct AgentGroup {
    id: String,
    label: String,
    is_root: bool,
    is_last_child: bool,
    details: Vec<String>,
}

struct App {
    span_tree: SpanTree,
    groups: Vec<AgentGroup>,
}

fn build_groups() -> Vec<AgentGroup> {
    let raw = lipsum::lipsum(120);
    let words: Vec<String> = raw
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect();
    let mut wi = 0;
    let mut t = |n: usize| -> String {
        let end = (wi + n).min(words.len());
        let phrase: Vec<&str> = words[wi..end].iter().map(|s| s.as_str()).collect();
        wi = end;
        phrase.join(" ")
    };

    vec![
        AgentGroup {
            id: "r0".into(),
            label: format!("#demiurge {}", t(5)),
            is_root: true,
            is_last_child: false,
            details: vec![],
        },
        AgentGroup {
            id: "a0".into(),
            label: format!("#demiurge.001 {} \u{2713}", t(4)),
            is_root: false,
            is_last_child: false,
            details: vec![
                format!("\u{2026}{}", t(8)),
                "{} hubris::task_decompose".into(),
            ],
        },
        AgentGroup {
            id: "a1".into(),
            label: format!("#demiurge.002 {}", t(5)),
            is_root: false,
            is_last_child: true,
            details: vec![
                format!("\u{2026}{}", t(6)),
                "hubris::task_decompose [exec]".into(),
            ],
        },
        AgentGroup {
            id: "r1".into(),
            label: format!("#demiurge {}", t(5)),
            is_root: true,
            is_last_child: false,
            details: vec![],
        },
        AgentGroup {
            id: "b0".into(),
            label: format!("#demiurge.003 {}", t(4)),
            is_root: false,
            is_last_child: false,
            details: vec![
                t(6),
                format!("\u{2026}{}", t(10)),
                "hubris::code_review [exec]".into(),
            ],
        },
        AgentGroup {
            id: "b1".into(),
            label: format!("#demiurge.004 {}", t(4)),
            is_root: false,
            is_last_child: false,
            details: vec![t(5)],
        },
        AgentGroup {
            id: "b2".into(),
            label: format!("#demiurge.005 {}", t(4)),
            is_root: false,
            is_last_child: true,
            details: vec![t(6), t(8), "hubris::planning [done]".into()],
        },
        AgentGroup {
            id: "r2".into(),
            label: format!("#demiurge {}", t(5)),
            is_root: true,
            is_last_child: true,
            details: vec![],
        },
        AgentGroup {
            id: "c0".into(),
            label: format!("#demiurge.006 {}", t(4)),
            is_root: false,
            is_last_child: false,
            details: vec![t(5)],
        },
        AgentGroup {
            id: "c1".into(),
            label: format!("#demiurge.007 {}", t(4)),
            is_root: false,
            is_last_child: true,
            details: vec![t(6), "hubris::analysis [running]".into()],
        },
    ]
}

fn build_entries(groups: &[AgentGroup]) -> Vec<SpanTreeEntry> {
    let muted = Color::DarkGray;
    groups
        .iter()
        .map(|g| {
            let mut lines: Vec<Vec<Span<'static>>> = Vec::new();

            if g.is_root {
                lines.push(vec![
                    Span::raw("  "),
                    Span::styled(
                        g.label.clone(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]);
            } else {
                let conn = if g.is_last_child {
                    BRANCH_END_SP
                } else {
                    BRANCH_MID_SP
                };
                lines.push(vec![
                    Span::raw("  "),
                    Span::styled(conn.to_string(), Style::default().fg(muted)),
                    Span::styled(
                        g.label.clone(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]);

                let stem = if g.is_last_child {
                    "   "
                } else {
                    &*format!("{}  ", VLINE)
                };
                let n = g.details.len();
                for (di, detail) in g.details.iter().enumerate() {
                    let dc = if di == n - 1 {
                        BRANCH_END_SP
                    } else {
                        BRANCH_MID_SP
                    };
                    lines.push(vec![
                        Span::raw("  "),
                        Span::styled(format!("{}{}", stem, dc), Style::default().fg(muted)),
                        Span::styled(detail.clone(), Style::default().fg(Color::White)),
                    ]);
                }
            }

            SpanTreeEntry::new(g.id.clone(), lines)
        })
        .collect()
}

fn run(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> anyhow::Result<()> {
    let groups = build_groups();
    let entries = build_entries(&groups);

    let mut span_tree = SpanTree::new()
        .with_cursor_style(Span::styled("▸ ", Style::default()), Span::raw("  "))
        .with_cursor_line_mode(CursorLineMode::HeaderOnly);

    span_tree.set_entries(entries);
    if !groups.is_empty() {
        span_tree.set_selected_index(0);
    }

    let mut app = App { span_tree, groups };

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
                KeyCode::Up | KeyCode::Char('k') => app.span_tree.navigate_up(),
                KeyCode::Down | KeyCode::Char('j') => app.span_tree.navigate_down(),
                KeyCode::Home => app.span_tree.navigate_to_first(),
                KeyCode::End => app.span_tree.navigate_to_last(),
                KeyCode::Char('c') => {
                    app.span_tree = SpanTree::new()
                        .with_cursor_style(Span::styled("▸ ", Style::default()), Span::raw("  "))
                        .with_cursor_line_mode(match app.span_tree.cursor_line_mode() {
                            CursorLineMode::HeaderOnly => CursorLineMode::AllLines,
                            CursorLineMode::AllLines => CursorLineMode::HeaderOnly,
                        });
                    app.span_tree.set_entries(build_entries(&app.groups));
                    if !app.groups.is_empty() {
                        app.span_tree.set_selected_index(0);
                    }
                }
                _ => {}
            }
        }
    }
}

fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let mode_label = match app.span_tree.cursor_line_mode() {
        CursorLineMode::HeaderOnly => "HeaderOnly",
        CursorLineMode::AllLines => "AllLines",
    };
    let title = format!(" Tree-Style List w/ Cursor ({}) ", mode_label);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    f.render_widget(block, area);
    app.span_tree.render(f, inner, area, &Theme);

    let status =
        " \u{2191}\u{2193}/jk navigate \u{00b7} Home/End \u{00b7} c:cursor mode \u{00b7} q quit ";
    let sa = Rect::new(area.x, area.height.saturating_sub(1), area.width, 1);
    f.render_widget(
        Paragraph::new(Span::styled(status, Style::default().fg(Color::DarkGray))),
        sa,
    );
}

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}
