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
    use anyhow::Context as _;

    #[test]
    fn test_parse_simple_flowchart() -> anyhow::Result<()> {
        let diagram = parser::parse("graph TD\nA[Start] --> B[End]")
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        assert_eq!(diagram.nodes.len(), 2, "expected 2 nodes, got {:?}", diagram.nodes);
        assert_eq!(diagram.edges.len(), 1, "expected 1 edge, got {:?}", diagram.edges);
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
}
