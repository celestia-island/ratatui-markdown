<div align="center">
  <img src="../../../examples/logo.webp" alt="ratatui-markdown logo" width="200" />
</div>

# ratatui-markdown

> ratatui 를 위한 Markdown 렌더링, Mermaid 다이어그램, 구문 강조, 접이식 JSON/TOML 트리, 그리고 풍부한 스크롤 위젯을 제공하는 Rust 라이브러리입니다.
>
> **빌드 기반**: [ratatui](https://github.com/ratatui/ratatui) 0.29 + 순수 Rust
>
> **최소 Rust 버전**: 1.74

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

## ratatui-markdown 이란?

ratatui-markdown 은 [ratatui](https://github.com/ratatui/ratatui) 위에 구축된 기능이 풍부한 터미널 UI 렌더링 라이브러리입니다. 여러 기능 모듈을 제공하며, 독립적으로 사용하거나 `MarkdownPreview` / `MarkdownViewer` 위젯으로 결합할 수 있습니다.

## 핵심 모듈

### Markdown 렌더링

Markdown 텍스트를 구문 분석하고 스타일이 적용된 터미널 출력으로 렌더링합니다:

- **제목**: H1 (`#`), H2 (`##`), H3 (`###`)
- **단락**: CJK 문자 너비를 고려한 자동 줄바꿈
- **인라인 서식**: `**굵게**`, `*기울임*`, `***굵게+기울임***`, `` `인라인 코드` ``
- **코드 블록**: 언어 레이블 포함 (mermaid 블록은 다이어그램으로 렌더링)
- **인용 블록** (`>`)
- **순서 없는 리스트** (`-`, `*`, `+`) 와 순서 있는 리스트 (`1.`, `2.`)
- **수평선** (`---`, `***`, `___`)
- **테이블**: 열 너비 비례 할당, 셀 줄바꿈 지원

### 접이식 트리 뷰

구조화된 데이터를 구문 분석하고 대화형으로 탐색합니다:

- **JSON** 과 **TOML** 을 접이식 트리로 구문 분석
- **펼치기 / 접기**: 개별 노드, 전체 펼치기, 전체 접기, 깊이 지정 펼치기
- **키 스타일**: JSON 모드(따옴표 키 + `:`) 또는 TOML 모드(베어 키 + `=`)
- **키보드 탐색**: 커서 기반 선택 및 토글
- **값 유형별 색상**: 문자열, 숫자, 불리언, null 각각에 테마 색상 적용

### 하이브리드 스크롤 시스템

자유로운 탐색과 아이템 내비게이션을 모두 처리하는 스마트 스크롤:

- **자유 스크롤 모드**: 콘텐츠를 자유롭게 스크롤
- **인게이지 모드**: 포커스 가능한 아이템이 뷰포트 중앙에 진입하면 자동 활성화
- **커서 내비게이션**: 키보드로 포커스 가능 아이템 간 이동
- **커서 표시기**: 인게이지된 행에 `> ` 접두사 표시
- **스크롤바**: 화살표 기반 오버레이 표시
- **페이지 이동**: `page_up` / `page_down` 지원

### Mermaid 다이어그램

터미널에서 직접 Mermaid 다이어그램 렌더링:

- **시퀀스 다이어그램**, **원형 차트**, **간트 차트**, **상태 다이어그램**
- ` ```mermaid ` 코드 블록으로 트리거
- 기능 플래그: `mermaid`

### 구문 강조

tree-sitter 기반 코드 블록 구문 강조:

- 언어별 기능 플래그 (`highlight-lang-rust`, `highlight-lang-python` 등)
- `highlight-lang-all`로 모든 언어 일괄 활성화
- `HighlightHooks`로 커스터마이즈 가능

### MarkdownPreview / MarkdownViewer 위젯

모든 것을 통합하는 고수준 위젯:

- 마크다운 콘텐츠, 트리 뷰, 액션 아이템을 단일 스크롤 가능한 레이아웃으로 렌더링
- **캐싱**: 콘텐츠, 너비 또는 테마 세대가 변경될 때만 출력 재구축
- **TOML 프론트매터 제거**: `+++` 로 구분된 TOML 프론트매터 자동 제거
- **액션 아이템**: 액션 ID가 있는 키보드 선택 가능한 레이블 아이템
- 모든 탐색을 `HybridScrollView` 에 위임

## 빠른 시작

```toml
[dependencies]
ratatui-markdown = "0.2"
```

### 예제

| 예제                 | 설명                               | 필요 기능 플래그               |
|----------------------|------------------------------------|-------------------------------|
| `basic`              | 최소 Markdown 렌더링               | —                             |
| `code`               | 구문 강조 코드 블록                | `highlight-lang-all`          |
| `custom_code_block`  | 커스텀 코드 블록 렌더링 훅          | —                             |
| `image`              | 이미지 임베딩 및 확대/축소          | `image`                       |
| `mermaid`            | Mermaid 다이어그램 렌더링           | `mermaid`                     |
| `tree_list`          | 접이식 JSON/TOML 트리 뷰           | —                             |

```bash
cargo run --example basic
cargo run --example code --features highlight-lang-all
cargo run --example image
cargo run --example mermaid
cargo run --example tree_list
```

## 기능 플래그

기본적으로 모든 기능이 활성화됩니다. 기본 기능을 비활성화하고 필요한 것만 활성화할 수 있습니다:

```toml
[dependencies]
ratatui-markdown = { version = "0.2", default-features = false, features = ["markdown"] }
```

| 기능                  | 의존성                             | 설명                                   | 기본값 |
|----------------------|------------------------------------|---------------------------------------|---------|
| `markdown`           | —                                  | Markdown 파서 및 렌더러                | ✓       |
| `image`              | —                                  | `ImageResolver`를 통한 이미지 해결      | ✓       |
| `scroll`             | —                                  | HybridScrollView, 스크롤 가능 리스트    | ✓       |
| `tree`               | `scroll`, `serde_json`, `toml`     | 접이식 JSON/TOML 트리                  | ✓       |
| `preview`            | `markdown`, `scroll`, `tree`       | MarkdownPreview 통합 위젯              | ✓       |
| `mermaid`            | `markdown`                         | Mermaid 다이어그램 렌더링               | ✓       |
| `viewer`             | `markdown`, `scroll`               | MarkdownViewer 위젯                    | ✓       |
| `highlight`          | —                                  | tree-sitter 기반 구문 강조              |         |
| `highlight-lang-*`   | `highlight`                        | 개별 언어 문법                         |         |
| `highlight-lang-all` | `highlight`                        | 모든 번들 언어 문법                     |         |

## 문서

| 가이드 | 설명 |
|--------|------|
| [시작하기](getting-started.md) | 설치 및 첫 렌더링 |
| [Markdown](markdown.md) | Markdown 구문 분석 및 렌더링 |
| [스크롤 시스템](scroll.md) | 하이브리드 스크롤, 내비게이션 |
| [트리 뷰](tree.md) | JSON/TOML 트리, 펼치기/접기 |
| [프리뷰 위젯](preview.md) | MarkdownPreview 로 모든 것 통합 |
| [테마](theme.md) | RichTextTheme 구현 |
| [기여하기](contributing.md) | 개발 및 기여 가이드 |

## 라이선스

MIT OR Apache-2.0 듀얼 라이선스.
