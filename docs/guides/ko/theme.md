# 테마 사용자 정의

> `RichTextTheme` 트레이트를 통한 커스텀 색상 구성표 구현.

## 개요

모든 색상 및 스타일 조회는 `RichTextTheme` 트레이트를 통해 이루어지며, 렌더링 로직과 외형을 분리합니다. 이 트레이트를 한 번 구현하고 모든 렌더 함수에 전달하세요.

## RichTextTheme 트레이트

```rust
pub trait RichTextTheme {
    fn generation(&self) -> Generation;

    // 일반 텍스트
    fn get_text_color(&self) -> Color;
    fn get_muted_text_color(&self) -> Color;
    fn get_background_color(&self) -> Color;

    // 강조 / 액센트
    fn get_primary_color(&self) -> Color;
    fn get_secondary_color(&self) -> Color;
    fn get_info_color(&self) -> Color;

    // 테두리
    fn get_border_color(&self) -> Color;
    fn get_focused_border_color(&self) -> Color;

    // 팝업 / 오버레이
    fn get_popup_selected_background(&self) -> Color;
    fn get_popup_selected_text_color(&self) -> Color;

    // JSON/트리 값
    fn get_json_key_color(&self) -> Color;
    fn get_json_string_color(&self) -> Color;
    fn get_json_number_color(&self) -> Color;
    fn get_json_bool_color(&self) -> Color;
    fn get_json_null_color(&self) -> Color;

    // 기타
    fn get_accent_yellow(&self) -> Color;
}
```

## 세대 토큰

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Generation(pub u64);

impl Generation {
    pub fn next(&self) -> Self { Self(self.0 + 1) }
}
```

`Generation` 토큰은 `MarkdownPreview` 의 캐시 무효화에 사용됩니다. 테마의 `generation()` 이 캐시된 값과 다른 값을 반환하면, 프리뷰는 렌더링된 모든 출력을 다시 빌드합니다.

**사용자가 런타임에 테마를 변경할 때마다 세대를 증가시키세요**:

```rust
struct AppTheme {
    generation: Generation,
    dark_mode: bool,
}

impl RichTextTheme for AppTheme {
    fn generation(&self) -> Generation { self.generation }

    fn get_text_color(&self) -> Color {
        if self.dark_mode { Color::White } else { Color::Black }
    }

    // ...
}

impl AppTheme {
    fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
        self.generation = self.generation.next();
    }
}
```

## 색상 슬롯 사용법

각 테마 색상의 실제 사용 방식은 다음과 같습니다:

| 메서드 | 사용처 |
|--------|--------|
| `get_text_color` | 기본 단락 텍스트, 리스트 아이템 텍스트 |
| `get_muted_text_color` | 코드 블록, 인용 블록 텍스트, 테이블 테두리 |
| `get_primary_color` | 제목 (H1-H3), 굵은 텍스트 액센트 |
| `get_secondary_color` | 인라인 코드, 보조 액센트 |
| `get_info_color` | 정보 수준 하이라이트 |
| `get_background_color` | 일반 배경 채우기 |
| `get_border_color` | 포커스되지 않은 테두리 (코드 블록, 테이블) |
| `get_focused_border_color` | 포커스/인게이지된 아이템 테두리, 스크롤바 |
| `get_popup_selected_background` | 선택된 팝업 아이템의 배경 |
| `get_popup_selected_text_color` | 선택된 팝업 아이템의 텍스트 색상 |
| `get_json_key_color` | 트리 노드 키 |
| `get_json_string_color` | 트리 내 문자열 값 |
| `get_json_number_color` | 트리 내 숫자 값 |
| `get_json_bool_color` | 트리 내 불리언 값 |
| `get_json_null_color` | 트리 내 null 값 |
| `get_accent_yellow` | 경고/하이라이트 액센트 |

## 예제: 완전한 테마

```rust
use ratatui::style::Color;
use ratatui_markdown::theme::{Generation, RichTextTheme};

struct CatppuccinMocha;

impl RichTextTheme for CatppuccinMocha {
    fn generation(&self) -> Generation { Generation(0) }

    fn get_text_color(&self) -> Color { Color::new(205, 214, 244) }       // text
    fn get_muted_text_color(&self) -> Color { Color::new(147, 153, 178) } // overlay2
    fn get_background_color(&self) -> Color { Color::new(30, 30, 46) }    // base
    fn get_primary_color(&self) -> Color { Color::new(137, 180, 250) }    // blue
    fn get_secondary_color(&self) -> Color { Color::new(203, 166, 247) }  // mauve
    fn get_info_color(&self) -> Color { Color::new(137, 220, 235) }       // sky
    fn get_border_color(&self) -> Color { Color::new(69, 71, 90) }        // surface1
    fn get_focused_border_color(&self) -> Color { Color::new(137, 180, 250) } // blue
    fn get_popup_selected_background(&self) -> Color { Color::new(69, 71, 90) }
    fn get_popup_selected_text_color(&self) -> Color { Color::new(205, 214, 244) }
    fn get_json_key_color(&self) -> Color { Color::new(137, 220, 235) }    // sky
    fn get_json_string_color(&self) -> Color { Color::new(166, 227, 161) } // green
    fn get_json_number_color(&self) -> Color { Color::new(250, 179, 135) } // peach
    fn get_json_bool_color(&self) -> Color { Color::new(203, 166, 247) }   // mauve
    fn get_json_null_color(&self) -> Color { Color::new(108, 112, 134) }   // surface2
    fn get_accent_yellow(&self) -> Color { Color::new(249, 226, 175) }     // yellow
}
```

## 성능 참고 사항

- 모든 테마 메서드는 렌더링 중에 자주 호출됩니다. 가볍게 유지하세요 — 상수나 간단한 필드 조회를 반환하세요.
- `Generation` 토큰 비교는 단순한 정수 동등성 검사이므로, 증가 연산은 매우 빠릅니다.
