# المساهمة

## إعداد التطوير

```bash
git clone https://github.com/celestia-island/ratatui-markdown.git
cd ratatui-markdown
cargo build
```

## فحوصات CI

قبل تقديم طلب سحب (PR)، تأكد من نجاح ما يلي:

```bash
cargo test --all-features
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings
cargo test --no-default-features
cargo test --no-default-features --features markdown
cargo test --no-default-features --features scroll
cargo test --no-default-features --features tree
cargo test --no-default-features --features mermaid
cargo test --no-default-features --features image
cargo test --no-default-features --features viewer
```

## اصطلاحات المشروع

### أسلوب الكود

- اتبع تعابير Rust القياسية (`cargo fmt` و `cargo clippy` يفرضان ذلك)
- لا توجد تعليقات في الكود إلا عند الضرورة القصوى
- استخدم `pub(crate)` للرؤية الداخلية؛ `pub` فقط لسطح API العام

### تنظيم الوحدات

تعيش كل وحدة ميزة تحت `src/{module}/`:

```
src/markdown/       # الميزة: markdown
  ├── mod.rs        # إعادة تصدير الأنواع العامة، تعريف MarkdownRenderer
  ├── parser.rs     # محلل على مستوى الكتل (impl MarkdownRenderer)
  ├── types.rs      # تعدادات MarkdownBlock و TextToken
  ├── render.rs     # عارض على مستوى الكتل
  ├── inline.rs     # محلل التنسيق المضمن
  ├── text.rs       # أدوات التفاف النص
  ├── tests.rs      # اختبارات المحلل/التكامل
  └── render_tests.rs  # اختبارات لقطة مخرجات العرض
```

تعيش الاختبارات بجانب الملفات المصدرية داخل كتل `#[cfg(test)] mod tests { }`، مع مجموعات اختبار أكبر في ملفات `tests.rs` / `render_tests.rs` مخصصة.

### علامات الميزات

جميع الميزات مفعلة افتراضيًا. استخدم `cfg(feature = "X")` لحصر الكود:

```rust
#[cfg(feature = "markdown")]
pub mod markdown;
```

تُعبر اعتماديات الميزات في `Cargo.toml`:

```toml
tree = ["dep:serde_json", "dep:toml", "scroll"]
preview = ["markdown", "scroll", "tree"]
```

### الاختبارات

شغّل جميع الاختبارات:

```bash
cargo test --all-features
```

اختبر كل تركيبة ميزات:

```bash
cargo test --no-default-features
cargo test --no-default-features --features markdown
cargo test --no-default-features --features scroll
cargo test --no-default-features --features tree
cargo test --no-default-features --features mermaid
cargo test --no-default-features --features image
cargo test --no-default-features --features viewer
cargo test --no-default-features --features preview  # يتضمن markdown, scroll, tree
```

### التوثيق

- توثيق API يتبع اصطلاحات تعليقات التوثيق في Rust
- التوثيق الموجه للمستخدم موجود تحت `docs/guides/`
- `docs/guides/en/` هو التوثيق القانوني (الإنجليزية)
- الترجمات مرحب بها للغات الأخرى تحت `docs/guides/{lang}/`

## أسلوب رسائل commit

اتبع تنسيق conventional commit:

```
type: وصف قصير

type: feat, fix, refactor, test, docs, chore, ci, style
```

## عملية الإصدار

النشر إلى crates.io يتم بواسطة سير عمل `publish.yml` في GitHub Actions عند دفع tag.

```bash
# قم بترقية الإصدار في Cargo.toml، ثم:
git tag v0.1.1
git push origin v0.1.1
```
