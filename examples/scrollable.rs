#[path = "_common/mod.rs"]
mod common;

use common::{AppState, Theme, draw_frame, poll_and_handle, setup_terminal, restore_terminal, lorem};
use ratatui_markdown::markdown::MarkdownRenderer;

const MARKDOWN_TEMPLATE: &str = r#"
# Scrollable Markdown Panel

This example demonstrates **vertical scrolling** with a scrollbar,
mouse wheel support, and keyboard navigation through a long document.

## Features

- Vertical scrollbar (right edge) using precise thumb-size formula
- Mouse wheel support (scrolls 3 lines per tick)
- Keyboard navigation (`j`/`k`, arrows, Page Up/Down, Home/End)
- Configurable padding inside the bordered frame
- 1px left/right content padding

## Long Lines Test

This is a deliberately long line that exceeds normal terminal width to demonstrate how pre-wrapped text is handled within the padded content area. The renderer wraps text at creation time so the scrollbar always reflects the true document height.

LOREM_4

## Code Block

```rust
fn render_scrollable(
    f: &mut Frame,
    area: Rect,
    lines: Vec<Line>,
    scroll: &mut AppState,
) {
    draw_frame(f, "Title", &lines, scroll, "hints");
}
```

## Nested List

- Level 0 Item 1
  - Level 1 Item A
    - Level 2 Item i
    - Level 2 Item ii
    - Level 2 Item iii (long text here to push width)
  - Level 1 Item B
  - Level 1 Item C
- Level 0 Item 2
  - Level 1 Item D
  - Level 1 Item E
- Level 0 Item 3
- Level 0 Item 4
- Level 0 Item 5
- Level 0 Item 6
- Level 0 Item 7
- Level 0 Item 8
- Level 0 Item 9
- Level 0 Item 10

## Blockquote

> This is a blockquote that contains enough text to potentially wrap across multiple lines, demonstrating how the scrollable panel handles wrapped content within a padded viewport area.

LOREM_5

## Table

| Feature | Status | Notes |
|---------|--------|-------|
| Parser  | Done   | Full pulldown-cmark coverage |
| Renderer| Done   | Pre-wrapped line output |
| Hooks   | Done   | Customizable per-element |
| Scroll  | Done   | Mouse + keyboard |
| Padding | Done   | 1px left/right |

LOREM_3

## Horizontal Rule

---

LOREM_4
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
                "Scrollable Panel",
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
