# Markdown 模組

> 解析並渲染 Markdown 文字為帶樣式的 `ratatui::text::Line`。

## 概覽

`markdown` 模組提供了一個專為終端輸出設計的自訂 Markdown 解析器和渲染器。它**不是** CommonMark 相容的解析器——它針對的是終端使用者介面中最實用的 Markdown 子集。

由 `markdown` 功能標誌控制（預設啟用）。

## MarkdownRenderer

```rust
pub struct MarkdownRenderer {
    max_width: usize,
}

impl MarkdownRenderer {
    pub fn new(max_width: usize) -> Self;
    pub fn parse(&self, markdown: &str) -> Vec<MarkdownBlock>;
    pub fn render(&self, blocks: &[MarkdownBlock], theme: &impl RichTextTheme) -> Vec<Line<'static>>;
}
```

### 建構函式

`MarkdownRenderer::new(max_width)` 接受可用的內容寬度（以列為單位）。此寬度用於段落文字換行（CJK 感知）和表格欄位大小調整。

### 解析

`parse()` 接受一個 `&str` 的 Markdown 文字並回傳 `Vec<MarkdownBlock>`。解析器是逐行處理的，按順序處理區塊。

### 渲染

`render()` 接受解析後的區塊和一個主題，產生 `Vec<Line<'static>>`，可直接用於 ratatui 組件中。

## MarkdownBlock 列舉

```rust
pub enum MarkdownBlock {
    Heading1(String),              // # 標題
    Heading2(String),              // ## 標題
    Heading3(String),              // ### 標題
    Paragraph(Vec<String>),        // 換行後的段落行
    CodeBlock(String, String),     // (語言, 內容)
    InlineCode(String),            // `行內程式碼`
    ListItem(String, u8),          // (內容, 縮排層級)
    Blockquote(String),            // > 引用文字
    HorizontalRule,                // --- 或 *** 或 ___
    BlankLine,                     // 空行
    Table {                        // | col1 | col2 |
        headers: Vec<String>,      //   |------|------|
        rows: Vec<Vec<String>>,    //   | val1 | val2 |
    },
}
```

### 區塊詳情

**標題** (H1-H3)：使用 `primary_color` 渲染，H1 使用 `Modifier::BOLD`。

**段落**：文字以 CJK 感知的方式自動換行至 `max_width`。每個換行後的行成為 `Vec<String>` 中的一個項目。

**程式碼塊**（以 ` ``` ` 圍欄）：在帶邊框的方塊內使用 `muted_text_color` 渲染，邊框使用製表符字元。Mermaid 程式碼塊會被靜默跳過。

**行內程式碼**：使用 `secondary_color` 和 `Modifier::DIM` 渲染。

**列表**：無序列表（`-`、`*`、`+`）和有序列表（`1.`、`2.`）。每個項目保留其縮排層級。子項目會視覺縮排。

**引用塊**：以帶顏色的 `│` 長條為前綴，使用 `muted_text_color` 渲染。

**表格**：根據內容寬度按比例分配欄寬。儲存格會換行，標題使用 `Modifier::BOLD`，邊框使用製表符字元。

## 內聯格式化

內聯格式化會套用在**段落和列表項文字內部**：

| Markdown        | 渲染效果                        |
|-----------------|--------------------------------|
| `**文字**`      | **粗體** (`Modifier::BOLD`)   |
| `*文字*`        | *斜體* (`Modifier::ITALIC`) |
| `***文字***`    | ***粗體+斜體***                |
| `` `程式碼` ``  | `行內程式碼` 樣式              |

```rust
pub fn parse_inline_formatting(text: &str, theme: &impl RichTextTheme) -> Vec<Span<'static>>;
```

此獨立函式也會重新匯出，以便在 `MarkdownRenderer` 之外使用。

## 範例

```rust
use ratatui_markdown::markdown::MarkdownRenderer;

let md = r#"
# 標題

這是一個包含 **粗體** 和 *斜體* 文字的段落。

## 程式碼

```rust
fn main() {
    println!("Hello!");
}
```

| 名稱     | 版本  |
|---------|------|
| ratatui | 0.29 |
| serde   | 1.0  |
"#;

let renderer = MarkdownRenderer::new(80);
let blocks = renderer.parse(md);
let lines = renderer.render(&blocks, theme);
// 在 ratatui 組件中使用 lines
```
