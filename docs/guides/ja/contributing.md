# コントリビューション

## 開発環境のセットアップ

```bash
git clone https://github.com/celestia-island/ratatui-markdown.git
cd ratatui-markdown
cargo build
```

## CI チェック

PR を提出する前に、以下が通過することを確認してください：

```bash
cargo test --all-features
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings
cargo test --no-default-features
cargo test --no-default-features --features markdown
cargo test --no-default-features --features scroll
cargo test --no-default-features --features tree
cargo test --no-default-features --features mermaid
cargo test --no-default-features --features image
cargo test --no-default-features --features viewer
```

## プロジェクトの規約

### コードスタイル

- 標準的な Rust のイディオムに従います（`cargo fmt` と `cargo clippy` がこれを強制します）
- 厳密に必要な場合を除き、コード内にコメントを付けないでください
- 内部の可視性には `pub(crate)` を使用し、公開 API サーフェスにのみ `pub` を使用します

### モジュール構成

各機能モジュールは `src/{module}/` の下に配置されます：

```
src/markdown/       # 機能: markdown
  ├── mod.rs        # 公開型の再エクスポート、MarkdownRenderer の定義
  ├── parser.rs     # ブロックレベルパーサー (impl MarkdownRenderer)
  ├── types.rs      # MarkdownBlock, TextToken 列挙型
  ├── render.rs     # ブロックレベルレンダラー
  ├── inline.rs     # インライン書式パーサー
  ├── text.rs       # テキスト折り返しユーティリティ
  ├── tests.rs      # パーサー/統合テスト
  └── render_tests.rs  # レンダリング出力のスナップショットテスト
```

テストはソースファイルと同じ場所にある `#[cfg(test)] mod tests { }` ブロック内に記述され、より大きなテストスイートは専用の `tests.rs` / `render_tests.rs` ファイルに配置されます。

### 機能フラグ

すべての機能はデフォルトで有効です。コードをゲートするには `cfg(feature = "X")` を使用します：

```rust
#[cfg(feature = "markdown")]
pub mod markdown;
```

機能の依存関係は `Cargo.toml` で表現されます：

```toml
tree = ["dep:serde_json", "dep:toml", "scroll"]
preview = ["markdown", "scroll", "tree"]
```

### テスト

すべてのテストを実行：

```bash
cargo test --all-features
```

各機能の組み合わせをテスト：

```bash
cargo test --no-default-features
cargo test --no-default-features --features markdown
cargo test --no-default-features --features scroll
cargo test --no-default-features --features tree
cargo test --no-default-features --features mermaid
cargo test --no-default-features --features image
cargo test --no-default-features --features viewer
cargo test --no-default-features --features preview  # markdown, scroll, tree を含む
```

### ドキュメント

- API ドキュメントは Rust のドキュメントコメント規約に従います
- ユーザー向けドキュメントは `docs/guides/` の下に配置されます
- `docs/guides/en/` が正規（英語）ドキュメントです
- 他の言語への翻訳は `docs/guides/{lang}/` の下に歓迎されます

## コミットメッセージのスタイル

conventional commit 形式に従います：

```
type: 短い説明

type: feat, fix, refactor, test, docs, chore, ci, style
```

## リリースプロセス

crates.io への公開は、タグプッシュ時の `publish.yml` GitHub Actions ワークフローによって処理されます。

```bash
# Cargo.toml のバージョンを上げてから：
git tag v0.1.1
git push origin v0.1.1
```
