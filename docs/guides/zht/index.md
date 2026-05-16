<div align="center">
  <img src="../../../examples/logo.webp" alt="ratatui-markdown logo" width="200" />
</div>

# ratatui-markdown

> 為 ratatui 提供 Markdown 渲染、Mermaid 圖表、語法高亮、可折疊 JSON/TOML 樹以及豐富滾動組件的 Rust 庫。
>
> **建構基礎**: [ratatui](https://github.com/ratatui/ratatui) 0.29 + 純 Rust
>
> **最低 Rust 版本**: 1.74

<div align="center">
  <p>
    <a href="../../README.md">English</a> |
    <a href="../zhs/index.md">简体中文</a> |
    <a href="../zht/index.md">繁體中文</a> |
    <a href="../ja/index.md">日本語</a> |
    <a href="../ko/index.md">한국어</a> |
    <a href="../fr/index.md">Français</a> |
    <a href="../es/index.md">Español</a> |
    <a href="../ru/index.md">Русский</a> |
    <a href="../ar/index.md">العربية</a>
  </p>
</div>

## 什麼是 ratatui-markdown？

ratatui-markdown 是一個功能豐富的終端使用者介面渲染庫，基於 [ratatui](https://github.com/ratatui/ratatui) 建構。它提供多個功能模組，可以獨立使用，也可以透過 `MarkdownPreview` / `MarkdownViewer` 組件組合使用。

## 核心模組

### Markdown 渲染

解析 Markdown 文字並渲染為帶樣式的終端輸出：

- **標題**: H1 (`#`), H2 (`##`), H3 (`###`)
- **段落**: 支援 CJK 字元寬度感知的自動換行
- **內聯格式**: `**粗體**`, `*斜體*`, `***粗體+斜體***`, `` `行內程式碼` ``
- **圍欄程式碼塊**: 可選語言標籤（mermaid 塊渲染為圖表）
- **引用塊** (`>`)
- **無序列表** (`-`, `*`, `+`) 和有序列表 (`1.`, `2.`)
- **水平分隔線** (`---`, `***`, `___`)
- **表格**: 按比例分配列寬，支援儲存格換行

### 可折疊樹檢視

解析並互動式瀏覽結構化資料：

- 解析 **JSON** 和 **TOML** 為可折疊樹
- **展開 / 折疊** 單個節點、全部展開、全部折疊、按深度展開
- **樣式鍵名**: JSON 模式（帶引號的鍵名 + `:`）或 TOML 模式（裸鍵名 + `=`）
- **鍵盤導航**: 基於游標的選擇和切換
- **值型別著色**: 字串、數字、布林值、null 各有獨立的主題顏色

### 混合捲動系統

智慧捲動，同時處理自由瀏覽和專案導航：

- **自由捲動模式**: 自由瀏覽內容
- **互動模式**: 當可聚焦專案進入視口中間時自動啟用
- **游標導航**: 透過鍵盤在可聚焦專案間移動
- **游標指示器**: 互動行顯示 `> ` 前綴
- **捲軸**: 基於箭頭的疊加顯示
- **翻頁**: 支援 `page_up` / `page_down`

### Mermaid 圖表

在終端中直接渲染 Mermaid 圖表：

- **時序圖**、**圓餅圖**、**甘特圖**、**狀態圖**
- 透過 ` ```mermaid ` 程式碼塊觸發
- 功能標誌：`mermaid`

### 語法高亮

基於 tree-sitter 的程式碼塊語法高亮：

- 按語言選擇的功能標誌（`highlight-lang-rust`、`highlight-lang-python` 等）
- `highlight-lang-all` 打包所有支援的語言
- 可透過 `HighlightHooks` 自訂

### MarkdownPreview / MarkdownViewer 組件

整合一切的高層組件：

- 將 markdown 內容、樹檢視和操作項渲染在單個可捲動佈局中
- **快取**: 僅在內容、寬度或主題世代變化時重建輸出
- **TOML 前言剝離**: 自動剝離 `+++` 分隔的 TOML 前言
- **操作項**: 可透過鍵盤選擇的帶 action ID 的標籤項
- 所有導航委託給 `HybridScrollView`

## 快速開始

```toml
[dependencies]
ratatui-markdown = "0.2"
```

### 範例

| 範例                 | 描述                               | 所需功能標誌                   |
|----------------------|------------------------------------|-------------------------------|
| `basic`              | 基礎 Markdown 渲染                 | —                             |
| `code`               | 語法高亮程式碼塊                   | `highlight-lang-all`          |
| `custom_code_block`  | 自訂程式碼塊渲染鉤子               | —                             |
| `image`              | 圖片嵌入和縮放                     | `image`                       |
| `mermaid`            | Mermaid 圖表渲染                   | `mermaid`                     |
| `tree_list`          | 可折疊 JSON/TOML 樹檢視            | —                             |

```bash
cargo run --example basic
cargo run --example code --features highlight-lang-all
cargo run --example image
cargo run --example mermaid
cargo run --example tree_list
```

## 功能標誌

預設情況下啟用所有功能。可以透過禁用預設功能來選擇性啟用：

```toml
[dependencies]
ratatui-markdown = { version = "0.2", default-features = false, features = ["markdown"] }
```

| 功能                | 依賴                               | 描述                               | 預設 |
|---------------------|------------------------------------|-----------------------------------|------|
| `markdown`          | —                                  | Markdown 解析器和渲染器             | ✓    |
| `image`             | —                                  | 透過 `ImageResolver` 解析圖片       | ✓    |
| `scroll`            | —                                  | HybridScrollView、可捲動列表、捲軸  | ✓    |
| `tree`              | `scroll`, `serde_json`, `toml`     | 可折疊 JSON/TOML 樹                 | ✓    |
| `preview`           | `markdown`, `scroll`, `tree`       | MarkdownPreview 統一組件            | ✓    |
| `mermaid`           | `markdown`                         | Mermaid 圖表渲染                    | ✓    |
| `viewer`            | `markdown`, `scroll`               | MarkdownViewer 組件                 | ✓    |
| `highlight`         | —                                  | 基於 tree-sitter 的語法高亮          |      |
| `highlight-lang-*`  | `highlight`                        | 單語言語法                          |      |
| `highlight-lang-all`| `highlight`                        | 所有內建語言語法                     |      |

## 文件

| 指南 | 描述 |
|------|------|
| [快速開始](getting-started.md) | 安裝和首次渲染 |
| [Markdown](markdown.md) | 解析和渲染 Markdown |
| [捲動系統](scroll.md) | 混合捲動、導航、捲軸 |
| [樹檢視](tree.md) | JSON/TOML 樹、展開/折疊 |
| [預覽組件](preview.md) | 使用 MarkdownPreview 整合一切 |
| [主題](theme.md) | 實現 RichTextTheme |
| [貢獻指南](contributing.md) | 開發和貢獻指南 |

## 授權條款

雙重授權 MIT OR Apache-2.0。
