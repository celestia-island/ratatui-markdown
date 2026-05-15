use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::layout::{Layout, LayoutEdge, LayoutNode};
use super::types::{Direction, EdgeType, NodeShape};
use crate::theme::RichTextTheme;

const HLINE: char = '─';
const VLINE: char = '│';
const TLC: char = '┌';
const TRC: char = '┐';
const BLC: char = '└';
const BRC: char = '┘';
const RTLC: char = '╭';
const RTRC: char = '╮';
const RBLC: char = '╰';
const RBRC: char = '╯';

#[derive(Clone)]
struct Cell {
    ch: char,
    style: Style,
}

#[derive(Clone, Copy, PartialEq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

pub fn render_layout(
    layout: &Layout,
    direction: &Direction,
    theme: &impl RichTextTheme,
) -> Vec<Line<'static>> {
    if layout.nodes.is_empty() {
        return vec![Line::from(Span::styled(
            "(empty diagram)",
            Style::default().fg(theme.get_muted_text_color()),
        ))];
    }

    let gw = layout.grid_width;
    let gh = layout.grid_height;
    if gw == 0 || gh == 0 {
        return Vec::new();
    }

    let mut grid = vec![vec![Cell { ch: ' ', style: Style::default() }; gw]; gh];

    for node in &layout.nodes {
        draw_node(&mut grid, node, theme);
    }

    let is_vertical = matches!(direction, Direction::TopDown | Direction::BottomUp);
    for edge in &layout.edges {
        draw_edge(&mut grid, edge, is_vertical, theme);
    }

    fix_grid_junctions(&mut grid);

    let mut lines = Vec::new();
    for row in grid.iter() {
        let spans: Vec<Span<'static>> = row
            .iter()
            .map(|cell| Span::styled(cell.ch.to_string(), cell.style))
            .collect();
        lines.push(Line::from(spans));
    }

    lines
}

fn draw_node(grid: &mut [Vec<Cell>], node: &LayoutNode, theme: &impl RichTextTheme) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    let (tl, tr, bl, br) = match node.shape {
        NodeShape::Rounded | NodeShape::Circle | NodeShape::Diamond => {
            (RTLC, RTRC, RBLC, RBRC)
        }
        NodeShape::Rect => (TLC, TRC, BLC, BRC),
    };

    let border_style = Style::default().fg(theme.get_muted_text_color());
    let text_style = Style::default().fg(theme.get_text_color());

    if y < grid.len() && x + w <= grid[0].len() {
        let row = &mut grid[y];
        row[x] = Cell {
            ch: tl,
            style: border_style,
        };
        row[x + w - 1] = Cell {
            ch: tr,
            style: border_style,
        };
        for cell in row.iter_mut().take(x + w - 1).skip(x + 1) {
            *cell = Cell {
                ch: HLINE,
                style: border_style,
            };
        }
    }

    let text_row = y + h / 2;
    if text_row < grid.len() && x + w <= grid[0].len() {
        let row = &mut grid[text_row];
        row[x] = Cell {
            ch: VLINE,
            style: border_style,
        };
        row[x + w - 1] = Cell {
            ch: VLINE,
            style: border_style,
        };
        let inner_w = w.saturating_sub(2);
        let label_chars: Vec<char> = node.label.chars().collect();
        let label_w = unicode_width::UnicodeWidthStr::width(node.label.as_str());
        let pad = if label_w < inner_w {
            (inner_w - label_w) / 2
        } else {
            0
        };
        let mut cx = x + 1;
        for _ in 0..pad {
            if cx < x + w - 1 {
                row[cx] = Cell {
                    ch: ' ',
                    style: text_style,
                };
                cx += 1;
            }
        }
        for ch in &label_chars {
            if cx < x + w - 1 {
                row[cx] = Cell {
                    ch: *ch,
                    style: text_style,
                };
                cx += 1;
            }
        }
        while cx < x + w - 1 {
            row[cx] = Cell {
                ch: ' ',
                style: text_style,
            };
            cx += 1;
        }
    }

    for vy in (y + 1)..(y + h - 1) {
        if vy == text_row {
            continue;
        }
        if vy < grid.len() && x + w <= grid[0].len() {
            let row = &mut grid[vy];
            row[x] = Cell {
                ch: VLINE,
                style: border_style,
            };
            row[x + w - 1] = Cell {
                ch: VLINE,
                style: border_style,
            };
            for cell in row.iter_mut().take(x + w - 1).skip(x + 1) {
                *cell = Cell {
                    ch: ' ',
                    style: text_style,
                };
            }
        }
    }

    let bottom_row = y + h - 1;
    if bottom_row < grid.len() && x + w <= grid[0].len() {
        let row = &mut grid[bottom_row];
        row[x] = Cell {
            ch: bl,
            style: border_style,
        };
        row[x + w - 1] = Cell {
            ch: br,
            style: border_style,
        };
        for cell in row.iter_mut().take(x + w - 1).skip(x + 1) {
            *cell = Cell {
                ch: HLINE,
                style: border_style,
            };
        }
    }
}

fn draw_edge(
    grid: &mut [Vec<Cell>],
    edge: &LayoutEdge,
    is_vertical: bool,
    theme: &impl RichTextTheme,
) {
    let wp = &edge.waypoints;
    if wp.len() < 2 {
        return;
    }

    let edge_style = Style::default().fg(theme.get_secondary_color());
    let arrow_style = Style::default()
        .fg(theme.get_primary_color())
        .add_modifier(Modifier::BOLD);

    let dirs: Vec<Dir> = (0..wp.len() - 1)
        .map(|i| {
            let (x1, y1) = wp[i];
            let (x2, y2) = wp[i + 1];
            if x1 == x2 {
                if y2 > y1 {
                    Dir::Down
                } else {
                    Dir::Up
                }
            } else if x2 > x1 {
                Dir::Right
            } else {
                Dir::Left
            }
        })
        .collect();

    for i in 0..dirs.len() {
        let (x1, y1) = wp[i];
        let (x2, y2) = wp[i + 1];
        let include_start = i == 0;

        if y1 == y2 {
            let (lo, hi) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
            let s = if include_start { lo } else { lo + 1 };
            for x in s..hi {
                set_if_empty(grid, x, y1, HLINE, edge_style);
            }
        } else {
            let (lo, hi) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
            let s = if include_start { lo } else { lo + 1 };
            for y in s..hi {
                set_if_empty(grid, x1, y, VLINE, edge_style);
            }
        }
    }

    for i in 1..wp.len() - 1 {
        let (x, y) = wp[i];
        let ch = junction_char(dirs[i - 1], dirs[i]);
        set_if_empty(grid, x, y, ch, edge_style);
    }

    if edge.edge_type == EdgeType::Arrow && !dirs.is_empty() {
        let last_dir = dirs[dirs.len() - 1];
        let &(tx, ty) = wp.last().unwrap();
        let arrow_ch = if is_vertical {
            match last_dir {
                Dir::Down => '▼',
                Dir::Up => '▲',
                _ => '▼',
            }
        } else {
            match last_dir {
                Dir::Right => '►',
                Dir::Left => '◄',
                _ => '►',
            }
        };
        force_set(grid, tx, ty, arrow_ch, arrow_style);
    }

    if let Some(ref label) = edge.label {
        if wp.len() >= 2 {
            let mid = wp.len() / 2;
            let (mx, my) = wp[mid];
            let label_style = Style::default()
                .fg(theme.get_info_color())
                .add_modifier(Modifier::ITALIC);
            let lw = unicode_width::UnicodeWidthStr::width(label.as_str());
            let lx = mx.saturating_sub(lw / 2);
            let ly = my.saturating_sub(1);
            let mut cx = lx;
            for ch in label.chars() {
                let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
                set_label_char(grid, cx, ly, ch, label_style);
                cx += cw;
            }
        }
    }
}

fn junction_char(from: Dir, to: Dir) -> char {
    match (from, to) {
        (Dir::Down, Dir::Right) | (Dir::Left, Dir::Up) => RTLC,
        (Dir::Down, Dir::Left) | (Dir::Right, Dir::Up) => RTRC,
        (Dir::Up, Dir::Left) | (Dir::Right, Dir::Down) => RBRC,
        (Dir::Up, Dir::Right) | (Dir::Left, Dir::Down) => RBLC,
        (Dir::Up, Dir::Down) | (Dir::Down, Dir::Up) => VLINE,
        (Dir::Left, Dir::Right) | (Dir::Right, Dir::Left) => HLINE,
        _ => HLINE,
    }
}

fn set_if_empty(grid: &mut [Vec<Cell>], x: usize, y: usize, ch: char, style: Style) {
    if y < grid.len() && x < grid[0].len() {
        let cell = &mut grid[y][x];
        if cell.ch == ' ' {
            cell.ch = ch;
            cell.style = style;
        }
    }
}

fn force_set(grid: &mut [Vec<Cell>], x: usize, y: usize, ch: char, style: Style) {
    if y < grid.len() && x < grid[0].len() {
        grid[y][x] = Cell { ch, style };
    }
}

fn set_label_char(grid: &mut [Vec<Cell>], x: usize, y: usize, ch: char, style: Style) {
    if y < grid.len() && x < grid[0].len() {
        let cell = &mut grid[y][x];
        if cell.ch == ' ' || is_line_drawing(cell.ch) {
            cell.ch = ch;
            cell.style = style;
        }
    }
}

fn is_line_drawing(ch: char) -> bool {
    matches!(
        ch,
        HLINE
            | VLINE
            | '┼'
            | TLC
            | TRC
            | BLC
            | BRC
            | '├'
            | '┤'
            | '┬'
            | '┴'
            | RTLC
            | RTRC
            | RBLC
            | RBRC
            | '▼'
            | '▲'
            | '►'
            | '◄'
    )
}

fn fix_grid_junctions(grid: &mut [Vec<Cell>]) {
    if grid.is_empty() || grid[0].is_empty() {
        return;
    }
    let gh = grid.len();
    let gw = grid[0].len();
    for y in 1..gh.saturating_sub(1) {
        for x in 1..gw.saturating_sub(1) {
            if grid[y][x].ch == '┼' {
                let up = is_line_drawing(grid[y - 1][x].ch);
                let down = is_line_drawing(grid[y + 1][x].ch);
                let left = is_line_drawing(grid[y][x - 1].ch);
                let right = is_line_drawing(grid[y][x + 1].ch);

                let style = grid[y][x].style;
                let new_ch = match (up, down, left, right) {
                    (true, true, true, true) => '┼',
                    (true, true, true, false) => '├',
                    (true, true, false, true) => '┤',
                    (true, false, true, true) => '┴',
                    (false, true, true, true) => '┬',
                    (true, false, true, false) => RBLC,
                    (true, false, false, true) => RBRC,
                    (false, true, true, false) => RTLC,
                    (false, true, false, true) => RTRC,
                    (true, true, false, false) => VLINE,
                    (false, false, true, true) => HLINE,
                    _ => '┼',
                };
                grid[y][x] = Cell {
                    ch: new_ch,
                    style,
                };
            }
        }
    }
}
