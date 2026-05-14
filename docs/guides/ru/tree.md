# Вид Дерева

> Интерактивное сворачиваемое дерево для данных JSON и TOML.

## Обзор

Модуль `tree` парсит JSON или TOML в интерактивное сворачиваемое дерево. Пользователи могут разворачивать/сворачивать узлы и перемещаться с клавиатуры в терминальном интерфейсе.

Ограничен функциональным флагом `tree` (требует зависимости `scroll`, `serde_json` и `toml`).

## CollapsibleTree

```rust
pub struct CollapsibleTree { /* поля */ }

impl CollapsibleTree {
    // Конструкторы
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error>;
    pub fn from_toml_str(toml: &str) -> Result<Self, Box<dyn Error>>;
    pub fn from_value(value: serde_json::Value, style: KeyStyle) -> Self;

    // Манипуляции с деревом
    pub fn toggle(&mut self, path: &str) -> bool;
    pub fn handle_toggle(&mut self, id: &str) -> bool;
    pub fn expand_all(&mut self);
    pub fn collapse_all(&mut self);
    pub fn expand_to_depth(&mut self, depth: usize);

    // Рендеринг
    pub fn render_lines(&self, width: usize, theme: &impl RichTextTheme) -> Vec<Line<'static>>;
    pub fn flatten(&self) -> Vec<FlatEntry>;
    pub fn build_focusable_items(&self) -> Vec<FocusableItemRange>;
}
```

### Конструкторы

- **`from_json_str`**: Парсит строку JSON в дерево. Ключи используют `KeyStyle::Json` (в кавычках, с разделителем `:`).
- **`from_toml_str`**: Парсит строку TOML (внутренне преобразует в JSON). Ключи используют `KeyStyle::Toml` (без кавычек, с разделителем `=`).
- **`from_value`**: Строит дерево из существующего `serde_json::Value` с выбранным стилем ключей.

### Манипуляции с Деревом

```rust
// Переключение с использованием пути, разделённого слэшами
tree.toggle("dependencies");          // переключить корневой ключ
tree.toggle("dependencies/serde");    // переключить вложенный ключ

// Удобные методы
tree.expand_all();
tree.collapse_all();
tree.expand_to_depth(2);

// В контексте прокрутки — используется ID выбранного элемента
if scroll.selected_item_id().is_some() {
    tree.handle_toggle(selected_id);
}
```

### Рендеринг

- **`render_lines`**: Создаёт стилизованные `Line` с соединителями дерева и цветными значениями.
- **`flatten`**: Возвращает плоский список всех видимых записей (учитывает состояние свёрнутости).
- **`build_focusable_items`**: Возвращает фокусируемые диапазоны для интеграции с `HybridScrollView`, с ID, соответствующими путям дерева.

## Типы Данных

```rust
pub struct FlatEntry {
    pub depth: usize,
    pub key: String,
    pub value: String,
    pub kind: EntryKind,
    pub value_type: ValueType,
    pub path: String,          // например, "dependencies/serde"
    pub is_last: bool,
}

pub enum EntryKind {
    Collapsed,  // имеет дочерние элементы, в данный момент свёрнут: [+]
    Expanded,   // имеет дочерние элементы, в данный момент развёрнут: [-]
    Leaf,       // нет дочерних элементов
}

pub enum ValueType {
    String, Number, Boolean, Null, Object, Array,
}

pub enum KeyStyle {
    Json,  // "ключ": значение
    Toml,  // ключ = значение
}
```

### Цвета Типов Значений

Каждый `ValueType` сопоставляется с соответствующим методом цвета темы:

| ValueType | Метод Темы              |
|-----------|-------------------------|
| String    | `get_json_string_color()` |
| Number    | `get_json_number_color()` |
| Boolean   | `get_json_bool_color()`   |
| Null      | `get_json_null_color()`   |

## Вспомогательные Функции Строк Дерева

Модуль `tree_lines` (реэкспортирован из `crate::tree`) предоставляет низкоуровневое построение строк:

```rust
pub fn make_body_line(key: &str, value: &str, style: &Style, value_style: &Style) -> Line<'static>;
pub fn make_status_line(status: &str, style: &Style) -> Line<'static>;
pub fn make_branch_dispatch_line(kind: EntryKind, ...) -> Line<'static>;
```

## Пример

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

// Развернуть всё
tree.expand_all();

// Рендеринг в строки
let lines = tree.render_lines(80, theme);

// Свернуть поддерево зависимостей
tree.toggle("dependencies");

// Повторный рендеринг — зависимости теперь свёрнуты
let lines = tree.render_lines(80, theme);

// Получить фокусируемые элементы для навигации с прокруткой
let items = tree.build_focusable_items();
```
