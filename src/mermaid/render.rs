use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::layout::{Layout, LayoutEdge, LayoutNode};
use super::types::{Direction, EdgeType, NodeShape};
use crate::theme::RichTextTheme;
use unicode_width::UnicodeWidthChar;
use std::collections::HashSet;

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

    // Phase 1: draw nodes (unchanged)
    for node in &layout.nodes {
        draw_node(&mut grid, node, theme);
    }

    // Phase 2: draw edges via connectivity-aware rasterization
    let is_vertical = matches!(direction, Direction::TopDown | Direction::BottomUp);
    for edge in &layout.edges {
        draw_edge_connectivity(&mut grid, edge, is_vertical, theme);
    }

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

// ── Node drawing (unchanged) ───────────────────────────────────────

fn draw_node(grid: &mut [Vec<Cell>], node: &LayoutNode, theme: &impl RichTextTheme) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    let (tl, tr, bl, br) = match node.shape {
        NodeShape::Rounded | NodeShape::Circle | NodeShape::Diamond => (RTLC, RTRC, RBLC, RBRC),
        NodeShape::Rect => (TLC, TRC, BLC, BRC),
    };

    let border_style = Style::default().fg(theme.get_muted_text_color());
    let text_style = Style::default().fg(theme.get_text_color());

    // top border
    if y < grid.len() && x + w <= grid[0].len() {
        let row = &mut grid[y];
        row[x] = Cell { ch: tl, style: border_style, is_edge: false };
        row[x + w - 1] = Cell { ch: tr, style: border_style, is_edge: false };
        for cell in row.iter_mut().take(x + w - 1).skip(x + 1) {
            *cell = Cell { ch: HLINE, style: border_style, is_edge: false };
        }
    }

    // text row
    let text_row = y + h / 2;
    if text_row < grid.len() && x + w <= grid[0].len() {
        let row = &mut grid[text_row];
        row[x] = Cell { ch: VLINE, style: border_style, is_edge: false };
        row[x + w - 1] = Cell { ch: VLINE, style: border_style, is_edge: false };
        let inner_w = w.saturating_sub(2);
        let label_chars: Vec<char> = node.label.chars().collect();
        let label_w = unicode_width::UnicodeWidthStr::width(node.label.as_str());
        let pad = if label_w < inner_w { (inner_w - label_w) / 2 } else { 0 };
        let mut cx = x + 1;
        for _ in 0..pad {
            if cx < x + w - 1 {
                row[cx] = Cell { ch: ' ', style: text_style, is_edge: false };
                cx += 1;
            }
        }
        for ch in &label_chars {
            if cx < x + w - 1 {
                row[cx] = Cell { ch: *ch, style: text_style, is_edge: false };
                cx += ch.width().unwrap_or(1);
            }
        }
        while cx < x + w - 1 {
            row[cx] = Cell { ch: ' ', style: text_style, is_edge: false };
            cx += 1;
        }
    }

    // side borders (non-text rows)
    for vy in (y + 1)..(y + h - 1) {
        if vy == text_row {
            continue;
        }
        if vy < grid.len() && x + w <= grid[0].len() {
            let row = &mut grid[vy];
            row[x] = Cell { ch: VLINE, style: border_style, is_edge: false };
            row[x + w - 1] = Cell { ch: VLINE, style: border_style, is_edge: false };
            for cell in row.iter_mut().take(x + w - 1).skip(x + 1) {
                *cell = Cell { ch: ' ', style: text_style, is_edge: false };
            }
        }
    }

    // bottom border
    let bottom_row = y + h - 1;
    if bottom_row < grid.len() && x + w <= grid[0].len() {
        let row = &mut grid[bottom_row];
        row[x] = Cell { ch: bl, style: border_style, is_edge: false };
        row[x + w - 1] = Cell { ch: br, style: border_style, is_edge: false };
        for cell in row.iter_mut().take(x + w - 1).skip(x + 1) {
            *cell = Cell { ch: HLINE, style: border_style, is_edge: false };
        }
    }
}

// ── Edge drawing: connectivity-based rasterization ───────────────

/// Draw an edge by rasterizing its path into cells, then resolving each
/// cell's character from its local connectivity pattern.
///
/// This replaces the old "draw segments → post-process junctions" approach.
/// Every cell knows exactly which neighbors are on the same path, so corners,
/// tees, and crosses are correct by construction — no fix-up pass needed.
fn draw_edge_connectivity(
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

    // Step 1: rasterize all segments → set of occupied cells
    let mut path_cells: HashSet<(usize, usize)> = HashSet::new();

    for i in 0..wp.len().saturating_sub(1) {
        let (x1, y1) = wp[i];
        let (x2, y2) = wp[i + 1];
        rasterize_segment(&mut path_cells, x1, y1, x2, y2);
    }

    // Step 1b: remove isolated cells (no orthogonal neighbor on the path).
    // Bresenham diagonal walks can produce cells that are only diagonally
    // adjacent to the main path — these would render as garbage.
    let gh = grid.len();
    let gw = grid[0].len();
    let isolated: Vec<(usize, usize)> = path_cells
        .iter()
        .filter(|&&(cx, cy)| {
            let has_neighbor =
                (cy > 0 && path_cells.contains(&(cx, cy - 1)))
                    || (cy + 1 < gh && path_cells.contains(&(cx, cy + 1)))
                    || (cx > 0 && path_cells.contains(&(cx - 1, cy)))
                    || (cx + 1 < gw && path_cells.contains(&(cx + 1, cy)));
            !has_neighbor
        })
        .copied()
        .collect();
    for cell in isolated {
        path_cells.remove(&cell);
    }

    // Step 2: resolve each cell's character from its neighbor connectivity
    let gh = grid.len();

    for &(cx, cy) in &path_cells {
        if cy >= gh || cx >= gw {
            continue;
        }
        // skip cells already occupied by non-edge content (node borders)
        if !grid[cy][cx].is_edge && grid[cy][cx].ch != ' ' {
            continue;
        }

        let up = cy > 0 && path_cells.contains(&(cx, cy - 1));
        let down = cy + 1 < gh && path_cells.contains(&(cx, cy + 1));
        let left = cx > 0 && path_cells.contains(&(cx - 1, cy));
        let right = cx + 1 < gw && path_cells.contains(&(cx + 1, cy));

        let ch = pick_line_char(up, down, left, right);
        grid[cy][cx] = Cell { ch, style: edge_style, is_edge: true };
    }

    // Step 3: place arrow at last waypoint (overwrites the line char there)
    if edge.edge_type == EdgeType::Arrow && wp.len() >= 2 {
        let last_idx = wp.len().saturating_sub(1);
        let &(ax, ay) = &wp[last_idx];
        let &(prev_x, prev_y) = &wp[wp.len() - 2];
        let arrow_ch = if is_vertical {
            if ay > prev_y { '▼' } else { '▲' }
        } else if ax > prev_x {
            '►'
        } else {
            '◄'
        };
        let arrow_style = Style::default()
            .fg(theme.get_primary_color())
            .add_modifier(Modifier::BOLD);
        if ay < gh && ax < gw {
            grid[ay][ax] = Cell { ch: arrow_ch, style: arrow_style, is_edge: true };
        }
    }

    // Step 4: place label near middle of path
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
                place_label_char(grid, cx, ly, ch, label_style);
                cx += cw;
            }
        }
    }
}

/// Walk from (x1,y1) to (x2,y2), collecting every grid cell touched.
///
/// Axis-aligned segments use range filling (guarantees orthogonal connectivity).
/// Diagonal segments use Bresenham walk; diagonal chars (╱╲ at ~26.6°
/// for 1:2 terminal aspect ratio, i.e. arctan(1/2)) are reserved for
/// future use — currently diagonal cells fall back to HLINE/VLINE.
fn rasterize_segment(cells: &mut HashSet<(usize, usize)>, x1: usize, y1: usize, x2: usize, y2: usize) {
    if x1 == x2 && y1 == y2 {
        cells.insert((x1, y1));
        return;
    }

    // Pure horizontal: range fill — every cell has left/right neighbors
    if y1 == y2 {
        let (lo, hi) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        for x in lo..=hi {
            cells.insert((x, y1));
        }
        return;
    }

    // Pure vertical: range fill — every cell has up/down neighbors
    if x1 == x2 {
        let (lo, hi) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        for y in lo..=hi {
            cells.insert((x1, y));
        }
        return;
    }

    // Genuine diagonal: Bresenham walk
    // Future: replace intermediate cells with ╱/╲ when diagonal rendering
    // is enabled. For now these cells get resolved by pick_line_char based on
    // whatever orthogonal neighbors they happen to have.
    let dx = x2.abs_diff(x1);
    let dy = y2.abs_diff(y1);
    let steps = dx.max(dy);
    for i in 0..=steps {
        let t = if steps > 0 { i as f64 / steps as f64 } else { 0.0 };
        let x = x1 as f64 + (x2 as f64 - x1 as f64) * t;
        let y = y1 as f64 + (y2 as f64 - y1 as f64) * t;
        cells.insert((x.round() as usize, y.round() as usize));
    }
}

/// Pick the correct box-drawing character for a cell given its four-way
/// connectivity to neighboring path cells.
///
/// This table covers all 16 combinations of (up,down,left,right).
/// Only patterns that actually occur in Manhattan paths are listed;
/// fallbacks handle rare cases gracefully.
fn pick_line_char(up: bool, down: bool, left: bool, right: bool) -> char {
    match (up, down, left, right) {
        // cross
        (true, true, true, true) => '┼',

        // T-junctions
        (true, true, true, false) => '├',
        (true, true, false, true) => '┤',
        (true, false, true, true) => '┴',
        (false, true, true, true) => '┬',

        // corners
        (true, false, true, false) => BRC,
        (true, false, false, true) => BLC,
        (false, true, true, false) => TRC,
        (false, true, false, true) => TLC,

        // straight lines
        (true, false, false, false) |
        (false, true, false, false) => VLINE,
        (false, false, true, false) |
        (false, false, false, true) => HLINE,

        // isolated / fallback (should not happen after cleanup pass)
        _ => HLINE,
    }
}

/// Place a label character; overwrites edge cells but never node borders.
fn place_label_char(grid: &mut [Vec<Cell>], x: usize, y: usize, ch: char, style: Style) {
    if y < grid.len() && x < grid[0].len() {
        let cell = &mut grid[y][x];
        if cell.ch == ' ' || cell.is_edge {
            cell.ch = ch;
            cell.style = style;
        }
    }
}
