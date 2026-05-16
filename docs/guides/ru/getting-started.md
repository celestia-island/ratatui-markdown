# Быстрый Старт

## Предварительные Требования

- **Rust** 1.74 или новее
- **ratatui** 0.29 (добавляется автоматически как зависимость)

## Установка

Добавьте в ваш `Cargo.toml`:

```toml
[dependencies]
ratatui-markdown = "0.1"
```

Это включает все возможности по умолчанию (`markdown`, `scroll`, `tree`, `preview`, `mermaid`, `image`, `viewer`).

### Выборочные Функции

Чтобы сократить время компиляции и количество зависимостей, включите только необходимое:

```toml
# Только рендеринг Markdown
ratatui-markdown = { version = "0.1", default-features = false, features = ["markdown"] }

# Только система прокрутки
ratatui-markdown = { version = "0.1", default-features = false, features = ["scroll"] }

# Вид дерева (добавляет scroll, serde_json и toml)
ratatui-markdown = { version = "0.1", default-features = false, features = ["tree"] }
```

## Базовое Использование

### Рендеринг Markdown

```rust
use ratatui_markdown::markdown::MarkdownRenderer;
use ratatui_markdown::theme::RichTextTheme;

// Создание рендерера с максимальной шириной содержимого
let renderer = MarkdownRenderer::new(80);

// Парсинг текста Markdown в блоки
let blocks = renderer.parse("# Привет\n\nЭто **жирный** и *курсивный* текст.");

// Рендеринг блоков в ratatui::text::Line<'static>
let lines = renderer.render(&blocks, &my_theme);
```

### Навигация по Дереву

```rust
use ratatui_markdown::tree::CollapsibleTree;

// Парсинг JSON в сворачиваемое дерево
let json_str = r#"{"name": "project", "deps": {"ratatui": "0.29", "serde": "1.0"}}"#;
let mut tree = CollapsibleTree::from_json_str(json_str).unwrap();

// Рендеринг строк дерева
let lines = tree.render_lines(80, &my_theme);

// Получение фокусируемых элементов для навигации
let items = tree.build_focusable_items();

// Переключение узла
tree.toggle("deps/serde");
```

### Использование Виджета MarkdownPreview

Виджет `MarkdownPreview` объединяет всё в единое прокручиваемое представление:

```rust
use ratatui_markdown::preview::MarkdownPreview;
use ratatui_markdown::theme::RichTextTheme;

let mut preview = MarkdownPreview::new()
    .with_left_padding(true);

// Установка содержимого Markdown
preview.set_content("# Добро пожаловать\n\n- Пункт один\n- Пункт два\n\n```rust\nlet x = 42;\n```");

// Установка сворачиваемого дерева (опционально)
let tree = CollapsibleTree::from_json_str(r#"{"config": {"port": 8080}}"#).unwrap();
preview.set_tree(Some(tree));

// Обработка ввода с клавиатуры
preview.scroll_up();
preview.scroll_down();
preview.page_up(10);
preview.page_down(10);
preview.toggle_tree_node(); // Клавиша Enter

// Рендеринг в цикле отрисовки ratatui
fn draw(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}
```

## Реализация Темы

Библиотека использует трейт для получения всех цветов:

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

Измените возвращаемое значение `generation()`, чтобы инвалидировать кэш виджета предпросмотра и принудительно выполнить повторный рендеринг (например, когда пользователь переключает темы во время выполнения).

## Дальнейшие Шаги

- [Модуль Markdown](markdown.md) — полное API парсинга и рендеринга Markdown
- [Система Прокрутки](scroll.md) — понимание архитектуры гибридной прокрутки
- [Вид Дерева](tree.md) — рендеринг и взаимодействие с деревьями JSON/TOML
- [Виджет Предпросмотра](preview.md) — высокоуровневый унифицированный виджет
- [Тема](theme.md) — полное руководство по настройке темы
