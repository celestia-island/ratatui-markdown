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

#[derive(Debug, Clone, PartialEq)]
pub struct SequenceDiagram {
    pub participants: Vec<String>,
    pub messages: Vec<SequenceMessage>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SequenceMessage {
    pub from: String,
    pub to: String,
    pub text: String,
    pub arrow_kind: SeqArrowKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeqArrowKind {
    Solid,
    Dotted,
    SolidOpen,
    DottedOpen,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PieChart {
    pub title: Option<String>,
    pub slices: Vec<(String, f64)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GanttChart {
    pub title: Option<String>,
    pub sections: Vec<GanttSection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GanttSection {
    pub name: String,
    pub tasks: Vec<GanttTask>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GanttTask {
    pub name: String,
    pub id: Option<String>,
    pub deps: Vec<String>,
    pub duration: Option<String>,
}
