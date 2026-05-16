# はじめに

## 前提条件

- **Rust** 1.74 以上
- **ratatui** 0.29（依存として自動的に取り込まれます）

## インストール

`Cargo.toml` に以下を追加します：

```toml
[dependencies]
ratatui-markdown = "0.1"
```

これにより、デフォルトですべての機能（`markdown`、`scroll`、`tree`、`preview`、`mermaid`、`image`、`viewer`）が有効になります。

### 機能の選択

コンパイル時間と依存関係を減らすには、必要なものだけを有効にします：

```toml
# Markdown レンダリングのみ
ratatui-markdown = { version = "0.1", default-features = false, features = ["markdown"] }

# スクロールシステムのみ
ratatui-markdown = { version = "0.1", default-features = false, features = ["scroll"] }

# ツリービュー (scroll, serde_json, toml を含む)
ratatui-markdown = { version = "0.1", default-features = false, features = ["tree"] }
```

## 基本的な使い方

### Markdown のレンダリング

```rust
use ratatui_markdown::markdown::MarkdownRenderer;
use ratatui_markdown::theme::RichTextTheme;

// 最大コンテンツ幅を指定してレンダラーを作成
let renderer = MarkdownRenderer::new(80);

// Markdown テキストをブロックに解析
let blocks = renderer.parse("# こんにちは\n\nこれは**太字**と*斜体*のテキストです。");

// ブロックを ratatui::text::Line<'static> にレンダリング
let lines = renderer.render(&blocks, &my_theme);
```

### ツリーの閲覧

```rust
use ratatui_markdown::tree::CollapsibleTree;

// JSON を折りたたみ可能なツリーに解析
let json_str = r#"{"name": "project", "deps": {"ratatui": "0.29", "serde": "1.0"}}"#;
let mut tree = CollapsibleTree::from_json_str(json_str).unwrap();

// ツリー行をレンダリング
let lines = tree.render_lines(80, &my_theme);

// ナビゲーション用のフォーカス可能アイテムを取得
let items = tree.build_focusable_items();

// ノードを切り替え
tree.toggle("deps/serde");
```

### MarkdownPreview ウィジェットの使用

`MarkdownPreview` ウィジェットは、すべてを単一のスクロール可能なビューに統合します：

```rust
use ratatui_markdown::preview::MarkdownPreview;
use ratatui_markdown::theme::RichTextTheme;

let mut preview = MarkdownPreview::new()
    .with_left_padding(true);

// Markdown コンテンツを設定
preview.set_content("# ようこそ\n\n- 項目1\n- 項目2\n\n```rust\nlet x = 42;\n```");

// 折りたたみツリーを設定（オプション）
let tree = CollapsibleTree::from_json_str(r#"{"config": {"port": 8080}}"#).unwrap();
preview.set_tree(Some(tree));

// キーボード入力を処理
preview.scroll_up();
preview.scroll_down();
preview.page_up(10);
preview.page_down(10);
preview.toggle_tree_node(); // Enter キー

// ratatui の描画ループ内でレンダリング
fn draw(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}
```

## テーマの実装

このライブラリはトレイトを使用してすべての色を参照します：

```rust
use ratatui::style::Color;
use ratatui_markdown::theme::{Generation, RichTextTheme};

struct MyTheme;

impl RichTextTheme for MyTheme {
    fn generation(&self) -> Generation { Generation(1) }
    fn get_text_color(&self) -> Color { Color::White }
    fn get_muted_text_color(&self) -> Color { Color::Gray }
    fn get_primary_color(&self) -> Color { Color::Cyan }
    fn get_secondary_color(&self) -> Color { Color::Blue }
    fn get_info_color(&self) -> Color { Color::LightBlue }
    fn get_background_color(&self) -> Color { Color::Black }
    fn get_border_color(&self) -> Color { Color::DarkGray }
    fn get_focused_border_color(&self) -> Color { Color::White }
    fn get_popup_selected_background(&self) -> Color { Color::DarkGray }
    fn get_popup_selected_text_color(&self) -> Color { Color::White }
    fn get_json_key_color(&self) -> Color { Color::LightCyan }
    fn get_json_string_color(&self) -> Color { Color::Green }
    fn get_json_number_color(&self) -> Color { Color::Yellow }
    fn get_json_bool_color(&self) -> Color { Color::Magenta }
    fn get_json_null_color(&self) -> Color { Color::DarkGray }
    fn get_accent_yellow(&self) -> Color { Color::Yellow }
}
```

`generation()` の戻り値を変更することで、プレビューウィジェットのキャッシュを無効化し、再レンダリングを強制できます（例：実行時にユーザーがテーマを切り替えた場合）。

## 次のステップ

- [Markdown モジュール](markdown.md) — Markdown の解析とレンダリング API の詳細
- [スクロールシステム](scroll.md) — ハイブリッドスクロールアーキテクチャの理解
- [ツリービュー](tree.md) — JSON/TOML ツリーのレンダリングと操作
- [プレビューウィジェット](preview.md) — 高レベルな統合ウィジェット
- [テーマ](theme.md) — テーマの完全なカスタマイズガイド
