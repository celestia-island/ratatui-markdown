# 快速开始

## 前置条件

- **Rust** 1.74 或更高版本
- **ratatui** 0.29（作为依赖自动引入）

## 安装

将以下内容添加到你的 `Cargo.toml`：

```toml
[dependencies]
ratatui-markdown = "0.1"
```

默认启用所有功能（`markdown`、`scroll`、`tree`、`preview`、`mermaid`、`image`、`viewer`）。

### 选择性功能

要减少编译时间和依赖项，只启用你需要的功能：

```toml
# 仅 Markdown 渲染
ratatui-markdown = { version = "0.1", default-features = false, features = ["markdown"] }

# 仅滚动系统
ratatui-markdown = { version = "0.1", default-features = false, features = ["scroll"] }

# 树视图（会引入 scroll、serde_json 和 toml）
ratatui-markdown = { version = "0.1", default-features = false, features = ["tree"] }
```

## 基本用法

### 渲染 Markdown

```rust
use ratatui_markdown::markdown::MarkdownRenderer;
use ratatui_markdown::theme::RichTextTheme;

// 创建渲染器，指定最大内容宽度
let renderer = MarkdownRenderer::new(80);

// 将 markdown 文本解析为块
let blocks = renderer.parse("# Hello\n\nThis is **bold** and *italic* text.");

// 将块渲染为 ratatui::text::Line<'static>
let lines = renderer.render(&blocks, &my_theme);
```

### 浏览树视图

```rust
use ratatui_markdown::tree::CollapsibleTree;

// 将 JSON 解析为可折叠树
let json_str = r#"{"name": "project", "deps": {"ratatui": "0.29", "serde": "1.0"}}"#;
let mut tree = CollapsibleTree::from_json_str(json_str).unwrap();

// 渲染树行
let lines = tree.render_lines(80, &my_theme);

// 获取可聚焦项用于导航
let items = tree.build_focusable_items();

// 切换节点
tree.toggle("deps/serde");
```

### 使用 MarkdownPreview 组件

`MarkdownPreview` 组件将所有功能整合到一个可滚动视图中：

```rust
use ratatui_markdown::preview::MarkdownPreview;
use ratatui_markdown::theme::RichTextTheme;

let mut preview = MarkdownPreview::new()
    .with_left_padding(true);

// 设置 markdown 内容
preview.set_content("# Welcome\n\n- Item one\n- Item two\n\n```rust\nlet x = 42;\n```");

// 设置可折叠树（可选）
let tree = CollapsibleTree::from_json_str(r#"{"config": {"port": 8080}}"#).unwrap();
preview.set_tree(Some(tree));

// 处理键盘输入
preview.scroll_up();
preview.scroll_down();
preview.page_up(10);
preview.page_down(10);
preview.toggle_tree_node(); // Enter 键

// 在 ratatui 绘制循环中渲染
fn draw(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}
```

## 实现主题

本库通过 trait 查找所有颜色：

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

修改 `generation()` 的返回值可以使预览组件的缓存失效并强制重新渲染（例如，用户在运行时切换主题时）。

## 下一步

- [Markdown 模块](markdown.md) — 完整的 Markdown 解析和渲染 API
- [滚动系统](scroll.md) — 理解混合滚动架构
- [树视图](tree.md) — JSON/TOML 树渲染与交互
- [预览组件](preview.md) — 高层统一组件
- [主题](theme.md) — 完整的主题定制指南
