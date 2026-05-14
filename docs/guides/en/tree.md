# Tree View

> Interactive collapsible tree for JSON and TOML data.

## Overview

The `tree` module parses JSON or TOML into an interactive collapsible tree. Users can expand/collapse nodes and navigate with keyboard in a terminal UI.

Gated behind the `tree` feature flag (requires `scroll`, `serde_json`, and `toml` dependencies).

## CollapsibleTree

```rust
pub struct CollapsibleTree { /* fields */ }

impl CollapsibleTree {
    // Constructors
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error>;
    pub fn from_toml_str(toml: &str) -> Result<Self, Box<dyn Error>>;
    pub fn from_value(value: serde_json::Value, style: KeyStyle) -> Self;

    // Tree manipulation
    pub fn toggle(&mut self, path: &str) -> bool;
    pub fn handle_toggle(&mut self, id: &str) -> bool;
    pub fn expand_all(&mut self);
    pub fn collapse_all(&mut self);
    pub fn expand_to_depth(&mut self, depth: usize);

    // Rendering
    pub fn render_lines(&self, width: usize, theme: &impl RichTextTheme) -> Vec<Line<'static>>;
    pub fn flatten(&self) -> Vec<FlatEntry>;
    pub fn build_focusable_items(&self) -> Vec<FocusableItemRange>;
}
```

### Constructors

- **`from_json_str`**: Parses a JSON string into a tree. Keys use `KeyStyle::Json` (quoted, with `:` separator).
- **`from_toml_str`**: Parses a TOML string (converts internally to JSON). Keys use `KeyStyle::Toml` (unquoted, with `=` separator).
- **`from_value`**: Builds a tree from an existing `serde_json::Value` with a chosen key style.

### Tree Manipulation

```rust
// Toggle using a slash-separated path
tree.toggle("dependencies");          // toggle root key
tree.toggle("dependencies/serde");    // toggle nested key

// Convenience methods
tree.expand_all();
tree.collapse_all();
tree.expand_to_depth(2);

// In a scroll context — uses the selected item's ID
if scroll.selected_item_id().is_some() {
    tree.handle_toggle(selected_id);
}
```

### Rendering

- **`render_lines`**: Produces styled `Line`s with tree connectors and colored values.
- **`flatten`**: Returns a flat list of all visible entries (respects collapsed state).
- **`build_focusable_items`**: Returns focusable ranges for `HybridScrollView` integration, with IDs matching tree paths.

## Data Types

```rust
pub struct FlatEntry {
    pub depth: usize,
    pub key: String,
    pub value: String,
    pub kind: EntryKind,
    pub value_type: ValueType,
    pub path: String,          // e.g. "dependencies/serde"
    pub is_last: bool,
}

pub enum EntryKind {
    Collapsed,  // has children, currently collapsed: [+]
    Expanded,   // has children, currently expanded: [-]
    Leaf,       // no children
}

pub enum ValueType {
    String, Number, Boolean, Null, Object, Array,
}

pub enum KeyStyle {
    Json,  // "key": value
    Toml,  // key = value
}
```

### Value Type Colors

Each `ValueType` maps to a corresponding theme color method:

| ValueType | Theme Method          |
|-----------|-----------------------|
| String    | `get_json_string_color()` |
| Number    | `get_json_number_color()` |
| Boolean   | `get_json_bool_color()`   |
| Null      | `get_json_null_color()`   |

## Tree Line Helpers

The `tree_lines` module (re-exported from `crate::tree`) provides low-level line construction:

```rust
pub fn make_body_line(key: &str, value: &str, style: &Style, value_style: &Style) -> Line<'static>;
pub fn make_status_line(status: &str, style: &Style) -> Line<'static>;
pub fn make_branch_dispatch_line(kind: EntryKind, ...) -> Line<'static>;
```

## Example

```rust
use ratatui_markdown::tree::CollapsibleTree;

let toml_content = r#"
[package]
name = "my-app"
version = "0.1.0"

[dependencies]
ratatui = "0.29"
serde = { version = "1.0", features = ["derive"] }

[features]
default = ["std"]
std = []
"#;

let mut tree = CollapsibleTree::from_toml_str(toml_content).unwrap();

// Expand everything
tree.expand_all();

// Render to lines
let lines = tree.render_lines(80, theme);

// Collapse the dependencies subtree
tree.toggle("dependencies");

// Re-render — dependencies are now collapsed
let lines = tree.render_lines(80, theme);

// Get focusable items for scroll navigation
let items = tree.build_focusable_items();
```
