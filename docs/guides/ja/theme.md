# テーマカスタマイズ

> `RichTextTheme` トレイトによるカスタムカラースキームの実装。

## 概要

すべての色とスタイルの参照は `RichTextTheme` トレイトを通じて行われ、レンダリングロジックと外観が分離されています。このトレイトを一度実装すれば、すべてのレンダリング関数に渡すことができます。

## RichTextTheme トレイト

```rust
pub trait RichTextTheme {
    fn generation(&self) -> Generation;

    // 一般テキスト
    fn get_text_color(&self) -> Color;
    fn get_muted_text_color(&self) -> Color;
    fn get_background_color(&self) -> Color;

    // アクセント / 強調
    fn get_primary_color(&self) -> Color;
    fn get_secondary_color(&self) -> Color;
    fn get_info_color(&self) -> Color;

    // 境界線
    fn get_border_color(&self) -> Color;
    fn get_focused_border_color(&self) -> Color;

    // ポップアップ / オーバーレイ
    fn get_popup_selected_background(&self) -> Color;
    fn get_popup_selected_text_color(&self) -> Color;

    // JSON/ツリーの値
    fn get_json_key_color(&self) -> Color;
    fn get_json_string_color(&self) -> Color;
    fn get_json_number_color(&self) -> Color;
    fn get_json_bool_color(&self) -> Color;
    fn get_json_null_color(&self) -> Color;

    // その他
    fn get_accent_yellow(&self) -> Color;
}
```

## Generation トークン

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Generation(pub u64);

impl Generation {
    pub fn next(&self) -> Self { Self(self.0 + 1) }
}
```

`Generation` トークンは `MarkdownPreview` のキャッシュ無効化に使用されます。テーマの `generation()` がキャッシュされた値と異なる値を返すと、プレビューはすべてのレンダリング出力を再構築します。

**実行時にユーザーがテーマを変更するたびに generation をインクリメントしてください**：

```rust
struct AppTheme {
    generation: Generation,
    dark_mode: bool,
}

impl RichTextTheme for AppTheme {
    fn generation(&self) -> Generation { self.generation }

    fn get_text_color(&self) -> Color {
        if self.dark_mode { Color::White } else { Color::Black }
    }

    // ...
}

impl AppTheme {
    fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
        self.generation = self.generation.next();
    }
}
```

## カラースロットの使用法

各テーマカラーが実際にどのように使用されるかを以下に示します：

| メソッド | 使用箇所 |
|--------|----------|
| `get_text_color` | デフォルトの段落テキスト、リストアイテムテキスト |
| `get_muted_text_color` | コードブロック、引用ブロックテキスト、テーブル境界線 |
| `get_primary_color` | 見出し（H1-H3）、太字テキストのアクセント |
| `get_secondary_color` | インラインコード、セカンダリアクセント |
| `get_info_color` | 情報レベルのハイライト |
| `get_background_color` | 一般的な背景の塗りつぶし |
| `get_border_color` | フォーカスされていない境界線（コードブロック、テーブル） |
| `get_focused_border_color` | フォーカス/エンゲージされたアイテムの境界線、スクロールバー |
| `get_popup_selected_background` | 選択されたポップアップアイテムの背景 |
| `get_popup_selected_text_color` | 選択されたポップアップアイテムのテキスト色 |
| `get_json_key_color` | ツリーノードのキー |
| `get_json_string_color` | ツリー内の文字列値 |
| `get_json_number_color` | ツリー内の数値 |
| `get_json_bool_color` | ツリー内の真偽値 |
| `get_json_null_color` | ツリー内の null 値 |
| `get_accent_yellow` | 警告/ハイライトのアクセント |

## 例：完全なテーマ

```rust
use ratatui::style::Color;
use ratatui_markdown::theme::{Generation, RichTextTheme};

struct CatppuccinMocha;

impl RichTextTheme for CatppuccinMocha {
    fn generation(&self) -> Generation { Generation(0) }

    fn get_text_color(&self) -> Color { Color::new(205, 214, 244) }       // text
    fn get_muted_text_color(&self) -> Color { Color::new(147, 153, 178) } // overlay2
    fn get_background_color(&self) -> Color { Color::new(30, 30, 46) }    // base
    fn get_primary_color(&self) -> Color { Color::new(137, 180, 250) }    // blue
    fn get_secondary_color(&self) -> Color { Color::new(203, 166, 247) }  // mauve
    fn get_info_color(&self) -> Color { Color::new(137, 220, 235) }       // sky
    fn get_border_color(&self) -> Color { Color::new(69, 71, 90) }        // surface1
    fn get_focused_border_color(&self) -> Color { Color::new(137, 180, 250) } // blue
    fn get_popup_selected_background(&self) -> Color { Color::new(69, 71, 90) }
    fn get_popup_selected_text_color(&self) -> Color { Color::new(205, 214, 244) }
    fn get_json_key_color(&self) -> Color { Color::new(137, 220, 235) }    // sky
    fn get_json_string_color(&self) -> Color { Color::new(166, 227, 161) } // green
    fn get_json_number_color(&self) -> Color { Color::new(250, 179, 135) } // peach
    fn get_json_bool_color(&self) -> Color { Color::new(203, 166, 247) }   // mauve
    fn get_json_null_color(&self) -> Color { Color::new(108, 112, 134) }   // surface2
    fn get_accent_yellow(&self) -> Color { Color::new(249, 226, 175) }     // yellow
}
```

## パフォーマンスに関する注意

- すべてのテーマメソッドはレンダリング中に頻繁に呼び出されます。定数を返すか、単純なフィールド参照のみを行うようにして、軽量に保ってください。
- `Generation` トークンの比較は単純な整数の等価チェックであるため、インクリメントは高速です。
