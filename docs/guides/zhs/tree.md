# 树视图

> 交互式可折叠树，支持 JSON 和 TOML 数据。

## 概述

`tree` 模块将 JSON 或 TOML 解析为交互式可折叠树。用户可以在终端 UI 中展开/折叠节点并使用键盘导航。

通过 `tree` 功能标志控制（需要 `scroll`、`serde_json` 和 `toml` 依赖）。

## CollapsibleTree

```rust
pub struct CollapsibleTree { /* 字段省略 */ }

impl CollapsibleTree {
    // 构造函数
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error>;
    pub fn from_toml_str(toml: &str) -> Result<Self, Box<dyn Error>>;
    pub fn from_value(value: serde_json::Value, style: KeyStyle) -> Self;

    // 树操作
    pub fn toggle(&mut self, path: &str) -> bool;
    pub fn handle_toggle(&mut self, id: &str) -> bool;
    pub fn expand_all(&mut self);
    pub fn collapse_all(&mut self);
    pub fn expand_to_depth(&mut self, depth: usize);

    // 渲染
    pub fn render_lines(&self, width: usize, theme: &impl RichTextTheme) -> Vec<Line<'static>>;
    pub fn flatten(&self) -> Vec<FlatEntry>;
    pub fn build_focusable_items(&self) -> Vec<FocusableItemRange>;
}
```

### 构造函数

- **`from_json_str`**：将 JSON 字符串解析为树。键名使用 `KeyStyle::Json` 样式（带引号的键名 + `:` 分隔符）。
- **`from_toml_str`**：将 TOML 字符串解析为树（内部转换为 JSON）。键名使用 `KeyStyle::Toml` 样式（裸键名 + `=` 分隔符）。
- **`from_value`**：从已有的 `serde_json::Value` 构建树，可指定键名样式。

### 树操作

```rust
// 使用斜杠分隔的路径切换节点
tree.toggle("dependencies");          // 切换根键
tree.toggle("dependencies/serde");    // 切换嵌套键

// 便捷方法
tree.expand_all();
tree.collapse_all();
tree.expand_to_depth(2);

// 在滚动上下文中 — 使用选中项的 ID
if scroll.selected_item_id().is_some() {
    tree.handle_toggle(selected_id);
}
```

### 渲染

- **`render_lines`**：生成带有树连接符和着色值的样式化 `Line`。
- **`flatten`**：返回所有可见条目的扁平列表（尊重折叠状态）。
- **`build_focusable_items`**：返回用于 `HybridScrollView` 集成的可聚焦范围，ID 与树路径匹配。

## 数据类型

```rust
pub struct FlatEntry {
    pub depth: usize,
    pub key: String,
    pub value: String,
    pub kind: EntryKind,
    pub value_type: ValueType,
    pub path: String,          // 例如 "dependencies/serde"
    pub is_last: bool,
}

pub enum EntryKind {
    Collapsed,  // 有子节点，当前折叠状态: [+]
    Expanded,   // 有子节点，当前展开状态: [-]
    Leaf,       // 叶子节点，无子节点
}

pub enum ValueType {
    String, Number, Boolean, Null, Object, Array,
}

pub enum KeyStyle {
    Json,  // "key": value
    Toml,  // key = value
}
```

### 值类型颜色

每个 `ValueType` 映射到对应的主题颜色方法：

| ValueType | 主题方法                  |
|-----------|---------------------------|
| String    | `get_json_string_color()` |
| Number    | `get_json_number_color()` |
| Boolean   | `get_json_bool_color()`   |
| Null      | `get_json_null_color()`   |

## 树行辅助函数

`tree_lines` 模块（从 `crate::tree` 重新导出）提供底层行构建：

```rust
pub fn make_body_line(key: &str, value: &str, style: &Style, value_style: &Style) -> Line<'static>;
pub fn make_status_line(status: &str, style: &Style) -> Line<'static>;
pub fn make_branch_dispatch_line(kind: EntryKind, ...) -> Line<'static>;
```

## 示例

```rust
use ratatui_markdown::tree::CollapsibleTree;

let toml_content = r#"
[package]
name = "my-app"
version = "0.1.0"

[dependencies]
ratatui = "0.29"
serde = { version = "1.0", features = ["derive"] }

[features]
default = ["std"]
std = []
"#;

let mut tree = CollapsibleTree::from_toml_str(toml_content).unwrap();

// 全部展开
tree.expand_all();

// 渲染为行
let lines = tree.render_lines(80, theme);

// 折叠 dependencies 子树
tree.toggle("dependencies");

// 重新渲染 — dependencies 现在处于折叠状态
let lines = tree.render_lines(80, theme);

// 获取可聚焦项用于滚动导航
let items = tree.build_focusable_items();
```
