use std::sync::Mutex;

use tree_sitter_highlight::Highlighter;

use super::{
    config::{highlight_to_style, HIGHLIGHT_NAMES},
    CodeHighlighter, StyleSegment,
};
use crate::theme::CodeColors;

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

#[cfg(any(
    feature = "highlight-lang-javascript",
    feature = "highlight-lang-c",
    feature = "highlight-lang-cpp",
    feature = "highlight-lang-bash",
    feature = "highlight-lang-solidity",
))]
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

        #[cfg(feature = "highlight-lang-kotlin")]
        "kotlin" | "kt" => Some(LangEntry {
            language: tree_sitter_kotlin_ng::LANGUAGE.into(),
            highlights_query: KOTLIN_HIGHLIGHTS,
        }),

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

        #[cfg(feature = "highlight-lang-cmake")]
        "cmake" => Some(LangEntry {
            language: tree_sitter_cmake::LANGUAGE.into(),
            highlights_query: CMAKE_HIGHLIGHTS,
        }),

        #[cfg(feature = "highlight-lang-proto")]
        "proto" | "protobuf" => Some(LangEntry {
            language: tree_sitter_proto::LANGUAGE.into(),
            highlights_query: PROTO_HIGHLIGHTS,
        }),

        _ => None,
    }
}

#[cfg(feature = "highlight-lang-kotlin")]
const KOTLIN_HIGHLIGHTS: &str = r#"
(line_comment) @comment
(multiline_comment) @comment

(simple_identifier) @variable
((simple_identifier) @variable.builtin (#eq? @variable.builtin "it"))
((simple_identifier) @variable.builtin (#eq? @variable.builtin "field"))
(this_expression) @variable.builtin
(super_expression) @variable.builtin

(class_parameter (simple_identifier) @property)
(class_body (property_declaration (variable_declaration (simple_identifier) @property)))
(_ (navigation_suffix (simple_identifier) @property))

(enum_entry (simple_identifier) @constant)
(type_identifier) @type

(package_header . (identifier)) @namespace
(import_header "import" @include)

(label) @label

(function_declaration . (simple_identifier) @function)
(getter ("get") @function.builtin)
(setter ("set") @function.builtin)
(primary_constructor) @constructor
(secondary_constructor ("constructor") @constructor)
(constructor_invocation (user_type (type_identifier) @constructor))

(parameter (simple_identifier) @variable.parameter)
(parameter_with_optional_type (simple_identifier) @variable.parameter)
(lambda_literal (lambda_parameters (variable_declaration (simple_identifier) @variable.parameter)))

(call_expression . (simple_identifier) @function)
(call_expression (navigation_expression (navigation_suffix (simple_identifier) @function) .))

(real_literal) @number
(integer_literal) @number
(long_literal) @number
(hex_literal) @number
(bin_literal) @number
(unsigned_literal) @number
(null_literal) @boolean
(boolean_literal) @boolean
(character_literal) @string
(string_literal) @string
(character_escape_seq) @string.escape

(type_alias "typealias" @keyword)
[
  (class_modifier) (member_modifier) (function_modifier)
  (property_modifier) (platform_modifier) (variance_modifier)
  (parameter_modifier) (visibility_modifier) (reification_modifier)
  (inheritance_modifier)
] @keyword
["val" "var" "enum" "class" "object" "interface"] @keyword
("fun") @keyword.function
(jump_expression) @exception
["if" "else" "when"] @conditional
["for" "do" "while"] @repeat
["try" "catch" "throw" "finally"] @exception

(annotation "@" @attribute (use_site_target)? @attribute)
(annotation (user_type (type_identifier) @attribute))
(annotation (constructor_invocation (user_type (type_identifier) @attribute)))
(file_annotation "@" @attribute "file" @attribute ":" @attribute)

["!" "!=" "!==" "=" "==" "===" ">" ">=" "<" "<=" "||" "&&"
 "+" "++" "+=" "-" "--" "-=" "*" "*=" "/" "/=" "%" "%="
 "?." "?:" "!!" "is" "in" "as" "as?" ".." "..<" "->"] @operator

["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["." "," ";" ":" "::"] @punctuation.delimiter
"#;

#[cfg(feature = "highlight-lang-cmake")]
const CMAKE_HIGHLIGHTS: &str = r#"
[
  (line_comment)
  (bracket_comment)
] @comment

(quoted_argument) @string
(bracket_argument) @string
(variable) @variable
(variable_ref) @variable

(normal_command (identifier) @function)

[
  "if" "elseif" "else" "endif"
  "foreach" "endforeach" "while" "endwhile"
  "function" "endfunction"
  "macro" "endmacro"
  "block" "endblock"
  "return" "break" "continue"
] @keyword

[
  "ENV" "CACHE"
] @namespace

["$" "{" "}"] @punctuation.special
["(" ")"] @punctuation.bracket
"#;

#[cfg(feature = "highlight-lang-proto")]
const PROTO_HIGHLIGHTS: &str = r#"
[
  "syntax" "package" "option" "import" "service" "rpc"
  "returns" "message" "enum" "oneof" "repeated"
  "reserved" "to" "stream" "map" "extend" "extensions"
  "optional" "required"
] @keyword

[(key_type) (type) (message_name) (enum_name) (service_name) (rpc_name)] @type
(string) @string
[(int_lit) (float_lit)] @number
[(true) (false)] @constant.builtin
(comment) @comment
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
"#;

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
    code_colors: CodeColors,
}

impl TreeSitterHighlighter {
    pub fn new() -> Self {
        Self {
            highlighter: Mutex::new(Highlighter::new()),
            code_colors: CodeColors::default(),
        }
    }

    pub fn with_code_colors(mut self, colors: CodeColors) -> Self {
        self.code_colors = colors;
        self
    }

    pub fn set_code_colors(&mut self, colors: CodeColors) {
        self.code_colors = colors;
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
        let mut hl = self.highlighter.lock().unwrap_or_else(|e| e.into_inner());

        let events = match hl.highlight(&config, code.as_bytes(), None, |_| None) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        let mut segments = Vec::new();
        let mut style_stack: Vec<usize> = Vec::new();

        for event in events {
            match event {
                Ok(tree_sitter_highlight::HighlightEvent::Source { start, end }) => {
                    let colors = &self.code_colors;
                    let style = style_stack
                        .last()
                        .map(|&idx| highlight_to_style(idx, colors))
                        .unwrap_or_default();
                    if start != end {
                        segments.push(StyleSegment { start, end, style });
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
