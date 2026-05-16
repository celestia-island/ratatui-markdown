#[path = "utils/mod.rs"]
mod common;

use common::{
    draw_frame, lorem, poll_and_handle, restore_terminal, setup_terminal, AppState, Theme,
};
use ratatui_markdown::markdown::MarkdownRenderer;

const MARKDOWN_TEMPLATE: &str = r#"
# Getting Started

This is a **basic** markdown rendering example using `ratatui-markdown`.

## Features

- Headings (H1, H2, H3)
- **Bold**, *italic*, and `inline code`
- Code blocks with syntax labels
- Blockquotes
- Tables
- Tree-style lists (see `tree_list` example)
- Image rendering (see `image` example)

### Code Example

```rust
fn main() {
    println!("Hello, world!");
}
```

## Overview

LOREM_4

### Table

| Feature | Status |
|---------|--------|
| Parser  | Done   |
| Renderer| Done   |
| Hooks   | Done   |
| Mermaid | WIP    |

> This is a blockquote. It supports *inline formatting* too.
> Multiple lines of quoted text demonstrate wrapping behavior
> inside the blockquote rendering pipeline.

## Details

LOREM_5

### Another Code Block

```python
def fibonacci(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a
```

LOREM_3
"#;

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;

    let md = MARKDOWN_TEMPLATE
        .replace("LOREM_3", &lorem(150))
        .replace("LOREM_4", &lorem(200))
        .replace("LOREM_5", &lorem(250));

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76);
    let blocks = renderer.parse(&md);
    let lines = renderer.render(&blocks, &theme);
    let mut state = AppState::new(lines.len());

    loop {
        terminal.draw(|f| {
            draw_frame(
                f,
                "Basic Markdown",
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
