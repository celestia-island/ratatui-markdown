# 스크롤 시스템

> 포커스 가능 아이템 내비게이션을 갖춘 스마트 하이브리드 스크롤.

## 개요

`scroll` 모듈은 두 가지 모드를 지원하는 하이브리드 스크롤 시스템을 제공합니다:

1. **자유 스크롤** — 뷰포트 내에 포커스 가능한 아이템이 없을 때, 콘텐츠가 자유롭게 스크롤됩니다
2. **인게이지** — 포커스 가능한 아이템이 뷰포트 중앙에 진입하면, 커서가 키보드 내비게이션을 위해 첫 번째 아이템에 고정됩니다

`scroll` 기능 플래그로 게이트됩니다 (기본 활성화).

## HybridScrollView

스크롤, 포커스 영역, 렌더링을 관리하는 핵심 위젯입니다:

```rust
pub struct HybridScrollView { /* 필드 */ }

impl HybridScrollView {
    pub fn new() -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;
    pub fn with_cursor_indicator(self, show: bool) -> Self;

    // 콘텐츠 관리
    pub fn set_content(&mut self, lines: Vec<Line<'static>>, regions: Vec<FocusableRegion>);
    pub fn set_lines(&mut self, lines: Vec<Line<'static>>);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // 스크롤 상태
    pub fn total_lines(&self) -> usize;
    pub fn get_scroll_offset(&self) -> usize;
    pub fn get_viewport_height(&self) -> usize;
    pub fn set_scroll_offset(&mut self, offset: usize);

    // 내비게이션
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // 인게이지먼트
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;
    pub fn engage_first(&mut self);
    pub fn engage_by_id(&mut self, id: &str) -> bool;

    // 렌더링
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

### 설정

- **with_left_padding**: 표시되는 모든 라인에 왼쪽 여백 1열을 추가합니다
- **with_cursor_indicator**: 인게이지된 커서 라인에 `> ` (2열)을 표시합니다 (`left_padding` 보다 우선)

`effective_padding()` 메서드는 실제 사용되는 여백을 반환합니다:
- 커서 표시기가 활성화된 경우 `2`
- 왼쪽 여백만 있는 경우 `1`
- 그 외의 경우 `0`

## 포커스 가능 영역과 아이템

```rust
pub struct FocusableItemRange {
    pub start_line: usize,    // 포함
    pub end_line: usize,      // 제외
    pub id: String,           // 고유 식별자
}

pub struct FocusableRegion {
    pub items: Vec<FocusableItemRange>,
}
```

영역은 상호작용 가능해지는 라인 범위를 정의합니다. 뷰포트 중앙이 영역을 통과하면 스크롤 뷰가 자동으로 인게이지되고 커서가 첫 번째 아이템에 위치합니다.

### 인게이지먼트 동작

- 영역으로 **아래로** 스크롤하면 **첫 번째** 아이템에 인게이지됩니다
- 영역으로 **위로** 스크롤하면 **마지막** 아이템에 인게이지됩니다
- 영역의 마지막 아이템을 지나 탐색하면 **디스인게이지** 되어 자유 스크롤로 돌아갑니다
- `scroll_to_top()` 과 `scroll_to_bottom()` 은 항상 디스인게이지됩니다
- 영역 내에서 `scroll_up`/`scroll_down` 은 아이템 간 커서를 이동합니다

## 기타 스크롤 위젯

### ScrollableList<T>

마우스/키보드 내비게이션과 선택적 테두리 렌더링을 갖춘 제네릭 스크롤 가능 리스트:

```rust
pub trait ListItemRenderer {
    fn render_item(&self, index: usize, is_selected: bool, width: usize) -> Line<'static>;
}
```

### ArrowScrollbar

유니코드 화살표 기호로 그려진 커스텀 스크롤바:

```rust
pub fn render_arrow_scrollbar(
    area: Rect,
    buf: &mut Buffer,
    top: usize,
    bottom: usize,
    theme: &impl RichTextTheme,
);
```

### FollowScrollState

콘텐츠 자동 추적용 (예: 스트리밍 출력):

```rust
pub struct FollowScrollState {
    // 뷰포트가 하단에 있는지 추적
}
```

### ScrollableRenderResult

간단한 스크롤 가능 패널 래퍼:

```rust
pub fn render_scrollable(
    lines: &[Line],
    scroll_offset: usize,
    area: Rect,
    buf: &mut Buffer,
) -> ScrollableRenderResult;
```

## 예제

```rust
use ratatui_markdown::scroll::{HybridScrollView, FocusableItemRange, FocusableRegion};

let mut scroll = HybridScrollView::new()
    .with_cursor_indicator(true);

// 콘텐츠 라인
let lines: Vec<Line> = (0..100)
    .map(|i| Line::raw(format!("Line {}", i)))
    .collect();

// 라인 30-32를 포커스 가능하게 설정
let region = FocusableRegion {
    items: vec![
        FocusableItemRange { start_line: 30, end_line: 31, id: "item-a".into() },
        FocusableItemRange { start_line: 31, end_line: 32, id: "item-b".into() },
        FocusableItemRange { start_line: 32, end_line: 33, id: "item-c".into() },
    ],
};

scroll.set_content(lines, vec![region]);

// 자유 스크롤로 아래로 이동; 라인 30이 뷰포트 중앙에 진입하면 자동 인게이지
scroll.scroll_down();

// 아이템 간 탐색
while scroll.scroll_down() == InputResult::Continue {
    if let Some(id) = scroll.selected_item_id() {
        println!("Selected: {}", id);
    }
}
```
