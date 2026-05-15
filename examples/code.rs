#[path = "utils/mod.rs"]
mod common;

use std::sync::Arc;

use common::{AppState, Theme, draw_frame, poll_and_handle, setup_terminal, restore_terminal};
use ratatui_markdown::highlight::TreeSitterHighlighter;
use ratatui_markdown::highlight::HighlightHooks;
use ratatui_markdown::markdown::{MarkdownRenderer, RenderHooks};

struct CodeHooks {
    inner: HighlightHooks,
}

impl RenderHooks for CodeHooks {
    fn render_code_block(
        &self,
        lang: &str,
        content: &str,
    ) -> Option<Vec<ratatui::text::Line<'static>>> {
        self.inner.render_code_block(lang, content)
    }
}

const MARKDOWN_TEMPLATE: &str = r#"
# Syntax Highlighting

This example demonstrates **syntax highlighting** for code blocks using
tree-sitter, the same engine that powers GitHub.com code rendering.

Tree-sitter provides incremental parsing, precise syntax highlighting,
and support for many languages via pluggable grammar crates.

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
"#;

fn main() -> anyhow::Result<()> {
    let highlighter = Arc::new(TreeSitterHighlighter::new());
    let hooks = HighlightHooks::new(highlighter, 74);

    let mut terminal = setup_terminal()?;

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76)
        .with_render_hooks(Box::new(CodeHooks { inner: hooks }));
    let blocks = renderer.parse(MARKDOWN_TEMPLATE);
    let lines = renderer.render(&blocks, &theme);
    let mut state = AppState::new(lines.len());

    loop {
        terminal.draw(|f| {
            draw_frame(
                f,
                "Code Highlighting (tree-sitter)",
                &lines,
                &mut state,
                "\u{2191}\u{2193}/jk scroll \u{00b7} PgUp/PgDn \u{00b7} Home/End \u{00b7} q quit",
            );
        })?;
        if poll_and_handle(&mut state)? {
            break;
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}
