# 貢獻指南

## 開發環境設定

```bash
git clone https://github.com/celestia-island/ratatui-markdown.git
cd ratatui-markdown
cargo build
```

## CI 檢查

在提交 PR 之前，請確保以下檢查通過：

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

## 專案慣例

### 程式碼風格

- 遵循標準 Rust 慣用寫法（由 `cargo fmt` 和 `cargo clippy` 強制執行）
- 除非絕對必要，否則程式碼中不加入註解
- 對內部可見性使用 `pub(crate)`；僅對公開 API 使用 `pub`

### 模組組織

每個功能模組位於 `src/{module}/` 下：

```
src/markdown/       # 功能：markdown
  ├── mod.rs        # 重新匯出公開型別，定義 MarkdownRenderer
  ├── parser.rs     # 區塊層級解析器（impl MarkdownRenderer）
  ├── types.rs      # MarkdownBlock、TextToken 列舉
  ├── render.rs     # 區塊層級渲染器
  ├── inline.rs     # 內聯格式化解析器
  ├── text.rs       # 文字換行工具
  ├── tests.rs      # 解析器/整合測試
  └── render_tests.rs  # 渲染輸出快照測試
```

測試與原始碼檔案放在一起，位於 `#[cfg(test)] mod tests { }` 區塊內，較大的測試套件則放在專門的 `tests.rs` / `render_tests.rs` 檔案中。

### 功能標誌

所有功能預設啟用。使用 `cfg(feature = "X")` 來控制程式碼：

```rust
#[cfg(feature = "markdown")]
pub mod markdown;
```

功能依賴在 `Cargo.toml` 中表達：

```toml
tree = ["dep:serde_json", "dep:toml", "scroll"]
preview = ["markdown", "scroll", "tree"]
```

### 測試

執行所有測試：

```bash
cargo test --all-features
```

測試每種功能組合：

```bash
cargo test --no-default-features
cargo test --no-default-features --features markdown
cargo test --no-default-features --features scroll
cargo test --no-default-features --features tree
cargo test --no-default-features --features mermaid
cargo test --no-default-features --features image
cargo test --no-default-features --features viewer
cargo test --no-default-features --features preview  # 包含 markdown, scroll, tree
```

### 文件

- API 文件遵循 Rust 的文件註解慣例
- 面向使用者的文件放在 `docs/guides/` 下
- `docs/guides/en/` 是標準（英文）文件
- 歡迎其他語言的翻譯，放在 `docs/guides/{lang}/` 下

## 提交訊息風格

遵循 Conventional Commit 格式：

```
type: 簡短描述

type: feat, fix, refactor, test, docs, chore, ci, style
```

## 發佈流程

發佈到 crates.io 由 `publish.yml` GitHub Actions 工作流程在推送標籤時處理。

```bash
# 在 Cargo.toml 中遞增版本號，然後：
git tag v0.1.1
git push origin v0.1.1
```
