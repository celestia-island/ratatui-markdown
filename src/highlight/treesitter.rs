use std::sync::Mutex;

use ratatui::style::{Color, Modifier, Style};
use tree_sitter_highlight::Highlighter;

use super::{CodeHighlighter, StyleSegment};

const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "boolean",
    "comment",
    "comment.documentation",
    "constant",
    "constant.builtin",
    "constructor",
    "function",
    "function.builtin",
    "keyword",
    "number",
    "operator",
    "property",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "string",
    "string.escape",
    "string.special",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.parameter",
    "variable.member",
    "tag",
    "label",
    "error",
];

struct LangEntry {
    language: tree_sitter::Language,
    highlights_query: &'static str,
}

macro_rules! lang_entry {
    ($lang_crate:ident) => {{
        LangEntry {
            language: $lang_crate::LANGUAGE.into(),
            highlights_query: $lang_crate::HIGHLIGHTS_QUERY,
        }
    }};
}

fn get_lang(lang: &str) -> Option<LangEntry> {
    match lang {
        #[cfg(feature = "highlight-lang-rust")]
        "rust" => Some(lang_entry!(tree_sitter_rust)),
        #[cfg(feature = "highlight-lang-python")]
        "python" | "py" => Some(lang_entry!(tree_sitter_python)),
        #[cfg(feature = "highlight-lang-go")]
        "go" | "golang" => Some(lang_entry!(tree_sitter_go)),
        #[cfg(feature = "highlight-lang-java")]
        "java" => Some(lang_entry!(tree_sitter_java)),
        #[cfg(feature = "highlight-lang-html")]
        "html" | "htm" => Some(lang_entry!(tree_sitter_html)),
        #[cfg(feature = "highlight-lang-css")]
        "css" | "scss" | "less" => Some(lang_entry!(tree_sitter_css)),
        #[cfg(feature = "highlight-lang-json")]
        "json" => Some(lang_entry!(tree_sitter_json)),
        #[cfg(feature = "highlight-lang-toml")]
        "toml" => Some(lang_entry!(tree_sitter_toml_ng)),
        #[cfg(feature = "highlight-lang-sql")]
        "sql" => Some(lang_entry!(tree_sitter_sequel)),
        _ => None,
    }
}

fn build_config(entry: &LangEntry) -> tree_sitter_highlight::HighlightConfiguration {
    let mut config = tree_sitter_highlight::HighlightConfiguration::new(
        entry.language.clone(),
        "",
        entry.highlights_query,
        "",
        "",
    )
    .expect("failed to create HighlightConfiguration");
    config.configure(HIGHLIGHT_NAMES);
    config
}

pub struct TreeSitterHighlighter {
    highlighter: Mutex<Highlighter>,
}

impl TreeSitterHighlighter {
    pub fn new() -> Self {
        Self {
            highlighter: Mutex::new(Highlighter::new()),
        }
    }
}

impl Default for TreeSitterHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeHighlighter for TreeSitterHighlighter {
    fn highlight(&self, lang: &str, code: &str) -> Vec<StyleSegment> {
        let entry = match get_lang(lang) {
            Some(e) => e,
            None => return Vec::new(),
        };
        let config = build_config(&entry);
        let mut hl = self.highlighter.lock().unwrap();

        let events = match hl.highlight(&config, code.as_bytes(), None, |_| None) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        let mut segments = Vec::new();
        let mut style_stack: Vec<usize> = Vec::new();

        for event in events {
            match event {
                Ok(tree_sitter_highlight::HighlightEvent::Source { start, end }) => {
                    let style = style_stack
                        .last()
                        .map(|&idx| highlight_to_style(idx))
                        .unwrap_or_default();
                    if start != end {
                        segments.push(StyleSegment {
                            start,
                            end,
                            style,
                        });
                    }
                }
                Ok(tree_sitter_highlight::HighlightEvent::HighlightStart(
                    tree_sitter_highlight::Highlight(idx),
                )) => {
                    style_stack.push(idx);
                }
                Ok(tree_sitter_highlight::HighlightEvent::HighlightEnd) => {
                    style_stack.pop();
                }
                Err(_) => break,
            }
        }

        segments
    }
}

fn highlight_to_style(idx: usize) -> Style {
    let name = HIGHLIGHT_NAMES.get(idx).unwrap_or(&"");
    match *name {
        "comment" | "comment.documentation" => Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
        "constant" | "constant.builtin" | "boolean" => Style::default().fg(Color::Yellow),
        "string" | "string.special" => Style::default().fg(Color::Green),
        "string.escape" => Style::default().fg(Color::LightGreen),
        "keyword" => Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
        "number" => Style::default().fg(Color::Yellow),
        "function" | "function.builtin" => Style::default().fg(Color::Cyan),
        "type" | "type.builtin" => Style::default().fg(Color::LightCyan),
        "variable" | "variable.builtin" | "variable.parameter" | "variable.member" => {
            Style::default().fg(Color::White)
        }
        "property" => Style::default().fg(Color::LightBlue),
        "operator" => Style::default().fg(Color::LightMagenta),
        "punctuation" | "punctuation.bracket" | "punctuation.delimiter" => {
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
