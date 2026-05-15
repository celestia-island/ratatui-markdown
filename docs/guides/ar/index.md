<div align="center">
  <img src="../../../examples/logo.webp" alt="ratatui-markdown logo" width="200" />
</div>

# ratatui-markdown

> مكتبة Rust توفر عرض Markdown ومخططات Mermaid وتلوين بناء الجملة وأشجار JSON/TOML قابلة للطي وعناصر تمرير غنية لـ ratatui.
>
> **مبنية على**: [ratatui](https://github.com/ratatui/ratatui) 0.29 + Rust خالص
>
> **أدنى إصدار Rust**: 1.74

<div align="center">
  <p>
    <a href="../../README.md">English</a> |
    <a href="../zhs/index.md">简体中文</a> |
    <a href="../zht/index.md">繁體中文</a> |
    <a href="../ja/index.md">日本語</a> |
    <a href="../ko/index.md">한국어</a> |
    <a href="../fr/index.md">Français</a> |
    <a href="../es/index.md">Español</a> |
    <a href="../ru/index.md">Русский</a> |
    <a href="../ar/index.md">العربية</a>
  </p>
</div>

## ما هو ratatui-markdown؟

ratatui-markdown هي مكتبة عرض غنية بالميزات لواجهات المستخدم الطرفية المبنية بـ [ratatui](https://github.com/ratatui/ratatui). توفر عدة وحدات وظيفية يمكن استخدامها بشكل مستقل أو دمجها عبر عنصري `MarkdownPreview` / `MarkdownViewer`.

## الوحدات الأساسية

### عرض Markdown

تحليل وعرض نص Markdown كمخرجات طرفية منسقة:

- **العناوين**: H1 (`#`), H2 (`##`), H3 (`###`)
- **الفقرات** مع التفاف تلقائي للنص يراعي عرض أحرف CJK
- **التنسيق المضمن**: `**غامق**`, `*مائل*`, `***غامق+مائل***`, `` `كود ضمني` ``
- **كتل الكود** مع تسميات لغة اختيارية (يتم عرض كتل mermaid كمخططات)
- **الاقتباسات** (`>`)
- **القوائم غير المرتبة** (`-`, `*`, `+`) والقوائم المرتبة (`1.`, `2.`)
- **الخطوط الأفقية** (`---`, `***`, `___`)
- **الجداول** مع عرض أعمدة تناسبي والتفاف الخلايا

### عرض الشجرة القابلة للطي

تحليل وتصفح تفاعلي للبيانات المنظمة:

- تحليل **JSON** و **TOML** إلى أشجار قابلة للطي
- **توسيع / طي** العقد الفردية، توسيع الكل، طي الكل، توسيع حسب العمق
- **مفاتيح منسقة**: وضع JSON (مفاتيح بين علامتي اقتباس + `:`) أو وضع TOML (مفاتيح عارية + `=`)
- **تنقل بلوحة المفاتيح**: تحديد وتبديل قائم على المؤشر
- **تلوين حسب نوع القيمة**: السلاسل، الأرقام، القيم المنطقية، null — لكل منها لون السمة الخاص به

### نظام التمرير الهجين

تمرير ذكي يتعامل مع كل من التصفح الحر والتنقل بين العناصر:

- **وضع التمرير الحر**: تصفح المحتوى بحرية
- **وضع التفاعل**: يتم تنشيطه تلقائيًا عند دخول العناصر القابلة للتركيز إلى منطقة العرض
- **تنقل بالمؤشر**: التنقل بين العناصر القابلة للتركيز بلوحة المفاتيح
- **مؤشر المؤشر**: بادئة مرئية `> ` على الأسطر المتفاعلة
- **شريط التمرير**: تراكب قائم على الأسهم
- **التصفح**: دعم `page_up` / `page_down`

### مخططات Mermaid

عرض مخططات Mermaid مباشرة في الطرفية:

- **مخططات التسلسل**، **المخططات الدائرية**، **مخططات غانت**، **مخططات الحالة**
- تُفعّل بكتل ` ```mermaid `
- علامة الميزة: `mermaid`

### تلوين بناء الجملة

تلوين بناء جملة كتل الكود باستخدام tree-sitter:

- علامات حسب اللغة (`highlight-lang-rust`، `highlight-lang-python`، إلخ)
- `highlight-lang-all` تُفعّل جميع اللغات
- قابلة للتخصيص عبر `HighlightHooks`

### عنصرا MarkdownPreview / MarkdownViewer

العنصر عالي المستوى الذي يدمج كل شيء معًا:

- عرض محتوى Markdown والأشجار وعناصر الإجراءات في تخطيط واحد قابل للتمرير
- **التخزين المؤقت**: يعيد بناء المخرجات فقط عند تغير المحتوى أو العرض أو جيل السمة
- **إزالة المقدمة TOML**: إزالة تلقائية لمقدمة TOML المحددة بـ `+++`
- **عناصر الإجراءات**: عناصر معنونة قابلة للتحديد بلوحة المفاتيح مع معرفات إجراءات
- تفويض كل التنقل إلى `HybridScrollView`

## بداية سريعة

```toml
[dependencies]
ratatui-markdown = "0.2"
```

### أمثلة

| مثال                 | الوصف                               | العلامات المطلوبة              |
|----------------------|------------------------------------|-------------------------------|
| `basic`              | عرض Markdown أساسي                 | —                             |
| `code`               | كتل كود مع تلوين بناء الجملة        | `highlight-lang-all`          |
| `custom_code_block`  | خطافات عرض كتل مخصصة               | —                             |
| `image`              | تضمين وتكبير الصور                  | `image`                       |
| `mermaid`            | عرض مخططات Mermaid                 | `mermaid`                     |
| `tree_list`          | عرض شجرة JSON/TOML قابلة للطي      | —                             |

```bash
cargo run --example basic
cargo run --example code --features highlight-lang-all
cargo run --example image --features image
cargo run --example mermaid --features mermaid
cargo run --example tree_list
```

## إشارات الميزات

جميع الميزات مفعلة افتراضيًا. قم بتعطيل الميزات الافتراضية لتفعيل ما تحتاج إليه فقط:

```toml
[dependencies]
ratatui-markdown = { version = "0.2", default-features = false, features = ["markdown"] }
```

| الميزة                | التبعيات                            | الوصف                                            |
|-----------------------|-------------------------------------|--------------------------------------------------|
| `markdown`            | —                                   | محلل وعارض Markdown                              |
| `image`               | —                                   | حل الصور عبر `ImageResolver`                    |
| `scroll`              | —                                   | HybridScrollView وقوائم قابلة للتمرير            |
| `tree`                | `scroll`, `serde_json`, `toml`      | شجرة JSON/TOML قابلة للطي                        |
| `preview`             | `markdown`, `scroll`, `tree`        | عنصر MarkdownPreview الموحد                      |
| `mermaid`             | `markdown`                          | عرض مخططات Mermaid                               |
| `viewer`              | `markdown`, `scroll`                | عنصر MarkdownViewer                              |
| `highlight`           | —                                   | تلوين بناء الجملة عبر tree-sitter                |
| `highlight-lang-*`    | `highlight`                         | قواعد لغوية فردية                                |
| `highlight-lang-all`  | `highlight`                         | جميع القواعد اللغوية المضمنة                      |

## التوثيق

| الدليل | الوصف |
|--------|-------|
| [البدء](getting-started.md) | التثبيت والعرض الأول |
| [Markdown](markdown.md) | تحليل وعرض Markdown |
| [نظام التمرير](scroll.md) | التمرير الهجين والتنقل |
| [عرض الشجرة](tree.md) | أشجار JSON/TOML والتوسيع/الطي |
| [عنصر المعاينة](preview.md) | دمج كل شيء مع MarkdownPreview |
| [السمة](theme.md) | تنفيذ RichTextTheme |
| [المساهمة](contributing.md) | دليل التطوير والمساهمة |

## الترخيص

ترخيص مزدوج MIT OR Apache-2.0.
