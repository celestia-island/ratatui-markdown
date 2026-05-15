#[path = "utils/mod.rs"]
mod common;

use std::sync::Arc;

use common::{AppState, Theme, draw_frame, poll_and_handle, setup_terminal, restore_terminal};
use ratatui::style::{Color, Modifier, Style};
use ratatui_markdown::highlight::{
    CodeHighlighter, HighlightHooks, StyleSegment, TreeSitterHighlighter,
};
use ratatui_markdown::markdown::{MarkdownRenderer, RenderHooks};

struct McfunctionHighlighter;

impl CodeHighlighter for McfunctionHighlighter {
    fn highlight(&self, lang: &str, code: &str) -> Vec<StyleSegment> {
        if lang != "mcfunction" && lang != "mcfc" {
            return Vec::new();
        }

        let mut segments = Vec::new();
        for line in code.lines() {
            let line_start = line.as_ptr() as usize - code.as_ptr() as usize;
            tokenize_mcfunction_line(line, line_start, &mut segments);
        }
        segments
    }
}

fn tokenize_mcfunction_line(
    line: &str,
    base: usize,
    segments: &mut Vec<StyleSegment>,
) {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();

    if trimmed.starts_with('#') {
        segments.push(StyleSegment { start: base, end: base + line.len(), style: comment_style() });
        return;
    }

    let mut chars = trimmed.char_indices().peekable();

    while let Some(&(i, ch)) = chars.peek() {
        match ch {
            '@' => {
                let start = i;
                chars.next();
                while let Some(&(_, c)) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '.' || c == '*' || c == '!' {
                        chars.next();
                    } else {
                        break;
                    }
                }
                if chars.peek().map(|&(_, c)| c) == Some('[') {
                    chars.next();
                    let mut depth = 1;
                    while let Some(&(_, c)) = chars.peek() {
                        chars.next();
                        if c == '[' { depth += 1; } else if c == ']' { depth -= 1; if depth == 0 { break; } }
                    }
                }
                let end = chars.peek().map(|&(j, _)| j).unwrap_or(trimmed.len());
                segments.push(StyleSegment { start: base + indent + start, end: base + indent + end, style: selector_style() });
            }
            '~' | '^' => {
                let start = i;
                chars.next();
                if chars.peek().map(|&(_, c)| c) == Some('-') { chars.next(); }
                while chars.peek().map(|&(_, c)| c.is_ascii_digit()) == Some(true) { chars.next(); }
                if chars.peek().map(|&(_, c)| c) == Some('.') {
                    chars.next();
                    while chars.peek().map(|&(_, c)| c.is_ascii_digit()) == Some(true) { chars.next(); }
                }
                let end = chars.peek().map(|&(j, _)| j).unwrap_or(trimmed.len());
                segments.push(StyleSegment { start: base + indent + start, end: base + indent + end, style: coord_style() });
            }
            '"' | '\'' => {
                let quote = ch;
                let start = i;
                chars.next();
                while let Some(&(_, c)) = chars.peek() { chars.next(); if c == quote { break; } }
                let end = chars.peek().map(|&(j, _)| j).unwrap_or(trimmed.len());
                segments.push(StyleSegment { start: base + indent + start, end: base + indent + end, style: string_style() });
            }
            '{' => {
                let start = i;
                let mut depth = 0;
                while let Some(&(_, c)) = chars.peek() {
                    chars.next();
                    if c == '{' { depth += 1; } else if c == '}' { depth -= 1; if depth == 0 { break; } }
                }
                let end = chars.peek().map(|&(j, _)| j).unwrap_or(trimmed.len());
                segments.push(StyleSegment { start: base + indent + start, end: base + indent + end, style: nbt_style() });
            }
            c if c.is_ascii_digit() || (c == '-' && i > 0) => {
                let start = i;
                if c == '-' { chars.next(); }
                while chars.peek().map(|&(_, c2)| c2.is_ascii_digit()) == Some(true) { chars.next(); }
                if chars.peek().map(|&(_, c2)| c2) == Some('.') {
                    chars.next();
                    while chars.peek().map(|&(_, c2)| c2.is_ascii_digit()) == Some(true) { chars.next(); }
                }
                let end = chars.peek().map(|&(j, _)| j).unwrap_or(trimmed.len());
                segments.push(StyleSegment { start: base + indent + start, end: base + indent + end, style: number_style() });
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while let Some(&(_, c2)) = chars.peek() {
                    if c2.is_alphanumeric() || c2 == '_' { chars.next(); } else { break; }
                }
                let end = chars.peek().map(|&(j, _)| j).unwrap_or(trimmed.len());
                let style = if start == 0 || trimmed[..start].trim().is_empty() { cmd_style() } else { Style::default() };
                segments.push(StyleSegment { start: base + indent + start, end: base + indent + end, style });
            }
            _ => { chars.next(); }
        }
    }
}

fn comment_style() -> Style { Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC) }
fn cmd_style() -> Style { Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD) }
fn selector_style() -> Style { Style::default().fg(Color::Yellow) }
fn string_style() -> Style { Style::default().fg(Color::Green) }
fn number_style() -> Style { Style::default().fg(Color::Yellow) }
fn coord_style() -> Style { Style::default().fg(Color::Cyan) }
fn nbt_style() -> Style { Style::default().fg(Color::LightBlue) }

struct BrainfuckHighlighter;

impl CodeHighlighter for BrainfuckHighlighter {
    fn highlight(&self, lang: &str, code: &str) -> Vec<StyleSegment> {
        if lang != "brainfuck" && lang != "bf" {
            return Vec::new();
        }

        let mut segments = Vec::new();
        let mut run_start: usize = 0;
        let mut prev: Option<Style> = None;

        for (i, ch) in code.char_indices() {
            let style = bf_char_style(ch);
            if prev != Some(style) {
                if let Some(ps) = prev {
                    segments.push(StyleSegment {
                        start: run_start,
                        end: i,
                        style: ps,
                    });
                }
                run_start = i;
                prev = Some(style);
            }
        }

        if let Some(style) = prev {
            segments.push(StyleSegment {
                start: run_start,
                end: code.len(),
                style,
            });
        }

        segments
    }
}

fn bf_char_style(ch: char) -> Style {
    match ch {
        '>' | '<' => Style::default().fg(Color::Cyan),
        '+' | '-' => Style::default().fg(Color::Green),
        '.' | ',' => Style::default().fg(Color::Yellow),
        '[' | ']' => Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
        _ => Style::default().fg(Color::DarkGray),
    }
}

struct CompositeHighlighter {
    treesitter: TreeSitterHighlighter,
    mcfunction: McfunctionHighlighter,
    brainfuck: BrainfuckHighlighter,
}

impl CodeHighlighter for CompositeHighlighter {
    fn highlight(&self, lang: &str, code: &str) -> Vec<StyleSegment> {
        let segs = self.treesitter.highlight(lang, code);
        if !segs.is_empty() {
            return segs;
        }
        let segs = self.mcfunction.highlight(lang, code);
        if !segs.is_empty() {
            return segs;
        }
        self.brainfuck.highlight(lang, code)
    }
}

struct CodeHooks {
    inner: HighlightHooks,
}

impl RenderHooks for CodeHooks {
    fn render_code_block(
        &self,
        lang: &str,
        content: &str,
    ) -> Option<Vec<ratatui::text::Line<'static>>> {
        self.inner.render_code_block(lang, content)
    }
}

const MARKDOWN_TEMPLATE: &str = r#"
# Syntax Highlighting

This example demonstrates **syntax highlighting** for code blocks using
three different approaches: tree-sitter, manual tokenization, and direct
segment construction.

## Rust (tree-sitter)

```rust
use std::collections::HashMap;

fn word_count(text: &str) -> HashMap<&str, usize> {
    let mut map = HashMap::new();
    for word in text.split_whitespace() {
        *map.entry(word).or_insert(0) += 1;
    }
    map
}
```

## mcfunction (manual tokenizer)

Uses **manual tokenization** to identify Minecraft command tokens:
commands, selectors, coordinates, NBT data, strings, and comments.

```mcfunction
# Teleport all players 10 blocks up
execute as @a at @s run tp ~ ~10 ~

give @p diamond_sword 1
scoreboard players set @a kills 0
fill ~1 ~-1 ~1 ~10 ~-1 ~10 stone
summon zombie ~ ~ ~ {CustomName:'"Bob"',Health:20}
kill @e[type=skeleton,distance=..10]
```

## brainfuck (segments)

Directly constructs `StyleSegment` from character analysis without any
parser framework. Pointer ops are cyan, value ops are green, I/O is
yellow, loops are magenta, and everything else is dimmed.

```brainfuck
[ Hello World ]
++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]
>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
```

```brainfuck
Multiply 3 x 5: result in cell 2
+++            cell 0 = 3
>+++++<        cell 1 = 5
[              loop while cell 1 != 0
  > ++++       add 3 to cell 2
  < -          decrement cell 1
]
>> .           cell 2 = 15
```
"#;

fn main() -> anyhow::Result<()> {
    let composite = Arc::new(CompositeHighlighter {
        treesitter: TreeSitterHighlighter::new(),
        mcfunction: McfunctionHighlighter,
        brainfuck: BrainfuckHighlighter,
    });
    let hooks = HighlightHooks::new(composite, 74);

    let mut terminal = setup_terminal()?;

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76)
        .with_render_hooks(Box::new(CodeHooks { inner: hooks }));
    let blocks = renderer.parse(MARKDOWN_TEMPLATE);
    let lines = renderer.render(&blocks, &theme);
    let mut state = AppState::new(lines.len());

    loop {
        terminal.draw(|f| {
            draw_frame(
                f,
                "Code Highlighting",
                &lines,
                &mut state,
                "\u{2191}\u{2193}/jk scroll \u{00b7} PgUp/PgDn \u{00b7} Home/End \u{00b7} q quit",
            );
        })?;
        if poll_and_handle(&mut state)? {
            break;
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}
