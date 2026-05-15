#[path = "utils/mod.rs"]
mod common;

use common::{AppState, Theme, draw_frame, poll_and_handle, setup_terminal, restore_terminal, lorem};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use ratatui_markdown::markdown::{MarkdownRenderer, RenderHooks};

struct TimelineCodeHooks;

impl RenderHooks for TimelineCodeHooks {
    fn code_block_header(&self, lang: &str) -> Option<Line<'static>> {
        let timestamp = "12:00:00";
        Some(Line::from(vec![
            Span::styled(
                format!("\u{256d} [{timestamp}] "),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                lang.to_string(),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]))
    }

    fn code_block_footer(&self, _lang: &str, _content_line_count: usize) -> Option<Line<'static>> {
        Some(Line::from(vec![
            Span::styled("\u{2570} ", Style::default().fg(Color::DarkGray)),
            Span::styled("\u{2191} ", Style::default().fg(Color::Green)),
            Span::styled("156 ", Style::default().fg(Color::DarkGray)),
            Span::styled("\u{2193} ", Style::default().fg(Color::Red)),
            Span::styled("234 ", Style::default().fg(Color::DarkGray)),
            Span::styled("\u{21c4} ", Style::default().fg(Color::Yellow)),
            Span::styled("1 23.5s", Style::default().fg(Color::DarkGray)),
        ]))
    }

    fn code_block_line_prefix(&self, _lang: &str) -> Option<String> {
        Some("\u{2502} ".to_string())
    }
}

const MARKDOWN_TEMPLATE: &str = r#"
# Timeline View

This example shows customized code block rendering with extra content
in the rounded border header and footer, inspired by a timeline view.

## Agent Skill Execution

```rust skill::read_file
use std::fs;
let content = fs::read_to_string("PLAN.md")?;
println!("{}", content);
```

The header shows a timestamp and tool name, while the footer
displays token usage statistics: \u{2191} output \u{2193} input \u{21c4} roundtrips duration.

## Analysis Block

```python skill::analyze
def analyze(data):
    results = []
    for item in data:
        processed = transform(item)
        results.append(processed)
    return aggregate(results)
```

LOREM_3

## Build Step

```bash skill::build
cargo build --release
echo "Build complete"
cp target/release/app /opt/bin/
```

LOREM_2

## Another Block

```javascript skill::render
function render(components) {
  return components
    .map(c => c.toString())
    .join('\n');
}
```

LOREM_3
"#;

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;

    let md = MARKDOWN_TEMPLATE
        .replace("LOREM_2", &lorem(100))
        .replace("LOREM_3", &lorem(150));

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76)
        .with_render_hooks(Box::new(TimelineCodeHooks));
    let blocks = renderer.parse(&md);
    let lines = renderer.render(&blocks, &theme);
    let mut state = AppState::new(lines.len());

    loop {
        terminal.draw(|f| {
            draw_frame(
                f,
                "Custom Code Block",
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
