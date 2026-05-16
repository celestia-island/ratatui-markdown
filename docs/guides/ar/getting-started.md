# البدء

## المتطلبات الأساسية

- **Rust** 1.74 أو أحدث
- **ratatui** 0.29 (يتم جلبه تلقائيًا كاعتمادية)

## التثبيت

أضف إلى `Cargo.toml`:

```toml
[dependencies]
ratatui-markdown = "0.1"
```

يفعّل هذا جميع الميزات افتراضيًا (`markdown`، `scroll`، `tree`، `preview`، `mermaid`، `image`، `viewer`).

### الميزات الانتقائية

لتقليل وقت التجميع والاعتماديات، فعّل ما تحتاجه فقط:

```toml
# عرض Markdown فقط
ratatui-markdown = { version = "0.1", default-features = false, features = ["markdown"] }

# نظام التمرير فقط
ratatui-markdown = { version = "0.1", default-features = false, features = ["scroll"] }

# عرض الشجرة (يجلب scroll و serde_json و toml)
ratatui-markdown = { version = "0.1", default-features = false, features = ["tree"] }
```

## الاستخدام الأساسي

### عرض Markdown

```rust
use ratatui_markdown::markdown::MarkdownRenderer;
use ratatui_markdown::theme::RichTextTheme;

// أنشئ عارضًا مع أقصى عرض للمحتوى
let renderer = MarkdownRenderer::new(80);

// حلل نص markdown إلى كتل
let blocks = renderer.parse("# مرحبًا\n\nهذا نص **غامق** ونص *مائل*.");

// اعرض الكتل إلى ratatui::text::Line<'static>
let lines = renderer.render(&blocks, &my_theme);
```

### تصفح شجرة

```rust
use ratatui_markdown::tree::CollapsibleTree;

// حلل JSON إلى شجرة قابلة للطي
let json_str = r#"{"name": "project", "deps": {"ratatui": "0.29", "serde": "1.0"}}"#;
let mut tree = CollapsibleTree::from_json_str(json_str).unwrap();

// اعرض أسطر الشجرة
let lines = tree.render_lines(80, &my_theme);

// احصل على العناصر القابلة للتركيز للتنقل
let items = tree.build_focusable_items();

// بدّل حالة العقدة
tree.toggle("deps/serde");
```

### استخدام عنصر MarkdownPreview

عنصر `MarkdownPreview` يدمج كل شيء في عرض واحد قابل للتمرير:

```rust
use ratatui_markdown::preview::MarkdownPreview;
use ratatui_markdown::theme::RichTextTheme;

let mut preview = MarkdownPreview::new()
    .with_left_padding(true);

// عيّن محتوى markdown
preview.set_content("# مرحبًا\n\n- عنصر واحد\n- عنصر اثنان\n\n```rust\nlet x = 42;\n```");

// عيّن شجرة قابلة للطي (اختياري)
let tree = CollapsibleTree::from_json_str(r#"{"config": {"port": 8080}}"#).unwrap();
preview.set_tree(Some(tree));

// تعامل مع إدخال لوحة المفاتيح
preview.scroll_up();
preview.scroll_down();
preview.page_up(10);
preview.page_down(10);
preview.toggle_tree_node(); // مفتاح Enter

// اعرض في حلقة الرسم الخاصة بـ ratatui
fn draw(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}
```

## تنفيذ سمة

تستخدم المكتبة trait للبحث عن جميع الألوان:

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

غيّر قيمة إرجاع `generation()` لإبطال ذاكرة التخزين المؤقت لعنصر المعاينة وفرض إعادة العرض (مثلًا، عندما يبدّل المستخدم السمات وقت التشغيل).

## الخطوات التالية

- [وحدة Markdown](markdown.md) — واجهة تحليل وعرض markdown الكاملة
- [نظام التمرير](scroll.md) — فهم بنية التمرير الهجين
- [عرض الشجرة](tree.md) — عرض وتفاعل أشجار JSON/TOML
- [عنصر المعاينة](preview.md) — العنصر الموحد عالي المستوى
- [السمة](theme.md) — دليل تخصيص السمة الكامل
