# Markdown モジュール

> Markdown テキストを解析し、スタイル付きの `ratatui::text::Line` としてレンダリングします。

## 概要

`markdown` モジュールは、ターミナル出力用に設計された独自の Markdown パーサーとレンダラーを提供します。CommonMark 準拠のパーサーでは**なく**、ターミナル UI で最も有用な Markdown のサブセットを対象としています。

`markdown` 機能フラグで制御されます（デフォルトで有効）。

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

### コンストラクタ

`MarkdownRenderer::new(max_width)` は利用可能なコンテンツ幅（列数）を受け取ります。この幅は段落テキストの折り返し（CJK 対応）やテーブルの列幅計算に使用されます。

### 解析

`parse()` は Markdown テキストの `&str` を受け取り、`Vec<MarkdownBlock>` を返します。パーサーは行指向で、ブロックを順次処理します。

### レンダリング

`render()` は解析されたブロックとテーマを受け取り、ratatui ウィジェットで直接使用できる `Vec<Line<'static>>` を生成します。

## MarkdownBlock 列挙型

```rust
pub enum MarkdownBlock {
    Heading1(String),              // # 見出し
    Heading2(String),              // ## 見出し
    Heading3(String),              // ### 見出し
    Paragraph(Vec<String>),        // 折り返された段落の行
    CodeBlock(String, String),     // (言語, 内容)
    InlineCode(String),            // `インラインコード`
    ListItem(String, u8),          // (内容, インデントレベル)
    Blockquote(String),            // > 引用テキスト
    HorizontalRule,                // --- または *** または ___
    BlankLine,                     // 空行
    Table {                        // | 列1 | 列2 |
        headers: Vec<String>,      //   |------|------|
        rows: Vec<Vec<String>>,    //   | 値1 | 値2 |
    },
}
```

### ブロックの詳細

**見出し** (H1-H3): `primary_color` でレンダリングされ、H1 は `Modifier::BOLD` が適用されます。

**段落**: テキストは CJK 文字幅を考慮して `max_width` で折り返されます。折り返された各行は `Vec<String>` のエントリになります。

**コードブロック** (`` ``` `` で囲まれたもの): ボックス描画文字を使用した境界線付きボックス内で `muted_text_color` でレンダリングされます。Mermaid コードブロックは暗黙的にスキップされます。

**インラインコード**: `secondary_color` と `Modifier::DIM` でレンダリングされます。

**リスト**: 順序なし（`-`、`*`、`+`）と順序付き（`1.`、`2.`）。各アイテムはインデントレベルを保持します。サブアイテムは視覚的にインデントされます。

**引用ブロック**: 色付きの `│` バーが先頭に付き、`muted_text_color` でレンダリングされます。

**テーブル**: 列はコンテンツ幅に基づいて比例配分されます。セルは折り返され、ヘッダーは `Modifier::BOLD` が適用され、境界線にはボックス描画文字が使用されます。

## インライン書式

インライン書式は段落およびリストアイテムのテキスト**内部**に適用されます：

| Markdown        | レンダリング効果                |
|-----------------|--------------------------------|
| `**text**`      | **太字** (`Modifier::BOLD`)   |
| `*text*`        | *斜体* (`Modifier::ITALIC`) |
| `***text***`    | ***太字+斜体***              |
| `` `code` ``    | `インラインコード` スタイル            |

```rust
pub fn parse_inline_formatting(text: &str, theme: &impl RichTextTheme) -> Vec<Span<'static>>;
```

このスタンドアロン関数は `MarkdownRenderer` の外部で使用できるように再エクスポートされています。

## 例

```rust
use ratatui_markdown::markdown::MarkdownRenderer;

let md = r#"
# タイトル

これは**太字**と*斜体*を含む段落です。

## コード

```rust
fn main() {
    println!("Hello!");
}
```

| 名前 | バージョン |
|------|---------|
| ratatui | 0.29 |
| serde | 1.0 |
"#;

let renderer = MarkdownRenderer::new(80);
let blocks = renderer.parse(md);
let lines = renderer.render(&blocks, theme);
// lines を ratatui ウィジェットで使用
```
