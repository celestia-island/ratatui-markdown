# 捲動系統

> 智慧混合捲動，具備可聚焦專案的鍵盤導航。

## 概覽

`scroll` 模組提供了一個混合捲動系統，支援兩種模式：

1. **自由捲動**——當沒有可聚焦專案在視口中時，內容自由捲動
2. **互動模式**——當可聚焦專案進入視口中間時，游標會鎖定在第一個專案上以進行鍵盤導航

由 `scroll` 功能標誌控制（預設啟用）。

## HybridScrollView

管理捲動、聚焦區域和渲染的核心組件：

```rust
pub struct HybridScrollView { /* 欄位 */ }

impl HybridScrollView {
    pub fn new() -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;
    pub fn with_cursor_indicator(self, show: bool) -> Self;

    // 內容管理
    pub fn set_content(&mut self, lines: Vec<Line<'static>>, regions: Vec<FocusableRegion>);
    pub fn set_lines(&mut self, lines: Vec<Line<'static>>);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // 捲動狀態
    pub fn total_lines(&self) -> usize;
    pub fn get_scroll_offset(&self) -> usize;
    pub fn get_viewport_height(&self) -> usize;
    pub fn set_scroll_offset(&mut self, offset: usize);

    // 導航
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // 互動
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;
    pub fn engage_first(&mut self);
    pub fn engage_by_id(&mut self, id: &str) -> bool;

    // 渲染
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

### 設定

- **with_left_padding**：為所有顯示行加入 1 列的左側內距
- **with_cursor_indicator**：在互動模式的游標行上顯示 `> `（2 列）（優先於 `left_padding`）

`effective_padding()` 方法回傳實際使用的內距：
- 若啟用游標指示器則為 `2`
- 若僅有左側內距則為 `1`
- 否則為 `0`

## 可聚焦區域與專案

```rust
pub struct FocusableItemRange {
    pub start_line: usize,    // 包含
    pub end_line: usize,      // 不包含
    pub id: String,           // 唯一識別碼
}

pub struct FocusableRegion {
    pub items: Vec<FocusableItemRange>,
}
```

區域定義了變為互動式的行範圍。當視口中間經過一個區域時，捲動檢視會自動進入互動模式，游標會落在第一個專案上。

### 互動行為

- 向**下**捲動進入區域時，會鎖定在**第一個**專案
- 向**上**捲動進入區域時，會鎖定在**最後一個**專案
- 導航超過區域中的最後一個專案會**退出互動模式**並返回自由捲動
- `scroll_to_top()` 和 `scroll_to_bottom()` 總是會退出互動模式
- 在區域內，`scroll_up` / `scroll_down` 在專案之間移動游標

## 其他捲動組件

### ScrollableList<T>

一個泛型可捲動列表，具備滑鼠/鍵盤導航和可選的帶邊框渲染：

```rust
pub trait ListItemRenderer {
    fn render_item(&self, index: usize, is_selected: bool, width: usize) -> Line<'static>;
}
```

### ArrowScrollbar

一個使用 Unicode 箭頭符號繪製的自訂捲軸：

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

用於自動跟隨內容（例如，串流輸出）：

```rust
pub struct FollowScrollState {
    // 追蹤視口是否在底部
}
```

### ScrollableRenderResult

一個簡單的可捲動面板包裝器：

```rust
pub fn render_scrollable(
    lines: &[Line],
    scroll_offset: usize,
    area: Rect,
    buf: &mut Buffer,
) -> ScrollableRenderResult;
```

## 範例

```rust
use ratatui_markdown::scroll::{HybridScrollView, FocusableItemRange, FocusableRegion};

let mut scroll = HybridScrollView::new()
    .with_cursor_indicator(true);

// 內容行
let lines: Vec<Line> = (0..100)
    .map(|i| Line::raw(format!("第 {} 行", i)))
    .collect();

// 將第 30-32 行設為可聚焦
let region = FocusableRegion {
    items: vec![
        FocusableItemRange { start_line: 30, end_line: 31, id: "item-a".into() },
        FocusableItemRange { start_line: 31, end_line: 32, id: "item-b".into() },
        FocusableItemRange { start_line: 32, end_line: 33, id: "item-c".into() },
    ],
};

scroll.set_content(lines, vec![region]);

// 向下自由捲動；當第 30 行進入視口中間時自動進入互動模式
scroll.scroll_down();

// 在專案之間導航
while scroll.scroll_down() == InputResult::Continue {
    if let Some(id) = scroll.selected_item_id() {
        println!("已選取：{}", id);
    }
}
```
