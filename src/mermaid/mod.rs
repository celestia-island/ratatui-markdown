mod gantt;
mod layout;
mod parser;
mod pie;
mod render;
mod sequence;
mod types;

pub use types::{
    Direction, EdgeType, GanttChart, GanttSection, GanttTask, MermaidDiagram, MermaidEdge,
    MermaidNode, NodeShape, PieChart, SeqArrowKind, SequenceDiagram, SequenceMessage,
};

use ratatui::text::Line;
use crate::theme::RichTextTheme;

pub fn render_mermaid(
    source: &str,
    max_width: usize,
    max_height: Option<usize>,
    theme: &impl RichTextTheme,
) -> Option<Vec<Line<'static>>> {
    let first_line = source.lines().next()?.trim();

    if first_line.starts_with("graph ") || first_line.starts_with("flowchart ") {
        render_flowchart(source, max_width, max_height, theme)
    } else if first_line == "sequenceDiagram" || first_line.starts_with("sequenceDiagram") {
        render_sequence_diagram(source, max_width, theme)
    } else if first_line.starts_with("pie") {
        render_pie_chart(source, max_width, theme)
    } else if first_line == "gantt" {
        render_gantt_chart(source, max_width, theme)
    } else if first_line.starts_with("stateDiagram") {
        render_state_diagram(source, max_width, max_height, theme)
    } else {
        render_flowchart(source, max_width, max_height, theme)
    }
}

fn render_flowchart(
    source: &str,
    max_width: usize,
    max_height: Option<usize>,
    theme: &impl RichTextTheme,
) -> Option<Vec<Line<'static>>> {
    let diagram = parser::parse(source).ok()?;
    let direction = diagram.direction.clone();
    let layout = layout::compute_layout(&diagram, max_width, max_height);
    let lines = render::render_layout(&layout, &direction, theme);
    Some(lines)
}

fn render_sequence_diagram(
    source: &str,
    max_width: usize,
    theme: &impl RichTextTheme,
) -> Option<Vec<Line<'static>>> {
    let diagram = sequence::parse_sequence(source)?;
    Some(sequence::render_sequence(&diagram, max_width, theme))
}

fn render_pie_chart(
    source: &str,
    max_width: usize,
    theme: &impl RichTextTheme,
) -> Option<Vec<Line<'static>>> {
    let chart = pie::parse_pie(source)?;
    Some(pie::render_pie(&chart, max_width, theme))
}

fn render_gantt_chart(
    source: &str,
    max_width: usize,
    theme: &impl RichTextTheme,
) -> Option<Vec<Line<'static>>> {
    let chart = gantt::parse_gantt(source)?;
    Some(gantt::render_gantt(&chart, max_width, theme))
}

fn render_state_diagram(
    source: &str,
    max_width: usize,
    max_height: Option<usize>,
    theme: &impl RichTextTheme,
) -> Option<Vec<Line<'static>>> {
    let diagram = parse_state_diagram(source)?;
    let direction = diagram.direction.clone();
    let layout = layout::compute_layout(&diagram, max_width, max_height);
    let lines = render::render_layout(&layout, &direction, theme);
    Some(lines)
}

fn parse_state_diagram(source: &str) -> Option<MermaidDiagram> {
    use std::collections::HashSet;
    use types::{EdgeType, MermaidEdge, MermaidNode, NodeShape};

    let mut nodes: Vec<MermaidNode> = Vec::new();
    let mut edges: Vec<MermaidEdge> = Vec::new();
    let mut node_set: HashSet<String> = HashSet::new();

    for line in source.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with("stateDiagram")
            || line.starts_with("state ")
            || line.starts_with("note ")
        {
            continue;
        }

        let arrow = line
            .find("-->")
            .map(|idx| (idx, "-->"))
            .or_else(|| line.find("---").map(|idx| (idx, "---")));

        if let Some((arrow_pos, arrow_str)) = arrow {
            let from_raw = line[..arrow_pos].trim();
            let to_raw = line[arrow_pos + arrow_str.len()..].trim();

            if from_raw.is_empty() || to_raw.is_empty() {
                continue;
            }

            let label_text = None;

            let (from_id, from_label, from_shape) = if from_raw == "[*]" {
                (
                    "__start__".to_string(),
                    "●".to_string(),
                    NodeShape::Circle,
                )
            } else {
                (
                    from_raw.to_string(),
                    from_raw.to_string(),
                    NodeShape::Rounded,
                )
            };

            let (to_id, to_label, to_shape) = if to_raw == "[*]" {
                (
                    "__end__".to_string(),
                    "●".to_string(),
                    NodeShape::Circle,
                )
            } else {
                (
                    to_raw.to_string(),
                    to_raw.to_string(),
                    NodeShape::Rounded,
                )
            };

            if !node_set.contains(&from_id) {
                node_set.insert(from_id.clone());
                nodes.push(MermaidNode {
                    id: from_id.clone(),
                    label: from_label,
                    shape: from_shape,
                });
            }
            if !node_set.contains(&to_id) {
                node_set.insert(to_id.clone());
                nodes.push(MermaidNode {
                    id: to_id.clone(),
                    label: to_label,
                    shape: to_shape,
                });
            }

            let edge_type = if arrow_str == "-->" {
                EdgeType::Arrow
            } else {
                EdgeType::Line
            };

            edges.push(MermaidEdge {
                source: from_id,
                target: to_id,
                label: label_text,
                edge_type,
            });
        }
    }

    if nodes.is_empty() {
        return None;
    }

    Some(MermaidDiagram {
        direction: types::Direction::TopDown,
        nodes,
        edges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_flowchart() -> anyhow::Result<()> {
        let diagram = parser::parse("graph TD\nA[Start] --> B[End]")
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        assert_eq!(
            diagram.nodes.len(),
            2,
            "expected 2 nodes, got {:?}",
            diagram.nodes
        );
        assert_eq!(
            diagram.edges.len(),
            1,
            "expected 1 edge, got {:?}",
            diagram.edges
        );
        assert_eq!(diagram.direction, Direction::TopDown);
        Ok(())
    }

    #[test]
    fn test_parse_with_labels() -> anyhow::Result<()> {
        let diagram = parser::parse("graph TD\nA -->|yes| B")
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        assert_eq!(diagram.nodes.len(), 2);
        assert_eq!(diagram.edges[0].label.as_deref(), Some("yes"));
        Ok(())
    }

    #[test]
    fn test_parse_lr_direction() -> anyhow::Result<()> {
        let diagram = parser::parse("graph LR\nA --> B")
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        assert_eq!(diagram.direction, Direction::LeftRight);
        Ok(())
    }

    #[test]
    fn test_parse_sequence_diagram() {
        let diagram = sequence::parse_sequence(
            "sequenceDiagram\n    Alice->>Bob: Hello\n    Bob-->>Alice: Hi",
        )
        .unwrap();
        assert_eq!(diagram.participants.len(), 2);
        assert_eq!(diagram.messages.len(), 2);
    }

    #[test]
    fn test_parse_pie_chart() {
        let chart = pie::parse_pie("pie title Pets\n    \"Dogs\" : 386\n    \"Cats\" : 85").unwrap();
        assert_eq!(chart.title.as_deref(), Some("Pets"));
        assert_eq!(chart.slices.len(), 2);
    }

    #[test]
    fn test_parse_gantt_chart() {
        let chart = gantt::parse_gantt(
            "gantt\ntitle Project\nsection Phase 1\nTask 1 :a1, 7d\nTask 2 :a2, after a1, 5d",
        )
        .unwrap();
        assert_eq!(chart.title.as_deref(), Some("Project"));
        assert_eq!(chart.sections.len(), 1);
        assert_eq!(chart.sections[0].tasks.len(), 2);
    }

    #[test]
    fn test_parse_state_diagram() {
        let diagram = parse_state_diagram(
            "stateDiagram-v2\n    [*] --> Idle\n    Idle --> Running\n    Running --> Idle",
        )
        .unwrap();
        assert_eq!(diagram.nodes.len(), 3);
        assert_eq!(diagram.edges.len(), 3);
    }
}
