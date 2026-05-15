#[path = "utils/mod.rs"]
mod common;

use common::{AppState, Theme, draw_frame, lorem, poll_and_handle, setup_terminal, restore_terminal};
use ratatui_markdown::{
    constants::{BRANCH_END_SP, BRANCH_FIRST_SP, BRANCH_MID_SP, VLINE},
    markdown::{MarkdownRenderer, RenderHooks},
};

struct TreeListHooks;

impl RenderHooks for TreeListHooks {
    fn list_item_marker(
        &self,
        indent: u8,
        is_last_in_group: bool,
        ancestors_are_last: &[bool],
        index_in_group: usize,
    ) -> Option<String> {
        let unit: usize = Self::tree_indent_unit(self).unwrap_or(3);
        let connector = if is_last_in_group {
            BRANCH_END_SP
        } else if indent == 0 && index_in_group == 0 {
            BRANCH_FIRST_SP
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

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn take(words: &[String], wi: &mut usize, n: usize) -> String {
    let end = (*wi + n).min(words.len());
    let phrase: Vec<&str> = words[*wi..end].iter().map(|s| s.as_str()).collect();
    *wi = end;
    phrase.join(" ")
}

fn build_list(items: &[(usize, String)], out: &mut String) {
    for (indent, text) in items {
        let spaces = "  ".repeat(*indent);
        out.push_str(&format!("{}- {}\n", spaces, text));
    }
}

fn generate_tree_markdown() -> String {
    let raw = lipsum::lipsum(160);
    let words: Vec<String> = raw.split_whitespace().map(|w| capitalize(w)).collect();
    let mut wi = 0;

    let mut t = |n: usize| -> String { take(&words, &mut wi, n) };

    let mut md = String::from(
        "# Tree-Style List Example\n\n\
         This example demonstrates nested list rendering with tree-style\n\
         branch connectors using `RenderHooks`.\n\n\
         ## Project TODO\n\n",
    );

    build_list(&[
        (0, t(8)),
        (1, t(6)),
        (1, t(6)),
        (2, t(5)),
        (2, t(5)),
        (2, t(5)),
        (0, t(8)),
        (1, t(6)),
        (2, t(6)),
        (2, t(6)),
        (2, t(6)),
        (1, t(6)),
        (2, t(6)),
        (2, t(6)),
        (2, t(6)),
        (1, t(6)),
        (2, t(5)),
        (2, t(5)),
        (0, t(6)),
        (1, t(6)),
        (1, t(6)),
        (1, t(6)),
        (0, t(6)),
        (1, t(5)),
        (1, t(5)),
        (1, t(5)),
        (0, t(6)),
        (1, t(5)),
        (1, t(5)),
        (1, t(5)),
        (1, t(5)),
        (0, t(6)),
        (1, t(5)),
        (1, t(5)),
        (1, t(5)),
    ], &mut md);

    md.push_str("\n\n");
    md.push_str(&lorem(60));
    md.push_str("\n\n## Additional Notes\n\n");

    let mut wi2 = 0;
    let mut t2 = |n: usize| -> String { take(&words, &mut wi2, n) };

    build_list(&[
        (0, t2(8)),
        (1, t2(6)),
        (1, t2(6)),
        (1, t2(6)),
        (0, t2(8)),
        (1, t2(6)),
        (1, t2(6)),
        (0, t2(8)),
        (1, t2(5)),
        (1, t2(5)),
    ], &mut md);

    md.push('\n');
    md.push_str(&lorem(150));
    md
}

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;

    let md = generate_tree_markdown();
    let theme = Theme;
    let renderer = MarkdownRenderer::new(76)
        .with_render_hooks(Box::new(TreeListHooks));
    let blocks = renderer.parse(&md);
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
