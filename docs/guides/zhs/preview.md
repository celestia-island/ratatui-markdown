# MarkdownPreview 组件

> 将 Markdown、树视图和操作项整合到一个可滚动视图中的高层统一组件。

## 概述

`MarkdownPreview` 是顶层集成组件。它将 Markdown 渲染、可折叠树显示和操作项组合到一个 `HybridScrollView` 中。这是大多数应用的推荐入口点。

通过 `preview` 功能标志控制（默认启用，需要所有其他功能）。

## API

```rust
pub struct ActionItem {
    pub id: String,
    pub label: String,
}

impl MarkdownPreview {
    pub fn new() -> Self;

    // 配置
    pub fn with_strip_frontmatter(self, strip: bool) -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;

    // 内容
    pub fn set_content(&mut self, content: &str);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // 树
    pub fn set_tree(&mut self, tree: Option<CollapsibleTree>);
    pub fn tree_mut(&mut self) -> Option<&mut CollapsibleTree>;
    pub fn has_tree(&self) -> bool;
    pub fn toggle_tree_node(&mut self) -> bool;

    // 操作项
    pub fn set_action_items(&mut self, items: Vec<ActionItem>);
    pub fn selected_action_id(&self) -> Option<&str>;

    // 导航
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // 状态
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

## 配置

### with_strip_frontmatter

启用后（默认），由 `+++` 分隔的内容会被视为 TOML 前言并在渲染前剥离：

```
+++
title = "My Document"
author = "Jane Doe"
+++

# Actual Content
This is the body.
```

输出中将只显示 `# Actual Content` 及其后续段落。

### with_left_padding

为渲染内容添加 1 列左边距（透传给 `HybridScrollView`）。

## 内容布局

组件按以下垂直顺序渲染内容：

1. **树视图**（如果存在）— 带有可聚焦节点的树行
2. **Markdown** — 解析并渲染的内容
3. **操作项** — 带有方括号包裹 `[label]` 的可聚焦操作标签

各节之间插入空行，每节在滚动视图中获得独立的 `FocusableRegion`。

## 缓存

`MarkdownPreview` 缓存渲染输出，仅在以下情况时重建：

- 内容变化（`set_content` 传入了不同的文本）
- 宽度变化（终端调整大小）
- 主题代次变化（`theme.generation()` 返回新值）
- 树被修改（`set_tree`、`toggle_tree_node`）
- 操作项被修改（`set_action_items`）

使用 `theme.generation()` 在主题变化后触发重新渲染。

## TOML 前言处理

前言内容假定为 TOML 格式。它**不会被解析**——只是从渲染输出中移除。第一个 `+++` 行开启前言模式，第二个 `+++` 行结束。前言块前后的行正常渲染。

如果内容不以 `+++` 开头，则不会发生剥离。

## 操作项

操作项在视图底部提供可通过键盘选择的选项：

```rust
preview.set_action_items(vec![
    ActionItem { id: "confirm".into(), label: " Confirm ".into() },
    ActionItem { id: "cancel".into(), label: " Cancel ".into() },
]);

// 在输入处理函数中:
if let Some("confirm") = preview.selected_action_id() {
    // 处理确认操作
}
```

操作项 ID 内部会添加 `action:` 前缀，以避免与树节点路径冲突。

## 示例

```rust
use ratatui_markdown::preview::{MarkdownPreview, ActionItem};
use ratatui_markdown::tree::CollapsibleTree;

let mut preview = MarkdownPreview::new()
    .with_strip_frontmatter(true)
    .with_left_padding(true);

// 设置 markdown 内容（带 TOML 前言）
preview.set_content(concat!(
    "+++\n",
    "title = \"My Doc\"\n",
    "+++\n",
    "\n",
    "# Hello\n\nContent here.\n",
));

// 设置树
let tree = CollapsibleTree::from_json_str(r#"{"config": {"theme": "dark"}}"#).unwrap();
preview.set_tree(Some(tree));

// 设置操作项
preview.set_action_items(vec![
    ActionItem { id: "edit".into(), label: " Edit ".into() },
    ActionItem { id: "save".into(), label: " Save ".into() },
]);

// 在 ratatui 应用的渲染函数中:
fn render_ui(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}

// 处理输入
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
