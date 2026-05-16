# 快速開始

## 前置需求

- **Rust** 1.74 或更高版本
- **ratatui** 0.29（會自動作為依賴引入）

## 安裝

在你的 `Cargo.toml` 中加入：

```toml
[dependencies]
ratatui-markdown = "0.1"
```

預設會啟用所有功能（`markdown`、`scroll`、`tree`、`preview`、`mermaid`、`image`、`viewer`）。

### 選擇性功能

若要減少編譯時間和依賴，僅啟用所需的功能：

```toml
# 僅 Markdown 渲染
ratatui-markdown = { version = "0.1", default-features = false, features = ["markdown"] }

# 僅捲動系統
ratatui-markdown = { version = "0.1", default-features = false, features = ["scroll"] }

# 樹檢視（會引入 scroll、serde_json 和 toml）
ratatui-markdown = { version = "0.1", default-features = false, features = ["tree"] }
```

## 基本用法

### 渲染 Markdown

```rust
use ratatui_markdown::markdown::MarkdownRenderer;
use ratatui_markdown::theme::RichTextTheme;

// 使用最大內容寬度建立渲染器
let renderer = MarkdownRenderer::new(80);

// 將 Markdown 文字解析為區塊
let blocks = renderer.parse("# Hello\n\nThis is **bold** and *italic* text.");

// 將區塊渲染為 ratatui::text::Line<'static>
let lines = renderer.render(&blocks, &my_theme);
```

### 瀏覽樹

```rust
use ratatui_markdown::tree::CollapsibleTree;

// 將 JSON 解析為可折疊樹
let json_str = r#"{"name": "project", "deps": {"ratatui": "0.29", "serde": "1.0"}}"#;
let mut tree = CollapsibleTree::from_json_str(json_str).unwrap();

// 渲染樹行
let lines = tree.render_lines(80, &my_theme);

// 取得可聚焦專案以進行導航
let items = tree.build_focusable_items();

// 切換節點
tree.toggle("deps/serde");
```

### 使用 MarkdownPreview 組件

`MarkdownPreview` 組件將所有功能整合到單一可捲動檢視中：

```rust
use ratatui_markdown::preview::MarkdownPreview;
use ratatui_markdown::theme::RichTextTheme;

let mut preview = MarkdownPreview::new()
    .with_left_padding(true);

// 設定 Markdown 內容
preview.set_content("# Welcome\n\n- Item one\n- Item two\n\n```rust\nlet x = 42;\n```");

// 設定可折疊樹（可選）
let tree = CollapsibleTree::from_json_str(r#"{"config": {"port": 8080}}"#).unwrap();
preview.set_tree(Some(tree));

// 處理鍵盤輸入
preview.scroll_up();
preview.scroll_down();
preview.page_up(10);
preview.page_down(10);
preview.toggle_tree_node(); // Enter 鍵

// 在 ratatui 繪製迴圈中渲染
fn draw(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}
```

## 實作主題

函式庫使用特徵來查詢所有顏色：

```rust
use ratatui::style::Color;
use ratatui_markdown::theme::{Generation, RichTextTheme};

struct MyTheme;

impl RichTextTheme for MyTheme {
    fn generation(&self) -> Generation { Generation(1) }
    fn get_text_color(&self) -> Color { Color::White }
    fn get_muted_text_color(&self) -> Color { Color::Gray }
    fn get_primary_color(&self) -> Color { Color::Cyan }
    fn get_secondary_color(&self) -> Color { Color::Blue }
    fn get_info_color(&self) -> Color { Color::LightBlue }
    fn get_background_color(&self) -> Color { Color::Black }
    fn get_border_color(&self) -> Color { Color::DarkGray }
    fn get_focused_border_color(&self) -> Color { Color::White }
    fn get_popup_selected_background(&self) -> Color { Color::DarkGray }
    fn get_popup_selected_text_color(&self) -> Color { Color::White }
    fn get_json_key_color(&self) -> Color { Color::LightCyan }
    fn get_json_string_color(&self) -> Color { Color::Green }
    fn get_json_number_color(&self) -> Color { Color::Yellow }
    fn get_json_bool_color(&self) -> Color { Color::Magenta }
    fn get_json_null_color(&self) -> Color { Color::DarkGray }
    fn get_accent_yellow(&self) -> Color { Color::Yellow }
}
```

變更 `generation()` 的回傳值可以讓預覽組件的快取失效並強制重新渲染（例如，當使用者在執行階段切換主題時）。

## 下一步

- [Markdown 模組](markdown.md) — 完整的 Markdown 解析和渲染 API
- [捲動系統](scroll.md) — 了解混合捲動架構
- [樹檢視](tree.md) — JSON/TOML 樹的渲染與互動
- [預覽組件](preview.md) — 高層級統一組件
- [主題](theme.md) — 完整的主題自訂指南
