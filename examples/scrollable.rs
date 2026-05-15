use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind, MouseEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame, Terminal,
};
use ratatui_markdown::{
    markdown::MarkdownRenderer,
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

struct ScrollState {
    v_offset: usize,
    h_offset: usize,
    total_lines: usize,
    max_line_width: usize,
    pad_top: u16,
    pad_bottom: u16,
    pad_left: u16,
    pad_right: u16,
}

impl ScrollState {
    fn new(total_lines: usize, max_line_width: usize) -> Self {
        Self {
            v_offset: 0,
            h_offset: 0,
            total_lines,
            max_line_width,
            pad_top: 1,
            pad_bottom: 1,
            pad_left: 2,
            pad_right: 2,
        }
    }

    fn viewport_height(&self, area_height: u16) -> usize {
        area_height.saturating_sub(self.pad_top + self.pad_bottom) as usize
    }

    fn viewport_width(&self, area_width: u16) -> usize {
        area_width.saturating_sub(self.pad_left + self.pad_right) as usize
    }

    fn max_v_offset(&self, area_height: u16) -> usize {
        self.total_lines.saturating_sub(self.viewport_height(area_height))
    }

    fn max_h_offset(&self, area_width: u16) -> usize {
        self.max_line_width.saturating_sub(self.viewport_width(area_width))
    }

    fn clamp(&mut self, area: Rect) {
        self.v_offset = self.v_offset.min(self.max_v_offset(area.height));
        self.h_offset = self.h_offset.min(self.max_h_offset(area.width));
    }

    fn scroll_v(&mut self, delta: isize, area: Rect) {
        if delta >= 0 {
            self.v_offset = self.v_offset.saturating_add(delta as usize);
        } else {
            self.v_offset = self.v_offset.saturating_sub((-delta) as usize);
        }
        self.clamp(area);
    }

    fn scroll_h(&mut self, delta: isize, area: Rect) {
        if delta >= 0 {
            self.h_offset = self.h_offset.saturating_add(delta as usize);
        } else {
            self.h_offset = self.h_offset.saturating_sub((-delta) as usize);
        }
        self.clamp(area);
    }

    fn page_up(&mut self, area: Rect) {
        let step = self.viewport_height(area.height).max(1);
        self.scroll_v(-(step as isize), area);
    }

    fn page_down(&mut self, area: Rect) {
        let step = self.viewport_height(area.height).max(1);
        self.scroll_v(step as isize, area);
    }
}

fn render_v_scrollbar(f: &mut Frame, area: Rect, state: &ScrollState) {
    let vp = state.viewport_height(area.height);
    let total = state.total_lines;
    if total <= vp || vp == 0 {
        return;
    }
    let sb_area = Rect::new(
        area.x + area.width.saturating_sub(1),
        area.y + state.pad_top,
        1,
        area.height.saturating_sub(state.pad_top + state.pad_bottom),
    );
    let max_pos = total.saturating_sub(1);
    let thumb_pos = if total > vp {
        let max_off = total - vp;
        if max_off > 0 && max_pos > 0 {
            (state.v_offset as u64 * max_pos as u64 / max_off as u64) as usize
        } else {
            0
        }
    } else {
        0
    };
    let sb = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .thumb_symbol("█")
        .track_symbol(Some("│"))
        .style(Style::default().fg(Color::DarkGray))
        .thumb_style(Style::default().fg(Color::Cyan));
    let mut sb_state = ScrollbarState::default()
        .content_length(total)
        .viewport_content_length(vp)
        .position(thumb_pos);
    f.render_stateful_widget(sb, sb_area, &mut sb_state);
}

fn render_h_scrollbar(f: &mut Frame, area: Rect, state: &ScrollState) {
    let vp = state.viewport_width(area.width);
    let total = state.max_line_width;
    if total <= vp || vp == 0 {
        return;
    }
    let sb_area = Rect::new(
        area.x + state.pad_left,
        area.y + area.height.saturating_sub(1),
        area.width.saturating_sub(state.pad_left + state.pad_right),
        1,
    );
    let max_pos = total.saturating_sub(1);
    let thumb_pos = if total > vp {
        let max_off = total - vp;
        if max_off > 0 && max_pos > 0 {
            (state.h_offset as u64 * max_pos as u64 / max_off as u64) as usize
        } else {
            0
        }
    } else {
        0
    };
    let sb = Scrollbar::default()
        .orientation(ScrollbarOrientation::HorizontalBottom)
        .thumb_symbol("█")
        .track_symbol(Some("─"))
        .style(Style::default().fg(Color::DarkGray))
        .thumb_style(Style::default().fg(Color::Cyan));
    let mut sb_state = ScrollbarState::default()
        .content_length(total)
        .viewport_content_length(vp)
        .position(thumb_pos);
    f.render_stateful_widget(sb, sb_area, &mut sb_state);
}

fn render_info_panel(f: &mut Frame, area: Rect, state: &ScrollState) {
    let vp_h = state.viewport_height(area.height);
    let vp_w = state.viewport_width(area.width);
    let max_v = state.max_v_offset(area.height);
    let max_h = state.max_h_offset(area.width);
    let info_lines = vec![
        Line::from(vec![
            Span::styled("─ Scroll Info ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!("┤ V:{}/{} H:{}/{} │ VP:{}x{} Content:{}x{} │ Pad:t{} b{} l{} r{}",
                state.v_offset, max_v,
                state.h_offset, max_h,
                vp_h, vp_w,
                state.total_lines, state.max_line_width,
                state.pad_top, state.pad_bottom, state.pad_left, state.pad_right,
            ), Style::default().fg(Color::DarkGray)),
        ]),
    ];
    let info_area = Rect::new(area.x, area.y + area.height.saturating_sub(3), area.width, 1);
    f.render_widget(Paragraph::new(info_lines), info_area);
}

fn clip_line_for_horizontal_scroll(line: &Line<'static>, h_offset: usize, viewport_w: usize) -> Line<'static> {
    let mut skipped_width = 0;
    let mut start_idx = None;
    for (i, span) in line.spans.iter().enumerate() {
        let sw = unicode_width::UnicodeWidthStr::width(span.content.as_ref());
        if skipped_width + sw > h_offset {
            start_idx = Some((i, h_offset.saturating_sub(skipped_width)));
            break;
        }
        skipped_width += sw;
    }
    let mut out_spans = Vec::new();
    let start = match start_idx {
        Some((idx, inner_skip)) => {
            let span = &line.spans[idx];
            let chars: Vec<char> = span.content.chars().collect();
            let remaining: String = chars[inner_skip.min(chars.len())..].iter().collect();
            if !remaining.is_empty() {
                out_spans.push(Span::styled(remaining, span.style));
            }
            idx + 1
        }
        None => {
            if h_offset < skipped_width {
                line.spans.len()
            } else {
                return Line::from(Span::raw(""));
            }
        }
    };
    let mut current_width = unicode_width::UnicodeWidthStr::width(
        out_spans.last().map(|s| s.content.as_ref()).unwrap_or("")
    );
    for i in start..line.spans.len() {
        if current_width >= viewport_w {
            break;
        }
        let span = &line.spans[i];
        let sw = unicode_width::UnicodeWidthStr::width(span.content.as_ref());
        if current_width + sw <= viewport_w || out_spans.is_empty() {
            out_spans.push(span.clone());
            current_width += sw;
        } else {
            let remaining = viewport_w.saturating_sub(current_width);
            let chars: Vec<char> = span.content.chars().collect();
            let mut taken = 0;
            let mut w = 0;
            for &ch in &chars {
                let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                if w + cw > remaining {
                    break;
                }
                taken += ch.len_utf8();
                w += cw;
            }
            if taken > 0 {
                let partial: String = chars[..taken].iter().collect();
                out_spans.push(Span::styled(partial, span.style));
            }
            break;
        }
    }
    Line::from(out_spans)
}

const MARKDOWN: &str = r#"
# Scrollable Markdown Panel

This example demonstrates **padding**, **vertical** and **horizontal** **scrollbars**, and **viewport info**.

## Features

- Vertical scrollbar (right edge) — tracks scroll position through content lines
- Horizontal scrollbar (bottom edge) — tracks horizontal scroll position
- Info panel — real-time readout of scroll state, viewport, and content dimensions
- Configurable padding (top/bottom/left/right)
- Mouse wheel support (vertical + Shift+wheel for horizontal)
- Keyboard navigation (arrows, Page Up/Down, Home/End)

## Long Lines Test

This is a deliberately long line that exceeds normal terminal width to demonstrate horizontal scrolling in action. The horizontal scrollbar at the bottom should become active when the content width exceeds the viewport width after accounting for left/right padding.

| Column A | Column B | Column C | Column D | Column E |
|----------|----------|----------|----------|----------|
| Alpha    | Bravo    | Charlie  | Delta    | Echo     |
| One      | Two      | Three    | Four     | Five     |
| First    | Second   | Third    | Fourth   | Fifth    |
| A        | B        | C        | D        | E        |
| 101      | 202      | 303      | 404      | 505      |

## Code Block

```rust
fn render_scrollable(
    f: &mut Frame,
    area: Rect,
    lines: Vec<Line>,
    scroll: &mut ScrollState,
) {
    let inner = apply_padding(area, scroll);
    let visible = clip_to_viewport(lines, scroll);
    f.render_widget(Paragraph::new(visible), inner);
    render_v_scrollbar(f, area, scroll);
    render_h_scrollbar(f, area, scroll);
    render_info_panel(f, area, scroll);
}
```

## Nested List

- Level 0 Item 1
  - Level 1 Item A
    - Level 2 Item i
    - Level 2 Item ii
    - Level 2 Item iii (long text here to push width)
  - Level 1 Item B
  - Level 1 Item C
- Level 0 Item 2
  - Level 1 Item D
  - Level 1 Item E
- Level 0 Item 3
- Level 0 Item 4
- Level 0 Item 5
- Level 0 Item 6
- Level 0 Item 7
- Level 0 Item 8
- Level 0 Item 9
- Level 0 Item 10
- Level 0 Item 11
- Level 0 Item 12
- Level 0 Item 13
- Level 0 Item 14
- Level 0 Item 15

## Blockquote

> This is a blockquote that contains enough text to potentially wrap across multiple lines, demonstrating how the scrollable panel handles wrapped content within a padded viewport area.

## Mouse Event Handling

Mouse events are handled by the application loop. Below is a summary:

| Event              | Action                        |
|--------------------|-------------------------------|
| Wheel Up           | Scroll up 3 lines             |
| Wheel Down         | Scroll down 3 lines           |
| Shift + Wheel Up   | Scroll left                   |
| Shift + Wheel Down | Scroll right                  |
| `↑` / `k`         | Scroll up 1 line              |
| `↓` / `j`         | Scroll down 1 line            |
| `←` / `h`         | Scroll left                  |
| `→` / `l`         | Scroll right                 |
| `Page Up`          | Page up (one viewport)       |
| `Page Down`        | Page down (one viewport)      |
| `Home`             | Scroll to top-left            |
| `End`              | Scroll to bottom-right        |
| `q`                | Quit                          |

Press `q` to quit.
"#;

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let theme = Theme;
    let renderer = MarkdownRenderer::new(120);
    let blocks = renderer.parse(MARKDOWN);
    let lines = renderer.render(&blocks, &theme);

    let max_line_width = lines.iter().map(|line| {
        line.spans.iter().map(|s| unicode_width::UnicodeWidthStr::width(s.content.as_ref())).sum::<usize>()
    }).max().unwrap_or(0);

    let mut scroll = ScrollState::new(lines.len(), max_line_width);
    let mut last_area: Option<Rect> = None;

    loop {
        terminal.draw(|f| {
            let area = f.area();
            last_area = Some(area);
            let inner = Rect::new(
                area.x + scroll.pad_left,
                area.y + scroll.pad_top,
                area.width.saturating_sub(scroll.pad_left + scroll.pad_right),
                area.height.saturating_sub(scroll.pad_top + scroll.pad_bottom + 1),
            );
            scroll.clamp(inner);

            let vp_h = inner.height as usize;
            let vp_w = inner.width as usize;
            let start = scroll.v_offset.min(scroll.total_lines.saturating_sub(vp_h.max(1)));
            let end = (start + vp_h).min(scroll.total_lines);

            let visible: Vec<Line<'static>> = lines[start..end]
                .iter()
                .map(|line| clip_line_for_horizontal_scroll(line, scroll.h_offset, vp_w))
                .collect();

            f.render_widget(
                Paragraph::new(visible)
                    .block(Block::default().borders(Borders::ALL).title(" Scrollable Panel Example ")),
                area,
            );

            render_v_scrollbar(f, area, &scroll);
            render_h_scrollbar(f, area, &scroll);
            render_info_panel(f, area, &scroll);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up | KeyCode::Char('k') => scroll.scroll_v(-1, last_area.unwrap_or_default()),
                    KeyCode::Down | KeyCode::Char('j') => scroll.scroll_v(1, last_area.unwrap_or_default()),
                    KeyCode::Left | KeyCode::Char('h') => scroll.scroll_h(-1, last_area.unwrap_or_default()),
                    KeyCode::Right | KeyCode::Char('l') => scroll.scroll_h(1, last_area.unwrap_or_default()),
                    KeyCode::PageUp => scroll.page_up(last_area.unwrap_or_default()),
                    KeyCode::PageDown => scroll.page_down(last_area.unwrap_or_default()),
                    KeyCode::Home => { scroll.v_offset = 0; scroll.h_offset = 0; }
                    KeyCode::End => {
                        let a = last_area.unwrap_or_default();
                        scroll.v_offset = scroll.max_v_offset(a.height);
                        scroll.h_offset = scroll.max_h_offset(a.width);
                    }
                    _ => {}
                },
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => scroll.scroll_v(-3, last_area.unwrap_or_default()),
                    MouseEventKind::ScrollDown => scroll.scroll_v(3, last_area.unwrap_or_default()),
                    MouseEventKind::ScrollLeft => scroll.scroll_h(-3, last_area.unwrap_or_default()),
                    MouseEventKind::ScrollRight => scroll.scroll_h(3, last_area.unwrap_or_default()),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    Ok(())
}
