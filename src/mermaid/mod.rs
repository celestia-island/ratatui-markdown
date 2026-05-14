mod layout;
mod parser;
mod render;
mod types;

pub use types::{Direction, EdgeType, MermaidDiagram, MermaidEdge, MermaidNode, NodeShape};

use ratatui::text::Line;
use crate::theme::RichTextTheme;

pub fn render_mermaid(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_flowchart() {
        let result = parser::parse("graph TD\nA[Start] --> B[End]");
        if let Err(ref e) = result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok(), "parse should succeed");
        let diagram = result.unwrap();
        assert_eq!(diagram.nodes.len(), 2, "expected 2 nodes, got {:?}", diagram.nodes);
        assert_eq!(diagram.edges.len(), 1, "expected 1 edge, got {:?}", diagram.edges);
        assert_eq!(diagram.direction, Direction::TopDown);
    }

    #[test]
    fn test_parse_with_labels() {
        let result = parser::parse("graph TD\nA -->|yes| B");
        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert_eq!(diagram.nodes.len(), 2);
        assert_eq!(diagram.edges[0].label.as_deref(), Some("yes"));
    }

    #[test]
    fn test_parse_lr_direction() {
        let result = parser::parse("graph LR\nA --> B");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().direction, Direction::LeftRight);
    }
}
