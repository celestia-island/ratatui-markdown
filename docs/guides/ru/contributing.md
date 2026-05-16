# Участие в Разработке

## Настройка Окружения Разработки

```bash
git clone https://github.com/celestia-island/ratatui-markdown.git
cd ratatui-markdown
cargo build
```

## Проверки CI

Перед отправкой PR убедитесь, что проходят следующие проверки:

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

## Соглашения Проекта

### Стиль Кода

- Следуйте стандартным идиомам Rust (`cargo fmt` и `cargo clippy` обеспечивают это)
- Без комментариев в коде, если они не являются строго необходимыми
- Используйте `pub(crate)` для внутренней видимости; `pub` только для публичного API

### Организация Модулей

Каждый функциональный модуль находится в `src/{module}/`:

```
src/markdown/       # Функция: markdown
  ├── mod.rs        # Реэкспорт публичных типов, определяет MarkdownRenderer
  ├── parser.rs     # Блочный парсер (impl MarkdownRenderer)
  ├── types.rs      # Перечисления MarkdownBlock, TextToken
  ├── render.rs     # Блочный рендерер
  ├── inline.rs     # Парсер встроенного форматирования
  ├── text.rs       # Утилиты переноса текста
  ├── tests.rs      # Тесты парсера/интеграции
  └── render_tests.rs  # Снапшот-тесты вывода рендеринга
```

Тесты располагаются рядом с исходными файлами внутри блоков `#[cfg(test)] mod tests { }`, а более крупные наборы тестов — в отдельных файлах `tests.rs` / `render_tests.rs`.

### Функциональные Флаги

Все функции включены по умолчанию. Используйте `cfg(feature = "X")` для условной компиляции кода:

```rust
#[cfg(feature = "markdown")]
pub mod markdown;
```

Зависимости функций описаны в `Cargo.toml`:

```toml
tree = ["dep:serde_json", "dep:toml", "scroll"]
preview = ["markdown", "scroll", "tree"]
```

### Тестирование

Запуск всех тестов:

```bash
cargo test --all-features
```

Тестирование каждой комбинации функций:

```bash
cargo test --no-default-features
cargo test --no-default-features --features markdown
cargo test --no-default-features --features scroll
cargo test --no-default-features --features tree
cargo test --no-default-features --features mermaid
cargo test --no-default-features --features image
cargo test --no-default-features --features viewer
cargo test --no-default-features --features preview  # подразумевает markdown, scroll, tree
```

### Документация

- Документация API соответствует соглашениям doc-комментариев Rust
- Пользовательская документация находится в `docs/guides/`
- `docs/guides/en/` — это каноническая (английская) документация
- Переводы на другие языки приветствуются в `docs/guides/{lang}/`

## Стиль Сообщений Коммитов

Следуйте формату conventional commits:

```
тип: краткое описание

тип: feat, fix, refactor, test, docs, chore, ci, style
```

## Процесс Релиза

Публикация на crates.io выполняется рабочим процессом GitHub Actions `publish.yml` при публикации тега.

```bash
# Обновите версию в Cargo.toml, затем:
git tag v0.1.1
git push origin v0.1.1
```
