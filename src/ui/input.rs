//! Text input widget for forms and dialogs.
//!
//! This module provides a [`TextInput`] widget for capturing text input
//! with cursor support and basic editing operations.
//!
//! ## Example
//!
//! ```rust,no_run
//! use ratado::ui::input::TextInput;
//!
//! let mut input = TextInput::new();
//! input.insert('H');
//! input.insert('i');
//! assert_eq!(input.value(), "Hi");
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

use super::theme;

/// A text input widget with cursor support.
///
/// `TextInput` provides a single-line text input field with:
/// - Character insertion at cursor position
/// - Backspace and delete operations
/// - Cursor movement (left, right, home, end)
/// - Visual cursor indicator when focused
#[derive(Debug, Clone, Default)]
pub struct TextInput {
    /// The current text value
    value: String,
    /// Cursor position (0 = before first char)
    cursor: usize,
    /// Placeholder text shown when empty
    placeholder: Option<String>,
}

impl TextInput {
    /// Creates a new empty text input.
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            placeholder: None,
        }
    }

    /// Creates a text input with an initial value.
    ///
    /// The cursor is positioned at the end of the value.
    pub fn with_value(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.len();
        Self {
            value,
            cursor,
            placeholder: None,
        }
    }

    /// Sets the placeholder text shown when the input is empty.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Returns the current text value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Sets the value and moves cursor to end.
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.value.len();
    }

    /// Clears the input and resets cursor.
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    /// Inserts a character at the cursor position.
    pub fn insert(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Deletes the character before the cursor (backspace).
    pub fn delete_backward(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.value.remove(self.cursor);
        }
    }

    /// Deletes the character at the cursor (delete key).
    pub fn delete_forward(&mut self) {
        if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
        }
    }

    /// Moves the cursor left by one position.
    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Moves the cursor right by one position.
    pub fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.value.len());
    }

    /// Moves the cursor to the start.
    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    /// Moves the cursor to the end.
    pub fn move_end(&mut self) {
        self.cursor = self.value.len();
    }

    /// Moves the cursor left by one word.
    ///
    /// A word boundary is defined as a transition between whitespace and non-whitespace.
    pub fn move_word_left(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let chars: Vec<char> = self.value.chars().collect();

        // Skip any whitespace immediately before cursor
        let mut pos = self.cursor;
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Skip non-whitespace characters (the word itself)
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        self.cursor = pos;
    }

    /// Moves the cursor right by one word.
    ///
    /// A word boundary is defined as a transition between whitespace and non-whitespace.
    pub fn move_word_right(&mut self) {
        let chars: Vec<char> = self.value.chars().collect();
        let len = chars.len();

        if self.cursor >= len {
            return;
        }

        let mut pos = self.cursor;

        // Skip non-whitespace characters (current word)
        while pos < len && !chars[pos].is_whitespace() {
            pos += 1;
        }

        // Skip whitespace to reach next word
        while pos < len && chars[pos].is_whitespace() {
            pos += 1;
        }

        self.cursor = pos;
    }

    /// Deletes the word before the cursor (Option+Delete / Alt+Backspace).
    pub fn delete_word_backward(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let chars: Vec<char> = self.value.chars().collect();
        let mut pos = self.cursor;

        // Skip any whitespace immediately before cursor
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Skip non-whitespace characters (the word itself)
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Delete from pos to cursor
        self.value = chars[..pos].iter().chain(chars[self.cursor..].iter()).collect();
        self.cursor = pos;
    }

    /// Renders the text input to a buffer.
    ///
    /// # Arguments
    ///
    /// * `area` - The area to render into
    /// * `buf` - The buffer to render to
    /// * `focused` - Whether this input is currently focused
    /// * `label` - Optional label to show in the border
    pub fn render_to_buffer(
        &self,
        area: Rect,
        buf: &mut Buffer,
        focused: bool,
        label: Option<&str>,
    ) {
        let text_style = if focused {
            Style::default().fg(theme::TEXT_PRIMARY)
        } else {
            Style::default().fg(theme::TEXT_SECONDARY)
        };

        let border_style = if focused {
            Style::default().fg(theme::PRIMARY_LIGHT)
        } else {
            Style::default().fg(theme::BORDER)
        };

        let label_style = if focused {
            Style::default()
                .fg(theme::PRIMARY_LIGHT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT_MUTED)
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        if let Some(label) = label {
            block = block.title(ratatui::text::Span::styled(
                format!(" {} ", label),
                label_style,
            ));
        }

        let inner = block.inner(area);
        block.render(area, buf);

        // Render the text or placeholder
        let is_empty = self.value.is_empty();
        let display_style = if is_empty {
            Style::default().fg(theme::TEXT_MUTED)
        } else {
            text_style
        };

        // Calculate visible portion based on cursor position
        let width = inner.width as usize;
        let visible_text = if is_empty {
            // Show placeholder when empty
            self.placeholder.as_deref().unwrap_or("").to_string()
        } else if width > 0 {
            self.calculate_visible_text(width).0
        } else {
            self.value.clone()
        };

        let cursor_pos = if !is_empty && width > 0 {
            self.calculate_visible_text(width).1
        } else {
            0
        };

        let paragraph = Paragraph::new(visible_text).style(display_style);
        paragraph.render(inner, buf);

        // Render cursor if focused
        if focused && inner.width > 0 {
            let cursor_x = inner.x + cursor_pos as u16;
            if cursor_x < inner.x + inner.width {
                let cursor_char = if self.cursor < self.value.len() {
                    self.value.chars().nth(self.cursor).unwrap_or(' ')
                } else {
                    ' '
                };
                buf[(cursor_x, inner.y)]
                    .set_char(cursor_char)
                    .set_style(Style::default().bg(theme::ACCENT).fg(Color::Black));
            }
        }
    }

    /// Calculates the visible portion of text and cursor position within it.
    fn calculate_visible_text(&self, width: usize) -> (String, usize) {
        if self.value.len() <= width {
            return (self.value.clone(), self.cursor);
        }

        // Scroll to keep cursor visible
        let scroll_margin = width / 4;
        let start = if self.cursor < width - scroll_margin {
            0
        } else {
            self.cursor.saturating_sub(width - scroll_margin - 1)
        };

        let end = (start + width).min(self.value.len());
        let visible: String = self.value.chars().skip(start).take(end - start).collect();
        let cursor_pos = self.cursor - start;

        (visible, cursor_pos)
    }
}

/// A renderable version of TextInput for use as a Widget.
pub struct TextInputWidget<'a> {
    input: &'a TextInput,
    focused: bool,
    label: Option<&'a str>,
}

impl<'a> TextInputWidget<'a> {
    /// Creates a new text input widget.
    pub fn new(input: &'a TextInput) -> Self {
        Self {
            input,
            focused: false,
            label: None,
        }
    }

    /// Sets whether the input is focused.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the label shown in the border.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

impl Widget for TextInputWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.input
            .render_to_buffer(area, buf, self.focused, self.label);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_input() {
        let input = TextInput::new();
        assert_eq!(input.value(), "");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_with_value() {
        let input = TextInput::with_value("Hello");
        assert_eq!(input.value(), "Hello");
        assert_eq!(input.cursor(), 5);
    }

    #[test]
    fn test_insert() {
        let mut input = TextInput::new();
        input.insert('H');
        input.insert('i');
        assert_eq!(input.value(), "Hi");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn test_delete_backward() {
        let mut input = TextInput::with_value("Hello");
        input.delete_backward();
        assert_eq!(input.value(), "Hell");
        assert_eq!(input.cursor(), 4);
    }

    #[test]
    fn test_delete_backward_at_start() {
        let mut input = TextInput::with_value("Hello");
        input.move_home();
        input.delete_backward();
        assert_eq!(input.value(), "Hello"); // No change
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_delete_forward() {
        let mut input = TextInput::with_value("Hello");
        input.move_home();
        input.delete_forward();
        assert_eq!(input.value(), "ello");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = TextInput::with_value("Hello");
        assert_eq!(input.cursor(), 5);

        input.move_left();
        assert_eq!(input.cursor(), 4);

        input.move_home();
        assert_eq!(input.cursor(), 0);

        input.move_right();
        assert_eq!(input.cursor(), 1);

        input.move_end();
        assert_eq!(input.cursor(), 5);
    }

    #[test]
    fn test_insert_at_cursor() {
        let mut input = TextInput::with_value("Hllo");
        input.cursor = 1;
        input.insert('e');
        assert_eq!(input.value(), "Hello");
    }

    #[test]
    fn test_clear() {
        let mut input = TextInput::with_value("Hello");
        input.clear();
        assert_eq!(input.value(), "");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_move_word_left() {
        let mut input = TextInput::with_value("Hello world test");
        assert_eq!(input.cursor(), 16); // At end

        input.move_word_left();
        assert_eq!(input.cursor(), 12); // Before "test"

        input.move_word_left();
        assert_eq!(input.cursor(), 6); // Before "world"

        input.move_word_left();
        assert_eq!(input.cursor(), 0); // Before "Hello"

        input.move_word_left();
        assert_eq!(input.cursor(), 0); // Stay at start
    }

    #[test]
    fn test_move_word_right() {
        let mut input = TextInput::with_value("Hello world test");
        input.move_home();
        assert_eq!(input.cursor(), 0);

        input.move_word_right();
        assert_eq!(input.cursor(), 6); // After "Hello "

        input.move_word_right();
        assert_eq!(input.cursor(), 12); // After "world "

        input.move_word_right();
        assert_eq!(input.cursor(), 16); // At end

        input.move_word_right();
        assert_eq!(input.cursor(), 16); // Stay at end
    }

    #[test]
    fn test_delete_word_backward() {
        let mut input = TextInput::with_value("Hello world test");
        assert_eq!(input.cursor(), 16); // At end

        input.delete_word_backward();
        assert_eq!(input.value(), "Hello world ");
        assert_eq!(input.cursor(), 12);

        input.delete_word_backward();
        assert_eq!(input.value(), "Hello ");
        assert_eq!(input.cursor(), 6);

        input.delete_word_backward();
        assert_eq!(input.value(), "");
        assert_eq!(input.cursor(), 0);
    }
}
