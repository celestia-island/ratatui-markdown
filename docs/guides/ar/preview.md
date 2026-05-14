# عنصر MarkdownPreview

> العنصر الموحد عالي المستوى الذي يدمج markdown والأشجار وعناصر الإجراءات في عرض واحد قابل للتمرير.

## نظرة عامة

`MarkdownPreview` هو عنصر التكامل عالي المستوى. يدمج عرض markdown وعرض الشجرة القابلة للطي وعناصر الإجراءات في `HybridScrollView` واحد. هذه هي نقطة الدخول الموصى بها لمعظم التطبيقات.

محصورة خلف علامة ميزة `preview` (مفعلة افتراضيًا، تتطلب جميع الميزات الأخرى).

## واجهة API

```rust
pub struct ActionItem {
    pub id: String,
    pub label: String,
}

impl MarkdownPreview {
    pub fn new() -> Self;

    // الإعدادات
    pub fn with_strip_frontmatter(self, strip: bool) -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;

    // المحتوى
    pub fn set_content(&mut self, content: &str);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // الشجرة
    pub fn set_tree(&mut self, tree: Option<CollapsibleTree>);
    pub fn tree_mut(&mut self) -> Option<&mut CollapsibleTree>;
    pub fn has_tree(&self) -> bool;
    pub fn toggle_tree_node(&mut self) -> bool;

    // عناصر الإجراءات
    pub fn set_action_items(&mut self, items: Vec<ActionItem>);
    pub fn selected_action_id(&self) -> Option<&str>;

    // التنقل
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // الحالة
    pub fn total_lines(&self) -> usize;
    pub fn scroll_offset(&self) -> usize;
    pub fn visible_height(&self) -> usize;
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;

    // العرض
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

## الإعدادات

### with_strip_frontmatter

عند التفعيل (افتراضيًا)، يُعامل المحتوى المحاط بـ `+++` كمقدمة TOML ويُزال قبل العرض:

```
+++
title = "مستندي"
author = "فلان الفلاني"
+++

# المحتوى الفعلي
هذا هو النص الأساسي.
```

سيظهر فقط `# المحتوى الفعلي` والفقرة التالية في المخرجات.

### with_left_padding

يضيف عمودًا واحدًا من الحشو الأيسر للمحتوى المعروض (يمرر إلى `HybridScrollView`).

## تخطيط المحتوى

يعرض العنصر المحتوى بترتيب عمودي:

1. **الشجرة** (إن وجدت) — أسطر الشجرة مع عقد قابلة للتركيز
2. **Markdown** — محتوى محلل ومعروض
3. **عناصر الإجراءات** — تسميات إجراءات قابلة للتركيز مع مغلفات أقواس `[تسمية]`

يُدرج سطر فارغ بين الأقسام، ويحصل كل قسم على `FocusableRegion` خاص به في عرض التمرير.

## التخزين المؤقت

يخزن `MarkdownPreview` المخرجات المعروضة مؤقتًا ولا يعيد البناء إلا عند:

- تغير المحتوى (`set_content` بنص مختلف)
- تغير العرض (تغيير حجم الطرفية)
- تغير جيل السمة (`theme.generation()` تعيد قيمة جديدة)
- تعديل الشجرة (`set_tree`، `toggle_tree_node`)
- تعديل عناصر الإجراءات (`set_action_items`)

استخدم `theme.generation()` لتشغيل إعادة العرض بعد تغييرات السمة.

## معالجة مقدمة TOML

تُفترض المقدمة أنها TOML. لا يتم **تحليلها** — يتم إزالتها ببساطة من المخرجات المعروضة. يبدأ سطر `+++` الأول وضع المقدمة، وينهيه سطر `+++` الثاني. تُعرض الأسطر قبل وبعد كتلة المقدمة بشكل طبيعي.

إذا لم يبدأ المحتوى بـ `+++`، لا تحدث أي إزالة.

## عناصر الإجراءات

توفر عناصر الإجراءات خيارات قابلة للتحديد بلوحة المفاتيح في أسفل العرض:

```rust
preview.set_action_items(vec![
    ActionItem { id: "confirm".into(), label: " تأكيد ".into() },
    ActionItem { id: "cancel".into(), label: " إلغاء ".into() },
]);

// في معالج الإدخال الخاص بك:
if let Some("confirm") = preview.selected_action_id() {
    // تعامل مع التأكيد
}
```

تُسبق معرفات عناصر الإجراءات بـ `action:` داخليًا لتجنب التعارض مع مسارات عقد الشجرة.

## مثال

```rust
use ratatui_markdown::preview::{MarkdownPreview, ActionItem};
use ratatui_markdown::tree::CollapsibleTree;

let mut preview = MarkdownPreview::new()
    .with_strip_frontmatter(true)
    .with_left_padding(true);

// عيّن محتوى markdown (مع مقدمة TOML)
preview.set_content(concat!(
    "+++\n",
    "title = \"مستندي\"\n",
    "+++\n",
    "\n",
    "# مرحبًا\n\nمحتوى هنا.\n",
));

// عيّن شجرة
let tree = CollapsibleTree::from_json_str(r#"{"config": {"theme": "dark"}}"#).unwrap();
preview.set_tree(Some(tree));

// عيّن عناصر الإجراءات
preview.set_action_items(vec![
    ActionItem { id: "edit".into(), label: " تعديل ".into() },
    ActionItem { id: "save".into(), label: " حفظ ".into() },
]);

// في دالة العرض لتطبيق ratatui:
fn render_ui(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}

// تعامل مع الإدخال
fn handle_input(key: rataui::crossterm::event::KeyCode, preview: &mut MarkdownPreview) {
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
