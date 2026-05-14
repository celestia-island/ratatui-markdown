# MarkdownPreview 위젯

> Markdown, 트리, 액션을 하나의 스크롤 가능한 뷰로 결합하는 고수준 통합 위젯.

## 개요

`MarkdownPreview` 는 최상위 통합 위젯입니다. Markdown 렌더링, 접이식 트리 표시, 액션 아이템을 단일 `HybridScrollView` 로 결합합니다. 대부분의 애플리케이션에서 권장되는 진입점입니다.

`preview` 기능 플래그로 게이트됩니다 (기본 활성화, 다른 모든 기능 필요).

## API

```rust
pub struct ActionItem {
    pub id: String,
    pub label: String,
}

impl MarkdownPreview {
    pub fn new() -> Self;

    // 설정
    pub fn with_strip_frontmatter(self, strip: bool) -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;

    // 콘텐츠
    pub fn set_content(&mut self, content: &str);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // 트리
    pub fn set_tree(&mut self, tree: Option<CollapsibleTree>);
    pub fn tree_mut(&mut self) -> Option<&mut CollapsibleTree>;
    pub fn has_tree(&self) -> bool;
    pub fn toggle_tree_node(&mut self) -> bool;

    // 액션 아이템
    pub fn set_action_items(&mut self, items: Vec<ActionItem>);
    pub fn selected_action_id(&self) -> Option<&str>;

    // 내비게이션
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // 상태
    pub fn total_lines(&self) -> usize;
    pub fn scroll_offset(&self) -> usize;
    pub fn visible_height(&self) -> usize;
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;

    // 렌더링
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

## 설정

### with_strip_frontmatter

활성화된 경우(기본값), `+++` 로 구분된 콘텐츠는 TOML 프론트매터로 처리되어 렌더링 전에 제거됩니다:

```
+++
title = "My Document"
author = "Jane Doe"
+++

# Actual Content
This is the body.
```

`# Actual Content` 와 그 다음 단락만 출력에 표시됩니다.

### with_left_padding

렌더링된 콘텐츠에 왼쪽 여백 1열을 추가합니다 (`HybridScrollView` 로 전달됨).

## 콘텐츠 레이아웃

위젯은 콘텐츠를 수직 순서로 렌더링합니다:

1. **트리** (있는 경우) — 포커스 가능 노드가 있는 트리 라인
2. **Markdown** — 파싱 및 렌더링된 콘텐츠
3. **액션 아이템** — 대괄호 래퍼 `[label]` 가 있는 포커스 가능 액션 레이블

각 섹션 사이에 빈 줄이 삽입되며, 각 섹션은 스크롤 뷰에서 자체 `FocusableRegion` 을 가집니다.

## 캐싱

`MarkdownPreview` 는 렌더링된 출력을 캐시하며 다음 경우에만 다시 빌드합니다:

- 콘텐츠 변경 (`set_content` 에 다른 텍스트 전달)
- 너비 변경 (터미널 크기 조정)
- 테마 세대 변경 (`theme.generation()` 이 새로운 값 반환)
- 트리 수정 (`set_tree`, `toggle_tree_node`)
- 액션 아이템 수정 (`set_action_items`)

테마 변경 후 다시 렌더링하려면 `theme.generation()` 을 사용하세요.

## TOML 프론트매터 처리

프론트매터는 TOML로 가정됩니다. 이는 **파싱되지 않으며** — 렌더링된 출력에서 단순히 제거됩니다. 첫 번째 `+++` 라인이 프론트매터 모드를 시작하고, 두 번째 `+++` 가 종료합니다. 프론트매터 블록 이전과 이후의 라인은 정상적으로 렌더링됩니다.

콘텐츠가 `+++` 로 시작하지 않으면 제거되지 않습니다.

## 액션 아이템

액션 아이템은 뷰 하단에 키보드로 선택 가능한 옵션을 제공합니다:

```rust
preview.set_action_items(vec![
    ActionItem { id: "confirm".into(), label: " Confirm ".into() },
    ActionItem { id: "cancel".into(), label: " Cancel ".into() },
]);

// 입력 핸들러에서:
if let Some("confirm") = preview.selected_action_id() {
    // confirm 처리
}
```

액션 아이템 ID는 트리 노드 경로와의 충돌을 방지하기 위해 내부적으로 `action:` 접두사가 붙습니다.

## 예제

```rust
use ratatui_markdown::preview::{MarkdownPreview, ActionItem};
use ratatui_markdown::tree::CollapsibleTree;

let mut preview = MarkdownPreview::new()
    .with_strip_frontmatter(true)
    .with_left_padding(true);

// Markdown 콘텐츠 설정 (TOML 프론트매터 포함)
preview.set_content(concat!(
    "+++\n",
    "title = \"My Doc\"\n",
    "+++\n",
    "\n",
    "# Hello\n\nContent here.\n",
));

// 트리 설정
let tree = CollapsibleTree::from_json_str(r#"{"config": {"theme": "dark"}}"#).unwrap();
preview.set_tree(Some(tree));

// 액션 아이템 설정
preview.set_action_items(vec![
    ActionItem { id: "edit".into(), label: " Edit ".into() },
    ActionItem { id: "save".into(), label: " Save ".into() },
]);

// ratatui 앱의 렌더 함수에서:
fn render_ui(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}

// 입력 처리
fn handle_input(key: ratatui::crossterm::event::KeyCode, preview: &mut MarkdownPreview) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => preview.scroll_up(),
        KeyCode::Down | KeyCode::Char('j') => preview.scroll_down(),
        KeyCode::PageUp => preview.page_up(20),
        KeyCode::PageDown => preview.page_down(20),
        KeyCode::Home => preview.scroll_to_top(),
        KeyCode::End => preview.scroll_to_bottom(),
        KeyCode::Enter => { preview.toggle_tree_node(); }
        _ => {}
    }
}
```
