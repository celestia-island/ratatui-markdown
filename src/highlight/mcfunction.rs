use pest::Parser;
use pest_derive::Parser;
use ratatui::style::{Color, Modifier, Style};

use super::{CodeHighlighter, StyleSegment};

#[derive(Parser)]
#[grammar = "highlight/mcfunction.pest"]
struct McfunctionParser;

pub struct McfunctionHighlighter;

impl CodeHighlighter for McfunctionHighlighter {
    fn highlight(&self, lang: &str, code: &str) -> Vec<StyleSegment> {
        if lang != "mcfunction" && lang != "mcfc" {
            return Vec::new();
        }

        let pairs = match McfunctionParser::parse(Rule::file, code) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        let mut segments = Vec::new();
        collect_segments(pairs, &mut segments);
        segments
    }
}

fn collect_segments(
    pairs: pest::iterators::Pairs<Rule>,
    segments: &mut Vec<StyleSegment>,
) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::comment
            | Rule::cmd_name
            | Rule::selector
            | Rule::string
            | Rule::number
            | Rule::coord
            | Rule::nbt => {
                let span = pair.as_span();
                segments.push(StyleSegment {
                    start: span.start(),
                    end: span.end(),
                    style: rule_style(pair.as_rule()),
                });
            }
            _ => {
                collect_segments(pair.into_inner(), segments);
            }
        }
    }
}

fn rule_style(rule: Rule) -> Style {
    match rule {
        Rule::comment => Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
        Rule::cmd_name => Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
        Rule::selector => Style::default().fg(Color::Yellow),
        Rule::string => Style::default().fg(Color::Green),
        Rule::number => Style::default().fg(Color::Yellow),
        Rule::coord => Style::default().fg(Color::Cyan),
        Rule::nbt => Style::default().fg(Color::LightBlue),
        _ => Style::default(),
    }
}
