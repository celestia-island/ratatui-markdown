# Theme Customization

> Implementing custom color schemes via the `RichTextTheme` trait.

## Overview

All color and style lookups go through the `RichTextTheme` trait, keeping rendering logic decoupled from appearance. Implement this trait once and pass it to all render functions.

## RichTextTheme Trait

```rust
pub trait RichTextTheme {
    fn generation(&self) -> Generation;

    // General text
    fn get_text_color(&self) -> Color;
    fn get_muted_text_color(&self) -> Color;
    fn get_background_color(&self) -> Color;

    // Accent / emphasis
    fn get_primary_color(&self) -> Color;
    fn get_secondary_color(&self) -> Color;
    fn get_info_color(&self) -> Color;

    // Borders
    fn get_border_color(&self) -> Color;
    fn get_focused_border_color(&self) -> Color;

    // Popups / overlays
    fn get_popup_selected_background(&self) -> Color;
    fn get_popup_selected_text_color(&self) -> Color;

    // JSON/Tree values
    fn get_json_key_color(&self) -> Color;
    fn get_json_string_color(&self) -> Color;
    fn get_json_number_color(&self) -> Color;
    fn get_json_bool_color(&self) -> Color;
    fn get_json_null_color(&self) -> Color;

    // Misc
    fn get_accent_yellow(&self) -> Color;
}
```

## Generation Token

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Generation(pub u64);

impl Generation {
    pub fn next(&self) -> Self { Self(self.0 + 1) }
}
```

The `Generation` token is used by `MarkdownPreview` for cache invalidation. When the theme's `generation()` returns a different value than the cached one, the preview rebuilds all rendered output.

**Increment the generation whenever the user changes theme** at runtime:

```rust
struct AppTheme {
    generation: Generation,
    dark_mode: bool,
}

impl RichTextTheme for AppTheme {
    fn generation(&self) -> Generation { self.generation }

    fn get_text_color(&self) -> Color {
        if self.dark_mode { Color::White } else { Color::Black }
    }

    // ...
}

impl AppTheme {
    fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
        self.generation = self.generation.next();
    }
}
```

## Color Slots Usage

Here's how each theme color is used in practice:

| Method | Used For |
|--------|----------|
| `get_text_color` | Default paragraph text, list item text |
| `get_muted_text_color` | Code blocks, blockquote text, table borders |
| `get_primary_color` | Headings (H1-H3), bold text accent |
| `get_secondary_color` | Inline code, secondary accents |
| `get_info_color` | Info-level highlights |
| `get_background_color` | General background fills |
| `get_border_color` | Unfocused borders (code blocks, tables) |
| `get_focused_border_color` | Focused/engaged item borders, scrollbar |
| `get_popup_selected_background` | Background of selected popup items |
| `get_popup_selected_text_color` | Text color of selected popup items |
| `get_json_key_color` | Tree node keys |
| `get_json_string_color` | String values in trees |
| `get_json_number_color` | Numeric values in trees |
| `get_json_bool_color` | Boolean values in trees |
| `get_json_null_color` | Null values in trees |
| `get_accent_yellow` | Warning/highlight accents |

## Example: Complete Theme

```rust
use ratatui::style::Color;
use ratatui_markdown::theme::{Generation, RichTextTheme};

struct CatppuccinMocha;

impl RichTextTheme for CatppuccinMocha {
    fn generation(&self) -> Generation { Generation(0) }

    fn get_text_color(&self) -> Color { Color::new(205, 214, 244) }       // text
    fn get_muted_text_color(&self) -> Color { Color::new(147, 153, 178) } // overlay2
    fn get_background_color(&self) -> Color { Color::new(30, 30, 46) }    // base
    fn get_primary_color(&self) -> Color { Color::new(137, 180, 250) }    // blue
    fn get_secondary_color(&self) -> Color { Color::new(203, 166, 247) }  // mauve
    fn get_info_color(&self) -> Color { Color::new(137, 220, 235) }       // sky
    fn get_border_color(&self) -> Color { Color::new(69, 71, 90) }        // surface1
    fn get_focused_border_color(&self) -> Color { Color::new(137, 180, 250) } // blue
    fn get_popup_selected_background(&self) -> Color { Color::new(69, 71, 90) }
    fn get_popup_selected_text_color(&self) -> Color { Color::new(205, 214, 244) }
    fn get_json_key_color(&self) -> Color { Color::new(137, 220, 235) }    // sky
    fn get_json_string_color(&self) -> Color { Color::new(166, 227, 161) } // green
    fn get_json_number_color(&self) -> Color { Color::new(250, 179, 135) } // peach
    fn get_json_bool_color(&self) -> Color { Color::new(203, 166, 247) }   // mauve
    fn get_json_null_color(&self) -> Color { Color::new(108, 112, 134) }   // surface2
    fn get_accent_yellow(&self) -> Color { Color::new(249, 226, 175) }     // yellow
}
```

## Performance Notes

- All theme methods are called frequently during rendering. Keep them cheap — return constants or simple field lookups.
- The `Generation` token comparison is a simple integer equality check, so incrementing it is fast.
