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

macro_rules! lang_entry_sq {
    ($lang_crate:ident) => {{
        LangEntry {
            language: $lang_crate::LANGUAGE.into(),
            highlights_query: $lang_crate::HIGHLIGHT_QUERY,
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

        #[cfg(feature = "highlight-lang-javascript")]
        "javascript" | "js" => Some(lang_entry_sq!(tree_sitter_javascript)),

        #[cfg(feature = "highlight-lang-typescript")]
        "typescript" | "ts" => Some(LangEntry {
            language: tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            highlights_query: tree_sitter_typescript::HIGHLIGHTS_QUERY,
        }),

        #[cfg(feature = "highlight-lang-typescript")]
        "tsx" => Some(LangEntry {
            language: tree_sitter_typescript::LANGUAGE_TSX.into(),
            highlights_query: tree_sitter_typescript::HIGHLIGHTS_QUERY,
        }),

        #[cfg(feature = "highlight-lang-c")]
        "c" => Some(lang_entry_sq!(tree_sitter_c)),

        #[cfg(feature = "highlight-lang-cpp")]
        "cpp" | "c++" | "cxx" => Some(lang_entry_sq!(tree_sitter_cpp)),

        #[cfg(feature = "highlight-lang-c-sharp")]
        "csharp" | "c#" | "cs" => Some(lang_entry!(tree_sitter_c_sharp)),

        #[cfg(feature = "highlight-lang-bash")]
        "bash" | "sh" | "shell" | "zsh" => Some(lang_entry_sq!(tree_sitter_bash)),

        #[cfg(feature = "highlight-lang-ruby")]
        "ruby" | "rb" => Some(lang_entry!(tree_sitter_ruby)),

        #[cfg(feature = "highlight-lang-swift")]
        "swift" => Some(lang_entry!(tree_sitter_swift)),

        #[cfg(feature = "highlight-lang-php")]
        "php" => Some(LangEntry {
            language: tree_sitter_php::LANGUAGE_PHP.into(),
            highlights_query: tree_sitter_php::HIGHLIGHTS_QUERY,
        }),

        #[cfg(feature = "highlight-lang-scala")]
        "scala" => Some(lang_entry!(tree_sitter_scala)),

        #[cfg(feature = "highlight-lang-lua")]
        "lua" => Some(lang_entry!(tree_sitter_lua)),

        #[cfg(feature = "highlight-lang-haskell")]
        "haskell" | "hs" => Some(lang_entry!(tree_sitter_haskell)),

        #[cfg(feature = "highlight-lang-elixir")]
        "elixir" | "ex" => Some(lang_entry!(tree_sitter_elixir)),

        #[cfg(feature = "highlight-lang-yaml")]
        "yaml" | "yml" => Some(lang_entry!(tree_sitter_yaml)),

        #[cfg(feature = "highlight-lang-dart")]
        "dart" => Some(lang_entry!(tree_sitter_dart)),

        #[cfg(feature = "highlight-lang-zig")]
        "zig" => Some(lang_entry!(tree_sitter_zig)),

        #[cfg(feature = "highlight-lang-r")]
        "r" => Some(lang_entry!(tree_sitter_r)),

        #[cfg(feature = "highlight-lang-ocaml")]
        "ocaml" => Some(LangEntry {
            language: tree_sitter_ocaml::LANGUAGE_OCAML.into(),
            highlights_query: tree_sitter_ocaml::HIGHLIGHTS_QUERY,
        }),

        #[cfg(feature = "highlight-lang-nix")]
        "nix" => Some(lang_entry!(tree_sitter_nix)),

        #[cfg(feature = "highlight-lang-html")]
        "html" | "htm" => Some(lang_entry!(tree_sitter_html)),

        #[cfg(feature = "highlight-lang-css")]
        "css" | "scss" | "less" => Some(lang_entry!(tree_sitter_css)),

        #[cfg(feature = "highlight-lang-xml")]
        "xml" | "svg" | "xsd" => Some(LangEntry {
            language: tree_sitter_xml::LANGUAGE_XML.into(),
            highlights_query: tree_sitter_xml::XML_HIGHLIGHT_QUERY,
        }),

        #[cfg(feature = "highlight-lang-json")]
        "json" => Some(lang_entry!(tree_sitter_json)),

        #[cfg(feature = "highlight-lang-toml")]
        "toml" => Some(lang_entry!(tree_sitter_toml_ng)),

        #[cfg(feature = "highlight-lang-sql")]
        "sql" => Some(lang_entry!(tree_sitter_sequel)),

        #[cfg(feature = "highlight-lang-solidity")]
        "solidity" | "sol" => Some(lang_entry_sq!(tree_sitter_solidity)),

        #[cfg(feature = "highlight-lang-diff")]
        "diff" | "patch" => Some(lang_entry!(tree_sitter_diff)),

        #[cfg(feature = "highlight-lang-regex")]
        "regex" | "regexp" => Some(lang_entry!(tree_sitter_regex)),

        #[cfg(feature = "highlight-lang-powershell")]
        "powershell" | "ps1" | "pwsh" => Some(lang_entry!(tree_sitter_powershell)),

        #[cfg(feature = "highlight-lang-objc")]
        "objc" | "objective-c" | "objectivec" => Some(lang_entry!(tree_sitter_objc)),

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
