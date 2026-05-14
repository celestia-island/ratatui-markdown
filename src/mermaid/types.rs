use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    TopDown,
    BottomUp,
    LeftRight,
    RightLeft,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeShape {
    Rect,
    Rounded,
    Diamond,
    Circle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MermaidNode {
    pub id: String,
    pub label: String,
    pub shape: NodeShape,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    Arrow,
    Line,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MermaidEdge {
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MermaidDiagram {
    pub direction: Direction,
    pub nodes: Vec<MermaidNode>,
    pub edges: Vec<MermaidEdge>,
}

impl MermaidDiagram {
    pub fn ensure_node(
        nodes: &mut Vec<MermaidNode>,
        map: &mut HashMap<String, usize>,
        id: &str,
        label: Option<&str>,
        shape: Option<NodeShape>,
    ) {
        if map.contains_key(id) {
            if let (Some(lbl), Some(&idx)) = (label, map.get(id)) {
                if !lbl.is_empty() {
                    nodes[idx].label = lbl.to_string();
                }
            }
            if let (Some(s), Some(&idx)) = (shape, map.get(id)) {
                nodes[idx].shape = s;
            }
            return;
        }
        let idx = nodes.len();
        map.insert(id.to_string(), idx);
        nodes.push(MermaidNode {
            id: id.to_string(),
            label: label.unwrap_or(id).to_string(),
            shape: shape.unwrap_or(NodeShape::Rect),
        });
    }
}
