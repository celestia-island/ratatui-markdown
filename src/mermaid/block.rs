use ratatui::text::Line;

use super::{
    graph, layout, render,
    types::{Direction, MermaidDiagram, MermaidEdge, MermaidNode, NodeShape},
};
use crate::theme::RichTextTheme;

#[derive(Debug, Clone, PartialEq)]
pub struct BlockDiagram {
    pub columns: usize,
    pub blocks: Vec<BlockEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockEntry {
    pub id: String,
    pub label: String,
}

pub fn parse_block(source: &str) -> Option<BlockDiagram> {
    let mut columns: usize = 1;
    let mut blocks: Vec<BlockEntry> = Vec::new();

    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('%') {
            continue;
        }

        let lower = line.to_lowercase();
        if lower.starts_with("block") && !lower.starts_with("block-beta") {
            continue;
        }
        if lower.starts_with("block-beta") {
            continue;
        }

        if let Some(rest) = line.strip_prefix("columns ") {
            if let Ok(n) = rest.trim().parse::<usize>() {
                columns = n.max(1);
            }
            continue;
        }

        if line.contains("columns") && !line.starts_with("columns") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in &parts {
                if part == &"columns" {
                    continue;
                }
                if let Ok(n) = part.parse::<usize>() {
                    columns = n.max(1);
                }
            }
            continue;
        }

        for token in line.split_whitespace() {
            let token = token.trim();
            if token.is_empty() || token == "columns" {
                continue;
            }
            if let Ok(_n) = token.parse::<usize>() {
                continue;
            }
            let cleaned = token.trim_matches('"').trim_matches('\'');
            blocks.push(BlockEntry {
                id: cleaned.to_string(),
                label: cleaned.to_string(),
            });
        }
    }

    if blocks.is_empty() {
        return None;
    }

    Some(BlockDiagram { columns, blocks })
}

pub fn convert_to_mermaid_diagram(block: &BlockDiagram) -> MermaidDiagram {
    use std::collections::HashMap;

    let mut nodes: Vec<MermaidNode> = Vec::new();
    let mut node_map: HashMap<String, usize> = HashMap::new();
    let mut edges: Vec<MermaidEdge> = Vec::new();

    for entry in &block.blocks {
        MermaidDiagram::ensure_node(
            &mut nodes,
            &mut node_map,
            &entry.id,
            Some(&entry.label),
            Some(NodeShape::Rounded),
        );
    }

    let cols = block.columns.max(1);
    for i in 0..block.blocks.len() {
        if i + cols < block.blocks.len() {
            edges.push(MermaidEdge {
                source: block.blocks[i].id.clone(),
                target: block.blocks[i + cols].id.clone(),
                label: None,
                edge_type: super::types::EdgeType::Line,
            });
        }
        if i + 1 < block.blocks.len() && (i + 1) % cols != 0 {
            edges.push(MermaidEdge {
                source: block.blocks[i].id.clone(),
                target: block.blocks[i + 1].id.clone(),
                label: None,
                edge_type: super::types::EdgeType::Line,
            });
        }
    }

    MermaidDiagram {
        direction: Direction::TopDown,
        nodes,
        edges,
    }
}

pub fn render_block_diagram(
    source: &str,
    max_width: usize,
    max_height: Option<usize>,
    theme: &impl RichTextTheme,
) -> Option<Vec<Line<'static>>> {
    let block = parse_block(source)?;
    let mermaid = convert_to_mermaid_diagram(&block);
    let direction = mermaid.direction.clone();
    let graph = graph::assign_layers(&mermaid);
    let layout = layout::compute_layout(&mermaid, &graph, max_width, max_height);
    Some(render::render_layout(&layout, &direction, theme))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_parse_simple_block() -> Result<()> {
        let source = "block-beta\n    A B C\n    D E F\n";
        let diagram =
            parse_block(source).ok_or_else(|| anyhow::anyhow!("failed to parse block"))?;
        assert_eq!(diagram.blocks.len(), 6);
        Ok(())
    }

    #[test]
    fn test_parse_block_with_columns() -> Result<()> {
        let source = "block-beta\n    columns 2\n    A B\n    C D\n";
        let diagram =
            parse_block(source).ok_or_else(|| anyhow::anyhow!("failed to parse block"))?;
        assert_eq!(diagram.columns, 2);
        assert_eq!(diagram.blocks.len(), 4);
        Ok(())
    }

    #[test]
    fn test_convert_to_mermaid() -> Result<()> {
        let source = "block-beta\n    A B\n    C D\n";
        let diagram =
            parse_block(source).ok_or_else(|| anyhow::anyhow!("failed to parse block"))?;
        let mermaid = convert_to_mermaid_diagram(&diagram);
        assert_eq!(mermaid.nodes.len(), 4);
        assert!(mermaid.edges.len() >= 3);
        Ok(())
    }
}
