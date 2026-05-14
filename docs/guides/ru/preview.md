# Виджет MarkdownPreview

> Высокоуровневый унифицированный виджет, объединяющий Markdown, деревья и действия в одном прокручиваемом представлении.

## Обзор

`MarkdownPreview` — это виджет интеграции верхнего уровня. Он объединяет рендеринг Markdown, отображение сворачиваемого дерева и элементы действий в единый `HybridScrollView`. Это рекомендуемая точка входа для большинства приложений.

Ограничен функциональным флагом `preview` (включён по умолчанию, требует все остальные функции).

## API

```rust
pub struct ActionItem {
    pub id: String,
    pub label: String,
}

impl MarkdownPreview {
    pub fn new() -> Self;

    // Конфигурация
    pub fn with_strip_frontmatter(self, strip: bool) -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;

    // Содержимое
    pub fn set_content(&mut self, content: &str);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // Дерево
    pub fn set_tree(&mut self, tree: Option<CollapsibleTree>);
    pub fn tree_mut(&mut self) -> Option<&mut CollapsibleTree>;
    pub fn has_tree(&self) -> bool;
    pub fn toggle_tree_node(&mut self) -> bool;

    // Элементы действий
    pub fn set_action_items(&mut self, items: Vec<ActionItem>);
    pub fn selected_action_id(&self) -> Option<&str>;

    // Навигация
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // Состояние
    pub fn total_lines(&self) -> usize;
    pub fn scroll_offset(&self) -> usize;
    pub fn visible_height(&self) -> usize;
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;

    // Рендеринг
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

## Конфигурация

### with_strip_frontmatter

Когда включено (по умолчанию), содержимое, ограниченное `+++`, рассматривается как преамбула TOML и удаляется перед рендерингом:

```
+++
title = "Мой Документ"
author = "Иван Иванов"
+++

# Фактическое Содержимое
Это тело документа.
```

Только `# Фактическое Содержимое` и следующий абзац появятся в выводе.

### with_left_padding

Добавляет 1 столбец левого отступа к рендеренному содержимому (передаётся в `HybridScrollView`).

## Макет Содержимого

Виджет рендерит содержимое в вертикальном порядке:

1. **Дерево** (если присутствует) — строки дерева с фокусируемыми узлами
2. **Markdown** — распарсенное и отрендеренное содержимое
3. **Элементы Действий** — фокусируемые метки действий с обёрткой в скобки `[метка]`

Между секциями вставляется пустая строка, и каждая секция получает собственную `FocusableRegion` в виде прокрутки.

## Кэширование

`MarkdownPreview` кэширует отрендеренный вывод и перестраивает его только при:

- Изменении содержимого (`set_content` с другим текстом)
- Изменении ширины (изменение размера терминала)
- Изменении поколения темы (`theme.generation()` возвращает новое значение)
- Изменении дерева (`set_tree`, `toggle_tree_node`)
- Изменении элементов действий (`set_action_items`)

Используйте `theme.generation()` для запуска повторного рендеринга после изменения темы.

## Обработка Преамбулы TOML

Предполагается, что преамбула имеет формат TOML. Она **не парсится** — просто удаляется из вывода. Первая строка `+++` начинает режим преамбулы, вторая `+++` заканчивает его. Строки до и после блока преамбулы рендерятся нормально.

Если содержимое не начинается с `+++`, удаления не происходит.

## Элементы Действий

Элементы действий предоставляют выбираемые с клавиатуры опции в нижней части представления:

```rust
preview.set_action_items(vec![
    ActionItem { id: "confirm".into(), label: " Подтвердить ".into() },
    ActionItem { id: "cancel".into(), label: " Отмена ".into() },
]);

// В вашем обработчике ввода:
if let Some("confirm") = preview.selected_action_id() {
    // обработка подтверждения
}
```

ID элементов действий внутренне предваряются префиксом `action:` для избежания коллизий с путями узлов дерева.

## Пример

```rust
use ratatui_markdown::preview::{MarkdownPreview, ActionItem};
use ratatui_markdown::tree::CollapsibleTree;

let mut preview = MarkdownPreview::new()
    .with_strip_frontmatter(true)
    .with_left_padding(true);

// Установка содержимого Markdown (с преамбулой TOML)
preview.set_content(concat!(
    "+++\n",
    "title = \"Мой Документ\"\n",
    "+++\n",
    "\n",
    "# Привет\n\nСодержимое здесь.\n",
));

// Установка дерева
let tree = CollapsibleTree::from_json_str(r#"{"config": {"theme": "dark"}}"#).unwrap();
preview.set_tree(Some(tree));

// Установка элементов действий
preview.set_action_items(vec![
    ActionItem { id: "edit".into(), label: " Редактировать ".into() },
    ActionItem { id: "save".into(), label: " Сохранить ".into() },
]);

// В функции рендеринга вашего приложения ratatui:
fn render_ui(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}

// Обработка ввода
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
