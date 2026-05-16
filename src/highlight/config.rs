use ratatui::style::{Color, Modifier, Style};

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

pub fn highlight_to_style(idx: usize) -> Style {
    let name = HIGHLIGHT_NAMES.get(idx).unwrap_or(&"");
    match *name {
        "comment" | "comment.documentation" => Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
        "constant" | "constant.builtin" | "boolean" => Style::default().fg(Color::Yellow),
        "string" | "string.special" => Style::default().fg(Color::Green),
        "string.escape" | "string.regex" => Style::default().fg(Color::LightGreen),
        "keyword" | "keyword.function" | "conditional" | "repeat" | "exception" | "include" => {
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD)
        }
        "number" => Style::default().fg(Color::Yellow),
        "function" | "function.builtin" => Style::default().fg(Color::Cyan),
        "type" | "type.builtin" | "namespace" => Style::default().fg(Color::LightCyan),
        "variable" | "variable.builtin" | "variable.parameter" | "variable.member" => {
            Style::default().fg(Color::White)
        }
        "property" => Style::default().fg(Color::LightBlue),
        "operator" => Style::default().fg(Color::LightMagenta),
        "punctuation" | "punctuation.bracket" | "punctuation.delimiter" | "punctuation.special" => {
            Style::default().fg(Color::DarkGray)
        }
        "attribute" => Style::default().fg(Color::LightYellow),
        "constructor" => Style::default().fg(Color::LightCyan),
        "tag" => Style::default().fg(Color::Cyan),
        "label" => Style::default().fg(Color::LightRed),
        "error" => Style::default().fg(Color::Red),
        _ => Style::default(),
    }
}
