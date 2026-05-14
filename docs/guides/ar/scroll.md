# نظام التمرير

> تمرير هجين ذكي مع تنقل بين العناصر القابلة للتركيز.

## نظرة عامة

توفر وحدة `scroll` نظام تمرير هجين يدعم وضعين:

1. **التمرير الحر** — عندما لا تكون هناك عناصر قابلة للتركيز في العرض، يتحرك المحتوى بحرية
2. **التفاعل** — عندما تدخل العناصر القابلة للتركيز إلى منتصف منطقة العرض، يثبت المؤشر على العنصر الأول للتنقل بلوحة المفاتيح

محصورة خلف علامة ميزة `scroll` (مفعلة افتراضيًا).

## HybridScrollView

العنصر الأساسي الذي يدير التمرير ومناطق التركيز والعرض:

```rust
pub struct HybridScrollView { /* الحقول */ }

impl HybridScrollView {
    pub fn new() -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;
    pub fn with_cursor_indicator(self, show: bool) -> Self;

    // إدارة المحتوى
    pub fn set_content(&mut self, lines: Vec<Line<'static>>, regions: Vec<FocusableRegion>);
    pub fn set_lines(&mut self, lines: Vec<Line<'static>>);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // حالة التمرير
    pub fn total_lines(&self) -> usize;
    pub fn get_scroll_offset(&self) -> usize;
    pub fn get_viewport_height(&self) -> usize;
    pub fn set_scroll_offset(&mut self, offset: usize);

    // التنقل
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // التفاعل
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;
    pub fn engage_first(&mut self);
    pub fn engage_by_id(&mut self, id: &str) -> bool;

    // العرض
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

### الإعدادات

- **with_left_padding**: يضيف عمودًا واحدًا من الحشو الأيسر لجميع الأسطر المعروضة
- **with_cursor_indicator**: يعرض `> ` (عمودان) على سطر المؤشر المتفاعل (له الأولوية على `left_padding`)

تعيد دالة `effective_padding()` الحشو الفعلي المستخدم:
- `2` إذا كان مؤشر المؤشر مفعلًا
- `1` إذا كان الحشو الأيسر فقط
- `0` في الحالات الأخرى

## المناطق والعناصر القابلة للتركيز

```rust
pub struct FocusableItemRange {
    pub start_line: usize,    // شامل
    pub end_line: usize,      // غير شامل
    pub id: String,           // معرف فريد
}

pub struct FocusableRegion {
    pub items: Vec<FocusableItemRange>,
}
```

تحدد المناطق نطاقات من الأسطر التي تصبح تفاعلية. عندما يمر منتصف منطقة العرض فوق منطقة، يتفاعل عرض التمرير تلقائيًا ويهبط المؤشر على العنصر الأول.

### سلوك التفاعل

- التمرير **لأسفل** إلى داخل منطقة يتفاعل على العنصر **الأول**
- التمرير **لأعلى** إلى داخل منطقة يتفاعل على العنصر **الأخير**
- التنقل بعد آخر عنصر في منطقة **يفك التفاعل** ويعود إلى التمرير الحر
- `scroll_to_top()` و `scroll_to_bottom()` تفك التفاعل دائمًا
- داخل المنطقة، `scroll_up`/`scroll_down` تحرك المؤشر بين العناصر

## عناصر تمرير أخرى

### ScrollableList<T>

قائمة قابلة للتمرير عامة مع تنقل بالفأرة/لوحة المفاتيح وعرض محاط اختياري:

```rust
pub trait ListItemRenderer {
    fn render_item(&self, index: usize, is_selected: bool, width: usize) -> Line<'static>;
}
```

### ArrowScrollbar

شريط تمرير مخصص مرسوم برموز أسهم Unicode:

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

للمتابعة التلقائية للمحتوى (مثلًا، الإخراج المتدفق):

```rust
pub struct FollowScrollState {
    // يتتبع ما إذا كانت منطقة العرض في الأسفل
}
```

### ScrollableRenderResult

مغلّف لوحة بسيط قابل للتمرير:

```rust
pub fn render_scrollable(
    lines: &[Line],
    scroll_offset: usize,
    area: Rect,
    buf: &mut Buffer,
) -> ScrollableRenderResult;
```

## مثال

```rust
use ratatui_markdown::scroll::{HybridScrollView, FocusableItemRange, FocusableRegion};

let mut scroll = HybridScrollView::new()
    .with_cursor_indicator(true);

// أسطر المحتوى
let lines: Vec<Line> = (0..100)
    .map(|i| Line::raw(format!("سطر {}", i)))
    .collect();

// اجعل الأسطر 30-32 قابلة للتركيز
let region = FocusableRegion {
    items: vec![
        FocusableItemRange { start_line: 30, end_line: 31, id: "item-a".into() },
        FocusableItemRange { start_line: 31, end_line: 32, id: "item-b".into() },
        FocusableItemRange { start_line: 32, end_line: 33, id: "item-c".into() },
    ],
};

scroll.set_content(lines, vec![region]);

// تمرير حر لأسفل؛ يتفاعل تلقائيًا عندما يدخل السطر 30 منتصف منطقة العرض
scroll.scroll_down();

// تنقل عبر العناصر
while scroll.scroll_down() == InputResult::Continue {
    if let Some(id) = scroll.selected_item_id() {
        println!("محدد: {}", id);
    }
}
```
