# MarkdownPreview Widget

> The high-level unified widget combining markdown, trees, and actions in one scrollable view.

## Overview

`MarkdownPreview` is the top-level integration widget. It combines markdown rendering, collapsible tree display, and action items into a single `HybridScrollView`. This is the recommended entry point for most applications.

Gated behind the `preview` feature flag (enabled by default, requires `markdown`, `scroll`, `tree`).

## API

```rust
pub struct ActionItem {
    pub id: String,
    pub label: String,
}

impl MarkdownPreview {
    pub fn new() -> Self;

    // Configuration
    pub fn with_strip_frontmatter(self, strip: bool) -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;

    // Content
    pub fn set_content(&mut self, content: &str);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // Tree
    pub fn set_tree(&mut self, tree: Option<CollapsibleTree>);
    pub fn tree_mut(&mut self) -> Option<&mut CollapsibleTree>;
    pub fn has_tree(&self) -> bool;
    pub fn toggle_tree_node(&mut self) -> bool;

    // Action items
    pub fn set_action_items(&mut self, items: Vec<ActionItem>);
    pub fn selected_action_id(&self) -> Option<&str>;

    // Navigation
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // State
    pub fn total_lines(&self) -> usize;
    pub fn scroll_offset(&self) -> usize;
    pub fn visible_height(&self) -> usize;
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;

    // Rendering
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

## Configuration

### with_strip_frontmatter

When enabled (default), content delimited by `+++` is treated as TOML frontmatter and stripped before rendering:

```
+++
title = "My Document"
author = "Jane Doe"
+++

# Actual Content
This is the body.
```

Only `# Actual Content` and the following paragraph would appear in the output.

### with_left_padding

Adds 1 column of left padding to the rendered content (passed through to `HybridScrollView`).

## Content Layout

The widget renders content in vertical order:

1. **Tree** (if present) — tree lines with focusable nodes
2. **Markdown** — parsed and rendered content
3. **Action Items** — focusable action labels with bracket wrappers `[label]`

A blank line is inserted between sections, and each section gets its own `FocusableRegion` in the scroll view.

## Caching

`MarkdownPreview` caches rendered output and only rebuilds when:

- Content changes (`set_content` with different text)
- Width changes (terminal resize)
- Theme generation changes (`theme.generation()` returns a new value)
- Tree is modified (`set_tree`, `toggle_tree_node`)
- Action items are modified (`set_action_items`)

Use `theme.generation()` to trigger a re-render after theme changes.

## TOML Frontmatter Handling

The frontmatter is assumed to be TOML. It is **not parsed** — it is simply removed from the rendered output. The first `+++` line starts frontmatter mode, the second `+++` ends it. Lines before and after the frontmatter block are rendered normally.

If the content does not start with `+++`, no stripping occurs.

## Action Items

Action items provide keyboard-selectable options at the bottom of the view:

```rust
preview.set_action_items(vec![
    ActionItem { id: "confirm".into(), label: " Confirm ".into() },
    ActionItem { id: "cancel".into(), label: " Cancel ".into() },
]);

// In your input handler:
if let Some("confirm") = preview.selected_action_id() {
    // handle confirm
}
```

Action item IDs are prefixed with `action:` internally to avoid collisions with tree node paths.

## Example

```rust
use ratatui_markdown::preview::{MarkdownPreview, ActionItem};
use ratatui_markdown::tree::CollapsibleTree;

let mut preview = MarkdownPreview::new()
    .with_strip_frontmatter(true)
    .with_left_padding(true);

// Set markdown content (with TOML frontmatter)
preview.set_content(concat!(
    "+++\n",
    "title = \"My Doc\"\n",
    "+++\n",
    "\n",
    "# Hello\n\nContent here.\n",
));

// Set a tree
let tree = CollapsibleTree::from_json_str(r#"{"config": {"theme": "dark"}}"#).unwrap();
preview.set_tree(Some(tree));

// Set action items
preview.set_action_items(vec![
    ActionItem { id: "edit".into(), label: " Edit ".into() },
    ActionItem { id: "save".into(), label: " Save ".into() },
]);

// In your ratatui app's render function:
fn render_ui(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}

// Handle input
fn handle_input(key: rataui::crossterm::event::KeyCode, preview: &mut MarkdownPreview) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => preview.scroll_up(),
        KeyCode::Down | KeyCode::Char('j') => preview.scroll_down(),
        KeyCode::PageUp => preview.page_up(20),
        KeyCode::PageDown => preview.page_down(20),
        KeyCode::Home => preview.scroll_to_top(),
        KeyCode::End => preview.scroll_to_bottom(),
        KeyCode::Enter => { preview.toggle_tree_node(); }
        _ => {}
    }
}
```
