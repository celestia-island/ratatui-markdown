#[path = "_common/mod.rs"]
mod common;

use std::sync::Arc;

use common::{AppState, Theme, draw_frame, poll_and_handle, setup_terminal, restore_terminal};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use ratatui_markdown::markdown::{MarkdownRenderer, RenderHooks};
use syntect::highlighting::{Color as SynColor, FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect_tui::translate_colour;

// HACK: syntect 的 Style.foreground 是 Color（非 Option），无法表达"无色"。
// 当前做法是往 theme.settings 里注入哨兵色，运行时比对哨兵来判断"未被高亮"。
// 这本质是绕过 syntect 的设计限制，后续应考虑：
//   1. 基于 tree-sitter + 自建 token→ratatui Style 映射，彻底绕开 syntect；
//   2. 或 fork/patch syntect 让 Style.foreground 支持 Option<Color>。
struct SentinelColors {
    fg: SynColor,
    bg: SynColor,
}

impl SentinelColors {
    fn pick(theme: &syntect::highlighting::Theme) -> Self {
        let mut used = std::collections::HashSet::new();
        if let Some(c) = theme.settings.foreground { used.insert(c); }
        if let Some(c) = theme.settings.background { used.insert(c); }
        for s in &theme.scopes {
            if let Some(c) = s.style.foreground { used.insert(c); }
            if let Some(c) = s.style.background { used.insert(c); }
        }

        let candidates: Vec<SynColor> = (0..u8::MAX)
            .map(|i| SynColor { r: i.wrapping_mul(3), g: i.wrapping_mul(5), b: i.wrapping_mul(7), a: 0xFF })
            .filter(|c| !used.contains(c))
            .take(3)
            .collect();

        Self {
            fg: candidates[0],
            bg: candidates[1],
        }
    }
}

struct SyntectHighlighter {
    syntax_set: SyntaxSet,
    theme: syntect::highlighting::Theme,
    sentinel: SentinelColors,
}

impl SyntectHighlighter {
    fn new(theme_name: &str) -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let mut theme = theme_set.themes[theme_name].clone();
        let sentinel = SentinelColors::pick(&theme);
        theme.settings.foreground = Some(sentinel.fg);
        theme.settings.background = Some(sentinel.bg);
        Self { syntax_set, theme, sentinel }
    }

    fn to_tui_style(&self, s: syntect::highlighting::Style) -> Style {
        if s.foreground == self.sentinel.fg {
            return Style::default();
        }
        let mut out = Style::default();
        if let Some(c) = translate_colour(s.foreground) {
            out.fg = Some(c);
        }
        if s.font_style != FontStyle::empty() {
            let mut m = Modifier::empty();
            if s.font_style.contains(FontStyle::BOLD) {
                m |= Modifier::BOLD;
            }
            if s.font_style.contains(FontStyle::ITALIC) {
                m |= Modifier::ITALIC;
            }
            if s.font_style.contains(FontStyle::UNDERLINE) {
                m |= Modifier::UNDERLINED;
            }
            if !m.is_empty() {
                out.add_modifier = m;
            }
        }
        out
    }

    fn render_code_block(&self, lang: &str, code: &str) -> Vec<Line<'static>> {
        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .or_else(|| self.syntax_set.find_syntax_by_extension(lang));

        let display_lang = if lang.is_empty() { "code" } else { lang };
        let border_style = Style::default().fg(Color::DarkGray);

        let mut lines = Vec::new();

        lines.push(Line::from(Span::styled(
            format!("\u{256d}\u{2500} {display_lang}"),
            border_style,
        )));

        let prefix = Span::styled("\u{2502} ".to_string(), border_style);

        match syntax {
            Some(syntax) => {
                let mut hl =
                    syntect::easy::HighlightLines::new(syntax, &self.theme);
                for raw_line in code.lines() {
                    let ranges = hl
                        .highlight_line(raw_line, &self.syntax_set)
                        .unwrap();
                    let mut spans: Vec<Span<'static>> = vec![prefix.clone()];
                    for (style, text) in ranges.iter() {
                        spans.push(Span::styled(
                            text.to_string(),
                            self.to_tui_style(*style),
                        ));
                    }
                    lines.push(Line::from(spans));
                }
            }
            None => {
                for raw_line in code.lines() {
                    lines.push(Line::from(vec![
                        prefix.clone(),
                        Span::styled(raw_line.to_string(), Style::default()),
                    ]));
                }
            }
        }

        lines.push(Line::from(Span::styled(
            format!("\u{2570}\u{2500}"),
            border_style,
        )));

        lines
    }
}

struct CodeHooks {
    highlighter: Arc<SyntectHighlighter>,
}

impl RenderHooks for CodeHooks {
    fn render_code_block(
        &self,
        lang: &str,
        content: &str,
    ) -> Option<Vec<Line<'static>>> {
        Some(self.highlighter.render_code_block(lang, content))
    }
}

const MARKDOWN_TEMPLATE: &str = r#"
# Syntax Highlighting

This example demonstrates **syntax highlighting** for code blocks using
[syntect](https://github.com/trishume/syntect), the same engine that powers
`bat` — a `cat(1)` clone with syntax highlighting.

Syntect bundles TextMate-compatible grammar files, supporting **100+ languages**
out of the box with zero configuration.

## Rust

```rust
use std::collections::HashMap;

fn word_count(text: &str) -> HashMap<&str, usize> {
    let mut map = HashMap::new();
    for word in text.split_whitespace() {
        *map.entry(word).or_insert(0) += 1;
    }
    map
}

fn main() {
    let text = "hello world hello rust world";
    for (word, count) in word_count(text) {
        println!("{word}: {count}");
    }
}
```

## Python

```python
from dataclasses import dataclass
from typing import Optional

@dataclass
class TreeNode:
    value: int
    left: Optional['TreeNode'] = None
    right: Optional['TreeNode'] = None

def inorder(node: Optional[TreeNode]) -> list[int]:
    if node is None:
        return []
    return inorder(node.left) + [node.value] + inorder(node.right)

# Build a simple BST
root = TreeNode(4, TreeNode(2, TreeNode(1), TreeNode(3)), TreeNode(6))
print(inorder(root))  # [1, 2, 3, 4, 6]
```

## JavaScript / TypeScript

```javascript
class EventEmitter {
  #listeners = new Map();

  on(event, callback) {
    if (!this.#listeners.has(event)) {
      this.#listeners.set(event, []);
    }
    this.#listeners.get(event).push(callback);
    return () => this.off(event, callback);
  }

  emit(event, ...args) {
    for (const cb of this.#listeners.get(event) ?? []) {
      cb(...args);
    }
  }

  off(event, callback) {
    const cbs = this.#listeners.get(event);
    if (cbs) {
      this.#listeners.set(event, cbs.filter(cb => cb !== callback));
    }
  }
}
```

## Go

```go
package main

import (
	"fmt"
	"sync"
	"time"
)

func fetch(id int, wg *sync.WaitGroup) {
	defer wg.Done()
	duration := time.Duration(id*100) * time.Millisecond
	time.Sleep(duration)
	fmt.Printf("Worker %d done (took %v)\n", id, duration)
}

func main() {
	var wg sync.WaitGroup
	for i := 1; i <= 5; i++ {
		wg.Add(1)
		go fetch(i, &wg)
	}
	wg.Wait()
	fmt.Println("All workers complete")
}
```

## C / C++

```cpp
#include <iostream>
#include <vector>
#include <algorithm>

template<typename T>
void quicksort(std::vector<T>& arr, int lo, int hi) {
    if (lo >= hi) return;
    T pivot = arr[(lo + hi) / 2];
    int i = lo, j = hi;
    while (i <= j) {
        while (arr[i] < pivot) i++;
        while (arr[j] > pivot) j--;
        if (i <= j) std::swap(arr[i++], arr[j--]);
    }
    quicksort(arr, lo, j);
    quicksort(arr, i, hi);
}

int main() {
    std::vector<int> data = {9, 3, 7, 1, 5, 8, 2, 6, 4};
    quicksort(data, 0, data.size() - 1);
    for (int x : data) std::cout << x << ' ';
    // 1 2 3 4 5 6 7 8 9
}
```

## Java

```java
import java.util.stream.*;
import java.util.List;

public class Streams {
    public static void main(String[] args) {
        List<String> names = List.of("Alice", "Bob", "Charlie", "Diana");

        List<String> upper = names.stream()
            .filter(n -> n.length() > 3)
            .map(String::toUpperCase)
            .sorted()
            .collect(Collectors.toList());

        upper.forEach(System.out::println);
        // ALICE, CHARLIE, DIANA
    }
}
```

## Shell / Bash

```bash
#!/usr/bin/env bash
set -euo pipefail

log() { echo "[$(date +%H:%M:%S)] $*"; }

backup_dir="$HOME/.backup/$(date +%Y%m%d)"
mkdir -p "$backup_dir"

for file in "$@"; do
    if [[ -f "$file" ]]; then
        cp -v "$file" "$backup_dir/"
        log "Backed up: $file"
    else
        log "Skipping (not found): $file"
    fi
done

log "Done. Files saved to $backup_dir"
```

## SQL

```sql
WITH monthly_revenue AS (
    SELECT
        DATE_TRUNC('month', order_date) AS month,
        SUM(quantity * unit_price)       AS revenue,
        COUNT(DISTINCT customer_id)      AS customers
    FROM orders
    WHERE order_date >= '2024-01-01'
    GROUP BY 1
)
SELECT
    TO_CHAR(month, 'YYYY-MM')  AS month,
    TO_CHAR(revenue, '$999,999') AS revenue,
    customers,
    LAG(revenue) OVER (ORDER BY month) AS prev_month
FROM monthly_revenue
ORDER BY month DESC
LIMIT 12;
```

## HTML

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Todo App</title>
  <style>
    .done { text-decoration: line-through; opacity: 0.6; }
  </style>
</head>
<body>
  <h1>Todo List</h1>
  <ul id="list"></ul>
  <input id="input" placeholder="Add item..." autofocus />
  <button onclick="addItem()">Add</button>
</body>
</html>
```

## CSS

```css
:root {
  --primary: #6366f1;
  --bg: #0f172a;
  --surface: #1e293b;
  --text: #e2e8f0;
}

.card {
  background: var(--surface);
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.3);
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 32px rgba(99, 102, 241, 0.25);
}
```

## TOML

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.29"
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }

[profile.release]
lto = true
strip = true
opt-level = 3
```

## JSON

```json
{
  "name": "syntax-highlight",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "test": "vitest run"
  },
  "devDependencies": {
    "typescript": "^5.4.0",
    "vite": "^5.2.0"
  }
}
```
"#;

fn main() -> anyhow::Result<()> {
    let highlighter = Arc::new(SyntectHighlighter::new("InspiredGitHub"));

    setup_terminal()?;

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76)
        .with_render_hooks(Box::new(CodeHooks {
            highlighter: highlighter.clone(),
        }));
    let blocks = renderer.parse(MARKDOWN_TEMPLATE);
    let lines = renderer.render(&blocks, &theme);
    let mut state = AppState::new(lines.len());

    loop {
        draw_frame(
            "Code Highlighting",
            &lines,
            &mut state,
            "\u{2191}\u{2193}/jk scroll \u{00b7} PgUp/PgDn \u{00b7} Home/End \u{00b7} q quit",
        )?;
        if poll_and_handle(&mut state)? {
            break;
        }
    }

    restore_terminal()?;
    Ok(())
}
