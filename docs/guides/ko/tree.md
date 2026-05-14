# 트리 뷰

> JSON 및 TOML 데이터를 위한 대화형 접이식 트리.

## 개요

`tree` 모듈은 JSON 또는 TOML을 대화형 접이식 트리로 파싱합니다. 사용자는 터미널 UI에서 노드를 펼치거나 접고 키보드로 탐색할 수 있습니다.

`tree` 기능 플래그로 게이트됩니다 (`scroll`, `serde_json`, `toml` 의존성 필요).

## CollapsibleTree

```rust
pub struct CollapsibleTree { /* 필드 */ }

impl CollapsibleTree {
    // 생성자
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error>;
    pub fn from_toml_str(toml: &str) -> Result<Self, Box<dyn Error>>;
    pub fn from_value(value: serde_json::Value, style: KeyStyle) -> Self;

    // 트리 조작
    pub fn toggle(&mut self, path: &str) -> bool;
    pub fn handle_toggle(&mut self, id: &str) -> bool;
    pub fn expand_all(&mut self);
    pub fn collapse_all(&mut self);
    pub fn expand_to_depth(&mut self, depth: usize);

    // 렌더링
    pub fn render_lines(&self, width: usize, theme: &impl RichTextTheme) -> Vec<Line<'static>>;
    pub fn flatten(&self) -> Vec<FlatEntry>;
    pub fn build_focusable_items(&self) -> Vec<FocusableItemRange>;
}
```

### 생성자

- **`from_json_str`**: JSON 문자열을 트리로 파싱합니다. 키는 `KeyStyle::Json` (따옴표 포함, `:` 구분자)을 사용합니다.
- **`from_toml_str`**: TOML 문자열을 파싱합니다 (내부적으로 JSON으로 변환). 키는 `KeyStyle::Toml` (따옴표 없음, `=` 구분자)을 사용합니다.
- **`from_value`**: 기존 `serde_json::Value` 에서 선택한 키 스타일로 트리를 생성합니다.

### 트리 조작

```rust
// 슬래시로 구분된 경로를 사용하여 토글
tree.toggle("dependencies");          // 루트 키 토글
tree.toggle("dependencies/serde");    // 중첩 키 토글

// 편의 메서드
tree.expand_all();
tree.collapse_all();
tree.expand_to_depth(2);

// 스크롤 컨텍스트에서 — 선택된 아이템의 ID 사용
if scroll.selected_item_id().is_some() {
    tree.handle_toggle(selected_id);
}
```

### 렌더링

- **`render_lines`**: 트리 연결선과 색상이 지정된 값으로 스타일이 적용된 `Line` 을 생성합니다.
- **`flatten`**: 모든 표시 항목의 플랫 리스트를 반환합니다 (접힌 상태를 반영).
- **`build_focusable_items`**: `HybridScrollView` 통합을 위한 포커스 가능 범위를 반환하며, ID는 트리 경로와 일치합니다.

## 데이터 타입

```rust
pub struct FlatEntry {
    pub depth: usize,
    pub key: String,
    pub value: String,
    pub kind: EntryKind,
    pub value_type: ValueType,
    pub path: String,          // 예: "dependencies/serde"
    pub is_last: bool,
}

pub enum EntryKind {
    Collapsed,  // 자식이 있으며 현재 접힘: [+]
    Expanded,   // 자식이 있으며 현재 펼쳐짐: [-]
    Leaf,       // 자식 없음
}

pub enum ValueType {
    String, Number, Boolean, Null, Object, Array,
}

pub enum KeyStyle {
    Json,  // "key": value
    Toml,  // key = value
}
```

### 값 유형별 색상

각 `ValueType` 은 해당하는 테마 색상 메서드에 매핑됩니다:

| ValueType | 테마 메서드                   |
|-----------|-------------------------------|
| String    | `get_json_string_color()`     |
| Number    | `get_json_number_color()`     |
| Boolean   | `get_json_bool_color()`       |
| Null      | `get_json_null_color()`       |

## 트리 라인 헬퍼

`tree_lines` 모듈 (`crate::tree` 에서 재내보내기됨)은 저수준 라인 구성을 제공합니다:

```rust
pub fn make_body_line(key: &str, value: &str, style: &Style, value_style: &Style) -> Line<'static>;
pub fn make_status_line(status: &str, style: &Style) -> Line<'static>;
pub fn make_branch_dispatch_line(kind: EntryKind, ...) -> Line<'static>;
```

## 예제

```rust
use ratatui_markdown::tree::CollapsibleTree;

let toml_content = r#"
[package]
name = "my-app"
version = "0.1.0"

[dependencies]
ratatui = "0.29"
serde = { version = "1.0", features = ["derive"] }

[features]
default = ["std"]
std = []
"#;

let mut tree = CollapsibleTree::from_toml_str(toml_content).unwrap();

// 모두 펼치기
tree.expand_all();

// 라인으로 렌더링
let lines = tree.render_lines(80, theme);

// dependencies 하위 트리 접기
tree.toggle("dependencies");

// 다시 렌더링 — dependencies가 이제 접힘
let lines = tree.render_lines(80, theme);

// 스크롤 내비게이션용 포커스 가능 아이템 가져오기
let items = tree.build_focusable_items();
```
