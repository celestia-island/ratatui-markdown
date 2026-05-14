# وحدة Markdown

> تحليل وعرض نص markdown إلى `ratatui::text::Line` منسقة.

## نظرة عامة

توفر وحدة `markdown` محللًا وعارضًا مخصصًا لـ markdown مصممًا للإخراج الطرفي. هو **ليس** محللًا متوافقًا مع CommonMark — يستهدف المجموعة الفرعية من markdown الأكثر فائدة في واجهات المستخدم الطرفية.

محصورة خلف علامة ميزة `markdown` (مفعلة افتراضيًا).

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

### المُنشئ

`MarkdownRenderer::new(max_width)` يأخذ عرض المحتوى المتاح بالأعمدة. يُستخدم هذا العرض لالتفاف نص الفقرات (مع مراعاة CJK) وتحديد حجم أعمدة الجدول.

### التحليل

`parse()` تأخذ `&str` من نص markdown وتعيد `Vec<MarkdownBlock>`. المحلل مبني على الأسطر ويعالج الكتل بالتسلسل.

### العرض

`render()` تأخذ الكتل المحللة وسمة، وتنتج `Vec<Line<'static>>` مناسبة للاستخدام المباشر في عناصر ratatui.

## تعداد MarkdownBlock

```rust
pub enum MarkdownBlock {
    Heading1(String),              // # عنوان
    Heading2(String),              // ## عنوان
    Heading3(String),              // ### عنوان
    Paragraph(Vec<String>),        // أسطر فقرة ملفوفة
    CodeBlock(String, String),     // (لغة، محتوى)
    InlineCode(String),            // `كود ضمني`
    ListItem(String, u8),          // (محتوى، مستوى_الإزاحة)
    Blockquote(String),            // > نص مقتبس
    HorizontalRule,                // --- أو *** أو ___
    BlankLine,                     // سطر فارغ
    Table {                        // | عمود1 | عمود2 |
        headers: Vec<String>,      //   |-------|-------|
        rows: Vec<Vec<String>>,    //   | قيمة1 | قيمة2 |
    },
}
```

### تفاصيل الكتل

**العناوين** (H1-H3): تُعرض بلون `primary_color`، مع استخدام `Modifier::BOLD` للعناوين H1.

**الفقرات**: يُلف النص مع مراعاة CJK إلى `max_width`. يصبح كل سطر ملفوف مدخلًا في `Vec<String>`.

**كتل الكود** (المحاطة بـ ` ``` `): تُعرض بلون `muted_text_color` داخل مربعات محاطة بأحرف رسم المربعات. يتم تخطي كتل كود mermaid بصمت.

**الكود المضمن**: يُعرض بلون `secondary_color` و `Modifier::DIM`.

**القوائم**: غير مرتبة (`-`, `*`, `+`) ومرتبة (`1.`, `2.`). يحافظ كل عنصر على مستوى إزاحته. العناصر الفرعية تُزاح بصريًا.

**الاقتباسات**: تُسبق بشريط `│` ملون وتُعرض بلون `muted_text_color`.

**الجداول**: تُحجّم الأعمدة نسبيًا بناءً على عرض المحتوى. تُلف الخلايا، وتستخدم العناوين `Modifier::BOLD`، وتستخدم الحدود أحرف رسم المربعات.

## التنسيق المضمن

يُطبق التنسيق المضمن **داخل** نص الفقرات وعناصر القوائم:

| Markdown        | التأثير المعروض                |
|-----------------|--------------------------------|
| `**نص**`        | **غامق** (`Modifier::BOLD`)   |
| `*نص*`          | *مائل* (`Modifier::ITALIC`)  |
| `***نص***`      | ***غامق+مائل***               |
| `` `كود` ``     | نمط `كود ضمني`                |

```rust
pub fn parse_inline_formatting(text: &str, theme: &impl RichTextTheme) -> Vec<Span<'static>>;
```

هذه الدالة المستقلة مُعاد تصديرها أيضًا للاستخدام خارج `MarkdownRenderer`.

## مثال

```rust
use ratatui_markdown::markdown::MarkdownRenderer;

let md = r#"
# عنوان

هذه فقرة تحتوي على نص **غامق** ونص *مائل*.

## كود

```rust
fn main() {
    println!("مرحبًا!");
}
```

| الاسم | الإصدار |
|-------|---------|
| ratatui | 0.29 |
| serde | 1.0 |
"#;

let renderer = MarkdownRenderer::new(80);
let blocks = renderer.parse(md);
let lines = renderer.render(&blocks, theme);
// استخدم الأسطر في عنصر ratatui
```
