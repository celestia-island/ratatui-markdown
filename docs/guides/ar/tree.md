# عرض الشجرة

> شجرة تفاعلية قابلة للطي لبيانات JSON و TOML.

## نظرة عامة

تحلل وحدة `tree` بيانات JSON أو TOML إلى شجرة تفاعلية قابلة للطي. يمكن للمستخدمين توسيع/طي العقد والتنقل باستخدام لوحة المفاتيح في واجهة مستخدم طرفية.

محصورة خلف علامة ميزة `tree` (تتطلب اعتماديات `scroll` و `serde_json` و `toml`).

## CollapsibleTree

```rust
pub struct CollapsibleTree { /* الحقول */ }

impl CollapsibleTree {
    // المنشآت
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error>;
    pub fn from_toml_str(toml: &str) -> Result<Self, Box<dyn Error>>;
    pub fn from_value(value: serde_json::Value, style: KeyStyle) -> Self;

    // التحكم في الشجرة
    pub fn toggle(&mut self, path: &str) -> bool;
    pub fn handle_toggle(&mut self, id: &str) -> bool;
    pub fn expand_all(&mut self);
    pub fn collapse_all(&mut self);
    pub fn expand_to_depth(&mut self, depth: usize);

    // العرض
    pub fn render_lines(&self, width: usize, theme: &impl RichTextTheme) -> Vec<Line<'static>>;
    pub fn flatten(&self) -> Vec<FlatEntry>;
    pub fn build_focusable_items(&self) -> Vec<FocusableItemRange>;
}
```

### المنشآت

- **`from_json_str`**: تحلل سلسلة JSON إلى شجرة. تستخدم المفاتيح `KeyStyle::Json` (بين علامتي اقتباس، مع فاصل `:`).
- **`from_toml_str`**: تحلل سلسلة TOML (تحول داخليًا إلى JSON). تستخدم المفاتيح `KeyStyle::Toml` (بدون علامات اقتباس، مع فاصل `=`).
- **`from_value`**: تبني شجرة من `serde_json::Value` موجود مع نمط مفاتيح مختار.

### التحكم في الشجرة

```rust
// تبديل باستخدام مسار مفصول بشرطة مائلة
tree.toggle("dependencies");          // تبديل المفتاح الجذري
tree.toggle("dependencies/serde");    // تبديل مفتاح متداخل

// دوال مساعدة
tree.expand_all();
tree.collapse_all();
tree.expand_to_depth(2);

// في سياق التمرير — تستخدم معرف العنصر المحدد
if scroll.selected_item_id().is_some() {
    tree.handle_toggle(selected_id);
}
```

### العرض

- **`render_lines`**: تنتج `Line`s منسقة مع موصلات شجرة وقيم ملونة.
- **`flatten`**: تعيد قائمة مسطحة لجميع المدخلات المرئية (تراعي حالة الطي).
- **`build_focusable_items`**: تعيد نطاقات قابلة للتركيز للتكامل مع `HybridScrollView`، مع معرفات تطابق مسارات الشجرة.

## أنواع البيانات

```rust
pub struct FlatEntry {
    pub depth: usize,
    pub key: String,
    pub value: String,
    pub kind: EntryKind,
    pub value_type: ValueType,
    pub path: String,          // مثلًا "dependencies/serde"
    pub is_last: bool,
}

pub enum EntryKind {
    Collapsed,  // لديه أبناء، مطوي حاليًا: [+]
    Expanded,   // لديه أبناء، موسع حاليًا: [-]
    Leaf,       // لا يوجد أبناء
}

pub enum ValueType {
    String, Number, Boolean, Null, Object, Array,
}

pub enum KeyStyle {
    Json,  // "مفتاح": قيمة
    Toml,  // مفتاح = قيمة
}
```

### ألوان أنواع القيم

كل `ValueType` يرتبط بدالة لون سمة مقابلة:

| ValueType | دالة السمة          |
|-----------|---------------------|
| String    | `get_json_string_color()` |
| Number    | `get_json_number_color()` |
| Boolean   | `get_json_bool_color()`   |
| Null      | `get_json_null_color()`   |

## دوال مساعدة لأسطر الشجرة

توفر وحدة `tree_lines` (المُعاد تصديرها من `crate::tree`) بناء أسطر منخفض المستوى:

```rust
pub fn make_body_line(key: &str, value: &str, style: &Style, value_style: &Style) -> Line<'static>;
pub fn make_status_line(status: &str, style: &Style) -> Line<'static>;
pub fn make_branch_dispatch_line(kind: EntryKind, ...) -> Line<'static>;
```

## مثال

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

// وسّع كل شيء
tree.expand_all();

// اعرض كأسطر
let lines = tree.render_lines(80, theme);

// اطوِ الشجرة الفرعية للاعتماديات
tree.toggle("dependencies");

// أعد العرض — الاعتماديات مطوية الآن
let lines = tree.render_lines(80, theme);

// احصل على العناصر القابلة للتركيز للتنقل بالتمرير
let items = tree.build_focusable_items();
```
