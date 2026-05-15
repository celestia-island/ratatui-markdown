#[path = "_common/mod.rs"]
mod common;

use common::{AppState, Theme, draw_frame, poll_and_handle, setup_terminal, restore_terminal};
use ratatui_markdown::{
    constants::{BRANCH_END_SP, BRANCH_MID_SP, VLINE},
    markdown::{MarkdownRenderer, RenderHooks},
};

struct TreeListHooks;

impl RenderHooks for TreeListHooks {
    fn list_item_marker(
        &self,
        indent: u8,
        is_last_in_group: bool,
        ancestors_are_last: &[bool],
        _index_in_group: usize,
    ) -> Option<String> {
        let unit: usize = Self::tree_indent_unit(self).unwrap_or(3);
        let connector = if is_last_in_group {
            BRANCH_END_SP
        } else {
            BRANCH_MID_SP
        };
        if indent == 0 {
            return Some(connector.to_string());
        }
        let mut prefix = String::new();
        for (i, &is_last_anc) in ancestors_are_last.iter().enumerate() {
            if i >= indent as usize {
                break;
            }
            if is_last_anc {
                for _ in 0..unit {
                    prefix.push(' ');
                }
            } else {
                prefix.push_str(VLINE);
                for _ in 1..unit {
                    prefix.push(' ');
                }
            }
        }
        if indent as usize > ancestors_are_last.len() {
            let extra = indent as usize - ancestors_are_last.len();
            for _ in 0..unit * extra {
                prefix.push(' ');
            }
        }
        Some(format!("{prefix}{connector}"))
    }

    fn tree_indent_unit(&self) -> Option<usize> {
        Some(3)
    }

    fn tree_continuation_prefix(
        &self,
        indent: u8,
        ancestors_are_last: &[bool],
    ) -> Option<String> {
        let unit: usize = Self::tree_indent_unit(self).unwrap_or(3);
        let mut prefix = String::new();
        for (i, &is_last_anc) in ancestors_are_last.iter().enumerate() {
            if i >= indent as usize {
                break;
            }
            if is_last_anc {
                for _ in 0..unit {
                    prefix.push(' ');
                }
            } else {
                prefix.push_str(VLINE);
                for _ in 1..unit {
                    prefix.push(' ');
                }
            }
        }
        for _ in 0..unit {
            prefix.push(' ');
        }
        Some(prefix)
    }
}

const MARKDOWN: &str = r#"
## Project TODO

- Setup project structure
  - Initialize Cargo workspace with members for core and crates
  - Add dependencies
    - ratatui for terminal UI rendering and display management
    - image crate for image support and protocol handling
    - crossterm for crossplatform terminal event handling
- Implement core features
  - Parser
    - Heading detection with nested component extraction logic
    - Code block parsing with language-aware fence matching rules
    - Image syntax for embedded visual content rendering pipeline
  - Renderer
    - Inline formatting with bold italic and code spans
    - Code block borders using rounded box drawing characters
    - Text wrapping engine that respects word boundaries and CJK width
  - Hooks system
    - RenderHooks trait for customizable list item markers and styling
    - Theme hooks for dynamic color palette switching at runtime
- Write tests
  - Unit tests for parser edge cases and corner conditions
  - Integration tests for full markdown document rendering pipeline
  - Visual regression tests for tree layout and wrap behavior verification
- Deploy to crates.io
  - Write comprehensive documentation with usage examples
  - Publish stable release with semver version bump
  - Create GitHub Actions CI pipeline for automated testing across platforms
- Community
  - Write contributing guidelines and code of conduct
  - Set up issue templates for bug reports and feature requests
  - Create example gallery showcasing different rendering configurations
  - Record screencast demos for README and documentation site
"#;

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76)
        .with_render_hooks(Box::new(TreeListHooks));
    let blocks = renderer.parse(MARKDOWN);
    let lines = renderer.render(&blocks, &theme);
    let mut state = AppState::new(lines.len());

    loop {
        terminal.draw(|f| {
            draw_frame(
                f,
                "Tree-Style List",
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
