use ratatui::style::Color;

#[cfg(feature = "mermaid")]
use crate::mermaid::theme::MermaidTheme;

#[derive(Debug, Clone, Copy)]
pub struct CodeColors {
    pub comment: Color,
    pub keyword: Color,
    pub string: Color,
    pub string_escape: Color,
    pub number: Color,
    pub constant: Color,
    pub function: Color,
    pub r#type: Color,
    pub variable: Color,
    pub property: Color,
    pub operator: Color,
    pub punctuation: Color,
    pub attribute: Color,
    pub tag: Color,
    pub label: Color,
    pub error: Color,
}

impl Default for CodeColors {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl CodeColors {
    pub const DEFAULT: Self = Self {
        comment: Color::DarkGray,
        keyword: Color::Magenta,
        string: Color::Green,
        string_escape: Color::LightGreen,
        number: Color::Yellow,
        constant: Color::Yellow,
        function: Color::Cyan,
        r#type: Color::LightCyan,
        variable: Color::White,
        property: Color::LightBlue,
        operator: Color::LightMagenta,
        punctuation: Color::DarkGray,
        attribute: Color::LightYellow,
        tag: Color::Cyan,
        label: Color::LightRed,
        error: Color::Red,
    };

    pub fn builder() -> CodeColorsBuilder {
        CodeColorsBuilder(Self::default())
    }
}

pub struct CodeColorsBuilder(CodeColors);

impl CodeColorsBuilder {
    pub fn comment(mut self, c: Color) -> Self {
        self.0.comment = c;
        self
    }
    pub fn keyword(mut self, c: Color) -> Self {
        self.0.keyword = c;
        self
    }
    pub fn string(mut self, c: Color) -> Self {
        self.0.string = c;
        self
    }
    pub fn string_escape(mut self, c: Color) -> Self {
        self.0.string_escape = c;
        self
    }
    pub fn number(mut self, c: Color) -> Self {
        self.0.number = c;
        self
    }
    pub fn constant(mut self, c: Color) -> Self {
        self.0.constant = c;
        self
    }
    pub fn function(mut self, c: Color) -> Self {
        self.0.function = c;
        self
    }
    pub fn r#type(mut self, c: Color) -> Self {
        self.0.r#type = c;
        self
    }
    pub fn variable(mut self, c: Color) -> Self {
        self.0.variable = c;
        self
    }
    pub fn property(mut self, c: Color) -> Self {
        self.0.property = c;
        self
    }
    pub fn operator(mut self, c: Color) -> Self {
        self.0.operator = c;
        self
    }
    pub fn punctuation(mut self, c: Color) -> Self {
        self.0.punctuation = c;
        self
    }
    pub fn attribute(mut self, c: Color) -> Self {
        self.0.attribute = c;
        self
    }
    pub fn tag(mut self, c: Color) -> Self {
        self.0.tag = c;
        self
    }
    pub fn label(mut self, c: Color) -> Self {
        self.0.label = c;
        self
    }
    pub fn error(mut self, c: Color) -> Self {
        self.0.error = c;
        self
    }
    pub fn build(self) -> CodeColors {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Generation(pub u64);

impl Generation {
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

pub trait RichTextTheme {
    fn generation(&self) -> Generation;
    fn get_text_color(&self) -> Color;
    fn get_muted_text_color(&self) -> Color;
    fn get_primary_color(&self) -> Color;
    fn get_popup_selected_background(&self) -> Color;
    fn get_border_color(&self) -> Color;
    fn get_focused_border_color(&self) -> Color;
    fn get_secondary_color(&self) -> Color;
    fn get_info_color(&self) -> Color;
    fn get_json_key_color(&self) -> Color;
    fn get_json_string_color(&self) -> Color;
    fn get_json_number_color(&self) -> Color;
    fn get_json_bool_color(&self) -> Color;
    fn get_json_null_color(&self) -> Color;
    fn get_accent_yellow(&self) -> Color;

    fn get_code_colors(&self) -> CodeColors {
        CodeColors::default()
    }

    fn get_popup_selected_text_color(&self) -> Color {
        Color::White
    }
    fn get_background_color(&self) -> Color {
        Color::Black
    }

    #[cfg(feature = "mermaid")]
    fn get_mermaid_theme(&self) -> MermaidTheme {
        MermaidTheme::for_background(self.get_background_color())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeConfig {
    pub gen: Generation,
    pub text_color: Color,
    pub muted_text_color: Color,
    pub primary_color: Color,
    pub popup_selected_background: Color,
    pub border_color: Color,
    pub focused_border_color: Color,
    pub secondary_color: Color,
    pub info_color: Color,
    pub json_key_color: Color,
    pub json_string_color: Color,
    pub json_number_color: Color,
    pub json_bool_color: Color,
    pub json_null_color: Color,
    pub accent_yellow: Color,
    pub code_colors: CodeColors,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            gen: Generation(1),
            text_color: Color::White,
            muted_text_color: Color::DarkGray,
            primary_color: Color::Cyan,
            popup_selected_background: Color::DarkGray,
            border_color: Color::DarkGray,
            focused_border_color: Color::White,
            secondary_color: Color::Blue,
            info_color: Color::LightBlue,
            json_key_color: Color::LightCyan,
            json_string_color: Color::Green,
            json_number_color: Color::Yellow,
            json_bool_color: Color::Magenta,
            json_null_color: Color::DarkGray,
            accent_yellow: Color::Yellow,
            code_colors: CodeColors::DEFAULT,
        }
    }
}

impl ThemeConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> ThemeBuilder {
        ThemeBuilder {
            config: Self::default(),
        }
    }

    pub fn with_generation(mut self, gen: Generation) -> Self {
        self.gen = gen;
        self
    }

    pub fn with_text_color(mut self, c: Color) -> Self {
        self.text_color = c;
        self
    }

    pub fn with_muted_text_color(mut self, c: Color) -> Self {
        self.muted_text_color = c;
        self
    }

    pub fn with_primary_color(mut self, c: Color) -> Self {
        self.primary_color = c;
        self
    }

    pub fn with_popup_selected_background(mut self, c: Color) -> Self {
        self.popup_selected_background = c;
        self
    }

    pub fn with_border_color(mut self, c: Color) -> Self {
        self.border_color = c;
        self
    }

    pub fn with_focused_border_color(mut self, c: Color) -> Self {
        self.focused_border_color = c;
        self
    }

    pub fn with_secondary_color(mut self, c: Color) -> Self {
        self.secondary_color = c;
        self
    }

    pub fn with_info_color(mut self, c: Color) -> Self {
        self.info_color = c;
        self
    }

    pub fn with_json_key_color(mut self, c: Color) -> Self {
        self.json_key_color = c;
        self
    }

    pub fn with_json_string_color(mut self, c: Color) -> Self {
        self.json_string_color = c;
        self
    }

    pub fn with_json_number_color(mut self, c: Color) -> Self {
        self.json_number_color = c;
        self
    }

    pub fn with_json_bool_color(mut self, c: Color) -> Self {
        self.json_bool_color = c;
        self
    }

    pub fn with_json_null_color(mut self, c: Color) -> Self {
        self.json_null_color = c;
        self
    }

    pub fn with_accent_yellow(mut self, c: Color) -> Self {
        self.accent_yellow = c;
        self
    }

    pub fn with_code_colors(mut self, colors: CodeColors) -> Self {
        self.code_colors = colors;
        self
    }
}

impl RichTextTheme for ThemeConfig {
    fn generation(&self) -> Generation {
        self.gen
    }
    fn get_text_color(&self) -> Color {
        self.text_color
    }
    fn get_muted_text_color(&self) -> Color {
        self.muted_text_color
    }
    fn get_primary_color(&self) -> Color {
        self.primary_color
    }
    fn get_popup_selected_background(&self) -> Color {
        self.popup_selected_background
    }
    fn get_border_color(&self) -> Color {
        self.border_color
    }
    fn get_focused_border_color(&self) -> Color {
        self.focused_border_color
    }
    fn get_secondary_color(&self) -> Color {
        self.secondary_color
    }
    fn get_info_color(&self) -> Color {
        self.info_color
    }
    fn get_json_key_color(&self) -> Color {
        self.json_key_color
    }
    fn get_json_string_color(&self) -> Color {
        self.json_string_color
    }
    fn get_json_number_color(&self) -> Color {
        self.json_number_color
    }
    fn get_json_bool_color(&self) -> Color {
        self.json_bool_color
    }
    fn get_json_null_color(&self) -> Color {
        self.json_null_color
    }
    fn get_accent_yellow(&self) -> Color {
        self.accent_yellow
    }
    fn get_code_colors(&self) -> CodeColors {
        self.code_colors
    }
}

pub struct ThemeBuilder {
    config: ThemeConfig,
}

impl ThemeBuilder {
    pub fn with_generation(mut self, gen: Generation) -> Self {
        self.config.gen = gen;
        self
    }

    pub fn with_text_color(mut self, c: Color) -> Self {
        self.config.text_color = c;
        self
    }

    pub fn with_muted_text_color(mut self, c: Color) -> Self {
        self.config.muted_text_color = c;
        self
    }

    pub fn with_primary_color(mut self, c: Color) -> Self {
        self.config.primary_color = c;
        self
    }

    pub fn with_popup_selected_background(mut self, c: Color) -> Self {
        self.config.popup_selected_background = c;
        self
    }

    pub fn with_border_color(mut self, c: Color) -> Self {
        self.config.border_color = c;
        self
    }

    pub fn with_focused_border_color(mut self, c: Color) -> Self {
        self.config.focused_border_color = c;
        self
    }

    pub fn with_secondary_color(mut self, c: Color) -> Self {
        self.config.secondary_color = c;
        self
    }

    pub fn with_info_color(mut self, c: Color) -> Self {
        self.config.info_color = c;
        self
    }

    pub fn with_json_key_color(mut self, c: Color) -> Self {
        self.config.json_key_color = c;
        self
    }

    pub fn with_json_string_color(mut self, c: Color) -> Self {
        self.config.json_string_color = c;
        self
    }

    pub fn with_json_number_color(mut self, c: Color) -> Self {
        self.config.json_number_color = c;
        self
    }

    pub fn with_json_bool_color(mut self, c: Color) -> Self {
        self.config.json_bool_color = c;
        self
    }

    pub fn with_json_null_color(mut self, c: Color) -> Self {
        self.config.json_null_color = c;
        self
    }

    pub fn with_accent_yellow(mut self, c: Color) -> Self {
        self.config.accent_yellow = c;
        self
    }

    pub fn with_code_colors(mut self, colors: CodeColors) -> Self {
        self.config.code_colors = colors;
        self
    }

    pub fn build(self) -> ThemeConfig {
        self.config
    }
}

#[deprecated(since = "0.3.0", note = "Use `ThemeConfig::default()` instead")]
pub struct DefaultTheme;

#[allow(deprecated)]
impl RichTextTheme for DefaultTheme {
    fn generation(&self) -> Generation {
        Generation(1)
    }
    fn get_text_color(&self) -> Color {
        Color::White
    }
    fn get_muted_text_color(&self) -> Color {
        Color::DarkGray
    }
    fn get_primary_color(&self) -> Color {
        Color::Cyan
    }
    fn get_secondary_color(&self) -> Color {
        Color::Blue
    }
    fn get_info_color(&self) -> Color {
        Color::LightBlue
    }
    fn get_border_color(&self) -> Color {
        Color::DarkGray
    }
    fn get_focused_border_color(&self) -> Color {
        Color::White
    }
    fn get_popup_selected_background(&self) -> Color {
        Color::DarkGray
    }
    fn get_json_key_color(&self) -> Color {
        Color::LightCyan
    }
    fn get_json_string_color(&self) -> Color {
        Color::Green
    }
    fn get_json_number_color(&self) -> Color {
        Color::Yellow
    }
    fn get_json_bool_color(&self) -> Color {
        Color::Magenta
    }
    fn get_json_null_color(&self) -> Color {
        Color::DarkGray
    }
    fn get_accent_yellow(&self) -> Color {
        Color::Yellow
    }
}
