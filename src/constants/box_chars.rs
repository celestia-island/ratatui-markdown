// ============================================================================
// 制表符（Box-Drawing Characters）常量映射
// ============================================================================
//
// Basic box-drawing characters.
// TUI 专用组合前缀（如 TL_BODY = "│  "）在本文件中额外定义。
//
// 命名规则：
//   BD_  = Box Drawing 前缀
//   方向 = 上(T) 下(B) 左(L) 右(R)
//   H    = 水平（Horizontal）
//   V    = 竖直（Vertical）
//   DR/DL/UR/UL = 对角方向（Down-Right / Down-Left / Up-Right / Up-Left）
// ============================================================================

// ─────────────────────────────────────────────────────────────────────
// Basic box-drawing characters
// ─────────────────────────────────────────────────────────────────────

pub const BD_DR: &str = "\u{250C}";
pub const BD_DL: &str = "\u{2510}";
pub const BD_UR: &str = "\u{2514}";
pub const BD_UL: &str = "\u{2518}";
pub const BD_H: &str = "\u{2500}";
pub const BD_V: &str = "\u{2502}";
pub const BD_T_LEFT: &str = "\u{251C}";
pub const BD_RND_TL: &str = "\u{256D}";
pub const BD_RND_BL: &str = "\u{2570}";
pub const TL_HEADER: &str = "\u{250C}";
pub const TL_SEP_CHAR: &str = "\u{251C}";
pub const ARROW_UP: &str = "\u{2191}";
pub const ARROW_DOWN: &str = "\u{2193}";
pub const ARROW_SWAP: &str = "\u{21C4}";
pub const CHECK: &str = "\u{2713}";
pub const CROSS: &str = "\u{2717}";
pub const DOT_FILLED: &str = "\u{25CF}";
pub const HLINE: &str = "\u{2500}";

// ─────────────────────────────────────────────────────────────────────
// TUI 专用：带空格的行前缀组合（ratatui rendering 使用）
// ─────────────────────────────────────────────────────────────────────

/// │  正文行前缀：竖线延续 + 双空格
pub const TL_BODY: &str = "\u{2502}  ";
/// └  末项闭合行前缀：下右角 + 单空格
pub const TL_CLOSE: &str = "\u{2514} ";
/// │ ╭ MCP 块首行前缀：竖线 + 空格 + 圆角左上
pub const TL_MCP_OPEN: &str = "\u{2502} \u{256D}";
/// │ │ MCP 块内行前缀：竖线 + 空格 + 竖线
pub const TL_MCP_INNER: &str = "\u{2502} \u{2502}";
/// │ ╰ MCP 块末行前缀：竖线 + 空格 + 圆角左下
pub const TL_MCP_CLOSE: &str = "\u{2502} \u{256F}";

// ─────────────────────────────────────────────────────────────────────
// Heavy & supplementary box-drawing characters
// ─────────────────────────────────────────────────────────────────────

pub const BD_H_HEAVY: &str = "\u{2501}";
pub const BD_V_HEAVY: &str = "\u{2503}";
pub const BD_DR_HEAVY: &str = "\u{250F}";
pub const BD_DL_HEAVY: &str = "\u{2513}";
pub const BD_UR_HEAVY: &str = "\u{2517}";
pub const BD_UL_HEAVY: &str = "\u{251B}";
pub const BD_T_RIGHT: &str = "\u{2524}";
pub const BD_T_UP: &str = "\u{252C}";
pub const BD_T_DOWN: &str = "\u{2534}";
pub const BD_T_LEFT_HEAVY: &str = "\u{2523}";
pub const BD_T_RIGHT_HEAVY: &str = "\u{2527}";
pub const BD_T_UP_HEAVY: &str = "\u{252F}";
pub const BD_T_DOWN_HEAVY: &str = "\u{253B}";
pub const BD_CROSS: &str = "\u{253C}";
pub const BD_CROSS_HEAVY: &str = "\u{253E}";
pub const BD_RND_TR: &str = "\u{256E}";
pub const BD_RND_BR: &str = "\u{2570}";
