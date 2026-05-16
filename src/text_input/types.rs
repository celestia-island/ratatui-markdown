use ratatui::style::{Color, Modifier};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorShape {
    #[default]
    Block,
    Bar,
    Underline,
    HollowBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorPosition {
    #[default]
    OnChar,
    BeforeChar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorStyle {
    pub shape: CursorShape,
    pub position: CursorPosition,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub modifier: Modifier,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self {
            shape: CursorShape::default(),
            position: CursorPosition::default(),
            fg: None,
            bg: None,
            modifier: Modifier::empty(),
        }
    }
}

impl CursorStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_shape(mut self, shape: CursorShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn with_position(mut self, position: CursorPosition) -> Self {
        self.position = position;
        self
    }

    pub fn with_fg(mut self, fg: Color) -> Self {
        self.fg = Some(fg);
        self
    }

    pub fn with_bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn with_modifier(mut self, modifier: Modifier) -> Self {
        self.modifier = modifier;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectionStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}

impl SelectionStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fg(mut self, fg: Color) -> Self {
        self.fg = Some(fg);
        self
    }

    pub fn with_bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Edit,
    Read,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn ordered(&self) -> (usize, usize) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }
}

pub trait CursorBlinkController {
    fn is_visible(&self) -> bool;
}
