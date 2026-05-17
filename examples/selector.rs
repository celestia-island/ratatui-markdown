#[path = "utils/mod.rs"]
mod common;

use std::collections::HashSet;

use common::{restore_terminal, setup_terminal, Theme};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseEventKind},
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use ratatui_markdown::{
    constants::{HLINE, ROUNDED_BL, ROUNDED_TL, VLINE},
    theme::RichTextTheme,
    tree::{CollapsibleTree, EntryKind, KeyStyle, ValueType},
};

const JSON_DATA: &str = r#"{
  "package": "ratatui-markdown",
  "version": "0.2.2",
  "metadata": {
    "edition": "2021",
    "license": "MIT OR Apache-2.0",
    "rust_version": "1.74"
  },
  "features": {
    "markdown": true,
    "tree": true,
    "scroll": true,
    "mermaid": true,
    "image": true,
    "preview": true
  },
  "dependencies": {
    "ratatui": "^0.29",
    "unicode-width": "^0.2"
  },
  "dev_dependencies": {
    "anyhow": "^1",
    "crossterm": "^0.28",
    "lipsum": "^0.9"
  }
}"#;

struct CodeBlockDef {
    lang: &'static str,
    code: &'static str,
}

const CODE_BLOCKS: &[CodeBlockDef] = &[
    CodeBlockDef {
        lang: "rust",
        code: "fn greet(name: &str) -> String {\n    format!(\"Hello, {}!\", name)\n}",
    },
    CodeBlockDef {
        lang: "toml",
        code: "[dependencies]\nratatui = \"^0.29\"\nunicode-width = \"^0.2\"",
    },
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectorMode {
    Gutter,
    Highlight,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TreeStyle {
    Connectors,
    Plain,
}

struct BlockInfo {
    start: usize,
    count: usize,
    path: String,
    collapsible: bool,
}

struct App {
    selector_mode: SelectorMode,
    tree_style: TreeStyle,
    focus: usize,
    selected: HashSet<String>,
    scroll: u16,
    content_h: u16,
    cursor_text: &'static str,
    cursor_every_line: bool,

    tree: CollapsibleTree,
    lines: Vec<Line<'static>>,
    blocks: Vec<BlockInfo>,
}

impl App {
    fn new(theme: &impl RichTextTheme) -> Self {
        let mut tree = CollapsibleTree::from_json_str(JSON_DATA)
            .unwrap()
            .with_show_root(false)
            .with_key_style(KeyStyle::Toml);
        tree.expand_to_depth(1);

        let mut app = Self {
            selector_mode: SelectorMode::Gutter,
            tree_style: TreeStyle::Connectors,
            focus: 0,
            selected: HashSet::new(),
            scroll: 0,
            content_h: 0,
            cursor_text: "> ",
            cursor_every_line: false,
            tree,
            lines: Vec::new(),
            blocks: Vec::new(),
        };
        app.rebuild(theme);
        app
    }

    fn rebuild(&mut self, theme: &impl RichTextTheme) {
        let old_focus_path = self
            .blocks
            .get(self.focus)
            .map(|b| b.path.clone())
            .unwrap_or_default();

        self.lines.clear();
        self.blocks.clear();

        let entries = self.tree.flatten();
        let tree_lines = match self.tree_style {
            TreeStyle::Connectors => self.tree.render_lines(74, theme),
            TreeStyle::Plain => render_tree_plain(&entries, theme, self.tree.key_style),
        };

        for entry in &entries {
            let collapsible = matches!(
                entry.kind,
                EntryKind::Collapsed { .. } | EntryKind::Expanded { .. }
            );
            self.blocks.push(BlockInfo {
                start: self.lines.len(),
                count: 1,
                path: entry.path.clone(),
                collapsible,
            });
            if let Some(line) = tree_lines.get(self.lines.len()) {
                self.lines.push(line.clone());
            } else {
                self.lines.push(Line::raw(""));
            }
        }

        self.lines.push(Line::raw(""));
        self.lines.push(Line::from(Span::styled(
            format!(" {HLINE} Code Examples {HLINE}"),
            Style::default().fg(theme.get_muted_text_color()),
        )));

        for (ci, cb) in CODE_BLOCKS.iter().enumerate() {
            let code_lines = build_code_block(cb.lang, cb.code, theme);
            let path = format!("__code_{ci}");
            self.blocks.push(BlockInfo {
                start: self.lines.len(),
                count: code_lines.len(),
                path,
                collapsible: false,
            });
            self.lines.extend(code_lines);
        }

        self.focus = self
            .blocks
            .iter()
            .position(|b| b.path == old_focus_path)
            .unwrap_or_else(|| {
                if self.focus < self.blocks.len() {
                    self.focus
                } else {
                    self.blocks.len().saturating_sub(1)
                }
            });
    }

    fn toggle_tree_node(&mut self, theme: &impl RichTextTheme) {
        if let Some(block) = self.blocks.get(self.focus) {
            if block.collapsible {
                self.tree.toggle(&block.path);
                self.rebuild(theme);
            }
        }
    }

    fn next_block(&mut self) {
        if self.focus < self.blocks.len() - 1 {
            self.focus += 1;
            self.ensure_visible();
        }
    }

    fn prev_block(&mut self) {
        if self.focus > 0 {
            self.focus -= 1;
            self.ensure_visible();
        }
    }

    fn toggle_select(&mut self) {
        if let Some(block) = self.blocks.get(self.focus) {
            if self.selected.contains(&block.path) {
                self.selected.remove(&block.path);
            } else {
                self.selected.insert(block.path.clone());
            }
        }
    }

    fn select_all(&mut self) {
        for b in &self.blocks {
            self.selected.insert(b.path.clone());
        }
    }

    fn deselect_all(&mut self) {
        self.selected.clear();
    }

    fn is_selected(&self, idx: usize) -> bool {
        self.blocks
            .get(idx)
            .map(|b| self.selected.contains(&b.path))
            .unwrap_or(false)
    }

    fn clamp_scroll(&mut self) {
        let max = self.lines.len().saturating_sub(self.content_h as usize);
        if self.scroll as usize > max {
            self.scroll = max as u16;
        }
    }

    fn ensure_visible(&mut self) {
        if self.focus >= self.blocks.len() {
            return;
        }
        let b = &self.blocks[self.focus];
        let start = b.start;
        let end = start + b.count;
        let scroll = self.scroll as usize;
        let visible = self.content_h as usize;

        if visible == 0 {
            return;
        }

        if start < scroll {
            self.scroll = start as u16;
        } else if end > scroll + visible {
            if b.count <= visible {
                self.scroll = (end - visible) as u16;
            } else {
                self.scroll = start as u16;
            }
        }
    }
}

fn render_tree_plain(
    entries: &[ratatui_markdown::tree::FlatEntry],
    theme: &impl RichTextTheme,
    key_style: KeyStyle,
) -> Vec<Line<'static>> {
    let key_color = theme.get_json_key_color();
    let muted = theme.get_muted_text_color();

    entries
        .iter()
        .map(|entry| {
            let indent = Span::raw("  ".repeat(entry.depth));

            match &entry.kind {
                EntryKind::Collapsed { label, count_str } => Line::from(vec![
                    indent,
                    Span::styled("\u{25b6} ", Style::default().fg(key_color)),
                    Span::styled(label.clone(), Style::default().fg(key_color)),
                    Span::styled(format!(" {}", count_str), Style::default().fg(muted)),
                ]),
                EntryKind::Expanded { label, count_str } => Line::from(vec![
                    indent,
                    Span::styled(
                        format!("{} ", ratatui_markdown::constants::TRIANGLE_DOWN),
                        Style::default().fg(key_color),
                    ),
                    Span::styled(label.clone(), Style::default().fg(key_color)),
                    Span::styled(format!(" {}", count_str), Style::default().fg(muted)),
                ]),
                EntryKind::Leaf {
                    key,
                    value,
                    value_type,
                } => {
                    let val_color = match value_type {
                        ValueType::String => theme.get_json_string_color(),
                        ValueType::Number => theme.get_json_number_color(),
                        ValueType::Boolean => theme.get_json_bool_color(),
                        ValueType::Null => theme.get_json_null_color(),
                    };
                    let (key_prefix, separator) = match key_style {
                        KeyStyle::Json => ("\"", "\": "),
                        KeyStyle::Toml => ("", " = "),
                    };
                    Line::from(vec![
                        indent,
                        Span::styled(
                            format!("{}{}{}", key_prefix, key, separator),
                            Style::default().fg(key_color),
                        ),
                        Span::styled(value.clone(), Style::default().fg(val_color)),
                    ])
                }
            }
        })
        .collect()
}

fn build_code_block(lang: &str, code: &str, theme: &impl RichTextTheme) -> Vec<Line<'static>> {
    let muted = theme.get_muted_text_color();
    let yellow = theme.get_accent_yellow();
    let mut lines = Vec::new();

    let header = if lang.is_empty() {
        format!("{ROUNDED_TL}{HLINE}")
    } else {
        format!("{ROUNDED_TL}{HLINE} {lang}")
    };
    lines.push(Line::from(Span::styled(header, Style::default().fg(muted))));

    let prefix = format!("{VLINE} ");
    for cl in code.lines() {
        lines.push(Line::from(vec![
            Span::styled(prefix.clone(), Style::default().fg(muted)),
            Span::styled(cl.to_string(), Style::default().fg(yellow)),
        ]));
    }

    lines.push(Line::from(Span::styled(
        format!("{ROUNDED_BL}{HLINE}"),
        Style::default().fg(muted),
    )));
    lines
}

fn find_block(blocks: &[BlockInfo], line: usize) -> Option<usize> {
    for (i, b) in blocks.iter().enumerate() {
        if line >= b.start && line < b.start + b.count {
            return Some(i);
        }
    }
    None
}

fn pad_or_truncate(spans: Vec<Span<'static>>, target_w: usize) -> Vec<Span<'static>> {
    let used: usize = spans.iter().map(|s| s.width()).sum();
    if used == target_w {
        return spans;
    }
    if used < target_w {
        let mut s = spans;
        s.push(Span::raw(" ".repeat(target_w - used)));
        return s;
    }
    let mut taken = 0usize;
    let mut out = Vec::new();
    for sp in spans {
        let sw = sp.width();
        if taken + sw > target_w {
            let keep = target_w - taken;
            let chop: String = sp.content.chars().take(keep).collect();
            out.push(Span::styled(chop, sp.style));
            break;
        }
        taken += sw;
        out.push(sp);
    }
    while taken < target_w {
        out.push(Span::raw(" "));
        taken += 1;
    }
    out
}

fn draw(f: &mut Frame, app: &mut App, theme: &Theme) {
    let area = f.area();
    let mode_label = match app.selector_mode {
        SelectorMode::Gutter => "Gutter",
        SelectorMode::Highlight => "Highlight",
    };
    let tree_label = match app.tree_style {
        TreeStyle::Connectors => "Tree",
        TreeStyle::Plain => "Plain",
    };
    let cursor_label = if app.cursor_every_line {
        "every-line"
    } else {
        "first-line"
    };
    let sel_count = app.selected.len();
    let total = app.blocks.len();

    let title = " Block Selector ";
    let status = format!(
        " {mode_label} · {tree_label} · {cursor_label} · {sel_count}/{total} selected │ ↑↓/jk focus · Space select · Enter expand · Tab mode · t tree · c cursor · a all · q quit "
    );

    let block_area = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1));
    let block_widget = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block_widget.inner(block_area);
    let content_h = inner.height;
    let inner_w = inner.width as usize;
    app.content_h = content_h;

    let scroll = app.scroll as usize;
    let visible = content_h as usize;
    let cursor_w = app.cursor_text.len();

    let primary = theme.get_primary_color();
    let secondary = theme.get_secondary_color();
    let highlight_bg = theme.get_popup_selected_background();

    let mut display: Vec<Line<'static>> = Vec::with_capacity(visible);

    for vi in 0..visible {
        let li = scroll + vi;

        if li >= app.lines.len() {
            display.push(Line::from(Span::raw(" ".repeat(inner_w))));
            continue;
        }

        let bidx = match find_block(&app.blocks, li) {
            Some(idx) => idx,
            None => {
                let spans = pad_or_truncate(app.lines[li].spans.clone(), inner_w);
                display.push(Line::from(spans));
                continue;
            }
        };
        let is_sel = app.is_selected(bidx);
        let is_focus = app.focus == bidx;
        let block = &app.blocks[bidx];
        let line_in_block = li - block.start;
        let is_first = line_in_block == 0;

        let original = app.lines[li].spans.clone();

        match app.selector_mode {
            SelectorMode::Gutter => {
                let gutter_str: String;
                let gutter_style: Style;

                if is_sel {
                    if is_first || app.cursor_every_line {
                        gutter_str = app.cursor_text.to_string();
                        gutter_style = Style::default().fg(primary);
                    } else {
                        gutter_str = " ".repeat(cursor_w);
                        gutter_style = Style::default();
                    }
                } else if is_focus {
                    if is_first {
                        gutter_str = app.cursor_text.to_string();
                        gutter_style = Style::default().fg(secondary);
                    } else {
                        gutter_str = " ".repeat(cursor_w);
                        gutter_style = Style::default();
                    }
                } else {
                    gutter_str = " ".repeat(cursor_w);
                    gutter_style = Style::default();
                };

                let content_w = inner_w.saturating_sub(cursor_w);
                let padded = pad_or_truncate(original, content_w);

                let mut spans = Vec::with_capacity(1 + padded.len());
                spans.push(Span::styled(gutter_str, gutter_style));
                spans.extend(padded);
                display.push(Line::from(spans));
            }
            SelectorMode::Highlight => {
                let bg = if is_sel {
                    highlight_bg
                } else if is_focus {
                    Color::Indexed(236)
                } else {
                    Color::Reset
                };

                let mut spans: Vec<Span<'static>> = original;
                if bg != Color::Reset {
                    for span in &mut spans {
                        span.style = span.style.bg(bg);
                    }
                }

                let used: usize = spans.iter().map(|s| s.width()).sum();
                if used < inner_w {
                    let pad_style = if bg != Color::Reset {
                        Style::default().bg(bg)
                    } else {
                        Style::default()
                    };
                    spans.push(Span::styled(" ".repeat(inner_w - used), pad_style));
                } else if used > inner_w {
                    let mut taken = 0usize;
                    let mut truncated = Vec::new();
                    for sp in spans {
                        let sw = sp.width();
                        if taken + sw > inner_w {
                            let keep = inner_w - taken;
                            let chop: String = sp.content.chars().take(keep).collect();
                            truncated.push(Span::styled(chop, sp.style));
                            break;
                        }
                        taken += sw;
                        truncated.push(sp);
                    }
                    let pad_style = if bg != Color::Reset {
                        Style::default().bg(bg)
                    } else {
                        Style::default()
                    };
                    while taken < inner_w {
                        truncated.push(Span::styled(" ", pad_style));
                        taken += 1;
                    }
                    spans = truncated;
                }

                display.push(Line::from(spans));
            }
        }
    }

    f.render_widget(block_widget, block_area);
    f.render_widget(Paragraph::new(display), inner);

    let doc_h = app.lines.len() as u16;
    if doc_h > content_h && content_h > 0 {
        let sb_col = block_area.x + block_area.width.saturating_sub(1);
        let sb_area = Rect::new(sb_col, inner.y, 1, content_h);
        let content_len = doc_h.saturating_sub(content_h).saturating_add(1);
        let sb = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .thumb_symbol("\u{2588}")
            .track_symbol(Some("\u{2502}"))
            .style(Style::default().fg(Color::DarkGray))
            .thumb_style(Style::default().fg(Color::Cyan));
        let mut sb_state = ScrollbarState::default()
            .content_length(content_len as usize)
            .viewport_content_length(content_h as usize)
            .position(app.scroll as usize);
        f.render_stateful_widget(sb, sb_area, &mut sb_state);
    }

    let status_area = Rect::new(area.x, area.height.saturating_sub(1), area.width, 1);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            status,
            Style::default().fg(Color::DarkGray),
        ))),
        status_area,
    );
}

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;
    let theme = Theme;

    let mut app = App::new(&theme);

    loop {
        terminal.draw(|f| draw(f, &mut app, &theme))?;

        if !event::poll(std::time::Duration::from_millis(50))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Up | KeyCode::Char('k') => {
                    app.prev_block();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.next_block();
                }
                KeyCode::Char('J') => {
                    app.scroll = app.scroll.saturating_add(1);
                    app.clamp_scroll();
                }
                KeyCode::Char('K') => {
                    app.scroll = app.scroll.saturating_sub(1);
                }
                KeyCode::PageUp => {
                    let step = app.content_h.max(1);
                    app.scroll = app.scroll.saturating_sub(step);
                }
                KeyCode::PageDown => {
                    let step = app.content_h.max(1);
                    app.scroll = app.scroll.saturating_add(step);
                    app.clamp_scroll();
                }
                KeyCode::Home => {
                    app.focus = 0;
                    app.ensure_visible();
                }
                KeyCode::End => {
                    app.focus = app.blocks.len().saturating_sub(1);
                    app.ensure_visible();
                }
                KeyCode::Char(' ') => app.toggle_select(),
                KeyCode::Enter => app.toggle_tree_node(&theme),
                KeyCode::Tab => {
                    app.selector_mode = match app.selector_mode {
                        SelectorMode::Gutter => SelectorMode::Highlight,
                        SelectorMode::Highlight => SelectorMode::Gutter,
                    };
                }
                KeyCode::Char('t') => {
                    app.tree_style = match app.tree_style {
                        TreeStyle::Connectors => TreeStyle::Plain,
                        TreeStyle::Plain => TreeStyle::Connectors,
                    };
                    app.rebuild(&theme);
                }
                KeyCode::Char('c') if app.selector_mode == SelectorMode::Gutter => {
                    app.cursor_every_line = !app.cursor_every_line;
                }
                KeyCode::Char('a') => {
                    if app.selected.len() == app.blocks.len() {
                        app.deselect_all();
                    } else {
                        app.select_all();
                    }
                }
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => {
                    app.scroll = app.scroll.saturating_sub(3);
                }
                MouseEventKind::ScrollDown => {
                    app.scroll = app.scroll.saturating_add(3);
                    app.clamp_scroll();
                }
                _ => {}
            },
            _ => {}
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}
