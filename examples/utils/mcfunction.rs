use pest::Parser;
use pest_derive::Parser;
use ratatui::style::{Color, Modifier, Style};
use ratatui_markdown::highlight::{pest_pairs_to_segments, CodeHighlighter, StyleSegment};

// pest_derive requires grammar files under src/ and only accepts string literals
// for grammar_inline, so the grammar is duplicated inline here.
// See examples/utils/mcfunction.pest for the same grammar in a standalone file.
#[derive(Parser)]
#[grammar_inline = r##"
WHITESPACE = _{ " " | "\t" }

file = { SOI ~ (line ~ NEWLINE?)* ~ EOI }

line = { comment | command }

comment = { "#" ~ (!NEWLINE ~ ANY)* }

command = { cmd_name ~ token* }

token = _{ selector | coord | nbt | str | number | word }

cmd_name = { ASCII_ALPHA+ }

selector = { "@" ~ ASCII_ALPHA* ~ ("[" ~ (!"]" ~ ANY)* ~ "]")? }

coord = { ("~" | "^") ~ "-"? ~ ASCII_DIGIT* ~ ("." ~ ASCII_DIGIT+)? }

nbt = { "{" ~ nbt_inner* ~ "}" }
nbt_inner = _{ nbt | str | number | (!("}" | "\"" | "'" | "{") ~ ANY) }

str = @{
    "\"" ~ (!("\"") ~ ANY)* ~ "\""
  | "'" ~ (!("'") ~ ANY)* ~ "'"
}

number = { "-"? ~ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }

word = { (ASCII_ALPHANUMERIC | "_" | ":" | "-" | "=" | "." | "/" | "<" | ">" | ",")+ }
"##]
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
