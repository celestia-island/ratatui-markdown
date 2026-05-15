use std::collections::{HashMap, HashSet};

use super::types::*;

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

const NODE_H_PADDING: usize = 2;
const NODE_V_HEIGHT: usize = 3;
const H_SPACING: usize = 4;
const V_SPACING: usize = 3;
const MIN_NODE_WIDTH: usize = 6;

pub fn compute_layout(
    diagram: &MermaidDiagram,
    max_width: usize,
    max_height: Option<usize>,
) -> Layout {
    if diagram.nodes.is_empty() {
        return Layout {
            nodes: Vec::new(),
            edges: Vec::new(),
            grid_width: 0,
            grid_height: 0,
        };
    }

    let (h_spacing, v_spacing) = adapt_spacing(diagram, max_width, max_height);

    let layers = assign_layers(diagram);

    let mut layout_nodes = Vec::new();
    let mut node_positions: HashMap<String, (usize, usize)> = HashMap::new();

    let is_vertical = matches!(
        diagram.direction,
        Direction::TopDown | Direction::BottomUp
    );

    let mut y_offset = 0usize;
    for layer in &layers {
        let node_count = layer.len();

        let mut node_widths: Vec<usize> = layer
            .iter()
            .map(|id| {
                let node = diagram.nodes.iter().find(|n| &n.id == id)
                    .expect("layer node must exist in diagram nodes");
                let text_w = unicode_width::UnicodeWidthStr::width(node.label.as_str());
                (text_w + NODE_H_PADDING * 2).max(MIN_NODE_WIDTH)
            })
            .collect();

        let total_w: usize = node_widths.iter().sum::<usize>() + h_spacing * (node_count - 1);
        let scale = if total_w > max_width && max_width > 0 {
            let available = max_width.saturating_sub(h_spacing * (node_count - 1));
            let min_total: usize = node_widths.len() * MIN_NODE_WIDTH;
            if available >= min_total {
                let current_total: usize = node_widths.iter().sum();
                if current_total > 0 {
                    available as f64 / current_total as f64
                } else {
                    1.0
                }
            } else {
                1.0
            }
        } else {
            1.0
        };

        if scale < 1.0 {
            for w in &mut node_widths {
                let scaled = (*w as f64 * scale) as usize;
                *w = scaled.max(MIN_NODE_WIDTH);
            }
        }

        let actual_total_w: usize = node_widths.iter().sum::<usize>() + h_spacing * (node_count.saturating_sub(1));

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
        for (i, id) in layer.iter().enumerate() {
            let node = diagram.nodes.iter().find(|n| &n.id == id)
                .expect("layer node must exist in diagram nodes");
            let w = node_widths[i];

            let (nx, ny) = if is_vertical {
                (x, y_offset)
            } else {
                (y_offset, x)
            };

            layout_nodes.push(LayoutNode {
                id: id.clone(),
                label: truncate_label(&node.label, w.saturating_sub(NODE_H_PADDING * 2)),
                shape: node.shape.clone(),
                x: nx,
                y: ny,
                width: w,
                height: NODE_V_HEIGHT,
            });
            node_positions.insert(id.clone(), (nx + w / 2, ny + NODE_V_HEIGHT / 2));
            x += w + h_spacing;
        }

        y_offset += NODE_V_HEIGHT + v_spacing;
    }

    let mut layout_edges = Vec::new();
    for edge in &diagram.edges {
        let waypoints = compute_edge_path(
            edge,
            &layout_nodes,
            &diagram.direction,
            v_spacing,
            h_spacing,
        );
        layout_edges.push(LayoutEdge {
            label: edge.label.clone(),
            edge_type: edge.edge_type.clone(),
            waypoints,
        });
    }

    let grid_w = layout_nodes.iter().map(|n| n.x + n.width).max().unwrap_or(0);
    let grid_h = layout_nodes
        .iter()
        .map(|n| n.y + n.height)
        .max()
        .unwrap_or(0);

    let grid_w = grid_w.min(max_width);

    Layout {
        nodes: layout_nodes,
        edges: layout_edges,
        grid_width: grid_w.max(1),
        grid_height: grid_h.max(1),
    }
}

fn adapt_spacing(
    diagram: &MermaidDiagram,
    max_width: usize,
    max_height: Option<usize>,
) -> (usize, usize) {
    let layers = assign_layers(diagram);
    let layer_count = layers.len().max(1);
    let max_layer_size = layers.iter().map(|l| l.len()).max().unwrap_or(1);

    let avg_node_w: usize = if diagram.nodes.is_empty() {
        MIN_NODE_WIDTH
    } else {
        diagram
            .nodes
            .iter()
            .map(|n| {
                let tw = unicode_width::UnicodeWidthStr::width(n.label.as_str());
                (tw + NODE_H_PADDING * 2).max(MIN_NODE_WIDTH)
            })
            .sum::<usize>()
            / diagram.nodes.len()
    };

    let natural_w = avg_node_w * max_layer_size + H_SPACING * (max_layer_size - 1);
    let natural_h = NODE_V_HEIGHT * layer_count + V_SPACING * (layer_count - 1);

    let mut hs = H_SPACING;
    let mut vs = V_SPACING;

    if natural_w > max_width && max_width > 0 {
        let needed = avg_node_w * max_layer_size;
        if needed < max_width {
            hs = (max_width - needed) / max_layer_size.saturating_sub(1).max(1);
            hs = hs.max(1);
        } else {
            hs = 1;
        }
    }

    if let Some(mh) = max_height {
        if natural_h > mh {
            let needed = NODE_V_HEIGHT * layer_count;
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

fn assign_layers(diagram: &MermaidDiagram) -> Vec<Vec<String>> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut reverse_adj: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut layer_map: HashMap<&str, usize> = HashMap::new();

    for node in &diagram.nodes {
        in_degree.insert(&node.id, 0);
        adj.insert(&node.id, Vec::new());
        reverse_adj.insert(&node.id, Vec::new());
        layer_map.insert(&node.id, 0);
    }

    let mut seen_edges: HashSet<(&str, &str)> = HashSet::new();
    for edge in &diagram.edges {
        if seen_edges.contains(&(&edge.source, &edge.target)) {
            continue;
        }
        seen_edges.insert((&edge.source, &edge.target));
        if let Some(deg) = in_degree.get_mut(edge.target.as_str()) {
            *deg += 1;
        }
        if let Some(neighbors) = adj.get_mut(edge.source.as_str()) {
            neighbors.push(&edge.target);
        }
        if let Some(ra) = reverse_adj.get_mut(edge.target.as_str()) {
            ra.push(&edge.source);
        }
    }

    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut processed = 0usize;
    while let Some(id) = queue.pop() {
        processed += 1;
        if let Some(neighbors) = adj.get(id) {
            for &target in neighbors {
                let parent_layer = layer_map[id];
                let current = layer_map[target];
                if parent_layer + 1 > current {
                    layer_map.insert(target, parent_layer + 1);
                }
                if let Some(deg) = in_degree.get_mut(target) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(target);
                    }
                }
            }
        }
    }

    if processed < diagram.nodes.len() {
        for node in &diagram.nodes {
            if !layer_map.contains_key(node.id.as_str()) {
                layer_map.insert(&node.id, 0);
            }
        }
    }

    let max_layer = layer_map.values().copied().max().unwrap_or(0);
    let mut layers: Vec<Vec<String>> = vec![Vec::new(); max_layer + 1];
    for node in &diagram.nodes {
        let l = layer_map[&node.id[..]];
        layers[l].push(node.id.clone());
    }

    for layer in &mut layers {
        layer.sort();
    }

    layers
}

fn compute_edge_path(
    edge: &MermaidEdge,
    nodes: &[LayoutNode],
    direction: &Direction,
    v_spacing: usize,
    _h_spacing: usize,
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
        let ty = target.y;

        let mid_y = sy + v_spacing / 2;

        if sx == tx {
            vec![(sx, sy), (sx, mid_y), (tx, ty)]
        } else {
            vec![(sx, sy), (sx, mid_y), (tx, mid_y), (tx, ty)]
        }
    } else {
        let sx = source.x + source.width;
        let sy = source.y + source.height / 2;
        let tx = target.x;
        let ty = target.y + target.height / 2;

        let mid_x = sx + v_spacing / 2;

        if sy == ty {
            vec![(sx, sy), (mid_x, sy), (tx, ty)]
        } else {
            vec![(sx, sy), (mid_x, sy), (mid_x, ty), (tx, ty)]
        }
    }
}

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
