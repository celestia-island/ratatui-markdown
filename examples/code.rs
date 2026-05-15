#[path = "utils/mod.rs"]
mod common;

use std::sync::Arc;

use common::{AppState, Theme, draw_frame, poll_and_handle, setup_terminal, restore_terminal};
use ratatui_markdown::highlight::{
    BrainfuckHighlighter, CodeHighlighter, HighlightHooks, McfunctionHighlighter,
    StyleSegment, TreeSitterHighlighter,
};
use ratatui_markdown::markdown::{MarkdownRenderer, RenderHooks};

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
three different approaches: tree-sitter, pest PEG parsing, and direct
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

## mcfunction (pest)

Uses a **PEG grammar** parsed by pest to identify Minecraft command tokens:
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
