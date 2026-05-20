use ratatui::text::Line;
use unicode_width::UnicodeWidthChar;

use super::{
    graph, layout, render,
    types::{Direction, EdgeType, MermaidDiagram, MermaidEdge, MermaidNode, NodeShape},
};
use crate::theme::RichTextTheme;

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDefinition {
    pub name: String,
    pub attributes: Vec<ClassMember>,
    pub methods: Vec<ClassMember>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassMember {
    pub visibility: Visibility,
    pub name: String,
    pub type_info: String,
    pub is_method: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelationshipType {
    Inheritance,
    Composition,
    Aggregation,
    Association,
    Dependency,
    Implements,
    DirectedAssociation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassRelationship {
    pub from: String,
    pub to: String,
    pub rel_type: RelationshipType,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDiagram {
    pub classes: Vec<ClassDefinition>,
    pub relationships: Vec<ClassRelationship>,
}

fn render_relationship_label(r: &RelationshipType) -> &'static str {
    match r {
        RelationshipType::Inheritance => "extends",
        RelationshipType::Composition => "has",
        RelationshipType::Aggregation => "has",
        RelationshipType::Association => "uses",
        RelationshipType::Dependency => "depends",
        RelationshipType::Implements => "impl",
        RelationshipType::DirectedAssociation => "uses",
    }
}

#[allow(dead_code)]
fn visibility_char(v: Visibility) -> char {
    match v {
        Visibility::Public => '+',
        Visibility::Private => '-',
        Visibility::Protected => '#',
        Visibility::Internal => '~',
    }
}

fn relationship_to_edge_type(r: RelationshipType) -> EdgeType {
    match r {
        RelationshipType::Inheritance => EdgeType::Arrow,
        RelationshipType::Composition => EdgeType::Arrow,
        RelationshipType::Aggregation => EdgeType::Arrow,
        RelationshipType::Association => EdgeType::Line,
        RelationshipType::Dependency => EdgeType::Line,
        RelationshipType::Implements => EdgeType::Arrow,
        RelationshipType::DirectedAssociation => EdgeType::Arrow,
    }
}

fn unicode_width(s: &str) -> usize {
    s.chars().map(|c| c.width().unwrap_or(0)).sum()
}

fn build_class_label(class: &ClassDefinition, max_width: usize) -> String {
    let title_content = format!(" {} ", class.name);
    let title_w = unicode_width(&title_content);

    let attr_texts: Vec<String> = class
        .attributes
        .iter()
        .map(|a| {
            let v = visibility_char(a.visibility);
            if a.type_info.is_empty() {
                format!("{} {}", v, a.name)
            } else {
                format!("{} {}: {}", v, a.name, a.type_info)
            }
        })
        .collect();

    let method_texts: Vec<String> = class
        .methods
        .iter()
        .map(|m| {
            let v = visibility_char(m.visibility);
            if m.type_info.is_empty() {
                format!("{} {}()", v, m.name)
            } else {
                format!("{} {}(): {}", v, m.name, m.type_info)
            }
        })
        .collect();

    let max_content = title_w
        .max(
            attr_texts
                .iter()
                .map(|t| unicode_width(t) + 1)
                .max()
                .unwrap_or(0),
        )
        .max(
            method_texts
                .iter()
                .map(|t| unicode_width(t) + 1)
                .max()
                .unwrap_or(0),
        );

    let available = max_content.max(max_width.saturating_sub(4)).clamp(4, 60);
    let sep = "\u{2500}".repeat(available);

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("\u{250c}{}\u{2510}", sep));

    let pad = available.saturating_sub(title_w);
    let left = pad / 2;
    let right = pad - left;
    lines.push(format!(
        "\u{2502}{}{}{}\u{2502}",
        " ".repeat(left),
        title_content,
        " ".repeat(right)
    ));

    lines.push(format!("\u{251c}{}\u{2524}", sep));

    for text in &attr_texts {
        let tw = unicode_width(text);
        let pad = available.saturating_sub(tw + 1);
        lines.push(format!("\u{2502} {}{}\u{2502}", text, " ".repeat(pad)));
    }

    if !class.attributes.is_empty() && !class.methods.is_empty() {
        lines.push(format!("\u{2502}{}\u{2502}", " ".repeat(available)));
    }

    for text in &method_texts {
        let tw = unicode_width(text);
        let pad = available.saturating_sub(tw + 1);
        lines.push(format!("\u{2502} {}{}\u{2502}", text, " ".repeat(pad)));
    }

    lines.push(format!("\u{2514}{}\u{2518}", sep));
    lines.join("\n")
}

pub fn parse_class_diagram(source: &str) -> Option<ClassDiagram> {
    let mut classes: Vec<ClassDefinition> = Vec::new();
    let mut relationships: Vec<ClassRelationship> = Vec::new();
    let mut current_class: Option<ClassDefinition> = None;
    let mut in_class_body = false;
    let mut brace_depth = 0;

    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("classDiagram") {
            continue;
        }

        if let Some(ref mut class) = current_class {
            if in_class_body {
                let trimmed = line.trim();
                if trimmed == "}" {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        in_class_body = false;
                        classes.push(class.clone());
                        current_class = None;
                        continue;
                    }
                }
                if brace_depth > 0 && !trimmed.is_empty() && !trimmed.starts_with('%') {
                    let member = parse_member(trimmed);
                    if let Some(m) = member {
                        if m.is_method {
                            class.methods.push(m);
                        } else {
                            class.attributes.push(m);
                        }
                    }
                }
                continue;
            }
        } else if line.starts_with("class ") && line.contains('{') {
            let rest = line[6..].trim();
            if let Some(name) = rest.split('{').next() {
                let name = name.trim().to_string();
                if !name.is_empty() {
                    in_class_body = true;
                    brace_depth = 1;
                    current_class = Some(ClassDefinition {
                        name,
                        attributes: Vec::new(),
                        methods: Vec::new(),
                    });
                    let body_part = line.split('{').nth(1).unwrap_or("");
                    if body_part.contains('}') {
                        if let Some(content) = body_part.split('}').next() {
                            for member_str in content.split([';', '\n']) {
                                let trimmed = member_str.trim();
                                if !trimmed.is_empty() {
                                    if let Some(m) = parse_member(trimmed) {
                                        if let Some(ref mut class) = current_class {
                                            if m.is_method {
                                                class.methods.push(m);
                                            } else {
                                                class.attributes.push(m);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        brace_depth = 0;
                        in_class_body = false;
                        if let Some(class) = current_class.take() {
                            classes.push(class);
                        }
                    }
                }
            }
        } else {
            let rel = parse_relationship(line);
            if let Some(r) = rel {
                relationships.push(r);
            }
        }
    }

    if current_class.is_some() {
        if let Some(class) = current_class.take() {
            if !class.attributes.is_empty() || !class.methods.is_empty() {
                classes.push(class);
            }
        }
    }

    if classes.is_empty() {
        return None;
    }

    Some(ClassDiagram {
        classes,
        relationships,
    })
}

fn parse_member(line: &str) -> Option<ClassMember> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('%') {
        return None;
    }

    let first = line.chars().next()?;
    let visibility = match first {
        '+' => Visibility::Public,
        '-' => Visibility::Private,
        '#' => Visibility::Protected,
        '~' => Visibility::Internal,
        _ => {
            return Some(ClassMember {
                visibility: Visibility::Internal,
                name: line.to_string(),
                type_info: String::new(),
                is_method: line.contains('('),
            })
        }
    };

    let rest = &line[1..].trim();
    let is_method = rest.contains('(');

    if is_method {
        let (name, type_info) = if let Some(paren) = rest.find('(') {
            let name = rest[..paren].trim();
            let after_paren = &rest[paren..];
            let type_info = if let Some(colon) = after_paren.rfind(':') {
                after_paren[colon + 1..].trim().to_string()
            } else {
                String::new()
            };
            (name.to_string(), type_info)
        } else {
            (rest.to_string(), String::new())
        };
        Some(ClassMember {
            visibility,
            name,
            type_info,
            is_method: true,
        })
    } else {
        let (name, type_info) = if let Some(colon) = rest.find(':') {
            let name = rest[..colon].trim().to_string();
            let t = rest[colon + 1..].trim().to_string();
            (name, t)
        } else {
            (rest.to_string(), String::new())
        };
        Some(ClassMember {
            visibility,
            name,
            type_info,
            is_method: false,
        })
    }
}

fn parse_relationship(line: &str) -> Option<ClassRelationship> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('%') {
        return None;
    }

    let patterns: &[(&str, RelationshipType)] = &[
        ("<|--", RelationshipType::Inheritance),
        ("*--", RelationshipType::Composition),
        ("o--", RelationshipType::Aggregation),
        ("..|>", RelationshipType::Implements),
        ("..", RelationshipType::Dependency),
        ("<--", RelationshipType::DirectedAssociation),
        ("-->", RelationshipType::DirectedAssociation),
        ("--", RelationshipType::Association),
    ];

    for (pattern, rel_type) in patterns {
        if let Some(pos) = line.find(pattern) {
            let from = line[..pos].trim();
            let after = &line[pos + pattern.len()..];
            let (to, label) = if let Some(colon) = after.find(':') {
                (
                    after[..colon].trim().to_string(),
                    Some(after[colon + 1..].trim().to_string()),
                )
            } else {
                (after.trim().to_string(), None)
            };
            if from.is_empty() || to.is_empty() {
                return None;
            }
            let from = from.trim_matches('"').to_string();
            let to = to.trim_matches('"').to_string();
            return Some(ClassRelationship {
                from,
                to,
                rel_type: *rel_type,
                label,
            });
        }
    }
    None
}

pub fn convert_to_mermaid_diagram(class_diagram: &ClassDiagram) -> MermaidDiagram {
    use std::collections::HashMap;

    let mut nodes: Vec<MermaidNode> = Vec::new();
    let mut node_map: HashMap<String, usize> = HashMap::new();
    let mut edges: Vec<MermaidEdge> = Vec::new();

    let max_label_width: usize = class_diagram
        .classes
        .iter()
        .map(|c| {
            let name_w = c.name.len();
            let attr_w = c
                .attributes
                .iter()
                .map(|a| a.name.len() + a.type_info.len() + 5)
                .max()
                .unwrap_or(0);
            let meth_w = c
                .methods
                .iter()
                .map(|m| m.name.len() + m.type_info.len() + 7)
                .max()
                .unwrap_or(0);
            (name_w + 4).max(attr_w).max(meth_w)
        })
        .max()
        .unwrap_or(20)
        .min(40);

    for class in &class_diagram.classes {
        let label = build_class_label(class, max_label_width);
        MermaidDiagram::ensure_node(
            &mut nodes,
            &mut node_map,
            &class.name,
            Some(&label),
            Some(NodeShape::Rect),
        );
    }

    for rel in &class_diagram.relationships {
        let edge_type = relationship_to_edge_type(rel.rel_type);
        let label = rel.label.clone().or_else(|| {
            if rel.from != rel.to {
                Some(render_relationship_label(&rel.rel_type).to_string())
            } else {
                None
            }
        });
        edges.push(MermaidEdge {
            source: rel.from.clone(),
            target: rel.to.clone(),
            label,
            edge_type,
        });
    }

    MermaidDiagram {
        direction: Direction::TopDown,
        nodes,
        edges,
    }
}

pub fn render_class_diagram(
    source: &str,
    max_width: usize,
    max_height: Option<usize>,
    theme: &impl RichTextTheme,
) -> Option<Vec<Line<'static>>> {
    let diagram = parse_class_diagram(source)?;
    let mermaid = convert_to_mermaid_diagram(&diagram);
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
    fn test_parse_simple_class_diagram() -> Result<()> {
        let source =
            "classDiagram\nclass Animal {\n  +String name\n  +int age\n  +makeSound() void\n}\n";
        let diagram = parse_class_diagram(source)
            .ok_or_else(|| anyhow::anyhow!("failed to parse class diagram"))?;
        assert_eq!(diagram.classes.len(), 1);
        assert_eq!(diagram.classes[0].name, "Animal");
        assert_eq!(diagram.classes[0].attributes.len(), 2);
        assert_eq!(diagram.classes[0].methods.len(), 1);
        Ok(())
    }

    #[test]
    fn test_parse_relationship() -> Result<()> {
        let rel = parse_relationship("Animal <|-- Dog")
            .ok_or_else(|| anyhow::anyhow!("failed to parse relationship"))?;
        assert_eq!(rel.from, "Animal");
        assert_eq!(rel.to, "Dog");
        assert_eq!(rel.rel_type, RelationshipType::Inheritance);
        Ok(())
    }

    #[test]
    fn test_parse_relationship_with_label() -> Result<()> {
        let rel = parse_relationship("Animal <|-- Dog : extends")
            .ok_or_else(|| anyhow::anyhow!("failed to parse relationship"))?;
        assert_eq!(rel.from, "Animal");
        assert_eq!(rel.to, "Dog");
        assert_eq!(rel.label.as_deref(), Some("extends"));
        Ok(())
    }

    #[test]
    fn test_parse_composition() -> Result<()> {
        let rel = parse_relationship("Car *-- Engine")
            .ok_or_else(|| anyhow::anyhow!("failed to parse relationship"))?;
        assert_eq!(rel.from, "Car");
        assert_eq!(rel.to, "Engine");
        assert_eq!(rel.rel_type, RelationshipType::Composition);
        Ok(())
    }

    #[test]
    fn test_parse_implements() -> Result<()> {
        let rel = parse_relationship("Dog ..|> Runnable")
            .ok_or_else(|| anyhow::anyhow!("failed to parse relationship"))?;
        assert_eq!(rel.from, "Dog");
        assert_eq!(rel.to, "Runnable");
        assert_eq!(rel.rel_type, RelationshipType::Implements);
        Ok(())
    }

    #[test]
    fn test_convert_to_mermaid() -> Result<()> {
        let source = "classDiagram\nclass Animal {\n  +String name\n}\nclass Dog {\n  +String breed\n}\nAnimal <|-- Dog\n";
        let diagram = parse_class_diagram(source)
            .ok_or_else(|| anyhow::anyhow!("failed to parse class diagram"))?;
        let mermaid = convert_to_mermaid_diagram(&diagram);
        assert_eq!(mermaid.nodes.len(), 2);
        assert_eq!(mermaid.edges.len(), 1);
        Ok(())
    }
}
