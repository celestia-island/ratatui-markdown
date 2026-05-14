# MarkdownPreview ウィジェット

> Markdown、ツリー、アクションを1つのスクロール可能なビューに統合する高レベルウィジェット。

## 概要

`MarkdownPreview` は最上位の統合ウィジェットです。Markdown レンダリング、折りたたみ可能ツリー表示、アクションアイテムを単一の `HybridScrollView` に統合します。ほとんどのアプリケーションで推奨されるエントリポイントです。

`preview` 機能フラグで制御されます（デフォルトで有効、他のすべての機能が必要です）。

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

    // コンテンツ
    pub fn set_content(&mut self, content: &str);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // ツリー
    pub fn set_tree(&mut self, tree: Option<CollapsibleTree>);
    pub fn tree_mut(&mut self) -> Option<&mut CollapsibleTree>;
    pub fn has_tree(&self) -> bool;
    pub fn toggle_tree_node(&mut self) -> bool;

    // アクションアイテム
    pub fn set_action_items(&mut self, items: Vec<ActionItem>);
    pub fn selected_action_id(&self) -> Option<&str>;

    // ナビゲーション
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // 状態
    pub fn total_lines(&self) -> usize;
    pub fn scroll_offset(&self) -> usize;
    pub fn visible_height(&self) -> usize;
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;

    // レンダリング
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

## 設定

### with_strip_frontmatter

有効時（デフォルト）、`+++` で区切られたコンテンツは TOML フロントマターとして扱われ、レンダリング前に除去されます：

```
+++
title = "My Document"
author = "Jane Doe"
+++

# 実際のコンテンツ
これが本文です。
```

`# 実際のコンテンツ` とそれに続く段落のみが出力に表示されます。

### with_left_padding

レンダリングされたコンテンツに左パディングを1列追加します（`HybridScrollView` に渡されます）。

## コンテンツレイアウト

ウィジェットはコンテンツを以下の順序で縦方向にレンダリングします：

1. **ツリー**（存在する場合） — フォーカス可能ノードを持つツリー行
2. **Markdown** — 解析・レンダリングされたコンテンツ
3. **アクションアイテム** — ブラケットラッパー `[label]` を持つフォーカス可能なアクションラベル

セクション間には空行が挿入され、各セクションはスクロールビュー内で独自の `FocusableRegion` を取得します。

## キャッシング

`MarkdownPreview` はレンダリング出力をキャッシュし、以下の場合のみ再構築します：

- コンテンツの変更（`set_content` で異なるテキストが渡された場合）
- 幅の変更（ターミナルのリサイズ）
- テーマ世代の変更（`theme.generation()` が新しい値を返した場合）
- ツリーの変更（`set_tree`、`toggle_tree_node`）
- アクションアイテムの変更（`set_action_items`）

テーマ変更後の再レンダリングをトリガーするには `theme.generation()` を使用します。

## TOML フロントマターの処理

フロントマターは TOML であると想定されます。これは**解析されず**、単にレンダリング出力から除去されます。最初の `+++` 行でフロントマターモードが開始され、2番目の `+++` で終了します。フロントマターブロックの前後の行は通常通りレンダリングされます。

コンテンツが `+++` で始まらない場合、除去は行われません。

## アクションアイテム

アクションアイテムは、ビューの下部にキーボード選択可能なオプションを提供します：

```rust
preview.set_action_items(vec![
    ActionItem { id: "confirm".into(), label: " Confirm ".into() },
    ActionItem { id: "cancel".into(), label: " Cancel ".into() },
]);

// 入力ハンドラ内で：
if let Some("confirm") = preview.selected_action_id() {
    // confirm の処理
}
```

アクションアイテムの ID は、ツリーノードパスとの衝突を避けるために内部的に `action:` プレフィックスが付加されます。

## 例

```rust
use ratatui_markdown::preview::{MarkdownPreview, ActionItem};
use ratatui_markdown::tree::CollapsibleTree;

let mut preview = MarkdownPreview::new()
    .with_strip_frontmatter(true)
    .with_left_padding(true);

// Markdown コンテンツを設定（TOML フロントマター付き）
preview.set_content(concat!(
    "+++\n",
    "title = \"My Doc\"\n",
    "+++\n",
    "\n",
    "# Hello\n\nContent here.\n",
));

// ツリーを設定
let tree = CollapsibleTree::from_json_str(r#"{"config": {"theme": "dark"}}"#).unwrap();
preview.set_tree(Some(tree));

// アクションアイテムを設定
preview.set_action_items(vec![
    ActionItem { id: "edit".into(), label: " Edit ".into() },
    ActionItem { id: "save".into(), label: " Save ".into() },
]);

// ratatui アプリのレンダリング関数内で：
fn render_ui(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}

// 入力の処理
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
