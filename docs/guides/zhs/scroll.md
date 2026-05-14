# 滚动系统

> 智能混合滚动，支持可聚焦项导航。

## 概述

`scroll` 模块提供了一个混合滚动系统，支持两种模式：

1. **自由滚动** — 当视口内无可聚焦项时，内容自由滚动
2. **交互模式** — 当可聚焦项进入视口中心时，光标自动锁定到第一个项目以支持键盘导航

通过 `scroll` 功能标志控制（默认启用）。

## HybridScrollView

管理滚动、焦点区域和渲染的核心组件：

```rust
pub struct HybridScrollView { /* 字段省略 */ }

impl HybridScrollView {
    pub fn new() -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;
    pub fn with_cursor_indicator(self, show: bool) -> Self;

    // 内容管理
    pub fn set_content(&mut self, lines: Vec<Line<'static>>, regions: Vec<FocusableRegion>);
    pub fn set_lines(&mut self, lines: Vec<Line<'static>>);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // 滚动状态
    pub fn total_lines(&self) -> usize;
    pub fn get_scroll_offset(&self) -> usize;
    pub fn get_viewport_height(&self) -> usize;
    pub fn set_scroll_offset(&mut self, offset: usize);

    // 导航
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // 交互状态
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;
    pub fn engage_first(&mut self);
    pub fn engage_by_id(&mut self, id: &str) -> bool;

    // 渲染
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

### 配置

- **with_left_padding**：为所有显示的行添加 1 列左边距
- **with_cursor_indicator**：在交互光标行显示 `> `（2 列）前缀（优先于 `left_padding`）

`effective_padding()` 方法返回实际使用的内边距：
- 启用光标指示器时返回 `2`
- 仅左边距时返回 `1`
- 否则返回 `0`

## 可聚焦区域和项目

```rust
pub struct FocusableItemRange {
    pub start_line: usize,    // 包含
    pub end_line: usize,      // 不包含
    pub id: String,           // 唯一标识符
}

pub struct FocusableRegion {
    pub items: Vec<FocusableItemRange>,
}
```

区域定义了一组变为可交互的行范围。当视口中心经过某个区域时，滚动视图会自动进入交互模式，光标落在第一个项目上。

### 交互行为

- 向**下**滚动进入区域时，光标定位到**第一个**项目
- 向**上**滚动进入区域时，光标定位到**最后一个**项目
- 导航超过区域的最后一个项目时，**退出交互模式**并返回自由滚动
- `scroll_to_top()` 和 `scroll_to_bottom()` 始终退出交互模式
- 在区域内，`scroll_up`/`scroll_down` 在项目之间移动光标

## 其他滚动组件

### ScrollableList\<T\>

通用可滚动列表，支持鼠标/键盘导航和可选的带边框渲染：

```rust
pub trait ListItemRenderer {
    fn render_item(&self, index: usize, is_selected: bool, width: usize) -> Line<'static>;
}
```

### ArrowScrollbar

使用 Unicode 箭头符号绘制的自定义滚动条：

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

用于自动跟随内容（例如流式输出）：

```rust
pub struct FollowScrollState {
    // 跟踪视口是否位于底部
}
```

### ScrollableRenderResult

一个简单的可滚动面板包装器：

```rust
pub fn render_scrollable(
    lines: &[Line],
    scroll_offset: usize,
    area: Rect,
    buf: &mut Buffer,
) -> ScrollableRenderResult;
```

## 示例

```rust
use ratatui_markdown::scroll::{HybridScrollView, FocusableItemRange, FocusableRegion};

let mut scroll = HybridScrollView::new()
    .with_cursor_indicator(true);

// 内容行
let lines: Vec<Line> = (0..100)
    .map(|i| Line::raw(format!("Line {}", i)))
    .collect();

// 使第 30-32 行可聚焦
let region = FocusableRegion {
    items: vec![
        FocusableItemRange { start_line: 30, end_line: 31, id: "item-a".into() },
        FocusableItemRange { start_line: 31, end_line: 32, id: "item-b".into() },
        FocusableItemRange { start_line: 32, end_line: 33, id: "item-c".into() },
    ],
};

scroll.set_content(lines, vec![region]);

// 自由向下滚动；当第 30 行进入视口中心时自动进入交互模式
scroll.scroll_down();

// 在项目之间导航
while scroll.scroll_down() == InputResult::Continue {
    if let Some(id) = scroll.selected_item_id() {
        println!("已选择: {}", id);
    }
}
```
