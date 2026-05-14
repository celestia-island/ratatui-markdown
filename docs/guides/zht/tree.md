# 樹檢視

> 用於 JSON 和 TOML 資料的互動式可折疊樹。

## 概覽

`tree` 模組將 JSON 或 TOML 解析為互動式可折疊樹。使用者可以在終端使用者介面中展開/折疊節點並使用鍵盤導航。

由 `tree` 功能標誌控制（需要 `scroll`、`serde_json` 和 `toml` 依賴）。

## CollapsibleTree

```rust
pub struct CollapsibleTree { /* 欄位 */ }

impl CollapsibleTree {
    // 建構函式
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error>;
    pub fn from_toml_str(toml: &str) -> Result<Self, Box<dyn Error>>;
    pub fn from_value(value: serde_json::Value, style: KeyStyle) -> Self;

    // 樹操作
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

### 建構函式

- **`from_json_str`**：將 JSON 字串解析為樹。鍵名使用 `KeyStyle::Json`（帶引號，以 `:` 分隔）。
- **`from_toml_str`**：將 TOML 字串解析為樹（內部轉換為 JSON）。鍵名使用 `KeyStyle::Toml`（不帶引號，以 `=` 分隔）。
- **`from_value`**：從現有的 `serde_json::Value` 以指定的鍵名樣式建構樹。

### 樹操作

```rust
// 使用斜線分隔的路徑進行切換
tree.toggle("dependencies");          // 切換根鍵名
tree.toggle("dependencies/serde");    // 切換巢狀鍵名

// 便利方法
tree.expand_all();
tree.collapse_all();
tree.expand_to_depth(2);

// 在捲動上下文中——使用所選專案的 ID
if scroll.selected_item_id().is_some() {
    tree.handle_toggle(selected_id);
}
```

### 渲染

- **`render_lines`**：產生帶有樹連接符和顏色值的樣式化 `Line`。
- **`flatten`**：回傳所有可見項目的平面列表（遵循折疊狀態）。
- **`build_focusable_items`**：回傳用於 `HybridScrollView` 整合的可聚焦範圍，其 ID 與樹路徑相符。

## 資料型別

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
    Collapsed,  // 有子節點，目前已折疊：[+]
    Expanded,   // 有子節點，目前已展開：[-]
    Leaf,       // 無子節點
}

pub enum ValueType {
    String, Number, Boolean, Null, Object, Array,
}

pub enum KeyStyle {
    Json,  // "key": value
    Toml,  // key = value
}
```

### 值型別顏色

每個 `ValueType` 對應到相應的主題顏色方法：

| ValueType | 主題方法               |
|-----------|------------------------|
| String    | `get_json_string_color()` |
| Number    | `get_json_number_color()` |
| Boolean   | `get_json_bool_color()`   |
| Null      | `get_json_null_color()`   |

## 樹行輔助函式

`tree_lines` 模組（從 `crate::tree` 重新匯出）提供低層級的行建構：

```rust
pub fn make_body_line(key: &str, value: &str, style: &Style, value_style: &Style) -> Line<'static>;
pub fn make_status_line(status: &str, style: &Style) -> Line<'static>;
pub fn make_branch_dispatch_line(kind: EntryKind, ...) -> Line<'static>;
```

## 範例

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

// 全部展開
tree.expand_all();

// 渲染為行
let lines = tree.render_lines(80, theme);

// 折疊 dependencies 子樹
tree.toggle("dependencies");

// 重新渲染——dependencies 現在已折疊
let lines = tree.render_lines(80, theme);

// 取得可聚焦專案以進行捲動導航
let items = tree.build_focusable_items();
```
