#[path = "utils/mod.rs"]
mod common;

use common::{
    draw_frame, lorem, poll_and_handle, restore_terminal, setup_terminal, AppState, Theme,
};
use ratatui_markdown::markdown::MarkdownRenderer;

const MARKDOWN_TEMPLATE: &str = r#"
# Mermaid Diagrams

This example renders **Mermaid diagrams** inline in markdown
using the `mermaid` feature of `ratatui-markdown`.

## Flowchart (TD)

```mermaid
graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Action A]
    B -->|No| D[Action B]
    C --> E[End]
    D --> E
```

LOREM_2

## Flowchart (LR)

```mermaid
graph LR
    Input --> Parser
    Parser --> AST
    AST --> Renderer
    Renderer --> Output
```

LOREM_2

## Complex Flowchart

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

## Sequence Diagram

```mermaid
sequenceDiagram
    participant Client
    participant Server
    participant DB
    Client->>Server: HTTP Request
    Server->>DB: Query
    DB-->>Server: Result
    Server-->>Client: Response
```

LOREM_2

## Pie Chart

```mermaid
pie title Technology Stack
    "Rust" : 40
    "TypeScript" : 25
    "Python" : 20
    "Go" : 15
```

LOREM_2

## Gantt Chart

```mermaid
gantt
    title Project Timeline
    section Planning
        Requirements :a1, 7d
        Design       :a2, after a1, 5d
    section Development
        Frontend     :a3, after a2, 10d
        Backend      :a4, after a2, 12d
    section Testing
        QA Testing   :a5, after a3, 5d
```

LOREM_3

## State Diagram

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Processing : submit
    Processing --> Success : ok
    Processing --> Failed : error
    Success --> Idle : reset
    Failed --> Idle : retry
```

LOREM_3
"#;

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;

    let md = MARKDOWN_TEMPLATE
        .replace("LOREM_2", &lorem(100))
        .replace("LOREM_3", &lorem(150));

    let theme = Theme;
    let content_width = terminal.size()?.width.saturating_sub(4) as usize;
    let renderer = MarkdownRenderer::new(content_width);
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
