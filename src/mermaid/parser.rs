use pest::Parser;
use pest_derive::Parser;

use super::types::*;

#[derive(Parser)]
#[grammar = "mermaid/grammar.pest"]
pub struct MermaidParser;

pub fn parse(source: &str) -> Result<MermaidDiagram, String> {
    let pairs = MermaidParser::parse(Rule::file, source)
        .map_err(|e| format!("mermaid parse error: {}", e))?;

    let mut direction = Direction::TopDown;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut node_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for pair in pairs {
        if pair.as_rule() == Rule::file {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::diagram {
                    for dchild in inner.into_inner() {
                        match dchild.as_rule() {
                            Rule::direction => {
                                direction = match dchild.as_str() {
                                    "TD" | "TB" => Direction::TopDown,
                                    "BT" => Direction::BottomUp,
                                    "LR" => Direction::LeftRight,
                                    "RL" => Direction::RightLeft,
                                    _ => Direction::TopDown,
                                };
                            }
                            Rule::stmts => {
                                for stmt in dchild.into_inner() {
                                    parse_stmt(stmt, &mut nodes, &mut edges, &mut node_map);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    Ok(MermaidDiagram {
        direction,
        nodes,
        edges,
    })
}

fn parse_stmt(
    pair: pest::iterators::Pair<Rule>,
    nodes: &mut Vec<MermaidNode>,
    edges: &mut Vec<MermaidEdge>,
    node_map: &mut std::collections::HashMap<String, usize>,
) {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::stmt_inner {
            for inner2 in inner.into_inner() {
                match inner2.as_rule() {
                    Rule::chain => {
                        parse_chain(inner2, nodes, edges, node_map);
                    }
                    Rule::nodedef => {
                        parse_nodedef(inner2, nodes, node_map);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn parse_chain(
    pair: pest::iterators::Pair<Rule>,
    nodes: &mut Vec<MermaidNode>,
    edges: &mut Vec<MermaidEdge>,
    node_map: &mut std::collections::HashMap<String, usize>,
) {
    let mut refs: Vec<(String, Option<String>, Option<NodeShape>)> = Vec::new();
    let mut edge_types: Vec<(EdgeType, Option<String>)> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::nref => {
                let (id, label, shape) = parse_nref(inner);
                MermaidDiagram::ensure_node(nodes, node_map, &id, label.as_deref(), shape.clone());
                refs.push((id, label, shape));
            }
            Rule::edge => {
                let (et, lbl) = parse_edge(inner);
                edge_types.push((et, lbl));
            }
            _ => {}
        }
    }

    for i in 0..edge_types.len() {
        if i + 1 < refs.len() {
            let (et, lbl) = &edge_types[i];
            edges.push(MermaidEdge {
                source: refs[i].0.clone(),
                target: refs[i + 1].0.clone(),
                label: lbl.clone(),
                edge_type: et.clone(),
            });
        }
    }
}

fn parse_nodedef(
    pair: pest::iterators::Pair<Rule>,
    nodes: &mut Vec<MermaidNode>,
    node_map: &mut std::collections::HashMap<String, usize>,
) {
    let mut id = String::new();
    let mut label: Option<String> = None;
    let mut shape: Option<NodeShape> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::nid => {
                id = inner.as_str().to_string();
            }
            Rule::shape => {
                let (s, l) = parse_shape(inner);
                shape = Some(s);
                label = l;
            }
            _ => {}
        }
    }

    MermaidDiagram::ensure_node(nodes, node_map, &id, label.as_deref(), shape);
}

fn parse_nref(pair: pest::iterators::Pair<Rule>) -> (String, Option<String>, Option<NodeShape>) {
    let mut id = String::new();
    let mut label: Option<String> = None;
    let mut shape: Option<NodeShape> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::nid => {
                id = inner.as_str().to_string();
            }
            Rule::shape => {
                let (s, l) = parse_shape(inner);
                shape = Some(s);
                label = l;
            }
            _ => {}
        }
    }

    (id, label, shape)
}

fn parse_shape(pair: pest::iterators::Pair<Rule>) -> (NodeShape, Option<String>) {
    for inner in pair.into_inner() {
        let text = inner
            .clone()
            .into_inner()
            .next()
            .map(|p| p.as_str().trim().to_string());
        let text = text.filter(|t| !t.is_empty());
        match inner.as_rule() {
            Rule::circ => return (NodeShape::Circle, text),
            Rule::rect => return (NodeShape::Rect, text),
            Rule::rnd => return (NodeShape::Rounded, text),
            Rule::diam => return (NodeShape::Diamond, text),
            _ => {}
        }
    }
    (NodeShape::Rect, None)
}

fn parse_edge(pair: pest::iterators::Pair<Rule>) -> (EdgeType, Option<String>) {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::arr | Rule::arr_lbl => {
                let label = inner.into_inner().next().map(|p| {
                    p.into_inner()
                        .next()
                        .map(|lp| lp.as_str().trim().to_string())
                        .unwrap_or_default()
                });
                return (EdgeType::Arrow, label.filter(|l| !l.is_empty()));
            }
            Rule::ln | Rule::ln_lbl => {
                let label = inner.into_inner().next().map(|p| {
                    p.into_inner()
                        .next()
                        .map(|lp| lp.as_str().trim().to_string())
                        .unwrap_or_default()
                });
                return (EdgeType::Line, label.filter(|l| !l.is_empty()));
            }
            _ => {}
        }
    }
    (EdgeType::Arrow, None)
}
