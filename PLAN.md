# ratatui-markdown — 项目状态与计划 (PLAN)

> 刷新于 2026-07-14。ratatui Markdown widget，嵌于 scriptum。

## 1. 项目概述

- **名称**：`ratatui-markdown`
- **简介**：在 ratatui 渲染面板中显示 Markdown 的 widget —— 支持 CommonMark、表格、代码块高亮、链接脚注、嵌套列表。
- **远程仓库**：https://github.com/celestia-island/ratatui-markdown.git
- **技术栈**：Rust / ratatui / pulldown-cmark / syntect（可选高亮）
- **类别**：library（widget）

## 2. 当前状态

- **当前分支**：`dev`
- **工作区**：有未提交改动（2 项）
- **最近提交时间**：2026-07-12
- **最近提交**：`🔧 Pin script recipes to the resolved Git Bash to survive WSL shadowing.`
- **本地领先 `origin/dev`**：0

## 3. 未提交改动

```
 M src/tree/collapsible_tree/mod.rs
 M src/tree/collapsible_tree/node_ops.rs
```

## 4. 近期进展

- `🔧 Pin script recipes to the resolved Git Bash to survive WSL shadowing.`
- `🔧 Switch the justfile to Git Bash and fetch devtools recipes on demand.`
- `♻️ Standardize windows-shell to pwsh.exe across celestia repos.`
- `🐛 Replace shebang recipes with [script(...)] to fix the Windows cygpath error.`
- `📝 Add FUNDING.yml for GitHub Sponsors.`

## 5. 后续计划

1. **collapsible tree 修复**：`node_ops.rs` 的未提交改动（修空节点 / 折叠状态持久化）随本轮 PLAN.md 一起提交到 dev。
2. **GFM 表格列宽自适应**：当前列宽按最长单元格计算，大表格时容易截断；改用软换行 + 横向滚动。
3. **代码块语法高亮**：可选 `syntect` feature，与 scriptum 的 TUI 主题协调。
4. **发布到 crates.io**：与 entelecheia 仓脱钩，单独发布以便社区复用。

## 6. 跨仓依赖

- 嵌于 scriptum，作为 TUI 渲染层一部分。

---

## 既有详细计划（存档）

公共 API 与示例在 `examples/` 与 `docs/en/`；本文件只承载"当前态 → 后续计划"两部分。
