# Настройка Темы

> Реализация пользовательских цветовых схем через трейт `RichTextTheme`.

## Обзор

Все запросы цветов и стилей проходят через трейт `RichTextTheme`, что позволяет отделить логику рендеринга от внешнего вида. Реализуйте этот трейт один раз и передавайте его во все функции рендеринга.

## Трейт RichTextTheme

```rust
pub trait RichTextTheme {
    fn generation(&self) -> Generation;

    // Основной текст
    fn get_text_color(&self) -> Color;
    fn get_muted_text_color(&self) -> Color;
    fn get_background_color(&self) -> Color;

    // Акценты / выделение
    fn get_primary_color(&self) -> Color;
    fn get_secondary_color(&self) -> Color;
    fn get_info_color(&self) -> Color;

    // Рамки
    fn get_border_color(&self) -> Color;
    fn get_focused_border_color(&self) -> Color;

    // Всплывающие окна / наложения
    fn get_popup_selected_background(&self) -> Color;
    fn get_popup_selected_text_color(&self) -> Color;

    // Значения JSON/Дерева
    fn get_json_key_color(&self) -> Color;
    fn get_json_string_color(&self) -> Color;
    fn get_json_number_color(&self) -> Color;
    fn get_json_bool_color(&self) -> Color;
    fn get_json_null_color(&self) -> Color;

    // Разное
    fn get_accent_yellow(&self) -> Color;
}
```

## Токен Поколения (Generation)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Generation(pub u64);

impl Generation {
    pub fn next(&self) -> Self { Self(self.0 + 1) }
}
```

Токен `Generation` используется `MarkdownPreview` для инвалидации кэша. Когда `generation()` темы возвращает значение, отличное от закэшированного, предпросмотр перестраивает весь отрендеренный вывод.

**Увеличивайте поколение каждый раз, когда пользователь меняет тему** во время выполнения:

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

## Использование Цветовых Слотов

Вот как каждый цвет темы используется на практике:

| Метод                         | Используется Для                                      |
|-------------------------------|-------------------------------------------------------|
| `get_text_color`              | Текст абзацев по умолчанию, текст элементов списка    |
| `get_muted_text_color`        | Блоки кода, текст цитат, границы таблиц               |
| `get_primary_color`           | Заголовки (H1-H3), акцент жирного текста              |
| `get_secondary_color`         | Встроенный код, вторичные акценты                     |
| `get_info_color`              | Информационные выделения                              |
| `get_background_color`        | Общие фоновые заливки                                 |
| `get_border_color`            | Нефокусированные рамки (блоки кода, таблицы)          |
| `get_focused_border_color`    | Рамки фокусированных/вовлечённых элементов, полоса прокрутки |
| `get_popup_selected_background` | Фон выбранных элементов всплывающих окон             |
| `get_popup_selected_text_color` | Цвет текста выбранных элементов всплывающих окон     |
| `get_json_key_color`          | Ключи узлов дерева                                    |
| `get_json_string_color`       | Строковые значения в деревьях                         |
| `get_json_number_color`       | Числовые значения в деревьях                          |
| `get_json_bool_color`         | Булевы значения в деревьях                            |
| `get_json_null_color`         | Null-значения в деревьях                              |
| `get_accent_yellow`           | Предупреждения/акценты выделения                      |

## Пример: Полная Тема

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

## Замечания по Производительности

- Все методы темы вызываются часто во время рендеринга. Делайте их лёгкими — возвращайте константы или простые обращения к полям.
- Сравнение токена `Generation` — это простая проверка равенства целых чисел, поэтому его увеличение выполняется быстро.
