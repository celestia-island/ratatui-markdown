# Scroll System

> Smart hybrid scrolling with focusable item navigation.

## Overview

The `scroll` module provides a hybrid scrolling system that supports two modes:

1. **Free-scroll** — when no focusable items are in view, content scrolls freely
2. **Engaged** — when focusable items enter the viewport center, the cursor latches onto the first item for keyboard navigation

Gated behind the `scroll` feature flag (enabled by default).

## HybridScrollView

The core widget that manages scrolling, focus regions, and rendering:

```rust
pub struct HybridScrollView { /* fields */ }

impl HybridScrollView {
    pub fn new() -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;
    pub fn with_cursor_indicator(self, show: bool) -> Self;

    // Content management
    pub fn set_content(&mut self, lines: Vec<Line<'static>>, regions: Vec<FocusableRegion>);
    pub fn set_lines(&mut self, lines: Vec<Line<'static>>);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // Scroll state
    pub fn total_lines(&self) -> usize;
    pub fn get_scroll_offset(&self) -> usize;
    pub fn get_viewport_height(&self) -> usize;
    pub fn set_scroll_offset(&mut self, offset: usize);

    // Navigation
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // Engagement
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;
    pub fn engage_first(&mut self);
    pub fn engage_by_id(&mut self, id: &str) -> bool;

    // Rendering
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

### Configuration

- **with_left_padding**: Adds 1 column of left padding to all displayed lines
- **with_cursor_indicator**: Shows `> ` (2 columns) on the engaged cursor line (takes precedence over `left_padding`)

The `effective_padding()` method returns the actual padding used:
- `2` if cursor indicator is enabled
- `1` if left padding only
- `0` otherwise

## Focusable Regions and Items

```rust
pub struct FocusableItemRange {
    pub start_line: usize,    // inclusive
    pub end_line: usize,      // exclusive
    pub id: String,           // unique identifier
}

pub struct FocusableRegion {
    pub items: Vec<FocusableItemRange>,
}
```

Regions define spans of lines that become interactive. When the viewport center passes over a region, the scroll view auto-engages and the cursor lands on the first item.

### Engagement Behavior

- Scrolling **down** into a region engages on the **first** item
- Scrolling **up** into a region engages on the **last** item
- Navigating past the last item in a region **disengages** and returns to free-scroll
- `scroll_to_top()` and `scroll_to_bottom()` always disengage
- Within a region, `scroll_up`/`scroll_down` move the cursor between items

## Other Scroll Widgets

### ScrollableList<T>

A generic scrollable list with mouse/keyboard navigation and optional bordered rendering:

```rust
pub trait ListItemRenderer {
    fn render_item(&self, index: usize, is_selected: bool, width: usize) -> Line<'static>;
}
```

### ArrowScrollbar

A custom scrollbar drawn with Unicode arrow symbols:

```rust
pub fn render_arrow_scrollbar(
    area: Rect,
    buf: &mut Buffer,
    top: usize,
    bottom: usize,
    theme: &impl RichTextTheme,
);
```

### FollowScrollState

For auto-following content (e.g., streaming output):

```rust
pub struct FollowScrollState {
    // tracks whether the viewport is at the bottom
}
```

### ScrollableRenderResult

A simple scrollable panel wrapper:

```rust
pub fn render_scrollable(
    lines: &[Line],
    scroll_offset: usize,
    area: Rect,
    buf: &mut Buffer,
) -> ScrollableRenderResult;
```

## Example

```rust
use ratatui_markdown::scroll::{HybridScrollView, FocusableItemRange, FocusableRegion};

let mut scroll = HybridScrollView::new()
    .with_cursor_indicator(true);

// Content lines
let lines: Vec<Line> = (0..100)
    .map(|i| Line::raw(format!("Line {}", i)))
    .collect();

// Make lines 30-32 focusable
let region = FocusableRegion {
    items: vec![
        FocusableItemRange { start_line: 30, end_line: 31, id: "item-a".into() },
        FocusableItemRange { start_line: 31, end_line: 32, id: "item-b".into() },
        FocusableItemRange { start_line: 32, end_line: 33, id: "item-c".into() },
    ],
};

scroll.set_content(lines, vec![region]);

// Free-scroll down; auto-engages when line 30 enters viewport center
scroll.scroll_down();

// Navigate through items
while scroll.scroll_down() == InputResult::Continue {
    if let Some(id) = scroll.selected_item_id() {
        println!("Selected: {}", id);
    }
}
```
