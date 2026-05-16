use std::collections::HashSet;

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};
use unicode_width::UnicodeWidthChar;

use super::{
    layout::{Layout, LayoutEdge, LayoutNode},
    types::{Direction, EdgeType, NodeShape},
};
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

    // Phase 1: draw nodes (unchanged)
    for node in &layout.nodes {
        draw_node(&mut grid, node, theme);
    }

    // Phase 2: draw edges — global accumulation → global resolution
    let is_vertical = matches!(direction, Direction::TopDown | Direction::BottomUp);
    draw_all_edges(&mut grid, &layout.edges, is_vertical, theme);

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

    if node.label.contains('\n') {
        draw_multiline_node(grid, node, theme);
        return;
    }

    let (tl, tr, bl, br) = match node.shape {
        NodeShape::Rounded | NodeShape::Circle | NodeShape::Diamond => (RTLC, RTRC, RBLC, RBRC),
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
                cx += ch.width().unwrap_or(1);
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

fn draw_multiline_node(grid: &mut [Vec<Cell>], node: &LayoutNode, theme: &impl RichTextTheme) {
    let x = node.x;
    let y = node.y;
    let w = node.width;

    let border_style = Style::default().fg(theme.get_muted_text_color());
    let grid_w = if !grid.is_empty() {
        grid[0].len()
    } else {
        return;
    };

    for (row_idx, line) in node.label.lines().enumerate() {
        let ry = y + row_idx;
        if ry >= grid.len() {
            break;
        }
        if x >= grid_w {
            break;
        }
        let row = &mut grid[ry];
        let mut cx = x;
        for ch in line.chars() {
            if cx >= x + w || cx >= grid_w {
                break;
            }
            let cw = ch.width().unwrap_or(1);
            if cx + cw > x + w {
                break;
            }
            let style = if is_box_drawing_char(ch) {
                border_style
            } else {
                Style::default().fg(theme.get_text_color())
            };
            row[cx] = Cell {
                ch,
                style,
                is_edge: false,
            };
            cx += cw;
        }
    }
}

fn is_box_drawing_char(ch: char) -> bool {
    matches!(
        ch,
        '─' | '│'
            | '┌'
            | '┐'
            | '└'
            | '┘'
            | '├'
            | '┤'
            | '┬'
            | '┴'
            | '┼'
            | '╭'
            | '╮'
            | '╰'
            | '╯'
            | '═'
            | '║'
            | '╔'
            | '╗'
            | '╚'
            | '╝'
            | '╠'
            | '╣'
            | '╦'
            | '╩'
            | '╬'
    )
}

// ── Edge drawing: global-accumulation pipeline ──────────────────────

/// Two-pass edge renderer that prevents per-edge character overwrites at
/// shared junction cells.
///
/// **Pass A** — Rasterizes every edge's waypoint-path cells into a **single
/// shared** `HashSet`.  Junction cells where 3+ edges meet can now see the
/// complete multi-edge neighbourhood.
///
/// **Pass B** — Stray cleanup (Bresenham diagonal-artefact cells with zero
/// orthogonal neighbours anywhere in the global set).
///
/// **Pass C** — Resolves each cell **once**, computing `(up,down,left,right)`
/// connectivity against the shared set.  A cross or T-junction rendered by
/// pass C can never be corrupted by a later edge.
///
/// Arrows and labels follow in passes D/E.
fn draw_all_edges(
    grid: &mut [Vec<Cell>],
    edges: &[LayoutEdge],
    is_vertical: bool,
    theme: &impl RichTextTheme,
) {
    if edges.is_empty() || grid.is_empty() {
        return;
    }
    let gh = grid.len();
    let gw = grid[0].len();

    let edge_style = Style::default().fg(theme.get_secondary_color());
    let arrow_style = Style::default()
        .fg(theme.get_primary_color())
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default()
        .fg(theme.get_info_color())
        .add_modifier(Modifier::ITALIC);

    // ════════════════════════════════════════════════════════════
    //  Pass A: Rasterize every edge into a SHARED global set
    // ════════════════════════════════════════════════════════════
    let mut global_cells: HashSet<(usize, usize)> = HashSet::new();

    struct EdgeMeta {
        waypoints: Vec<(usize, usize)>,
        has_arrow: bool,
        label: Option<String>,
    }
    let mut meta: Vec<EdgeMeta> = Vec::with_capacity(edges.len());

    for edge in edges {
        let wp = &edge.waypoints;
        if wp.len() < 2 {
            meta.push(EdgeMeta {
                waypoints: wp.clone(),
                has_arrow: false,
                label: None,
            });
            continue;
        }

        meta.push(EdgeMeta {
            waypoints: wp.clone(),
            has_arrow: edge.edge_type == EdgeType::Arrow,
            label: edge.label.clone(),
        });

        for i in 0..wp.len().saturating_sub(1) {
            let (x1, y1) = wp[i];
            let (x2, y2) = wp[i + 1];
            if x1 == x2 || y1 == y2 {
                rasterize_segment(&mut global_cells, x1, y1, x2, y2);
            } else {
                rasterize_segment(&mut global_cells, x1, y1, x1, y2);
                rasterize_segment(&mut global_cells, x1, y2, x2, y2);
            }
        }

        // Waypoints are always present (half-open segment ranges exclude ends)
        for &pt in wp {
            global_cells.insert(pt);
        }
    }

    // ════════════════════════════════════════════════════════════
    //  Pass B: Stray cleanup (Bresenham artefacts)
    // ════════════════════════════════════════════════════════════
    let stray: Vec<(usize, usize)> = global_cells
        .iter()
        .copied()
        .filter(|&(cx, cy)| {
            let has_ortho = (cy > 0 && global_cells.contains(&(cx, cy.saturating_sub(1))))
                || (cy + 1 < gh && global_cells.contains(&(cx, cy + 1)))
                || (cx > 0 && global_cells.contains(&(cx.saturating_sub(1), cy)))
                || (cx + 1 < gw && global_cells.contains(&(cx + 1, cy)));
            !has_ortho
        })
        .collect();
    for s in stray {
        global_cells.remove(&s);
    }

    global_cells.retain(|&(cx, cy)| {
        if cy >= gh || cx >= gw {
            return false;
        }
        grid[cy][cx].is_edge || grid[cy][cx].ch == ' '
    });

    // ════════════════════════════════════════════════════════════
    //  Pass C: Global resolution — one cell, one char, correct
    // ════════════════════════════════════════════════════════════
    for &(cx, cy) in &global_cells {
        if cy >= gh || cx >= gw {
            continue;
        }
        if !grid[cy][cx].is_edge && grid[cy][cx].ch != ' ' {
            continue;
        }

        let up = cy > 0 && global_cells.contains(&(cx, cy.saturating_sub(1)));
        let down = cy + 1 < gh && global_cells.contains(&(cx, cy + 1));
        let left = cx > 0 && global_cells.contains(&(cx.saturating_sub(1), cy));
        let right = cx + 1 < gw && global_cells.contains(&(cx + 1, cy));

        let ch = resolve_edge_char(up, down, left, right);
        grid[cy][cx] = Cell {
            ch,
            style: edge_style,
            is_edge: true,
        };
    }

    // ════════════════════════════════════════════════════════════
    //  Pass D: Arrows (overwrite edge cells at terminal waypoints)
    // ════════════════════════════════════════════════════════════
    for m in &meta {
        if !m.has_arrow || m.waypoints.len() < 2 {
            continue;
        }
        let wp = &m.waypoints;
        let last = wp[wp.len() - 1];
        let prev = wp[wp.len() - 2];
        let arrow_ch = if is_vertical {
            if last.1 > prev.1 {
                ARROW_DOWN
            } else {
                ARROW_UP
            }
        } else if last.0 > prev.0 {
            ARROW_RIGHT
        } else {
            ARROW_LEFT
        };
        if last.1 < gh && last.0 < gw {
            grid[last.1][last.0] = Cell {
                ch: arrow_ch,
                style: arrow_style,
                is_edge: true,
            };
        }
    }

    // ════════════════════════════════════════════════════════════
    //  Pass E: Labels (near mid-point of each edge path)
    // ════════════════════════════════════════════════════════════
    for m in &meta {
        if let Some(ref label) = m.label {
            let wp = &m.waypoints;
            if wp.len() >= 2 {
                let mid = wp.len() / 2;
                let (mx, my) = wp[mid];
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
}

/// Walk the grid cells touched by the segment (x1,y1)→(x2,y2).
///
/// Axis-aligned segments use **half-open** ranges `[start, end)` so that
/// a shared corner waypoint receives axial neighbours from **only one**
/// adjacent segment, yielding correct 2-way corner characters (└┐┌┘)
/// instead of spurious 3-way T-junctions (├┤┬┴).
///
/// Diagonal segments use Bresenham linear interpolation; stray cells
/// are cleaned up by the caller (global pass B).
fn rasterize_segment(
    cells: &mut HashSet<(usize, usize)>,
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
) {
    if x1 == x2 && y1 == y2 {
        cells.insert((x1, y1));
        return;
    }

    // Pure horizontal: half-open [x1, x2) — excludes x2 (next segment start)
    if y1 == y2 {
        let (lo, hi) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        for x in lo..hi {
            cells.insert((x, y1));
        }
        return;
    }

    // Pure vertical: half-open [y1, y2) — excludes y2
    if x1 == x2 {
        let (lo, hi) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        for y in lo..hi {
            cells.insert((x1, y));
        }
        return;
    }

    // Diagonal: Bresenham walk (strays cleaned up after global merge)
    let dx = x2.abs_diff(x1);
    let dy = y2.abs_diff(y1);
    let steps = dx.max(dy);
    for i in 0..=steps {
        let t = if steps > 0 {
            i as f64 / steps as f64
        } else {
            0.0
        };
        let x = x1 as f64 + (x2 as f64 - x1 as f64) * t;
        let y = y1 as f64 + (y2 as f64 - y1 as f64) * t;
        cells.insert((x.round() as usize, y.round() as usize));
    }
}

// ── Box-drawing named constants (T/C junctions) ───────────────────
#[allow(dead_code)]
const TEE_UP: char = '┴'; // U+2534  connects up+left+right (no down stem)
const TEE_DOWN: char = '┬'; // U+252C  connects down+left+right (no up stem)
#[allow(dead_code)]
const TEE_LEFT: char = '┤'; // U+2524  tee pointing left (reserved for RightLeft)
const TEE_RIGHT: char = '├'; // U+251C  tee pointing right (up+down+left)
const CROSS: char = '┼'; // U+253C  four-way junction
const ARROW_DOWN: char = '▼';
const ARROW_UP: char = '▲';
const ARROW_RIGHT: char = '►';
const ARROW_LEFT: char = '◄';

/// Complete 16-entry truth table: (up, down, left, right) → box-drawing
/// character.  Every combination is mapped explicitly — no hidden fallback
/// can quietly render a wrong char.
#[rustfmt::skip]
fn resolve_edge_char(up: bool, down: bool, left: bool, right: bool) -> char {
    match (up, down, left, right) {
        // ── 4-way ──────────────────────────────────────────────
        (true,  true,  true,  true ) => CROSS,

        // ── 3-way T-junctions ──────────────────────────────────
        // Stem always points in the diagram's primary axis direction
        // (down for TD, right for LR) so that merge points visually
        // flow toward their target, not away from it.
        (true,  true,  true,  false) => TEE_LEFT,
        (true,  true,  false, true ) => TEE_RIGHT,
        (true,  false, true,  true ) => TEE_UP,
        (false, true,  true,  true ) => TEE_DOWN,

        // ── 2-way straight (mid-segment) ──────────────────────
        (true,  true,  false, false) => VLINE,
        (false, false, true,  true ) => HLINE,

        // ── 2-way corners ─────────────────────────────────────
        (true,  false, true,  false) => BRC,   // up+left  → ┘
        (true,  false, false, true ) => BLC,   // up+right → └
        (false, true,  true,  false) => TRC,   // down+left→ ┐
        (false, true,  false, true ) => TLC,   // down+right→┌

        // ── 1-way straight (dead-end / endpoint) ──────────────
        (true,  false, false, false) |
        (false, true,  false, false) => VLINE,
        (false, false, true,  false) |
        (false, false, false, true ) => HLINE,

        // ── Isolated (Bresenham cleanup fallback) ─────────────
        (false, false, false, false) => HLINE,
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
