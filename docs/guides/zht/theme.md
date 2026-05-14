# 主題自訂

> 透過 `RichTextTheme` 特徵實作自訂色彩配置。

## 概覽

所有顏色和樣式查詢都透過 `RichTextTheme` 特徵進行，將渲染邏輯與外觀解耦。實作此特徵一次，然後傳遞給所有渲染函式。

## RichTextTheme 特徵

```rust
pub trait RichTextTheme {
    fn generation(&self) -> Generation;

    // 一般文字
    fn get_text_color(&self) -> Color;
    fn get_muted_text_color(&self) -> Color;
    fn get_background_color(&self) -> Color;

    // 強調 / 重點
    fn get_primary_color(&self) -> Color;
    fn get_secondary_color(&self) -> Color;
    fn get_info_color(&self) -> Color;

    // 邊框
    fn get_border_color(&self) -> Color;
    fn get_focused_border_color(&self) -> Color;

    // 彈出視窗 / 覆蓋層
    fn get_popup_selected_background(&self) -> Color;
    fn get_popup_selected_text_color(&self) -> Color;

    // JSON / 樹值
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

`Generation` 令牌被 `MarkdownPreview` 用於快取失效。當主題的 `generation()` 回傳與快取值不同的數值時，預覽組件會重新建構所有渲染輸出。

**當使用者在執行階段變更主題時，遞增世代計數**：

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

## 顏色欄位使用方式

以下是每個主題顏色在實務中的使用方式：

| 方法 | 用於 |
|------|------|
| `get_text_color` | 預設段落文字、列表項文字 |
| `get_muted_text_color` | 程式碼塊、引用塊文字、表格邊框 |
| `get_primary_color` | 標題 (H1-H3)、粗體文字強調 |
| `get_secondary_color` | 行內程式碼、次要強調 |
| `get_info_color` | 資訊級別高亮 |
| `get_background_color` | 一般背景填充 |
| `get_border_color` | 未聚焦邊框（程式碼塊、表格） |
| `get_focused_border_color` | 已聚焦/互動中專案邊框、捲軸 |
| `get_popup_selected_background` | 已選取彈出視窗項目的背景 |
| `get_popup_selected_text_color` | 已選取彈出視窗項目的文字顏色 |
| `get_json_key_color` | 樹節點鍵名 |
| `get_json_string_color` | 樹中的字串值 |
| `get_json_number_color` | 樹中的數值 |
| `get_json_bool_color` | 樹中的布林值 |
| `get_json_null_color` | 樹中的 Null 值 |
| `get_accent_yellow` | 警告/高亮強調 |

## 範例：完整主題

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

## 效能注意事項

- 所有主題方法在渲染期間被頻繁呼叫。保持它們低開銷——回傳常數或簡單的欄位查詢。
- `Generation` 令牌的比較是簡單的整數相等檢查，因此遞增它非常快速。
