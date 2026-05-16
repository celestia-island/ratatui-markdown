# 기여하기

## 개발 환경 설정

```bash
git clone https://github.com/celestia-island/ratatui-markdown.git
cd ratatui-markdown
cargo build
```

## CI 검사

PR을 제출하기 전에 다음이 통과하는지 확인하세요:

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

## 프로젝트 규칙

### 코드 스타일

- 표준 Rust 관용구를 따릅니다 (`cargo fmt` 와 `cargo clippy` 가 이를 강제합니다)
- 꼭 필요한 경우가 아니면 코드에 주석을 달지 않습니다
- 내부 공개에는 `pub(crate)` 를 사용하고, 공개 API 표면에만 `pub` 를 사용합니다

### 모듈 구성

각 기능 모듈은 `src/{module}/` 아래에 위치합니다:

```
src/markdown/       # 기능: markdown
  ├── mod.rs        # 공개 타입 재내보내기, MarkdownRenderer 정의
  ├── parser.rs     # 블록 레벨 파서 (impl MarkdownRenderer)
  ├── types.rs      # MarkdownBlock, TextToken 열거형
  ├── render.rs     # 블록 레벨 렌더러
  ├── inline.rs     # 인라인 서식 파서
  ├── text.rs       # 텍스트 줄바꿈 유틸리티
  ├── tests.rs      # 파서/통합 테스트
  └── render_tests.rs  # 렌더 출력 스냅샷 테스트
```

테스트는 `#[cfg(test)] mod tests { }` 블록 내에서 소스 파일과 함께 위치하며, 더 큰 테스트 스위트는 전용 `tests.rs` / `render_tests.rs` 파일에 있습니다.

### 기능 플래그

모든 기능은 기본적으로 활성화됩니다. 코드를 게이트하려면 `cfg(feature = "X")` 를 사용하세요:

```rust
#[cfg(feature = "markdown")]
pub mod markdown;
```

기능 의존성은 `Cargo.toml` 에 표현됩니다:

```toml
tree = ["dep:serde_json", "dep:toml", "scroll"]
preview = ["markdown", "scroll", "tree"]
```

### 테스트

모든 테스트 실행:

```bash
cargo test --all-features
```

각 기능 조합 테스트:

```bash
cargo test --no-default-features
cargo test --no-default-features --features markdown
cargo test --no-default-features --features scroll
cargo test --no-default-features --features tree
cargo test --no-default-features --features mermaid
cargo test --no-default-features --features image
cargo test --no-default-features --features viewer
cargo test --no-default-features --features preview  # markdown, scroll, tree 포함
```

### 문서

- API 문서는 Rust의 doc-comment 규칙을 따릅니다
- 사용자 대상 문서는 `docs/guides/` 아래에 위치합니다
- `docs/guides/en/` 이 표준(영어) 문서입니다
- 다른 언어로의 번역은 `docs/guides/{lang}/` 아래에 환영합니다

## 커밋 메시지 스타일

컨벤셔널 커밋 형식을 따르세요:

```
type: 짧은 설명

type: feat, fix, refactor, test, docs, chore, ci, style
```

## 릴리스 프로세스

crates.io 배포는 태그 푸시 시 `publish.yml` GitHub Actions 워크플로우에 의해 처리됩니다.

```bash
# Cargo.toml에서 버전을 올린 후:
git tag v0.1.1
git push origin v0.1.1
```
