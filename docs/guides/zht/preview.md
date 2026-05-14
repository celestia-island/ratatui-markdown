# MarkdownPreview 組件

> 高層級統一組件，將 Markdown、樹和操作項整合在單一可捲動檢視中。

## 概覽

`MarkdownPreview` 是頂層整合組件。它將 Markdown 渲染、可折疊樹顯示和操作項目組合在單一 `HybridScrollView` 中。這是大多數應用程式建議的進入點。

由 `preview` 功能標誌控制（預設啟用，需要所有其他功能）。

## API

```rust
pub struct ActionItem {
    pub id: String,
    pub label: String,
}

impl MarkdownPreview {
    pub fn new() -> Self;

    // 設定
    pub fn with_strip_frontmatter(self, strip: bool) -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;

    // 內容
    pub fn set_content(&mut self, content: &str);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // 樹
    pub fn set_tree(&mut self, tree: Option<CollapsibleTree>);
    pub fn tree_mut(&mut self) -> Option<&mut CollapsibleTree>;
    pub fn has_tree(&self) -> bool;
    pub fn toggle_tree_node(&mut self) -> bool;

    // 操作項
    pub fn set_action_items(&mut self, items: Vec<ActionItem>);
    pub fn selected_action_id(&self) -> Option<&str>;

    // 導航
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // 狀態
    pub fn total_lines(&self) -> usize;
    pub fn scroll_offset(&self) -> usize;
    pub fn visible_height(&self) -> usize;
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;

    // 渲染
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

## 設定

### with_strip_frontmatter

啟用時（預設），以 `+++` 分隔的內容會被視為 TOML 前言並在渲染前剝離：

```
+++
title = "My Document"
author = "Jane Doe"
+++

# 實際內容
這是內文。
```

輸出中只會出現 `# 實際內容` 和後續的段落。

### with_left_padding

為渲染內容加入 1 列的左側內距（傳遞給 `HybridScrollView`）。

## 內容佈局

組件按垂直順序渲染內容：

1. **樹**（如果有）——帶有可聚焦節點的樹行
2. **Markdown**——解析並渲染的內容
3. **操作項**——帶有方括號包裝 `[標籤]` 的可聚焦操作標籤

在各區段之間會插入空行，每個區段在捲動檢視中擁有自己的 `FocusableRegion`。

## 快取

`MarkdownPreview` 會快取已渲染的輸出，僅在以下情況重新建構：

- 內容變更（`set_content` 傳入不同的文字）
- 寬度變更（終端機調整大小）
- 主題世代變更（`theme.generation()` 回傳新值）
- 樹被修改（`set_tree`、`toggle_tree_node`）
- 操作項被修改（`set_action_items`）

使用 `theme.generation()` 在主題變更後觸發重新渲染。

## TOML 前言處理

前言被假設為 TOML 格式。它**不會被解析**——只會從渲染輸出中移除。第一個 `+++` 行開始前言模式，第二個 `+++` 結束它。前言區塊前後的行會正常渲染。

如果內容不以 `+++` 開頭，則不會進行剝離。

## 操作項

操作項在檢視底部提供可透過鍵盤選取的選項：

```rust
preview.set_action_items(vec![
    ActionItem { id: "confirm".into(), label: " Confirm ".into() },
    ActionItem { id: "cancel".into(), label: " Cancel ".into() },
]);

// 在你的輸入處理器中：
if let Some("confirm") = preview.selected_action_id() {
    // 處理確認
}
```

操作項 ID 在內部會以 `action:` 前綴，以避免與樹節點路徑衝突。

## 範例

```rust
use ratatui_markdown::preview::{MarkdownPreview, ActionItem};
use ratatui_markdown::tree::CollapsibleTree;

let mut preview = MarkdownPreview::new()
    .with_strip_frontmatter(true)
    .with_left_padding(true);

// 設定 Markdown 內容（附帶 TOML 前言）
preview.set_content(concat!(
    "+++\n",
    "title = \"My Doc\"\n",
    "+++\n",
    "\n",
    "# Hello\n\nContent here.\n",
));

// 設定樹
let tree = CollapsibleTree::from_json_str(r#"{"config": {"theme": "dark"}}"#).unwrap();
preview.set_tree(Some(tree));

// 設定操作項
preview.set_action_items(vec![
    ActionItem { id: "edit".into(), label: " Edit ".into() },
    ActionItem { id: "save".into(), label: " Save ".into() },
]);

// 在 ratatui 應用程式的渲染函式中：
fn render_ui(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}

// 處理輸入
fn handle_input(key: rataui::crossterm::event::KeyCode, preview: &mut MarkdownPreview) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => preview.scroll_up(),
        KeyCode::Down | KeyCode::Char('j') => preview.scroll_down(),
        KeyCode::PageUp => preview.page_up(20),
        KeyCode::PageDown => preview.page_down(20),
        KeyCode::Home => preview.scroll_to_top(),
        KeyCode::End => preview.scroll_to_bottom(),
        KeyCode::Enter => { preview.toggle_tree_node(); }
        _ => {}
    }
}
```
