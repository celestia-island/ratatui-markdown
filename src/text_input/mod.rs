mod cursor;
mod edit_render;
mod read_render;
mod types;

pub use types::{
    CursorBlinkController, CursorPosition, CursorShape, CursorStyle, InputMode, Selection,
    SelectionStyle,
};

use std::rc::Rc;

use ratatui::{layout::Rect, Frame};

use crate::theme::RichTextTheme;

pub struct TextInput {
    text: String,
    cursor_char_idx: usize,
    selection: Option<Selection>,
    mode: InputMode,
    cursor_style: CursorStyle,
    selection_style: SelectionStyle,
    blink_controller: Option<Rc<dyn CursorBlinkController>>,
    horizontal_scroll: usize,
    scroll_offset: usize,
    placeholder: Option<String>,
    password: bool,
    max_width: usize,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor_char_idx: 0,
            selection: None,
            mode: InputMode::default(),
            cursor_style: CursorStyle::default(),
            selection_style: SelectionStyle::default(),
            blink_controller: None,
            horizontal_scroll: 0,
            scroll_offset: 0,
            placeholder: None,
            password: false,
            max_width: usize::MAX,
        }
    }

    pub fn with_mode(mut self, mode: InputMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_cursor_style(mut self, style: CursorStyle) -> Self {
        self.cursor_style = style;
        self
    }

    pub fn with_selection_style(mut self, style: SelectionStyle) -> Self {
        self.selection_style = style;
        self
    }

    pub fn with_blink_controller(mut self, ctrl: Rc<dyn CursorBlinkController>) -> Self {
        self.blink_controller = Some(ctrl);
        self
    }

    pub fn with_placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    pub fn with_password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    pub fn with_max_width(mut self, width: usize) -> Self {
        self.max_width = width;
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        let len = self.text.chars().count();
        if self.cursor_char_idx > len {
            self.cursor_char_idx = len;
        }
    }

    pub fn cursor_char_idx(&self) -> usize {
        self.cursor_char_idx
    }

    pub fn set_cursor_char_idx(&mut self, idx: usize) {
        let len = self.text.chars().count();
        self.cursor_char_idx = idx.min(len);
    }

    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }

    pub fn set_selection(&mut self, sel: Option<Selection>) {
        self.selection = sel;
    }

    pub fn mode(&self) -> InputMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    pub fn horizontal_scroll(&self) -> usize {
        self.horizontal_scroll
    }

    pub fn set_horizontal_scroll(&mut self, offset: usize) {
        self.horizontal_scroll = offset;
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    pub fn insert_char(&mut self, ch: char) {
        if self.mode != InputMode::Edit {
            return;
        }
        let byte_pos = char_idx_to_byte(&self.text, self.cursor_char_idx);
        self.text.insert(byte_pos, ch);
        self.cursor_char_idx += 1;
        self.selection = None;
    }

    pub fn delete_char_backward(&mut self) {
        if self.mode != InputMode::Edit || self.cursor_char_idx == 0 {
            return;
        }
        self.cursor_char_idx -= 1;
        let byte_pos = char_idx_to_byte(&self.text, self.cursor_char_idx);
        self.text.remove(byte_pos);
        self.selection = None;
    }

    pub fn delete_char_forward(&mut self) {
        if self.mode != InputMode::Edit {
            return;
        }
        let len = self.text.chars().count();
        if self.cursor_char_idx >= len {
            return;
        }
        let byte_pos = char_idx_to_byte(&self.text, self.cursor_char_idx);
        self.text.remove(byte_pos);
        self.selection = None;
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_char_idx > 0 {
            self.cursor_char_idx -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let len = self.text.chars().count();
        if self.cursor_char_idx < len {
            self.cursor_char_idx += 1;
        }
    }

    pub fn move_cursor_to_start(&mut self) {
        self.cursor_char_idx = 0;
    }

    pub fn move_cursor_to_end(&mut self) {
        self.cursor_char_idx = self.text.chars().count();
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &impl RichTextTheme) {
        let effective_width = if self.max_width < area.width as usize {
            self.max_width
        } else {
            area.width as usize
        };

        match self.mode {
            InputMode::Edit => {
                let mut lines = edit_render::render_edit_mode(
                    &self.text,
                    self.cursor_char_idx,
                    self.horizontal_scroll,
                    effective_width,
                    self.password,
                    self.placeholder.as_deref(),
                    theme,
                );

                let blink_visible = self
                    .blink_controller
                    .as_ref()
                    .map_or(true, |ctrl| ctrl.is_visible());

                if let Some(line) = lines.first_mut() {
                    cursor::apply_cursor_and_selection(
                        line,
                        self.cursor_char_idx,
                        self.horizontal_scroll,
                        &self.cursor_style,
                        self.selection.as_ref(),
                        &self.selection_style,
                        blink_visible,
                        theme,
                    );
                }

                let paragraph = ratatui::widgets::Paragraph::new(lines);
                f.render_widget(paragraph, area);
            }
            InputMode::Read => {
                #[cfg(feature = "markdown")]
                {
                    read_render::render_read_mode(
                        &self.text,
                        f,
                        area,
                        self.scroll_offset,
                        theme,
                    );
                }
                #[cfg(not(feature = "markdown"))]
                {
                    let _ = (effective_width, theme);
                    let paragraph = ratatui::widgets::Paragraph::new(&*self.text);
                    f.render_widget(paragraph, area);
                }
            }
        }
    }

    #[cfg(feature = "markdown")]
    pub fn rendered_height(&self, width: usize, theme: &impl RichTextTheme) -> u16 {
        read_render::rendered_height(&self.text, width, theme)
    }

    #[cfg(not(feature = "markdown"))]
    pub fn rendered_height(&self, _width: usize, _theme: &impl RichTextTheme) -> u16 {
        self.text.lines().count().max(1) as u16
    }
}

fn char_idx_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_defaults() {
        let input = TextInput::new();
        assert_eq!(input.text(), "");
        assert_eq!(input.cursor_char_idx(), 0);
        assert_eq!(input.mode(), InputMode::Edit);
    }

    #[test]
    fn set_text_updates_content() {
        let mut input = TextInput::new();
        input.set_text("hello");
        assert_eq!(input.text(), "hello");
    }

    #[test]
    fn cursor_clamps_on_set_text() {
        let mut input = TextInput::new();
        input.set_text("hello");
        input.set_cursor_char_idx(5);
        input.set_text("hi");
        assert!(input.cursor_char_idx() <= 2);
    }

    #[test]
    fn insert_char_appends() {
        let mut input = TextInput::new();
        input.insert_char('a');
        input.insert_char('b');
        assert_eq!(input.text(), "ab");
        assert_eq!(input.cursor_char_idx(), 2);
    }

    #[test]
    fn insert_char_in_middle() {
        let mut input = TextInput::new();
        input.set_text("ac");
        input.set_cursor_char_idx(1);
        input.insert_char('b');
        assert_eq!(input.text(), "abc");
        assert_eq!(input.cursor_char_idx(), 2);
    }

    #[test]
    fn delete_char_backward() {
        let mut input = TextInput::new();
        input.set_text("abc");
        input.set_cursor_char_idx(2);
        input.delete_char_backward();
        assert_eq!(input.text(), "ac");
        assert_eq!(input.cursor_char_idx(), 1);
    }

    #[test]
    fn delete_char_forward() {
        let mut input = TextInput::new();
        input.set_text("abc");
        input.set_cursor_char_idx(1);
        input.delete_char_forward();
        assert_eq!(input.text(), "ac");
        assert_eq!(input.cursor_char_idx(), 1);
    }

    #[test]
    fn move_cursor_left_right() {
        let mut input = TextInput::new();
        input.set_text("hello");
        input.set_cursor_char_idx(3);
        input.move_cursor_left();
        assert_eq!(input.cursor_char_idx(), 2);
        input.move_cursor_right();
        assert_eq!(input.cursor_char_idx(), 3);
    }

    #[test]
    fn move_cursor_bounds() {
        let mut input = TextInput::new();
        input.set_text("hi");
        input.move_cursor_left();
        assert_eq!(input.cursor_char_idx(), 0);
        input.move_cursor_to_end();
        assert_eq!(input.cursor_char_idx(), 2);
        input.move_cursor_right();
        assert_eq!(input.cursor_char_idx(), 2);
    }

    #[test]
    fn mode_switching() {
        let mut input = TextInput::new();
        assert_eq!(input.mode(), InputMode::Edit);
        input.set_mode(InputMode::Read);
        assert_eq!(input.mode(), InputMode::Read);
    }

    #[test]
    fn insert_in_read_mode_noop() {
        let mut input = TextInput::new().with_mode(InputMode::Read);
        input.set_text("hello");
        input.insert_char('x');
        assert_eq!(input.text(), "hello");
    }

    #[test]
    fn builder_methods() {
        let input = TextInput::new()
            .with_mode(InputMode::Read)
            .with_password(true)
            .with_placeholder("enter text")
            .with_max_width(100)
            .with_cursor_style(CursorStyle::new().with_shape(CursorShape::Bar))
            .with_selection_style(SelectionStyle::new().with_bg(ratatui::style::Color::Blue));
        assert_eq!(input.mode(), InputMode::Read);
        assert!(input.password);
        assert_eq!(input.max_width, 100);
    }

    #[test]
    fn selection_set_clear() {
        let mut input = TextInput::new();
        assert!(input.selection().is_none());
        input.set_selection(Some(Selection::new(0, 5)));
        assert_eq!(input.selection().unwrap().start, 0);
        assert_eq!(input.selection().unwrap().end, 5);
        input.set_selection(None);
        assert!(input.selection().is_none());
    }

    #[test]
    fn selection_insert_clears() {
        let mut input = TextInput::new();
        input.set_text("hello");
        input.set_selection(Some(Selection::new(0, 3)));
        input.insert_char('x');
        assert!(input.selection().is_none());
    }

    #[test]
    fn char_idx_to_byte_multibyte() {
        let s = "héllo";
        assert_eq!(char_idx_to_byte(s, 0), 0);
        assert_eq!(char_idx_to_byte(s, 1), 1);
        assert_eq!(char_idx_to_byte(s, 2), 3);
    }
}
