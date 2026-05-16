# 贡献指南

## 开发环境搭建

```bash
git clone https://github.com/celestia-island/ratatui-markdown.git
cd ratatui-markdown
cargo build
```

## CI 检查

提交 PR 前，请确保以下检查全部通过：

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

## 项目约定

### 代码风格

- 遵循标准 Rust 习惯用法（由 `cargo fmt` 和 `cargo clippy` 强制执行）
- 除非绝对必要，否则不要在代码中添加注释
- 内部可见性使用 `pub(crate)`；仅对公共 API 层面使用 `pub`

### 模块组织

每个功能模块位于 `src/{module}/` 目录下：

```
src/markdown/       # 功能: markdown
  ├── mod.rs        # 重新导出公共类型，定义 MarkdownRenderer
  ├── parser.rs     # 块级解析器 (impl MarkdownRenderer)
  ├── types.rs      # MarkdownBlock、TextToken 枚举
  ├── render.rs     # 块级渲染器
  ├── inline.rs     # 内联格式化解析器
  ├── text.rs       # 文本换行工具
  ├── tests.rs      # 解析器/集成测试
  └── render_tests.rs  # 渲染输出快照测试
```

测试文件与源文件放在一起，位于 `#[cfg(test)] mod tests { }` 块中，较大的测试套件则放在专用的 `tests.rs` / `render_tests.rs` 文件中。

### 功能标志

所有功能默认启用。使用 `cfg(feature = "X")` 来条件编译代码：

```rust
#[cfg(feature = "markdown")]
pub mod markdown;
```

功能依赖关系在 `Cargo.toml` 中表达：

```toml
tree = ["dep:serde_json", "dep:toml", "scroll"]
preview = ["markdown", "scroll", "tree"]
```

### 测试

运行所有测试：

```bash
cargo test --all-features
```

测试每种功能组合：

```bash
cargo test --no-default-features
cargo test --no-default-features --features markdown
cargo test --no-default-features --features scroll
cargo test --no-default-features --features tree
cargo test --no-default-features --features mermaid
cargo test --no-default-features --features image
cargo test --no-default-features --features viewer
cargo test --no-default-features --features preview  # 隐含 markdown, scroll, tree
```

### 文档

- API 文档遵循 Rust 的文档注释约定
- 面向用户的文档位于 `docs/guides/` 目录下
- `docs/guides/en/` 是规范（英文）文档
- 欢迎其他语言的翻译，放在 `docs/guides/{lang}/` 目录下

## 提交信息风格

遵循 conventional commit 格式：

```
type: 简短描述

类型: feat、fix、refactor、test、docs、chore、ci、style
```

## 发布流程

发布到 crates.io 由 `publish.yml` GitHub Actions 工作流在推送标签时自动处理。

```bash
# 在 Cargo.toml 中更新版本号，然后：
git tag v0.1.1
git push origin v0.1.1
```
