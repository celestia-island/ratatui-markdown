<div align="center">
  <img src="../../../examples/logo.webp" alt="ratatui-markdown logo" width="200" />
</div>

# ratatui-markdown

> 为 ratatui 提供 Markdown 渲染、Mermaid 图表、语法高亮、可折叠 JSON/TOML 树以及丰富滚动组件的 Rust 库。
>
> **构建基础**: [ratatui](https://github.com/ratatui/ratatui) 0.29 + 纯 Rust
>
> **最低 Rust 版本**: 1.74

<div align="center">
  <p>
    <a href="../../README.md">English</a> |
    <a href="../zhs/index.md">简体中文</a> |
    <a href="../zht/index.md">繁體中文</a> |
    <a href="../ja/index.md">日本語</a> |
    <a href="../ko/index.md">한국어</a> |
    <a href="../fr/index.md">Français</a> |
    <a href="../es/index.md">Español</a> |
    <a href="../ru/index.md">Русский</a> |
    <a href="../ar/index.md">العربية</a>
  </p>
</div>

## 什么是 ratatui-markdown？

ratatui-markdown 是一个功能丰富的终端用户界面渲染库，基于 [ratatui](https://github.com/ratatui/ratatui) 构建。它提供多个功能模块，可以独立使用，也可以通过 `MarkdownPreview` / `MarkdownViewer` 组件组合使用。

## 核心模块

### Markdown 渲染

解析 Markdown 文本并渲染为带样式的终端输出：

- **标题**: H1 (`#`), H2 (`##`), H3 (`###`)
- **段落**: 支持 CJK 字符宽度感知的自动换行
- **内联格式**: `**粗体**`, `*斜体*`, `***粗体+斜体***`, `` `行内代码` ``
- **围栏代码块**: 可选语言标签（mermaid 块渲染为图表）
- **引用块** (`>`)
- **无序列表** (`-`, `*`, `+`) 和有序列表 (`1.`, `2.`)
- **水平分割线** (`---`, `***`, `___`)
- **表格**: 按比例分配列宽，支持单元格换行

### 可折叠树视图

解析并交互式浏览结构化数据：

- 解析 **JSON** 和 **TOML** 为可折叠树
- **展开 / 折叠** 单个节点、全部展开、全部折叠、按深度展开
- **样式键名**: JSON 模式（带引号的键名 + `:`）或 TOML 模式（裸键名 + `=`）
- **键盘导航**: 基于光标的选择和切换
- **值类型着色**: 字符串、数字、布尔值、null 各有独立的主题颜色

### 混合滚动系统

智能滚动，同时处理自由浏览和项目导航：

- **自由滚动模式**: 自由浏览内容
- **交互模式**: 当可聚焦项目进入视口中心时自动激活
- **光标导航**: 通过键盘在可聚焦项目间移动
- **光标指示器**: 交互行显示 `> ` 前缀
- **滚动条**: 基于箭头的叠加显示
- **翻页**: 支持 `page_up` / `page_down`

### Mermaid 图表

在终端中直接渲染 Mermaid 图表：

- **时序图**、**饼图**、**甘特图**、**状态图**
- 通过 ` ```mermaid ` 代码块触发
- 功能标志：`mermaid`

### 语法高亮

基于 tree-sitter 的代码块语法高亮：

- 按语言选择的功能标志（`highlight-lang-rust`、`highlight-lang-python` 等）
- `highlight-lang-all` 打包所有支持的语言
- 可通过 `HighlightHooks` 自定义

### MarkdownPreview / MarkdownViewer 组件

整合一切的高层组件：

- 将 markdown 内容、树视图和操作项渲染在单个可滚动布局中
- **缓存**: 仅在内容、宽度或主题代次变化时重建输出
- **TOML 前言剥离**: 自动剥离 `+++` 分隔的 TOML 前言
- **操作项**: 可通过键盘选择的带 action ID 的标签项
- 所有导航委托给 `HybridScrollView`

## 快速开始

```toml
[dependencies]
ratatui-markdown = "0.2"
```

### 示例

| 示例                 | 描述                               | 所需功能标志                   |
|----------------------|------------------------------------|-------------------------------|
| `basic`              | 基础 Markdown 渲染                 | —                             |
| `code`               | 语法高亮代码块                     | `highlight-lang-all`          |
| `custom_code_block`  | 自定义代码块渲染钩子               | —                             |
| `image`              | 图片嵌入和缩放                     | `image`                       |
| `mermaid`            | Mermaid 图表渲染                   | `mermaid`                     |
| `tree_list`          | 可折叠 JSON/TOML 树视图            | —                             |

```bash
cargo run --example basic
cargo run --example code --features highlight-lang-all
cargo run --example image --features image
cargo run --example mermaid --features mermaid
cargo run --example tree_list
```

## 功能标志

默认情况下启用所有功能。可以通过禁用默认功能来选择性启用：

```toml
[dependencies]
ratatui-markdown = { version = "0.2", default-features = false, features = ["markdown"] }
```

| 功能                | 依赖                               | 描述                               |
|---------------------|------------------------------------|-----------------------------------|
| `markdown`          | —                                  | Markdown 解析器和渲染器             |
| `image`             | —                                  | 通过 `ImageResolver` 解析图片       |
| `scroll`            | —                                  | HybridScrollView、可滚动列表、滚动条  |
| `tree`              | `scroll`, `serde_json`, `toml`     | 可折叠 JSON/TOML 树                 |
| `preview`           | `markdown`, `scroll`, `tree`       | MarkdownPreview 统一组件            |
| `mermaid`           | `markdown`                         | Mermaid 图表渲染                    |
| `viewer`            | `markdown`, `scroll`               | MarkdownViewer 组件                 |
| `highlight`         | —                                  | 基于 tree-sitter 的语法高亮          |
| `highlight-lang-*`  | `highlight`                        | 单语言语法                          |
| `highlight-lang-all`| `highlight`                        | 所有内置语言语法                     |

## 文档

| 指南 | 描述 |
|------|------|
| [快速开始](getting-started.md) | 安装和首次渲染 |
| [Markdown](markdown.md) | 解析和渲染 Markdown |
| [滚动系统](scroll.md) | 混合滚动、导航、滚动条 |
| [树视图](tree.md) | JSON/TOML 树、展开/折叠 |
| [预览组件](preview.md) | 使用 MarkdownPreview 整合一切 |
| [主题](theme.md) | 实现 RichTextTheme |
| [贡献指南](contributing.md) | 开发和贡献指南 |

## 许可证

双重许可 MIT OR Apache-2.0。
