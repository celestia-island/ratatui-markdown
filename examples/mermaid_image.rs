#[path = "utils/mod.rs"]
mod common;

use common::{
    draw_frame, lorem, poll_and_handle, restore_terminal, setup_terminal, AppState, Theme,
};
use ratatui_markdown::markdown::MarkdownRenderer;

const MARKDOWN_TEMPLATE: &str = r#"
# Mermaid Image Rendering

This demo renders mermaid diagrams as **actual images** (PNG) via the
`mmdc` CLI, then displays them inline using `ratatui-image`.

If `mmdc` is not installed, diagrams fall back to TUI character art.

## Cache Flow

```mermaid
graph TD
    A{Cache Hit?} -->|No| B[Compute Result]
    B --> C[Update Cache]
    A -->|Yes| D[Return Cached]
    C --> D
    D --> E[Response]
```

LOREM_2

## Pipeline

```mermaid
graph LR
    Input --> Parser --> AST --> Renderer --> Output
```

LOREM_2

## Pie Chart

```mermaid
pie title Languages
    "Rust" : 40
    "TypeScript" : 25
    "Python" : 20
    "Go" : 15
```

LOREM_2

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

LOREM_3
"#;

fn render_mermaid_to_image(
    source: &str,
    cache_dir: &std::path::Path,
) -> Option<image::DynamicImage> {
    use std::process::Command;

    let hash = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    };

    let cached = cache_dir.join(format!("mermaid_{hash}.png"));
    if cached.exists() {
        if let Ok(reader) = image::ImageReader::open(&cached) {
            if let Ok(img) = reader.decode() {
                return Some(img);
            }
        }
    }

    let mmd = cache_dir.join(format!("mermaid_{hash}.mmd"));
    std::fs::write(&mmd, source).ok()?;

    let status = Command::new("mmdc")
        .args([
            "-i",
            mmd.to_str()?,
            "-o",
            cached.to_str()?,
            "-w",
            "800",
            "-b",
            "transparent",
        ])
        .output()
        .ok()?;

    if !status.status.success() {
        return None;
    }

    image::ImageReader::open(&cached).ok()?.decode().ok()
}

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

    let cache_dir = std::env::temp_dir().join("ratatui-mermaid-image-cache");
    std::fs::create_dir_all(&cache_dir).ok();

    loop {
        terminal.draw(|f| {
            draw_frame(
                f,
                "Mermaid Image Rendering",
                &lines,
                &mut state,
                "↑↓/jk scroll · PgUp/PgDn · Home/End · q quit",
            );
        })?;
        if poll_and_handle(&mut state)? {
            break;
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}
