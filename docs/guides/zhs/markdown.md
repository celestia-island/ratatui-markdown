# Markdown 模块

> 解析 Markdown 文本并将其渲染为带样式的 `ratatui::text::Line`。

## 概述

`markdown` 模块提供了一个专为终端输出设计的自定义 Markdown 解析器和渲染器。它**并非** CommonMark 兼容的解析器——它针对的是终端 UI 中最常用的 Markdown 子集。

通过 `markdown` 功能标志控制（默认启用）。

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

### 构造函数

`MarkdownRenderer::new(max_width)` 接收可用的内容宽度（以列为单位）。此宽度用于段落文本换行（支持 CJK 字符宽度感知）和表格列宽计算。

### 解析

`parse()` 接收 markdown 文本的 `&str` 引用，返回 `Vec<MarkdownBlock>`。解析器是面向行的，按顺序处理块。

### 渲染

`render()` 接收解析后的块和主题，生成 `Vec<Line<'static>>`，可直接用于 ratatui 组件中。

## MarkdownBlock 枚举

```rust
pub enum MarkdownBlock {
    Heading1(String),              // # 标题
    Heading2(String),              // ## 标题
    Heading3(String),              // ### 标题
    Paragraph(Vec<String>),        // 换行后的段落
    CodeBlock(String, String),     // (语言, 内容)
    InlineCode(String),            // `行内代码`
    ListItem(String, u8),          // (内容, 缩进级别)
    Blockquote(String),            // > 引用文本
    HorizontalRule,                // --- 或 *** 或 ___
    BlankLine,                     // 空行
    Table {                        // | 列1 | 列2 |
        headers: Vec<String>,      //   |------|------|
        rows: Vec<Vec<String>>,    //   | 值1 | 值2 |
    },
}
```

### 块详情

**标题** (H1-H3)：使用 `primary_color` 渲染，H1 额外应用 `Modifier::BOLD`。

**段落**：文本按 CJK 字符宽度感知自动换行到 `max_width`。每个换行行都会成为 `Vec<String>` 中的一条。

**代码块**（以 ` ``` ` 围栏包裹）：在带有制表符边框的框内使用 `muted_text_color` 渲染。Mermaid 代码块会被静默跳过。

**行内代码**：使用 `secondary_color` 和 `Modifier::DIM` 渲染。

**列表**：支持无序列表（`-`、`*`、`+`）和有序列表（`1.`、`2.`）。每个列表项保留其缩进级别。子项在视觉上会进一步缩进。

**引用块**：以前缀带有颜色的 `│` 竖线和使用 `muted_text_color` 渲染。

**表格**：列宽根据内容宽度按比例分配。单元格支持换行，表头使用 `Modifier::BOLD`，边框使用制表符字符。

## 内联格式化

内联格式化应用于**段落和列表项**文本内部：

| Markdown        | 渲染效果                          |
|-----------------|--------------------------------|
| `**text**`      | **粗体** (`Modifier::BOLD`)   |
| `*text*`        | *斜体* (`Modifier::ITALIC`)   |
| `***text***`    | ***粗体+斜体***                |
| `` `code` ``    | `行内代码` 样式                 |

```rust
pub fn parse_inline_formatting(text: &str, theme: &impl RichTextTheme) -> Vec<Span<'static>>;
```

此独立函数也会被重新导出，以便在 `MarkdownRenderer` 外部使用。

## 示例

```rust
use ratatui_markdown::markdown::MarkdownRenderer;

let md = r#"
# Title

This is a paragraph with **bold** and *italic* text.

## Code

```rust
fn main() {
    println!("Hello!");
}
```

| Name | Version |
|------|---------|
| ratatui | 0.29 |
| serde | 1.0 |
"#;

let renderer = MarkdownRenderer::new(80);
let blocks = renderer.parse(md);
let lines = renderer.render(&blocks, theme);
// 在 ratatui 组件中使用 lines
```
