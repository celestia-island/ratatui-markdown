# 시작하기

## 사전 준비

- **Rust** 1.74 이상
- **ratatui** 0.29 (의존성으로 자동 포함됨)

## 설치

`Cargo.toml` 에 추가하세요:

```toml
[dependencies]
ratatui-markdown = "0.1"
```

기본적으로 모든 기능(`markdown`, `scroll`, `tree`, `preview`, `mermaid`, `image`, `viewer`)이 활성화됩니다.

### 선택적 기능

컴파일 시간과 의존성을 줄이려면 필요한 것만 활성화하세요:

```toml
# Markdown 렌더링만
ratatui-markdown = { version = "0.1", default-features = false, features = ["markdown"] }

# 스크롤 시스템만
ratatui-markdown = { version = "0.1", default-features = false, features = ["scroll"] }

# 트리 뷰 (scroll, serde_json, toml 포함)
ratatui-markdown = { version = "0.1", default-features = false, features = ["tree"] }
```

## 기본 사용법

### Markdown 렌더링

```rust
use ratatui_markdown::markdown::MarkdownRenderer;
use ratatui_markdown::theme::RichTextTheme;

// 최대 콘텐츠 너비로 렌더러 생성
let renderer = MarkdownRenderer::new(80);

// Markdown 텍스트를 블록으로 파싱
let blocks = renderer.parse("# Hello\n\nThis is **bold** and *italic* text.");

// 블록을 ratatui::text::Line<'static> 으로 렌더링
let lines = renderer.render(&blocks, &my_theme);
```

### 트리 탐색

```rust
use ratatui_markdown::tree::CollapsibleTree;

// JSON을 접이식 트리로 파싱
let json_str = r#"{"name": "project", "deps": {"ratatui": "0.29", "serde": "1.0"}}"#;
let mut tree = CollapsibleTree::from_json_str(json_str).unwrap();

// 트리 라인 렌더링
let lines = tree.render_lines(80, &my_theme);

// 내비게이션용 포커스 가능 아이템 가져오기
let items = tree.build_focusable_items();

// 노드 토글
tree.toggle("deps/serde");
```

### MarkdownPreview 위젯 사용

`MarkdownPreview` 위젯은 모든 기능을 하나의 스크롤 가능한 뷰로 결합합니다:

```rust
use ratatui_markdown::preview::MarkdownPreview;
use ratatui_markdown::theme::RichTextTheme;

let mut preview = MarkdownPreview::new()
    .with_left_padding(true);

// Markdown 콘텐츠 설정
preview.set_content("# Welcome\n\n- Item one\n- Item two\n\n```rust\nlet x = 42;\n```");

// 접이식 트리 설정 (선택 사항)
let tree = CollapsibleTree::from_json_str(r#"{"config": {"port": 8080}}"#).unwrap();
preview.set_tree(Some(tree));

// 키보드 입력 처리
preview.scroll_up();
preview.scroll_down();
preview.page_up(10);
preview.page_down(10);
preview.toggle_tree_node(); // Enter 키

// ratatui 그리기 루프에서 렌더링
fn draw(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}
```

## 테마 구현

라이브러리는 모든 색상을 조회하기 위해 트레이트를 사용합니다:

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

`generation()` 반환값을 변경하면 프리뷰 위젯의 캐시가 무효화되고 다시 렌더링됩니다 (예: 사용자가 런타임에 테마를 전환할 때).

## 다음 단계

- [Markdown 모듈](markdown.md) — 전체 Markdown 파싱 및 렌더링 API
- [스크롤 시스템](scroll.md) — 하이브리드 스크롤 아키텍처 이해
- [트리 뷰](tree.md) — JSON/TOML 트리 렌더링 및 상호작용
- [프리뷰 위젯](preview.md) — 고수준 통합 위젯
- [테마](theme.md) — 완전한 테마 사용자 정의 가이드
