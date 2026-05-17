use ratatui::style::{Modifier, Style};

use crate::theme::CodeColors;

pub const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "boolean",
    "comment",
    "comment.documentation",
    "conditional",
    "constant",
    "constant.builtin",
    "constructor",
    "exception",
    "function",
    "function.builtin",
    "include",
    "keyword",
    "keyword.function",
    "label",
    "namespace",
    "number",
    "operator",
    "property",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "repeat",
    "string",
    "string.escape",
    "string.regex",
    "string.special",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.member",
    "variable.parameter",
    "error",
];

pub fn highlight_to_style(idx: usize, colors: &CodeColors) -> Style {
    let name = HIGHLIGHT_NAMES.get(idx).unwrap_or(&"");
    match *name {
        "comment" | "comment.documentation" => Style::default()
            .fg(colors.comment)
            .add_modifier(Modifier::ITALIC),
        "constant" | "constant.builtin" | "boolean" => Style::default().fg(colors.constant),
        "string" | "string.special" => Style::default().fg(colors.string),
        "string.escape" | "string.regex" => Style::default().fg(colors.string_escape),
        "keyword" | "keyword.function" | "conditional" | "repeat" | "exception" | "include" => {
            Style::default()
                .fg(colors.keyword)
                .add_modifier(Modifier::BOLD)
        }
        "number" => Style::default().fg(colors.number),
        "function" | "function.builtin" => Style::default().fg(colors.function),
        "type" | "type.builtin" | "namespace" | "constructor" => Style::default().fg(colors.r#type),
        "variable" | "variable.builtin" | "variable.parameter" | "variable.member" => {
            Style::default().fg(colors.variable)
        }
        "property" => Style::default().fg(colors.property),
        "operator" => Style::default().fg(colors.operator),
        "punctuation" | "punctuation.bracket" | "punctuation.delimiter" | "punctuation.special" => {
            Style::default().fg(colors.punctuation)
        }
        "attribute" => Style::default().fg(colors.attribute),
        "tag" => Style::default().fg(colors.tag),
        "label" => Style::default().fg(colors.label),
        "error" => Style::default().fg(colors.error),
        _ => Style::default(),
    }
}
