# ツリービュー

> JSON および TOML データのインタラクティブな折りたたみ可能ツリー。

## 概要

`tree` モジュールは JSON または TOML を解析し、インタラクティブな折りたたみ可能ツリーに変換します。ユーザーはターミナル UI でノードの展開/折りたたみとキーボードナビゲーションを行うことができます。

`tree` 機能フラグで制御されます（`scroll`、`serde_json`、`toml` の依存が必要です）。

## CollapsibleTree

```rust
pub struct CollapsibleTree { /* fields */ }

impl CollapsibleTree {
    // コンストラクタ
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error>;
    pub fn from_toml_str(toml: &str) -> Result<Self, Box<dyn Error>>;
    pub fn from_value(value: serde_json::Value, style: KeyStyle) -> Self;

    // ツリー操作
    pub fn toggle(&mut self, path: &str) -> bool;
    pub fn handle_toggle(&mut self, id: &str) -> bool;
    pub fn expand_all(&mut self);
    pub fn collapse_all(&mut self);
    pub fn expand_to_depth(&mut self, depth: usize);

    // レンダリング
    pub fn render_lines(&self, width: usize, theme: &impl RichTextTheme) -> Vec<Line<'static>>;
    pub fn flatten(&self) -> Vec<FlatEntry>;
    pub fn build_focusable_items(&self) -> Vec<FocusableItemRange>;
}
```

### コンストラクタ

- **`from_json_str`**: JSON 文字列をツリーに解析します。キーは `KeyStyle::Json`（引用符付き、`:` 区切り）を使用します。
- **`from_toml_str`**: TOML 文字列を解析します（内部的に JSON に変換）。キーは `KeyStyle::Toml`（引用符なし、`=` 区切り）を使用します。
- **`from_value`**: 既存の `serde_json::Value` から、選択したキースタイルでツリーを構築します。

### ツリー操作

```rust
// スラッシュ区切りのパスを使用して切り替え
tree.toggle("dependencies");          // ルートキーを切り替え
tree.toggle("dependencies/serde");    // ネストされたキーを切り替え

// 便利なメソッド
tree.expand_all();
tree.collapse_all();
tree.expand_to_depth(2);

// スクロールコンテキストで — 選択されたアイテムのIDを使用
if scroll.selected_item_id().is_some() {
    tree.handle_toggle(selected_id);
}
```

### レンダリング

- **`render_lines`**: ツリーコネクタと色付きの値を持つ、スタイル付きの `Line` を生成します。
- **`flatten`**: 表示されているすべてのエントリのフラットなリストを返します（折りたたみ状態を反映）。
- **`build_focusable_items`**: ツリーパスに一致する ID を持つ、`HybridScrollView` 統合用のフォーカス可能範囲を返します。

## データ型

```rust
pub struct FlatEntry {
    pub depth: usize,
    pub key: String,
    pub value: String,
    pub kind: EntryKind,
    pub value_type: ValueType,
    pub path: String,          // 例: "dependencies/serde"
    pub is_last: bool,
}

pub enum EntryKind {
    Collapsed,  // 子を持つが現在折りたたまれている: [+]
    Expanded,   // 子を持ち現在展開されている: [-]
    Leaf,       // 子なし
}

pub enum ValueType {
    String, Number, Boolean, Null, Object, Array,
}

pub enum KeyStyle {
    Json,  // "key": value
    Toml,  // key = value
}
```

### 値の型による色分け

各 `ValueType` は対応するテーマカラーメソッドにマッピングされます：

| ValueType | テーマメソッド          |
|-----------|-----------------------|
| String    | `get_json_string_color()` |
| Number    | `get_json_number_color()` |
| Boolean   | `get_json_bool_color()`   |
| Null      | `get_json_null_color()`   |

## ツリー行ヘルパー

`tree_lines` モジュール（`crate::tree` から再エクスポート）は低レベルの行構築を提供します：

```rust
pub fn make_body_line(key: &str, value: &str, style: &Style, value_style: &Style) -> Line<'static>;
pub fn make_status_line(status: &str, style: &Style) -> Line<'static>;
pub fn make_branch_dispatch_line(kind: EntryKind, ...) -> Line<'static>;
```

## 例

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

// すべて展開
tree.expand_all();

// 行にレンダリング
let lines = tree.render_lines(80, theme);

// dependencies サブツリーを折りたたむ
tree.toggle("dependencies");

// 再レンダリング — dependencies は折りたたまれた状態に
let lines = tree.render_lines(80, theme);

// スクロールナビゲーション用のフォーカス可能アイテムを取得
let items = tree.build_focusable_items();
```
