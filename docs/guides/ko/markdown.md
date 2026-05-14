# Markdown 모듈

> Markdown 텍스트를 파싱하여 스타일이 적용된 `ratatui::text::Line` 으로 렌더링합니다.

## 개요

`markdown` 모듈은 터미널 출력을 위해 설계된 커스텀 Markdown 파서 및 렌더러를 제공합니다. 이는 CommonMark 호환 파서가 **아닙니다** — 터미널 UI에서 가장 유용한 Markdown 하위 집합을 대상으로 합니다.

`markdown` 기능 플래그로 게이트됩니다 (기본 활성화).

## MarkdownRenderer

```rust
pub struct MarkdownRenderer {
    max_width: usize,
}

impl MarkdownRenderer {
    pub fn new(max_width: usize) -> Self;
    pub fn parse(&self, markdown: &str) -> Vec<MarkdownBlock>;
    pub fn render(&self, blocks: &[MarkdownBlock], theme: &impl RichTextTheme) -> Vec<Line<'static>>;
}
```

### 생성자

`MarkdownRenderer::new(max_width)` 는 사용 가능한 콘텐츠 너비를 열 단위로 받습니다. 이 너비는 단락 텍스트 줄바꿈(CJK 인식)과 테이블 열 크기 조정에 사용됩니다.

### 파싱

`parse()` 는 Markdown 텍스트 `&str` 을 받아 `Vec<MarkdownBlock>` 을 반환합니다. 파서는 라인 지향적이며 블록을 순차적으로 처리합니다.

### 렌더링

`render()` 는 파싱된 블록과 테마를 받아 ratatui 위젯에서 직접 사용할 수 있는 `Vec<Line<'static>>` 을 생성합니다.

## MarkdownBlock 열거형

```rust
pub enum MarkdownBlock {
    Heading1(String),              // # 제목
    Heading2(String),              // ## 제목
    Heading3(String),              // ### 제목
    Paragraph(Vec<String>),        // 줄바꿈된 단락 라인
    CodeBlock(String, String),     // (언어, 내용)
    InlineCode(String),            // `인라인 코드`
    ListItem(String, u8),          // (내용, 들여쓰기 레벨)
    Blockquote(String),            // > 인용 텍스트
    HorizontalRule,                // --- 또는 *** 또는 ___
    BlankLine,                     // 빈 줄
    Table {                        // | col1 | col2 |
        headers: Vec<String>,      //   |------|------|
        rows: Vec<Vec<String>>,    //   | val1 | val2 |
    },
}
```

### 블록 상세

**제목** (H1-H3): `primary_color` 로 렌더링되며, H1은 `Modifier::BOLD` 를 사용합니다.

**단락**: 텍스트는 CJK 인식 줄바꿈으로 `max_width` 에 맞춰 처리됩니다. 각 줄바꿈된 라인은 `Vec<String>` 의 항목이 됩니다.

**코드 블록** (`` ``` `` 으로 감싸짐): 상자 그리기 문자를 사용한 테두리 상자 안에 `muted_text_color` 로 렌더링됩니다. Mermaid 코드 블록은 조용히 건너뜁니다.

**인라인 코드**: `secondary_color` 와 `Modifier::DIM` 으로 렌더링됩니다.

**리스트**: 순서 없는 리스트 (`-`, `*`, `+`) 와 순서 있는 리스트 (`1.`, `2.`). 각 아이템은 들여쓰기 레벨을 유지합니다. 하위 아이템은 시각적으로 들여쓰기됩니다.

**인용 블록**: 색상이 지정된 `│` 막대가 접두사로 붙고 `muted_text_color` 로 렌더링됩니다.

**테이블**: 열은 콘텐츠 너비에 따라 비례적으로 크기가 조정됩니다. 셀은 줄바꿈되며, 헤더는 `Modifier::BOLD` 를 사용하고 테두리는 상자 그리기 문자를 사용합니다.

## 인라인 서식

인라인 서식은 단락 및 리스트 아이템 텍스트 **내부에** 적용됩니다:

| Markdown        | 렌더링 효과                            |
|-----------------|----------------------------------------|
| `**text**`      | **굵게** (`Modifier::BOLD`)           |
| `*text*`        | *기울임* (`Modifier::ITALIC`)         |
| `***text***`    | ***굵게+기울임***                      |
| `` `code` ``    | `인라인 코드` 스타일                    |

```rust
pub fn parse_inline_formatting(text: &str, theme: &impl RichTextTheme) -> Vec<Span<'static>>;
```

이 독립 함수는 `MarkdownRenderer` 외부에서도 사용할 수 있도록 재내보내기됩니다.

## 예제

```rust
use ratatui_markdown::markdown::MarkdownRenderer;

let md = r#"
# Title

This is a paragraph with **bold** and *italic* text.

## Code

```rust
fn main() {
    println!("Hello!");
}
```

| Name | Version |
|------|---------|
| ratatui | 0.29 |
| serde | 1.0 |
"#;

let renderer = MarkdownRenderer::new(80);
let blocks = renderer.parse(md);
let lines = renderer.render(&blocks, theme);
// lines를 ratatui 위젯에서 사용
```
