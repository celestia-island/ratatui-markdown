use pest::Parser;
use pest_derive::Parser;
use ratatui::style::{Color, Modifier, Style};
use ratatui_markdown::highlight::{pest_pairs_to_segments, CodeHighlighter, StyleSegment};

// Option A: reference an external .pest file (must reside under src/)
#[derive(Parser)]
#[grammar = "highlight/mcfunction.pest"]
// Option B: inline the grammar directly (no external file needed)
// #[grammar_inline = r##" ... "##]
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
        pest_pairs_to_segments(pairs, rule_style)
    }
}

fn rule_style(rule: Rule) -> Option<Style> {
    match rule {
        Rule::comment => Some(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ),
        Rule::cmd_name => Some(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Rule::selector => Some(Style::default().fg(Color::Yellow)),
        Rule::str => Some(Style::default().fg(Color::Green)),
        Rule::number => Some(Style::default().fg(Color::Yellow)),
        Rule::coord => Some(Style::default().fg(Color::Cyan)),
        Rule::nbt => Some(Style::default().fg(Color::LightBlue)),
        _ => None,
    }
}
