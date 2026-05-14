<div align="center">
  <img src="../../../examples/logo.webp" alt="ratatui-markdown logo" width="200" />
</div>

# ratatui-markdown

> 为 ratatui 提供 Markdown 渲染、可折叠 JSON/TOML 树以及丰富滚动组件的 Rust 库。
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

ratatui-markdown 是一个功能丰富的终端用户界面渲染库，基于 [ratatui](https://github.com/ratatui/ratatui) 构建。它提供四个主要功能模块，可以独立使用，也可以通过 `MarkdownPreview` 组件组合使用。

## 核心模块

### Markdown 渲染

解析 Markdown 文本并渲染为带样式的终端输出：

- **标题**: H1 (`#`), H2 (`##`), H3 (`###`)
- **段落**: 支持 CJK 字符宽度感知的自动换行
- **内联格式**: `**粗体**`, `*斜体*`, `***粗体+斜体***`, `` `行内代码` ``
- **围栏代码块**: 可选语言标签（mermaid 块会跳过）
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

### MarkdownPreview 组件

整合一切的高层组件：

- 将 markdown 内容、树视图和操作项渲染在单个可滚动布局中
- **缓存**: 仅在内容、宽度或主题代次变化时重建输出
- **TOML 前言剥离**: 自动剥离 `+++` 分隔的 TOML 前言
- **操作项**: 可通过键盘选择的带 action ID 的标签项
- 所有导航委托给 `HybridScrollView`

## 快速开始

```toml
[dependencies]
ratatui-markdown = "0.1"
```

```rust
use ratatui_markdown::preview::MarkdownPreview;

let mut preview = MarkdownPreview::new();
preview.set_content("# 你好，世界！\n\n这是一个段落。");
// 在 ratatui 应用循环中渲染并处理输入
```

## 功能标志

默认情况下启用所有功能。可以通过禁用默认功能来选择性启用：

```toml
[dependencies]
ratatui-markdown = { version = "0.1", default-features = false, features = ["markdown"] }
```

| 功能       | 依赖             | 描述                               |
|------------|------------------|-----------------------------------|
| `markdown` | —                | Markdown 解析器和渲染器             |
| `scroll`   | —                | HybridScrollView、可滚动列表、滚动条  |
| `tree`     | `scroll`, `serde_json`, `toml` | 可折叠 JSON/TOML 树    |
| `preview`  | `markdown`, `scroll`, `tree` | MarkdownPreview 统一组件 |

## 项目结构

```
ratatui-markdown/
  src/
   ├── lib.rs                  # 库入口：按功能开关组织模块
   ├── theme.rs                # RichTextTheme trait、Generation 令牌
   ├── constants/
   │   ├── mod.rs              # 重导出
   │   ├── box_chars.rs        # 制表符常量
   │   └── list_prefix.rs      # 树连接符、箭头、标记
   ├── markdown/
   │   ├── mod.rs              # MarkdownRenderer 结构体
   │   ├── parser.rs           # 块级 Markdown 解析器
   │   ├── types.rs            # MarkdownBlock 枚举、TextToken
   │   ├── render.rs           # 块级渲染器（含表格）
   │   ├── inline.rs           # 内联格式化解析器
   │   └── text.rs             # CJK 感知的文本换行
   ├── scroll/
   │   ├── mod.rs              # 重导出
   │   ├── hybrid_scroll/      # HybridScrollView（核心组件）
   │   ├── scrollable_list.rs  # 泛型 ScrollableList<T>
   │   ├── scrollable_panel.rs # 简单滚动辅助
   │   ├── focusable_list.rs   # FocusableItemList 渲染器
   │   ├── follow_scroll.rs    # FollowScrollState
   │   └── scrollbar.rs        # ArrowScrollbar 组件
   ├── tree/
   │   ├── mod.rs              # 重导出
   │   ├── tree_lines.rs       # 树行构建
   │   └── collapsible_tree/   # CollapsibleTree + 节点操作 + 渲染
   └── preview/
       └── mod.rs              # MarkdownPreview 统一组件
```

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
