// ============================================================================
// 常量：光标符号
// ============================================================================

/// 选中/聚焦行的光标符号
pub const CURSOR_SELECTED: &str = ">";
/// 未选中行的光标占位（与 CURSOR_SELECTED 等宽）
pub const CURSOR_BLANK: &str = " ";

// ============================================================================
// 常量：Box-drawing 单字符
// ============================================================================

/// ─ 水平线（light horizontal）
pub const HLINE: &str = "─";
/// │ 竖线（light vertical）
pub const VLINE: &str = "│";
/// ┌ 左上角（light down and right）
pub const CORNER_TL: &str = "┌";
/// ┐ 右上角（light down and left）
pub const CORNER_TR: &str = "┐";
/// └ 左下角（light up and right）
pub const CORNER_BL: &str = "└";
/// ┘ 右下角（light up and left）
pub const CORNER_BR: &str = "┘";
/// ┬ 上中（light down and horizontal）
pub const TOP_MID: &str = "┬";
/// ┴ 下中（light up and horizontal）
pub const BOTTOM_MID: &str = "┴";
/// ├ 左中（light right and horizontal）
pub const MID_LEFT: &str = "├";
/// ┤ 右中（light left and horizontal）
pub const MID_RIGHT: &str = "┤";
/// ┼ 十字交叉（light）
pub const CROSS_MID: &str = "┼";

// Rounded corners
/// ╭ 圆角左上
pub const ROUNDED_TL: &str = "╭";
/// ╰ 圆角左下
pub const ROUNDED_BL: &str = "╰";

// Heavy box-drawing
/// ━ 粗水平线
pub const HLINE_HEAVY: &str = "━";
/// ┃ 粗竖线
pub const VLINE_HEAVY: &str = "┃";
/// ┏ 粗左上角
pub const HEAVY_TL: &str = "┏";
/// ┓ 粗右上角
pub const HEAVY_TR: &str = "┓";
/// ┗ 粗左下角
pub const HEAVY_BL: &str = "┗";
/// ┛ 粗右下角
pub const HEAVY_BR: &str = "┛";

// ============================================================================
// 常量：树状连接符（组合）
// ============================================================================

/// ├─ 非末项子节点
pub const BRANCH_MID: &str = "├─";
/// └─ 末项子节点 / 组头（上右角）
pub const BRANCH_END: &str = "└─";
/// ┌─ 首项子节点
pub const BRANCH_FIRST: &str = "┌─";

/// ┌─ 带尾随空格（首项，无向上突起）
pub const BRANCH_FIRST_SP: &str = "┌─ ";

/// ├─ 带尾随空格（与 BRANCH_END_SP / BRANCH_VERT_PAD 等宽）
pub const BRANCH_MID_SP: &str = "├─ ";
/// └─ 带尾随空格（与 BRANCH_MID_SP / BRANCH_VERT_PAD 等宽）
pub const BRANCH_END_SP: &str = "└─ ";
/// │  纵向延续线 + 2空格（与 BRANCH_MID_SP / BRANCH_END_SP 等宽）
pub const BRANCH_VERT_PAD: &str = "│  ";
/// │ （带尾随空格）用于深层缩进 "│ ".repeat(depth)
pub const INDENT_VERT: &str = "│ ";
/// 子项竖线延续：1空格缩进 + │ + 2空格（与 tree_child_prefix 等宽）
pub const CHILD_VERT_PAD: &str = " │  ";

// ============================================================================
// 常量：状态/指示符号
// ============================================================================

/// ✓ 勾号
pub const CHECK: &str = "✓";
/// ✗ 叉号
pub const CROSS: &str = "✗";
/// ● 实心圆
pub const DOT_FILLED: &str = "●";
/// ○ 空心圆
pub const DOT_EMPTY: &str = "○";
/// ◉ 空心带点（用于闪烁动画）
pub const DOT_ALT: &str = "◉";

// ============================================================================
// Direction arrows
// ============================================================================

pub const ARROW_DOWN: &str = "\u{2193}";
pub const ARROW_SWAP: &str = "\u{21C4}";
pub const ARROW_UP: &str = "\u{2191}";

/// ← 左箭头
pub const ARROW_LEFT: &str = "←";
/// → 右箭头
pub const ARROW_RIGHT: &str = "→";
/// ▲ 实心上三角
pub const TRIANGLE_UP: &str = "▲";
/// ▼ 实心下三角
pub const TRIANGLE_DOWN: &str = "▼";

// ============================================================================
// 常量：填充/进度条
// ============================================================================

/// █ 全方块
pub const BLOCK_FULL: &str = "█";

// ============================================================================
// 树状列表节点类型
// ============================================================================

/// 树状列表中各节点的类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeNodeKind {
    /// └─ 组头部（所有组头统一使用上右角）
    Header,
    /// ├─ 子项：非末项
    ChildMiddle,
    /// └─ 子项：末项
    ChildLast,
    /// │  纵向延续线（子行之间的连接，与 Mid/End 等宽）
    Continuation,
}

/// 统一树状连接符（带尾随空格），用于分组列表场景
///
/// 效果（带光标的分组列表）:
/// \> └─ Group A (3)
/// \> ├─ item 1
/// \> ├─ item 2
/// \> └─ item 3
/// \> └─ Group B (2)
/// \> ├─ item 4
/// \> └─ item 5
///
/// 效果（对话面板，光标始终在组头）:
/// \> └─ 用户 [12:00]
/// \> ├─ 内容行
/// \> └─ 最后一行
/// \> └─ 助手 [12:01]
/// \> ├─ 回复内容
/// \> └─ 完成
pub fn tree_connector(kind: TreeNodeKind) -> &'static str {
    match kind {
        TreeNodeKind::Header => BRANCH_END_SP,
        TreeNodeKind::ChildMiddle => BRANCH_MID_SP,
        TreeNodeKind::ChildLast => BRANCH_END_SP,
        TreeNodeKind::Continuation => BRANCH_VERT_PAD,
    }
}

/// 组内子项的前缀：1个空格 + 连接符 + 尾随空格
pub fn tree_child_prefix(kind: TreeNodeKind) -> &'static str {
    match kind {
        TreeNodeKind::ChildMiddle => " ├─ ",
        TreeNodeKind::ChildLast => " └─ ",
        TreeNodeKind::Header => BRANCH_END_SP,
        TreeNodeKind::Continuation => CHILD_VERT_PAD,
    }
}

use ratatui::text::{Line, Span};

/// 构建一行树状详情子行：`<cursor_blank> <indent><connector><content...>`
///
/// 用于设备面板等需要多层缩进 + 分支连接符 + 文本内容的场景。
/// indent 通常为 `INDENT_VERT.repeat(level - 1)` + " "。
pub fn tree_detail_line<'a>(indent: &str, kind: TreeNodeKind, content: Vec<Span<'a>>) -> Line<'a> {
    let mut spans: Vec<Span<'a>> = vec![
        Span::raw(CURSOR_BLANK.to_string()),
        Span::raw(" "),
        Span::raw(indent.to_string()),
        Span::raw(tree_connector(kind).to_string()),
    ];
    spans.extend(content);
    Line::from(spans)
}

/// 构建一行树状头部行：`<cursor> <indent><content...>`
///
/// 头部行没有分支连接符，直接跟在缩进后面。
pub fn tree_header_line<'a>(
    cursor: &'static str,
    indent: &str,
    content: Vec<Span<'a>>,
) -> Line<'a> {
    let mut spans: Vec<Span<'a>> = vec![
        Span::raw(cursor.to_string()),
        Span::raw(" "),
        Span::raw(indent.to_string()),
    ];
    spans.extend(content);
    Line::from(spans)
}
