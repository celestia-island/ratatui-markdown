use std::collections::HashMap;

use unicode_width::UnicodeWidthStr;

use super::{graph::LayeredGraph, types::*};

fn label_display_width(label: &str) -> usize {
    label.lines().map(UnicodeWidthStr::width).max().unwrap_or(0)
}

fn label_line_count(label: &str) -> usize {
    let n = label.lines().count();
    if n == 0 {
        1
    } else {
        n
    }
}

// ── pixel-space types ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: String,
    pub label: String,
    pub shape: NodeShape,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone)]
pub struct LayoutEdge {
    pub label: Option<String>,
    pub edge_type: EdgeType,
    pub waypoints: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<LayoutEdge>,
    pub grid_width: usize,
    pub grid_height: usize,
}

// ── constants ──────────────────────────────────────────────────

const NODE_H_PADDING: usize = 2;
const MIN_NODE_WIDTH: usize = 6;
const NODE_V_HEIGHT: usize = 3; // vertical direction
const NODE_V_HEIGHT_LR: usize = 5; // horizontal direction
const H_SPACING: usize = 4;
const V_SPACING: usize = 3;
const MIN_GAP: usize = 3; // minimum cells between node borders (line + arrow + margin)

// ── public entry point ─────────────────────────────────────────

pub fn compute_layout(
    diagram: &MermaidDiagram,
    graph: &LayeredGraph,
    max_width: usize,
    max_height: Option<usize>,
) -> Layout {
    if diagram.nodes.is_empty() || graph.layers.is_empty() {
        return Layout {
            nodes: Vec::new(),
            edges: Vec::new(),
            grid_width: 0,
            grid_height: 0,
        };
    }

    let is_vertical = matches!(diagram.direction, Direction::TopDown | Direction::BottomUp);
    let node_v_height = if is_vertical {
        NODE_V_HEIGHT
    } else {
        NODE_V_HEIGHT_LR
    };

    let (h_spacing, v_spacing) =
        adapt_spacing(diagram, &graph.layers, node_v_height, max_width, max_height);

    let mut layout_nodes = Vec::new();
    let mut node_positions: HashMap<String, (usize, usize)> = HashMap::new();

    let mut y_offset = 0usize;
    for layer in &graph.layers {
        if layer.is_empty() {
            continue;
        }
        let node_count = layer.len();

        let mut node_widths: Vec<usize> = layer
            .iter()
            .map(|id| {
                let node = diagram
                    .nodes
                    .iter()
                    .find(|n| &n.id == id)
                    .expect("layer node must exist in diagram nodes");
                let text_w = if node.label.contains('\n') {
                    label_display_width(&node.label)
                } else {
                    unicode_width::UnicodeWidthStr::width(node.label.as_str())
                };
                (text_w + NODE_H_PADDING * 2).max(MIN_NODE_WIDTH)
            })
            .collect();

        let total_w: usize =
            node_widths.iter().sum::<usize>() + h_spacing * node_count.saturating_sub(1);
        let scale = scale_factor(total_w, max_width, &node_widths, h_spacing);

        if scale < 1.0 {
            for w in &mut node_widths {
                let scaled = (*w as f64 * scale) as usize;
                *w = scaled.max(MIN_NODE_WIDTH);
            }
        }

        let actual_total_w: usize =
            node_widths.iter().sum::<usize>() + h_spacing * node_count.saturating_sub(1);

        let x_start = if is_vertical {
            if actual_total_w < max_width {
                (max_width - actual_total_w) / 2
            } else {
                0
            }
        } else {
            0
        };

        let mut x = x_start;
        let mut layer_max_h = 0usize;
        for (i, id) in layer.iter().enumerate() {
            let node = diagram
                .nodes
                .iter()
                .find(|n| &n.id == id)
                .expect("layer node must exist in diagram nodes");
            let w = node_widths[i];
            let is_multiline = node.label.contains('\n');
            let h = if is_multiline {
                label_line_count(&node.label)
            } else {
                node_v_height
            };
            if h > layer_max_h {
                layer_max_h = h;
            }
            let label = if is_multiline {
                node.label.clone()
            } else {
                truncate_label(&node.label, w.saturating_sub(NODE_H_PADDING * 2))
            };

            let (nx, ny) = if is_vertical {
                (x, y_offset)
            } else {
                (y_offset, x)
            };

            layout_nodes.push(LayoutNode {
                id: id.clone(),
                label,
                shape: node.shape.clone(),
                x: nx,
                y: ny,
                width: w,
                height: h,
            });
            node_positions.insert(id.clone(), (nx + w / 2, ny + h / 2));
            x += w + h_spacing;
        }

        if is_vertical {
            y_offset += layer_max_h + v_spacing;
        } else {
            let max_w = node_widths.iter().copied().max().unwrap_or(0);
            y_offset += max_w + h_spacing;
        }
    }

    // edge paths
    let mut layout_edges = Vec::new();
    for edge in &diagram.edges {
        let waypoints = compute_edge_path(edge, &layout_nodes, &diagram.direction, v_spacing);
        layout_edges.push(LayoutEdge {
            label: edge.label.clone(),
            edge_type: edge.edge_type.clone(),
            waypoints,
        });
    }

    // grid dimensions
    let grid_w = layout_nodes
        .iter()
        .map(|n| n.x + n.width)
        .max()
        .unwrap_or(0);
    let grid_h = layout_nodes
        .iter()
        .map(|n| n.y + n.height)
        .max()
        .unwrap_or(0);

    Layout {
        nodes: layout_nodes,
        edges: layout_edges,
        grid_width: grid_w.min(max_width).max(1),
        grid_height: grid_h.max(1),
    }
}

// ── spacing adaptation ─────────────────────────────────────────

fn adapt_spacing(
    diagram: &MermaidDiagram,
    layers: &[Vec<String>],
    node_v_height: usize,
    max_width: usize,
    max_height: Option<usize>,
) -> (usize, usize) {
    let layer_count = layers.len().max(1);
    let max_layer_size = layers.iter().map(|l| l.len()).max().unwrap_or(1);

    let avg_node_w: usize = if diagram.nodes.is_empty() {
        MIN_NODE_WIDTH
    } else {
        diagram
            .nodes
            .iter()
            .map(|n| {
                let tw = if n.label.contains('\n') {
                    label_display_width(&n.label)
                } else {
                    unicode_width::UnicodeWidthStr::width(n.label.as_str())
                };
                (tw + NODE_H_PADDING * 2).max(MIN_NODE_WIDTH)
            })
            .sum::<usize>()
            / diagram.nodes.len()
    };

    let natural_w = avg_node_w * max_layer_size + H_SPACING * max_layer_size.saturating_sub(1);
    let natural_h = node_v_height * layer_count + V_SPACING * layer_count.saturating_sub(1);

    let mut hs = H_SPACING;
    let mut vs = V_SPACING;

    if natural_w > max_width && max_width > 0 {
        let needed = avg_node_w * max_layer_size;
        if needed < max_width {
            hs = (max_width - needed) / max_layer_size.saturating_sub(1).max(1);
            hs = hs.max(MIN_GAP);
        } else {
            hs = MIN_GAP;
        }
    } else {
        hs = hs.max(MIN_GAP);
    }

    if let Some(mh) = max_height {
        if natural_h > mh {
            let needed = node_v_height * layer_count;
            if needed < mh {
                vs = (mh - needed) / layer_count.saturating_sub(1).max(1);
                vs = vs.max(1);
            } else {
                vs = 1;
            }
        }
    }

    (hs, vs)
}

// ── scale factor ───────────────────────────────────────────────

fn scale_factor(total_w: usize, max_width: usize, node_widths: &[usize], h_spacing: usize) -> f64 {
    if total_w <= max_width || max_width == 0 {
        return 1.0;
    }
    let node_count = node_widths.len().max(1);
    let available = max_width.saturating_sub(h_spacing * node_count.saturating_sub(1));
    let min_total: usize = node_widths.len() * MIN_NODE_WIDTH;
    if available < min_total {
        return 1.0;
    }
    let current_total: usize = node_widths.iter().sum();
    if current_total == 0 {
        1.0
    } else {
        available as f64 / current_total as f64
    }
}

// ── edge path geometry ─────────────────────────────────────────

fn compute_edge_path(
    edge: &MermaidEdge,
    nodes: &[LayoutNode],
    direction: &Direction,
    v_spacing: usize,
) -> Vec<(usize, usize)> {
    let source = match nodes.iter().find(|n| n.id == edge.source) {
        Some(n) => n,
        None => return Vec::new(),
    };
    let target = match nodes.iter().find(|n| n.id == edge.target) {
        Some(n) => n,
        None => return Vec::new(),
    };

    let is_vertical = matches!(direction, Direction::TopDown | Direction::BottomUp);

    if is_vertical {
        let sx = source.x + source.width / 2;
        let sy = source.y + source.height;
        let tx = target.x + target.width / 2;
        let ty = target.y.saturating_sub(1); // stop one cell ABOVE target top border

        let mid_y = if ty > sy {
            (sy + ty) / 2
        } else {
            sy + v_spacing / 2
        };

        if sx == tx {
            vec![(sx, sy), (sx, mid_y), (tx, ty)]
        } else {
            vec![(sx, sy), (sx, mid_y), (tx, mid_y), (tx, ty)]
        }
    } else {
        let sx = source.x + source.width;
        let sy = source.y + source.height / 2;
        let tx = target.x.saturating_sub(1); // stop one cell LEFT of target left border
        let ty = target.y + target.height / 2;

        let mid_x = if tx > sx {
            (sx + tx) / 2
        } else {
            sx + v_spacing / 2
        };

        if sy == ty {
            vec![(sx, sy), (mid_x, sy), (tx, ty)]
        } else {
            vec![(sx, sy), (mid_x, sy), (mid_x, ty), (tx, ty)]
        }
    }
}

// ── label truncation ───────────────────────────────────────────

fn truncate_label(label: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let width = unicode_width::UnicodeWidthStr::width(label);
    if width <= max_chars {
        return label.to_string();
    }
    let mut result = String::new();
    let mut w = 0;
    for ch in label.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw > max_chars.saturating_sub(1) {
            break;
        }
        result.push(ch);
        w += cw;
    }
    if !result.is_empty() {
        result.push('…');
    }
    result
}
