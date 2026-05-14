# 主题定制

> 通过 `RichTextTheme` trait 实现自定义配色方案。

## 概述

所有颜色和样式的查找都通过 `RichTextTheme` trait 进行，使渲染逻辑与外观解耦。实现一次该 trait，然后传递给所有渲染函数即可。

## RichTextTheme Trait

```rust
pub trait RichTextTheme {
    fn generation(&self) -> Generation;

    // 通用文本
    fn get_text_color(&self) -> Color;
    fn get_muted_text_color(&self) -> Color;
    fn get_background_color(&self) -> Color;

    // 强调色
    fn get_primary_color(&self) -> Color;
    fn get_secondary_color(&self) -> Color;
    fn get_info_color(&self) -> Color;

    // 边框
    fn get_border_color(&self) -> Color;
    fn get_focused_border_color(&self) -> Color;

    // 弹窗/叠加层
    fn get_popup_selected_background(&self) -> Color;
    fn get_popup_selected_text_color(&self) -> Color;

    // JSON/树值
    fn get_json_key_color(&self) -> Color;
    fn get_json_string_color(&self) -> Color;
    fn get_json_number_color(&self) -> Color;
    fn get_json_bool_color(&self) -> Color;
    fn get_json_null_color(&self) -> Color;

    // 其他
    fn get_accent_yellow(&self) -> Color;
}
```

## Generation 令牌

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Generation(pub u64);

impl Generation {
    pub fn next(&self) -> Self { Self(self.0 + 1) }
}
```

`Generation` 令牌由 `MarkdownPreview` 用于缓存失效。当主题的 `generation()` 返回的值与缓存值不同时，预览组件会重建所有渲染输出。

**在用户运行时切换主题时，递增 generation 值**：

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

## 颜色槽位用途

以下是每种主题颜色在实际中的使用方式：

| 方法 | 用途 |
|--------|----------|
| `get_text_color` | 默认段落文本、列表项文本 |
| `get_muted_text_color` | 代码块、引用块文本、表格边框 |
| `get_primary_color` | 标题 (H1-H3)、粗体文本强调色 |
| `get_secondary_color` | 行内代码、次要强调色 |
| `get_info_color` | 信息级高亮 |
| `get_background_color` | 通用背景填充 |
| `get_border_color` | 未聚焦边框（代码块、表格） |
| `get_focused_border_color` | 聚焦/交互项边框、滚动条 |
| `get_popup_selected_background` | 弹窗中选中项的背景色 |
| `get_popup_selected_text_color` | 弹窗中选中项的文本色 |
| `get_json_key_color` | 树节点键名 |
| `get_json_string_color` | 树中的字符串值 |
| `get_json_number_color` | 树中的数值 |
| `get_json_bool_color` | 树中的布尔值 |
| `get_json_null_color` | 树中的空值 |
| `get_accent_yellow` | 警告/高亮强调色 |

## 示例：完整主题

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

## 性能说明

- 所有主题方法在渲染期间会被频繁调用。请保持它们的开销低廉——返回常量或简单的字段查找即可。
- `Generation` 令牌的比较是简单的整数相等检查，因此递增它非常快速。
