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
    is_edge: bool,
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

    let blank = Cell {
        ch: ' ',
        style: Style::default(),
        is_edge: false,
    };
    let mut grid = vec![vec![blank; gw]; gh];

    for node in &layout.nodes {
        draw_node(&mut grid, node, theme);
    }

    let is_vertical = matches!(direction, Direction::TopDown | Direction::BottomUp);
    for edge in &layout.edges {
        draw_edge(&mut grid, edge, is_vertical, theme);
    }

    fix_edge_junctions(&mut grid);

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
            is_edge: false,
        };
        row[x + w - 1] = Cell {
            ch: tr,
            style: border_style,
            is_edge: false,
        };
        for cell in row.iter_mut().take(x + w - 1).skip(x + 1) {
            *cell = Cell {
                ch: HLINE,
                style: border_style,
                is_edge: false,
            };
        }
    }

    let text_row = y + h / 2;
    if text_row < grid.len() && x + w <= grid[0].len() {
        let row = &mut grid[text_row];
        row[x] = Cell {
            ch: VLINE,
            style: border_style,
            is_edge: false,
        };
        row[x + w - 1] = Cell {
            ch: VLINE,
            style: border_style,
            is_edge: false,
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
                    is_edge: false,
                };
                cx += 1;
            }
        }
        for ch in &label_chars {
            if cx < x + w - 1 {
                row[cx] = Cell {
                    ch: *ch,
                    style: text_style,
                    is_edge: false,
                };
                cx += 1;
            }
        }
        while cx < x + w - 1 {
            row[cx] = Cell {
                ch: ' ',
                style: text_style,
                is_edge: false,
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
                is_edge: false,
            };
            row[x + w - 1] = Cell {
                ch: VLINE,
                style: border_style,
                is_edge: false,
            };
            for cell in row.iter_mut().take(x + w - 1).skip(x + 1) {
                *cell = Cell {
                    ch: ' ',
                    style: text_style,
                    is_edge: false,
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
            is_edge: false,
        };
        row[x + w - 1] = Cell {
            ch: br,
            style: border_style,
            is_edge: false,
        };
        for cell in row.iter_mut().take(x + w - 1).skip(x + 1) {
            *cell = Cell {
                ch: HLINE,
                style: border_style,
                is_edge: false,
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

    for i in 0..wp.len() - 1 {
        let (x1, y1) = wp[i];
        let (x2, y2) = wp[i + 1];

        if y1 == y2 {
            let (lo, hi) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
            for x in lo..=hi {
                set_if_empty(grid, x, y1, HLINE, edge_style);
            }
        } else if x1 == x2 {
            let (lo, hi) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
            for y in lo..=hi {
                set_if_empty(grid, x1, y, VLINE, edge_style);
            }
        }
    }

    if edge.edge_type == EdgeType::Arrow && wp.len() >= 2 {
        let &(sx, sy) = &wp[wp.len() - 2];
        let &(tx, ty) = wp.last().unwrap();
        let arrow_ch = if is_vertical {
            if ty > sy {
                '▼'
            } else {
                '▲'
            }
        } else if tx > sx {
            '►'
        } else {
            '◄'
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

fn set_if_empty(grid: &mut [Vec<Cell>], x: usize, y: usize, ch: char, style: Style) {
    if y < grid.len() && x < grid[0].len() {
        let cell = &mut grid[y][x];
        if cell.ch == ' ' {
            cell.ch = ch;
            cell.style = style;
            cell.is_edge = true;
        }
    }
}

fn force_set(grid: &mut [Vec<Cell>], x: usize, y: usize, ch: char, style: Style) {
    if y < grid.len() && x < grid[0].len() {
        grid[y][x] = Cell {
            ch,
            style,
            is_edge: true,
        };
    }
}

fn set_label_char(grid: &mut [Vec<Cell>], x: usize, y: usize, ch: char, style: Style) {
    if y < grid.len() && x < grid[0].len() {
        let cell = &mut grid[y][x];
        if cell.ch == ' ' || cell.is_edge {
            cell.ch = ch;
            cell.style = style;
        }
    }
}

fn is_neighbor_connector(ch: char) -> bool {
    matches!(
        ch,
        HLINE | VLINE | '┼' | TLC | TRC | BLC | BRC | '├' | '┤' | '┬' | '┴'
            | RTLC | RTRC | RBLC | RBRC
            | '▼' | '▲' | '►' | '◄'
            | '╌'
    )
}

fn fix_edge_junctions(grid: &mut [Vec<Cell>]) {
    let gh = grid.len();
    if gh == 0 {
        return;
    }
    let gw = grid[0].len();
    if gw == 0 {
        return;
    }

    for y in 0..gh {
        for x in 0..gw {
            if !grid[y][x].is_edge {
                continue;
            }
            let ch = grid[y][x].ch;
            if !matches!(ch, HLINE | VLINE) {
                continue;
            }

            let up = y > 0 && is_neighbor_connector(grid[y - 1][x].ch);
            let down = y + 1 < gh && is_neighbor_connector(grid[y + 1][x].ch);
            let left = x > 0 && is_neighbor_connector(grid[y][x - 1].ch);
            let right = x + 1 < gw && is_neighbor_connector(grid[y][x + 1].ch);

            let new_ch = pick_junction_char(up, down, left, right);
            if new_ch != ch {
                grid[y][x].ch = new_ch;
            }
        }
    }
}

fn pick_junction_char(up: bool, down: bool, left: bool, right: bool) -> char {
    match (up, down, left, right) {
        (true, true, true, true) => '┼',
        (true, true, true, false) => '├',
        (true, true, false, true) => '┤',
        (true, false, true, true) => '┴',
        (false, true, true, true) => '┬',
        (true, false, true, false) => BRC,
        (true, false, false, true) => BLC,
        (false, true, true, false) => TRC,
        (false, true, false, true) => TLC,
        (true, true, false, false) => VLINE,
        (false, false, true, true) => HLINE,
        _ => VLINE,
    }
}
