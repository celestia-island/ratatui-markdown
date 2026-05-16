use ratatui::style::Color;

pub const PRIMARY_TEXT_DARK: Color = Color::Rgb(51, 51, 51);
pub const PRIMARY_TEXT_LIGHT: Color = Color::Rgb(15, 23, 42);
pub const LINE_COLOR_DARK: Color = Color::Rgb(47, 59, 77);
pub const LINE_COLOR_LIGHT: Color = Color::Rgb(100, 116, 139);
pub const BORDER_COLOR_DARK: Color = Color::Rgb(123, 136, 168);
pub const BORDER_COLOR_LIGHT: Color = Color::Rgb(148, 163, 184);
pub const EDGE_LABEL_BG_DARK: Color = Color::Rgb(248, 250, 252);
pub const EDGE_LABEL_BG_LIGHT: Color = Color::Rgb(255, 255, 255);
pub const CLUSTER_BG_DARK: Color = Color::Rgb(255, 255, 222);
pub const CLUSTER_BG_LIGHT: Color = Color::Rgb(241, 245, 249);
pub const CLUSTER_BORDER_DARK: Color = Color::Rgb(170, 170, 51);
pub const CLUSTER_BORDER_LIGHT: Color = Color::Rgb(203, 213, 225);
pub const BACKGROUND_DARK: Color = Color::Rgb(255, 255, 255);
pub const BACKGROUND_LIGHT: Color = Color::Rgb(255, 255, 255);
pub const ACTOR_FILL_DARK: Color = Color::Rgb(234, 234, 234);
pub const ACTOR_FILL_LIGHT: Color = Color::Rgb(248, 250, 252);
pub const ACTOR_BORDER_DARK: Color = Color::Rgb(102, 102, 102);
pub const ACTOR_BORDER_LIGHT: Color = Color::Rgb(148, 163, 184);
pub const ACTOR_LINE_DARK: Color = Color::Rgb(153, 153, 153);
pub const ACTOR_LINE_LIGHT: Color = Color::Rgb(100, 116, 139);
pub const NOTE_FILL_DARK: Color = Color::Rgb(255, 245, 173);
pub const NOTE_FILL_LIGHT: Color = Color::Rgb(255, 247, 237);
pub const NOTE_BORDER_DARK: Color = Color::Rgb(170, 170, 51);
pub const NOTE_BORDER_LIGHT: Color = Color::Rgb(253, 186, 116);
pub const ACTIVATION_FILL_DARK: Color = Color::Rgb(244, 244, 244);
pub const ACTIVATION_FILL_LIGHT: Color = Color::Rgb(226, 232, 240);
pub const ACTIVATION_BORDER_DARK: Color = Color::Rgb(102, 102, 102);
pub const ACTIVATION_BORDER_LIGHT: Color = Color::Rgb(148, 163, 184);
pub const TEXT_COLOR_DARK: Color = Color::Rgb(51, 51, 51);
pub const TEXT_COLOR_LIGHT: Color = Color::Rgb(15, 23, 42);
pub const PIE_STROKE_DARK: Color = Color::Rgb(0, 0, 0);
pub const PIE_STROKE_LIGHT: Color = Color::Rgb(51, 65, 85);
pub const PIE_OUTER_STROKE_DARK: Color = Color::Rgb(0, 0, 0);
pub const PIE_OUTER_STROKE_LIGHT: Color = Color::Rgb(203, 213, 225);

pub const PRIMARY_DARK: Color = Color::Rgb(236, 236, 255);
pub const SECONDARY_DARK: Color = Color::Rgb(255, 255, 222);
pub const TERTIARY_DARK: Color = Color::Rgb(236, 236, 255);
pub const PRIMARY_LIGHT: Color = Color::Rgb(59, 130, 246);
pub const SECONDARY_LIGHT: Color = Color::Rgb(16, 185, 129);
pub const TERTIARY_LIGHT: Color = Color::Rgb(245, 158, 11);

pub const PIE_COLORS_DARK: [Color; 12] = [
    Color::Rgb(236, 236, 255),
    Color::Rgb(255, 255, 222),
    Color::Rgb(236, 236, 255),
    Color::Rgb(214, 214, 245),
    Color::Rgb(233, 233, 200),
    Color::Rgb(214, 214, 245),
    Color::Rgb(139, 139, 195),
    Color::Rgb(59, 59, 139),
    Color::Rgb(139, 59, 139),
    Color::Rgb(118, 118, 178),
    Color::Rgb(39, 39, 118),
    Color::Rgb(139, 39, 118),
];

pub const PIE_COLORS_LIGHT: [Color; 12] = [
    Color::Rgb(59, 130, 246),
    Color::Rgb(16, 185, 129),
    Color::Rgb(245, 158, 11),
    Color::Rgb(99, 102, 241),
    Color::Rgb(236, 72, 153),
    Color::Rgb(20, 184, 166),
    Color::Rgb(168, 85, 247),
    Color::Rgb(234, 179, 8),
    Color::Rgb(6, 182, 212),
    Color::Rgb(249, 115, 22),
    Color::Rgb(132, 204, 22),
    Color::Rgb(217, 70, 239),
];

pub const GIT_COLORS_HSL: [&str; 8] = [
    "hsl(240, 100%, 46.3%)",
    "hsl(60, 100%, 43.5%)",
    "hsl(80, 100%, 46.3%)",
    "hsl(210, 100%, 46.3%)",
    "hsl(180, 100%, 46.3%)",
    "hsl(150, 100%, 46.3%)",
    "hsl(300, 100%, 46.3%)",
    "hsl(0, 100%, 46.3%)",
];

pub const GIT_COMMIT_LABEL_COLOR: Color = Color::Rgb(0, 0, 33);
pub const GIT_COMMIT_LABEL_BG: Color = Color::Rgb(255, 255, 222);
pub const GIT_TAG_LABEL_COLOR: Color = Color::Rgb(19, 19, 0);
pub const GIT_TAG_LABEL_BG: Color = Color::Rgb(236, 236, 255);

#[derive(Debug, Clone)]
pub struct MermaidTheme {
    pub background: Color,
    pub primary_color: Color,
    pub primary_text_color: Color,
    pub primary_border_color: Color,
    pub line_color: Color,
    pub secondary_color: Color,
    pub tertiary_color: Color,
    pub edge_label_background: Color,
    pub cluster_background: Color,
    pub cluster_border: Color,
    pub text_color: Color,
    pub actor_fill: Color,
    pub actor_border: Color,
    pub actor_line: Color,
    pub note_fill: Color,
    pub note_border: Color,
    pub activation_fill: Color,
    pub activation_border: Color,
    pub pie_colors: [Color; 12],
    pub pie_stroke_color: Color,
    pub pie_outer_stroke_color: Color,
    pub git_commit_label_color: Color,
    pub git_commit_label_bg: Color,
    pub git_tag_label_color: Color,
    pub git_tag_label_bg: Color,
}

impl Default for MermaidTheme {
    fn default() -> Self {
        Self::dark_bg()
    }
}

impl MermaidTheme {
    pub fn dark_bg() -> Self {
        Self {
            background: BACKGROUND_DARK,
            primary_color: PRIMARY_DARK,
            primary_text_color: PRIMARY_TEXT_DARK,
            primary_border_color: BORDER_COLOR_DARK,
            line_color: LINE_COLOR_DARK,
            secondary_color: SECONDARY_DARK,
            tertiary_color: TERTIARY_DARK,
            edge_label_background: EDGE_LABEL_BG_DARK,
            cluster_background: CLUSTER_BG_DARK,
            cluster_border: CLUSTER_BORDER_DARK,
            text_color: TEXT_COLOR_DARK,
            actor_fill: ACTOR_FILL_DARK,
            actor_border: ACTOR_BORDER_DARK,
            actor_line: ACTOR_LINE_DARK,
            note_fill: NOTE_FILL_DARK,
            note_border: NOTE_BORDER_DARK,
            activation_fill: ACTIVATION_FILL_DARK,
            activation_border: ACTIVATION_BORDER_DARK,
            pie_colors: PIE_COLORS_DARK,
            pie_stroke_color: PIE_STROKE_DARK,
            pie_outer_stroke_color: PIE_OUTER_STROKE_DARK,
            git_commit_label_color: GIT_COMMIT_LABEL_COLOR,
            git_commit_label_bg: GIT_COMMIT_LABEL_BG,
            git_tag_label_color: GIT_TAG_LABEL_COLOR,
            git_tag_label_bg: GIT_TAG_LABEL_BG,
        }
    }

    pub fn light_bg() -> Self {
        Self {
            background: BACKGROUND_LIGHT,
            primary_color: PRIMARY_LIGHT,
            primary_text_color: Color::White,
            primary_border_color: Color::Rgb(29, 78, 216),
            line_color: LINE_COLOR_LIGHT,
            secondary_color: SECONDARY_LIGHT,
            tertiary_color: TERTIARY_LIGHT,
            edge_label_background: EDGE_LABEL_BG_LIGHT,
            cluster_background: CLUSTER_BG_LIGHT,
            cluster_border: CLUSTER_BORDER_LIGHT,
            text_color: TEXT_COLOR_LIGHT,
            actor_fill: ACTOR_FILL_LIGHT,
            actor_border: ACTOR_BORDER_LIGHT,
            actor_line: ACTOR_LINE_LIGHT,
            note_fill: NOTE_FILL_LIGHT,
            note_border: NOTE_BORDER_LIGHT,
            activation_fill: ACTIVATION_FILL_LIGHT,
            activation_border: ACTIVATION_BORDER_LIGHT,
            pie_colors: PIE_COLORS_LIGHT,
            pie_stroke_color: PIE_STROKE_LIGHT,
            pie_outer_stroke_color: PIE_OUTER_STROKE_LIGHT,
            git_commit_label_color: GIT_COMMIT_LABEL_COLOR,
            git_commit_label_bg: GIT_COMMIT_LABEL_BG,
            git_tag_label_color: GIT_TAG_LABEL_COLOR,
            git_tag_label_bg: GIT_TAG_LABEL_BG,
        }
    }

    pub fn for_background(bg: Color) -> Self {
        if luminance(bg) > 0.5 {
            Self::light_bg()
        } else {
            Self::dark_bg()
        }
    }
}

pub fn luminance(c: Color) -> f64 {
    match c {
        Color::Rgb(r, g, b) => (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64) / 255.0,
        Color::Black => 0.0,
        Color::White => 1.0,
        _ => 0.0,
    }
}

pub fn color_to_hex(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("{r:02X}{g:02X}{b:02X}"),
        Color::Black => "000000".to_string(),
        Color::White => "FFFFFF".to_string(),
        _ => "000000".to_string(),
    }
}
