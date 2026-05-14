# Модуль Markdown

> Парсинг и рендеринг текста Markdown в стилизованные `ratatui::text::Line`.

## Обзор

Модуль `markdown` предоставляет собственный парсер и рендерер Markdown, предназначенный для терминального вывода. Он **не** является парсером, совместимым с CommonMark — он нацелен на подмножество Markdown, наиболее полезное в терминальных интерфейсах.

Ограничен функциональным флагом `markdown` (включён по умолчанию).

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

### Конструктор

`MarkdownRenderer::new(max_width)` принимает доступную ширину содержимого в столбцах. Эта ширина используется для переноса текста абзацев (с учётом CJK) и определения ширины столбцов таблиц.

### Парсинг

`parse()` принимает `&str` текста Markdown и возвращает `Vec<MarkdownBlock>`. Парсер ориентирован на строки и обрабатывает блоки последовательно.

### Рендеринг

`render()` принимает распарсенные блоки и тему, создавая `Vec<Line<'static>>`, пригодный для непосредственного использования в виджетах ratatui.

## Перечисление MarkdownBlock

```rust
pub enum MarkdownBlock {
    Heading1(String),              // # Заголовок
    Heading2(String),              // ## Заголовок
    Heading3(String),              // ### Заголовок
    Paragraph(Vec<String>),        // Перенесённые строки абзаца
    CodeBlock(String, String),     // (язык, содержимое)
    InlineCode(String),            // `встроенный код`
    ListItem(String, u8),          // (содержимое, уровень_отступа)
    Blockquote(String),            // > цитируемый текст
    HorizontalRule,                // --- или *** или ___
    BlankLine,                     // пустая строка
    Table {                        // | кол1 | кол2 |
        headers: Vec<String>,      //   |------|------|
        rows: Vec<Vec<String>>,    //   | знч1 | знч2 |
    },
}
```

### Детали Блоков

**Заголовки** (H1-H3): Рендерятся с `primary_color`, при этом H1 использует `Modifier::BOLD`.

**Абзацы**: Текст переносится по словам с учётом CJK до `max_width`. Каждая перенесённая строка становится записью в `Vec<String>`.

**Блоки Кода** (ограниченные ` ``` `): Рендерятся с `muted_text_color` внутри рамок, нарисованных символами псевдографики. Блоки кода Mermaid молча пропускаются.

**Встроенный Код**: Рендерится с `secondary_color` и `Modifier::DIM`.

**Списки**: Неупорядоченные (`-`, `*`, `+`) и упорядоченные (`1.`, `2.`). Каждый элемент сохраняет свой уровень отступа. Подэлементы визуально смещены.

**Цитаты**: Предваряются цветной полосой `│` и рендерятся с `muted_text_color`.

**Таблицы**: Столбцы имеют пропорциональный размер на основе ширины содержимого. Ячейки переносятся, заголовки используют `Modifier::BOLD`, а границы используют символы псевдографики.

## Встроенное Форматирование

Встроенное форматирование применяется **внутри** текста абзацев и элементов списка:

| Markdown        | Визуальный Эффект              |
|-----------------|--------------------------------|
| `**текст**`     | **Жирный** (`Modifier::BOLD`) |
| `*текст*`       | *Курсив* (`Modifier::ITALIC`) |
| `***текст***`   | ***Жирный+Курсив***            |
| `` `код` ``     | Стиль `Встроенного Кода`      |

```rust
pub fn parse_inline_formatting(text: &str, theme: &impl RichTextTheme) -> Vec<Span<'static>>;
```

Эта автономная функция также реэкспортируется для использования вне `MarkdownRenderer`.

## Пример

```rust
use ratatui_markdown::markdown::MarkdownRenderer;

let md = r#"
# Заголовок

Это абзац с **жирным** и *курсивным* текстом.

## Код

```rust
fn main() {
    println!("Привет!");
}
```

| Имя     | Версия |
|---------|--------|
| ratatui | 0.29   |
| serde   | 1.0    |
"#;

let renderer = MarkdownRenderer::new(80);
let blocks = renderer.parse(md);
let lines = renderer.render(&blocks, theme);
// использование lines в виджете ratatui
```
