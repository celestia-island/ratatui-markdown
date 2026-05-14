# Система Прокрутки

> Умная гибридная прокрутка с навигацией по фокусируемым элементам.

## Обзор

Модуль `scroll` предоставляет гибридную систему прокрутки, поддерживающую два режима:

1. **Свободная прокрутка** — когда фокусируемые элементы отсутствуют в области просмотра, содержимое прокручивается свободно
2. **Вовлечённый режим** — когда фокусируемые элементы попадают в центр области просмотра, курсор фиксируется на первом элементе для навигации с клавиатуры

Ограничен функциональным флагом `scroll` (включён по умолчанию).

## HybridScrollView

Основной виджет, управляющий прокруткой, областями фокуса и рендерингом:

```rust
pub struct HybridScrollView { /* поля */ }

impl HybridScrollView {
    pub fn new() -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;
    pub fn with_cursor_indicator(self, show: bool) -> Self;

    // Управление содержимым
    pub fn set_content(&mut self, lines: Vec<Line<'static>>, regions: Vec<FocusableRegion>);
    pub fn set_lines(&mut self, lines: Vec<Line<'static>>);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // Состояние прокрутки
    pub fn total_lines(&self) -> usize;
    pub fn get_scroll_offset(&self) -> usize;
    pub fn get_viewport_height(&self) -> usize;
    pub fn set_scroll_offset(&mut self, offset: usize);

    // Навигация
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // Вовлечение
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;
    pub fn engage_first(&mut self);
    pub fn engage_by_id(&mut self, id: &str) -> bool;

    // Рендеринг
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

### Конфигурация

- **with_left_padding**: Добавляет 1 столбец левого отступа ко всем отображаемым строкам
- **with_cursor_indicator**: Показывает `> ` (2 столбца) на строке вовлечённого курсора (имеет приоритет над `left_padding`)

Метод `effective_padding()` возвращает фактически используемый отступ:
- `2`, если включён индикатор курсора
- `1`, если только левый отступ
- `0` в противном случае

## Фокусируемые Области и Элементы

```rust
pub struct FocusableItemRange {
    pub start_line: usize,    // включительно
    pub end_line: usize,      // исключительно
    pub id: String,           // уникальный идентификатор
}

pub struct FocusableRegion {
    pub items: Vec<FocusableItemRange>,
}
```

Области определяют диапазоны строк, которые становятся интерактивными. Когда центр области просмотра проходит над областью, вид прокрутки автоматически вовлекается, и курсор попадает на первый элемент.

### Поведение Вовлечения

- Прокрутка **вниз** в область вовлекает на **первом** элементе
- Прокрутка **вверх** в область вовлекает на **последнем** элементе
- Навигация за пределы последнего элемента в области **отключает вовлечение** и возвращает к свободной прокрутке
- `scroll_to_top()` и `scroll_to_bottom()` всегда отключают вовлечение
- Внутри области `scroll_up`/`scroll_down` перемещают курсор между элементами

## Другие Виджеты Прокрутки

### ScrollableList<T>

Обобщённый прокручиваемый список с навигацией мышью/клавиатурой и опциональным рендерингом с рамкой:

```rust
pub trait ListItemRenderer {
    fn render_item(&self, index: usize, is_selected: bool, width: usize) -> Line<'static>;
}
```

### ArrowScrollbar

Пользовательская полоса прокрутки, отрисованная символами стрелок Unicode:

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

Для автоматического следования за содержимым (например, потоковый вывод):

```rust
pub struct FollowScrollState {
    // отслеживает, находится ли область просмотра внизу
}
```

### ScrollableRenderResult

Простая обёртка прокручиваемой панели:

```rust
pub fn render_scrollable(
    lines: &[Line],
    scroll_offset: usize,
    area: Rect,
    buf: &mut Buffer,
) -> ScrollableRenderResult;
```

## Пример

```rust
use ratatui_markdown::scroll::{HybridScrollView, FocusableItemRange, FocusableRegion};

let mut scroll = HybridScrollView::new()
    .with_cursor_indicator(true);

// Строки содержимого
let lines: Vec<Line> = (0..100)
    .map(|i| Line::raw(format!("Строка {}", i)))
    .collect();

// Делаем строки 30-32 фокусируемыми
let region = FocusableRegion {
    items: vec![
        FocusableItemRange { start_line: 30, end_line: 31, id: "item-a".into() },
        FocusableItemRange { start_line: 31, end_line: 32, id: "item-b".into() },
        FocusableItemRange { start_line: 32, end_line: 33, id: "item-c".into() },
    ],
};

scroll.set_content(lines, vec![region]);

// Свободная прокрутка вниз; авто-вовлечение, когда строка 30 попадает в центр области просмотра
scroll.scroll_down();

// Навигация по элементам
while scroll.scroll_down() == InputResult::Continue {
    if let Some(id) = scroll.selected_item_id() {
        println!("Выбрано: {}", id);
    }
}
```
