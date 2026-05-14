# تخصيص السمة

> تنفيذ مخططات ألوان مخصصة عبر trait `RichTextTheme`.

## نظرة عامة

تمر جميع عمليات البحث عن الألوان والأنماط عبر trait `RichTextTheme`، مما يفصل منطق العرض عن المظهر. نفّذ هذا الـ trait مرة واحدة ومرره إلى جميع دوال العرض.

## RichTextTheme Trait

```rust
pub trait RichTextTheme {
    fn generation(&self) -> Generation;

    // النص العام
    fn get_text_color(&self) -> Color;
    fn get_muted_text_color(&self) -> Color;
    fn get_background_color(&self) -> Color;

    // التمييز / التأكيد
    fn get_primary_color(&self) -> Color;
    fn get_secondary_color(&self) -> Color;
    fn get_info_color(&self) -> Color;

    // الحدود
    fn get_border_color(&self) -> Color;
    fn get_focused_border_color(&self) -> Color;

    // النوافذ المنبثقة / التراكبات
    fn get_popup_selected_background(&self) -> Color;
    fn get_popup_selected_text_color(&self) -> Color;

    // قيم JSON/الشجرة
    fn get_json_key_color(&self) -> Color;
    fn get_json_string_color(&self) -> Color;
    fn get_json_number_color(&self) -> Color;
    fn get_json_bool_color(&self) -> Color;
    fn get_json_null_color(&self) -> Color;

    // متنوع
    fn get_accent_yellow(&self) -> Color;
}
```

## رمز الجيل

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Generation(pub u64);

impl Generation {
    pub fn next(&self) -> Self { Self(self.0 + 1) }
}
```

يُستخدم رمز `Generation` من قبل `MarkdownPreview` لإبطال ذاكرة التخزين المؤقت. عندما تعيد `generation()` الخاصة بالسمة قيمة مختلفة عن القيمة المخزنة مؤقتًا، يعيد عنصر المعاينة بناء كل المخرجات المعروضة.

**قم بزيادة الجيل كلما غيّر المستخدم السمة** وقت التشغيل:

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

## استخدام خانات الألوان

إليك كيفية استخدام كل لون سمة عمليًا:

| الدالة | تُستخدم لـ |
|--------|------------|
| `get_text_color` | نص الفقرات الافتراضي، نص عناصر القائمة |
| `get_muted_text_color` | كتل الكود، نص الاقتباسات، حدود الجداول |
| `get_primary_color` | العناوين (H1-H3)، تمييز النص الغامق |
| `get_secondary_color` | الكود المضمن، التمييزات الثانوية |
| `get_info_color` | تمييزات مستوى المعلومات |
| `get_background_color` | تعبئة الخلفية العامة |
| `get_border_color` | الحدود غير المركزة (كتل الكود، الجداول) |
| `get_focused_border_color` | حدود العناصر المركزة/المتفاعلة، شريط التمرير |
| `get_popup_selected_background` | خلفية عناصر النوافذ المنبثقة المحددة |
| `get_popup_selected_text_color` | لون نص عناصر النوافذ المنبثقة المحددة |
| `get_json_key_color` | مفاتيح عقد الشجرة |
| `get_json_string_color` | قيم السلاسل في الأشجار |
| `get_json_number_color` | القيم الرقمية في الأشجار |
| `get_json_bool_color` | القيم المنطقية في الأشجار |
| `get_json_null_color` | القيم الفارغة في الأشجار |
| `get_accent_yellow` | تمييزات التحذير/التأكيد |

## مثال: سمة كاملة

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

## ملاحظات الأداء

- جميع دوال السمة تُستدعى بشكل متكرر أثناء العرض. اجعلها خفيفة — أرجع ثوابت أو عمليات بحث بسيطة في الحقول.
- مقارنة رمز `Generation` هي فحص مساواة بسيط لعدد صحيح، لذا فإن زيادته سريعة.
