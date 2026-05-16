<div align="center">
  <img src="../../../examples/logo.webp" alt="ratatui-markdown logo" width="200" />
</div>

# ratatui-markdown

> ratatui 向けの Markdown レンダリング、Mermaid ダイアグラム、シンタックスハイライト、折りたたみ可能な JSON/TOML ツリー、そしてリッチなスクロールウィジェットを提供する Rust ライブラリです。
>
> **ビルド基盤**: [ratatui](https://github.com/ratatui/ratatui) 0.29 + 純粋 Rust
>
> **最小 Rust バージョン**: 1.74

<div align="center">
  <p>
    <a href="../../README.md">English</a> |
    <a href="../zhs/index.md">简体中文</a> |
    <a href="../zht/index.md">繁體中文</a> |
    <a href="../ja/index.md">日本語</a> |
    <a href="../ko/index.md">한국어</a> |
    <a href="../fr/index.md">Français</a> |
    <a href="../es/index.md">Español</a> |
    <a href="../ru/index.md">Русский</a> |
    <a href="../ar/index.md">العربية</a>
  </p>
</div>

## ratatui-markdown とは？

ratatui-markdown は、[ratatui](https://github.com/ratatui/ratatui) 上に構築された、機能豊富なターミナル UI レンダリングライブラリです。複数の機能モジュールを提供し、独立して使用することも、`MarkdownPreview` / `MarkdownViewer` ウィジェットで組み合わせることもできます。

## コアモジュール

### Markdown レンダリング

Markdown テキストを解析し、スタイル付きのターミナル出力としてレンダリングします：

- **見出し**: H1 (`#`), H2 (`##`), H3 (`###`)
- **段落**: CJK 文字幅を考慮した自動折り返し
- **インライン書式**: `**太字**`, `*斜体*`, `***太字+斜体***`, `` `インラインコード` ``
- **フェンスコードブロック**: 言語ラベル付き（mermaid ブロックは図としてレンダリング）
- **引用ブロック** (`>`)
- **順序なしリスト** (`-`, `*`, `+`) と順序付きリスト (`1.`, `2.`)
- **水平線** (`---`, `***`, `___`)
- **テーブル**: 列幅の比例配分、セル折り返し対応

### 折りたたみツリービュー

構造化データを解析し、インタラクティブに閲覧します：

- **JSON** と **TOML** を折りたたみ可能なツリーに解析
- **展開 / 折りたたみ**: 個別ノード、全展開、全折りたたみ、深さ指定展開
- **キースタイル**: JSON モード（引用符付きキー + `:`）または TOML モード（裸キー + `=`）
- **キーボードナビゲーション**: カーソルベースの選択と切り替え
- **値の型による色分け**: 文字列、数値、真偽値、null それぞれにテーマカラー

### ハイブリッドスクロールシステム

自由な閲覧とアイテムナビゲーションの両方を処理するスマートスクロール：

- **自由スクロールモード**: コンテンツを自由にスクロール
- **エンゲージモード**: フォーカス可能なアイテムがビューポート中央に入ると自動的に有効化
- **カーソルナビゲーション**: キーボードでフォーカス可能アイテム間を移動
- **カーソルインジケーター**: エンゲージされた行に `> ` プレフィックスを表示
- **スクロールバー**: 矢印ベースのオーバーレイ表示
- **ページネーション**: `page_up` / `page_down` 対応

### Mermaid ダイアグラム

ターミナルで直接 Mermaid ダイアグラムをレンダリング：

- **シーケンス図**、**円グラフ**、**ガントチャート**、**状態遷移図**
- ` ```mermaid ` コードブロックでトリガー
- 機能フラグ：`mermaid`

### シンタックスハイライト

tree-sitter ベースのコードブロックハイライト：

- 言語ごとの機能フラグ（`highlight-lang-rust`、`highlight-lang-python` など）
- `highlight-lang-all` で全言語を一括有効化
- `HighlightHooks` でカスタマイズ可能

### MarkdownPreview / MarkdownViewer ウィジェット

すべてを統合する高レベルウィジェット：

- マークダウンコンテンツ、ツリービュー、アクションアイテムを単一のスクロール可能なレイアウトにレンダリング
- **キャッシング**: コンテンツ、幅、またはテーマ世代が変更された場合のみ出力を再構築
- **TOML フロントマターの除去**: `+++` で区切られた TOML フロントマターを自動除去
- **アクションアイテム**: アクション ID 付きのキーボード選択可能なラベルアイテム
- すべてのナビゲーションを `HybridScrollView` に委譲

## クイックスタート

```toml
[dependencies]
ratatui-markdown = "0.2"
```

### 例

| 例                   | 説明                               | 必要な機能フラグ                |
|----------------------|------------------------------------|-------------------------------|
| `basic`              | 最小限の Markdown レンダリング      | —                             |
| `code`               | シンタックスハイライトコードブロック | `highlight-lang-all`          |
| `custom_code_block`  | カスタムコードブロックレンダリング   | —                             |
| `image`              | 画像の埋め込みとズーム              | `image`                       |
| `mermaid`            | Mermaid ダイアグラムレンダリング    | `mermaid`                     |
| `tree_list`          | 折りたたみ JSON/TOML ツリービュー   | —                             |

```bash
cargo run --example basic
cargo run --example code --features highlight-lang-all
cargo run --example image --features image
cargo run --example mermaid --features mermaid
cargo run --example tree_list
```

## 機能フラグ

デフォルトですべての機能が有効です。デフォルト機能を無効にして必要なものだけを有効にすることもできます：

```toml
[dependencies]
ratatui-markdown = { version = "0.2", default-features = false, features = ["markdown"] }
```

| 機能                 | 依存関係                            | 説明                                  |
|----------------------|-------------------------------------|--------------------------------------|
| `markdown`           | —                                   | Markdown パーサーとレンダラー          |
| `image`              | —                                   | `ImageResolver` による画像解決        |
| `scroll`             | —                                   | HybridScrollView、スクロール可能リスト |
| `tree`               | `scroll`, `serde_json`, `toml`      | 折りたたみ JSON/TOML ツリー           |
| `preview`            | `markdown`, `scroll`, `tree`        | MarkdownPreview 統合ウィジェット      |
| `mermaid`            | `markdown`                          | Mermaid ダイアグラムレンダリング       |
| `viewer`             | `markdown`, `scroll`                | MarkdownViewer ウィジェット           |
| `highlight`          | —                                   | tree-sitter ベースのシンタックスハイライト |
| `highlight-lang-*`   | `highlight`                         | 個別言語グラマー                      |
| `highlight-lang-all` | `highlight`                         | 全バンドル言語グラマー                 |

## ドキュメント

| ガイド | 説明 |
|--------|------|
| [はじめに](getting-started.md) | セットアップと最初のレンダリング |
| [Markdown](markdown.md) | Markdown の解析とレンダリング |
| [スクロールシステム](scroll.md) | ハイブリッドスクロール、ナビゲーション |
| [ツリービュー](tree.md) | JSON/TOML ツリー、展開/折りたたみ |
| [プレビューウィジェット](preview.md) | MarkdownPreview ですべてを統合 |
| [テーマ](theme.md) | RichTextTheme の実装 |
| [コントリビューション](contributing.md) | 開発とコントリビューションガイド |

## ライセンス

MIT OR Apache-2.0 のデュアルライセンス。
