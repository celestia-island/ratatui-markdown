#[path = "_common/mod.rs"]
mod common;

use common::{AppState, Theme, draw_frame, poll_and_handle, setup_terminal, restore_terminal, lorem};
use ratatui_markdown::markdown::MarkdownRenderer;

const MARKDOWN_TEMPLATE: &str = r#"
# Mermaid Diagrams

This example renders **Mermaid flowcharts** inline in markdown
using the `mermaid` feature of `ratatui-markdown`.

## Simple Flow

```mermaid
graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Action A]
    B -->|No| D[Action B]
    C --> E[End]
    D --> E
```

The diagram above shows a basic top-down flowchart with a decision node
and two branches converging on an end state.

LOREM_2

## Left-to-Right Flow

```mermaid
graph LR
    Input --> Parser
    Parser --> AST
    AST --> Renderer
    Renderer --> Output
```

LOREM_2

## Complex Flow

```mermaid
graph TD
    A[User Request] --> B[Auth Check]
    B -->|Authorized| C[Route Handler]
    B -->|Denied| D[403 Error]
    C --> E[Business Logic]
    E --> F{Cache Hit?}
    F -->|Yes| G[Return Cached]
    F -->|No| H[Compute Result]
    H --> I[Update Cache]
    I --> G
    G --> J[Response]
```

LOREM_3

## Another Example

```mermaid
graph LR
    Client --> Gateway
    Gateway --> ServiceA
    Gateway --> ServiceB
    ServiceA --> DB
    ServiceB --> DB
    ServiceA --> Cache
```

LOREM_3
"#;

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;

    let md = MARKDOWN_TEMPLATE
        .replace("LOREM_2", &lorem(2))
        .replace("LOREM_3", &lorem(3));

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76);
    let blocks = renderer.parse(&md);
    let lines = renderer.render(&blocks, &theme);
    let mut state = AppState::new(lines.len());

    loop {
        terminal.draw(|f| {
            draw_frame(
                f,
                "Mermaid Diagrams",
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
