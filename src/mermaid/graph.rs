use std::collections::{HashMap, HashSet};

use super::types::*;

/// Pure graph output: nodes grouped into topological layers.
/// No pixel coordinates — just node IDs per layer.
#[derive(Debug, Clone)]
pub struct LayeredGraph {
    pub layers: Vec<Vec<String>>,
}

/// Assign nodes to topological layers via BFS.
///
/// Nodes with in-degree 0 go to layer 0. Edges push targets to
/// `max(layer, parent_layer + 1)`. Unreachable nodes default to layer 0.
/// Empty nodes list → empty layers.
pub fn assign_layers(diagram: &MermaidDiagram) -> LayeredGraph {
    if diagram.nodes.is_empty() {
        return LayeredGraph { layers: Vec::new() };
    }

    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut layer_map: HashMap<&str, usize> = HashMap::new();

    for node in &diagram.nodes {
        in_degree.insert(&node.id, 0);
        adj.insert(&node.id, Vec::new());
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
                let parent_layer = *layer_map.get(id).unwrap_or(&0);
                let current = *layer_map.get(target).unwrap_or(&0);
                if parent_layer + 1 > current {
                    layer_map.insert(target, parent_layer + 1);
                }
                if let Some(deg) = in_degree.get_mut(target) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        queue.push(target);
                    }
                }
            }
        }
    }

    // catch isolated / unreachable nodes
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
        let l = *layer_map.get(&node.id[..]).unwrap_or(&0);
        layers[l].push(node.id.clone());
    }

    // stable ordering within each layer
    for layer in &mut layers {
        layer.sort();
    }

    // trim trailing empty layers
    while layers.last().is_some_and(|l| l.is_empty()) {
        layers.pop();
    }

    LayeredGraph { layers }
}
