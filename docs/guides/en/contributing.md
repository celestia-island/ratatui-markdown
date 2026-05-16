# Contributing

## Development Setup

```bash
git clone https://github.com/celestia-island/ratatui-markdown.git
cd ratatui-markdown
cargo build
```

## CI Checks

Before submitting a PR, ensure the following pass:

```bash
cargo test --all-features
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings
cargo test --no-default-features
cargo test --no-default-features --features markdown
cargo test --no-default-features --features scroll
cargo test --no-default-features --features tree
```

## Project Conventions

### Code Style

- Follow standard Rust idioms (`cargo fmt` and `cargo clippy` enforce this)
- No comments in code unless strictly necessary
- Use `pub(crate)` for internal visibility; `pub` only for the public API surface

### Module Organization

Each feature module lives under `src/{module}/`:

```
src/markdown/       # Feature: markdown
  ├── mod.rs        # Re-exports public types, defines MarkdownRenderer
  ├── parser.rs     # Block-level parser (impl MarkdownRenderer)
  ├── types.rs      # MarkdownBlock, TextToken enums
  ├── render.rs     # Block-level renderer
  ├── inline.rs     # Inline formatting parser
  ├── text.rs       # Text wrapping utilities
  ├── tests.rs      # Parser/integration tests
  └── render_tests.rs  # Render output snapshot tests
```

Tests live alongside source files inside `#[cfg(test)] mod tests { }` blocks, with larger test suites in dedicated `tests.rs` / `render_tests.rs` files.

### Feature Flags

All features are enabled by default. Use `cfg(feature = "X")` to gate code:

```rust
#[cfg(feature = "markdown")]
pub mod markdown;
```

Feature dependencies are expressed in `Cargo.toml`:

```toml
tree = ["dep:serde_json", "dep:toml", "scroll"]
preview = ["markdown", "scroll", "tree"]
```

### Testing

Run all tests:

```bash
cargo test --all-features
```

Test each feature combination:

```bash
cargo test --no-default-features
cargo test --no-default-features --features markdown
cargo test --no-default-features --features scroll
cargo test --no-default-features --features tree
cargo test --no-default-features --features mermaid
cargo test --no-default-features --features image
cargo test --no-default-features --features viewer
cargo test --no-default-features --features preview  # implies markdown, scroll, tree
```

### Documentation

- API docs follow Rust's doc-comment conventions
- User-facing documentation lives under `docs/guides/`
- `docs/guides/en/` is the canonical (English) documentation
- Translations welcome for other languages under `docs/guides/{lang}/`

## Commit Message Style

Follow conventional commit format:

```
type: short description

type: feat, fix, refactor, test, docs, chore, ci, style
```

## Release Process

Publishing to crates.io is handled by the `publish.yml` GitHub Actions workflow on tag push.

```bash
# Bump version in Cargo.toml, then:
git tag v0.1.1
git push origin v0.1.1
```
