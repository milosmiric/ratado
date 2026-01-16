//! Description textarea with clickable link support.
//!
//! A multi-line text input for task descriptions that supports:
//! - Multiple lines of text
//! - Link detection and highlighting
//! - Opening links in the default browser with Ctrl+O
//!
//! # Example
//!
//! ```rust,no_run
//! use ratado::ui::description_textarea::DescriptionTextArea;
//!
//! let mut textarea = DescriptionTextArea::new();
//! textarea.set_text("Check out https://example.com for more info");
//! ```

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
};

use super::theme;

/// Result of handling a key event in the textarea.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextAreaAction {
    /// No special action, continue editing
    None,
    /// User wants to open a link (Ctrl+O)
    OpenLink(String),
    /// User pressed Tab to move to next field
    NextField,
    /// User pressed Shift+Tab to move to previous field
    PrevField,
}

/// A multi-line textarea for task descriptions with link support.
#[derive(Debug, Clone)]
pub struct DescriptionTextArea {
    /// Lines of text
    lines: Vec<String>,
    /// Cursor row (0-indexed)
    cursor_row: usize,
    /// Cursor column (0-indexed)
    cursor_col: usize,
    /// Detected links in the text
    links: Vec<LinkSpan>,
    /// Scroll offset for viewing
    scroll_offset: usize,
}

/// Represents a link found in the text.
#[derive(Debug, Clone)]
struct LinkSpan {
    /// The URL text
    url: String,
    /// Line number (0-indexed)
    line: usize,
    /// Start column in the line
    start: usize,
    /// End column in the line
    end: usize,
}

impl DescriptionTextArea {
    /// Creates a new empty textarea.
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            links: Vec::new(),
            scroll_offset: 0,
        }
    }

    /// Creates a textarea with initial text.
    pub fn with_text(text: &str) -> Self {
        let mut ta = Self::new();
        ta.set_text(text);
        ta
    }

    /// Sets the text content.
    pub fn set_text(&mut self, text: &str) {
        self.lines = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(|s| s.to_string()).collect()
        };
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
        self.update_links();
    }

    /// Gets the text content as a single string.
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Returns true if the textarea is empty.
    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    /// Updates the list of detected links.
    fn update_links(&mut self) {
        self.links.clear();
        let lines_copy: Vec<String> = self.lines.clone();

        for (line_idx, line) in lines_copy.iter().enumerate() {
            self.detect_links_in_line(line, line_idx);
        }
    }

    /// Detects URLs in a single line.
    fn detect_links_in_line(&mut self, line: &str, line_idx: usize) {
        let mut start = 0;
        while start < line.len() {
            // Find the next occurrence of http:// or https://
            let search_str = &line[start..];
            let https_pos = search_str.find("https://");
            let http_pos = search_str.find("http://");

            // Get the earliest match (prefer https:// if both match at same position via prefix)
            let url_pos = match (https_pos, http_pos) {
                (Some(https), Some(http)) => Some(https.min(http)),
                (Some(pos), None) | (None, Some(pos)) => Some(pos),
                (None, None) => None,
            };

            let Some(rel_pos) = url_pos else {
                break;
            };

            let abs_start = start + rel_pos;
            let url_start = abs_start;
            let rest = &line[url_start..];
            let url_end = rest
                .find(|c: char| c.is_whitespace() || c == ')' || c == ']' || c == '>' || c == '"' || c == '\'')
                .map(|pos| url_start + pos)
                .unwrap_or(line.len());

            if url_end > url_start {
                let url = line[url_start..url_end].to_string();
                self.links.push(LinkSpan {
                    url,
                    line: line_idx,
                    start: url_start,
                    end: url_end,
                });
            }

            start = url_end;
        }
    }

    /// Gets the link at the current cursor position, if any.
    pub fn link_at_cursor(&self) -> Option<&str> {
        for link in &self.links {
            if link.line == self.cursor_row && self.cursor_col >= link.start && self.cursor_col < link.end {
                return Some(&link.url);
            }
        }
        None
    }

    /// Gets all detected links.
    pub fn links(&self) -> Vec<&str> {
        self.links.iter().map(|l| l.url.as_str()).collect()
    }

    /// Inserts a character at the cursor position.
    fn insert_char(&mut self, c: char) {
        if self.cursor_row < self.lines.len() {
            let line = &mut self.lines[self.cursor_row];
            if self.cursor_col <= line.len() {
                line.insert(self.cursor_col, c);
                self.cursor_col += 1;
            }
        }
        self.update_links();
    }

    /// Inserts a newline at the cursor position.
    fn insert_newline(&mut self) {
        if self.cursor_row < self.lines.len() {
            let current_line = &self.lines[self.cursor_row];
            let rest = current_line[self.cursor_col..].to_string();
            self.lines[self.cursor_row] = current_line[..self.cursor_col].to_string();
            self.cursor_row += 1;
            self.lines.insert(self.cursor_row, rest);
            self.cursor_col = 0;
        }
        self.update_links();
    }

    /// Deletes the character before the cursor.
    fn delete_backward(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_row];
            line.remove(self.cursor_col - 1);
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            // Join with previous line
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current_line);
        }
        self.update_links();
    }

    /// Deletes the character at the cursor.
    fn delete_forward(&mut self) {
        if self.cursor_row < self.lines.len() {
            let line = &self.lines[self.cursor_row];
            if self.cursor_col < line.len() {
                self.lines[self.cursor_row].remove(self.cursor_col);
            } else if self.cursor_row < self.lines.len() - 1 {
                // Join with next line
                let next_line = self.lines.remove(self.cursor_row + 1);
                self.lines[self.cursor_row].push_str(&next_line);
            }
        }
        self.update_links();
    }

    /// Moves the cursor left.
    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    /// Moves the cursor right.
    fn move_right(&mut self) {
        if self.cursor_row < self.lines.len() {
            if self.cursor_col < self.lines[self.cursor_row].len() {
                self.cursor_col += 1;
            } else if self.cursor_row < self.lines.len() - 1 {
                self.cursor_row += 1;
                self.cursor_col = 0;
            }
        }
    }

    /// Moves the cursor up.
    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        }
    }

    /// Moves the cursor down.
    fn move_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        }
    }

    /// Moves the cursor to the start of the line.
    fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    /// Moves the cursor to the end of the line.
    fn move_end(&mut self) {
        if self.cursor_row < self.lines.len() {
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    /// Moves the cursor left by one word.
    fn move_word_left(&mut self) {
        if self.cursor_row >= self.lines.len() {
            return;
        }

        let line = &self.lines[self.cursor_row];
        let chars: Vec<char> = line.chars().collect();

        if self.cursor_col == 0 {
            // Move to end of previous line
            if self.cursor_row > 0 {
                self.cursor_row -= 1;
                self.cursor_col = self.lines[self.cursor_row].len();
            }
            return;
        }

        let mut pos = self.cursor_col;

        // Skip any whitespace immediately before cursor
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Skip non-whitespace characters (the word itself)
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        self.cursor_col = pos;
    }

    /// Moves the cursor right by one word.
    fn move_word_right(&mut self) {
        if self.cursor_row >= self.lines.len() {
            return;
        }

        let line = &self.lines[self.cursor_row];
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();

        if self.cursor_col >= len {
            // Move to start of next line
            if self.cursor_row < self.lines.len() - 1 {
                self.cursor_row += 1;
                self.cursor_col = 0;
            }
            return;
        }

        let mut pos = self.cursor_col;

        // Skip non-whitespace characters (current word)
        while pos < len && !chars[pos].is_whitespace() {
            pos += 1;
        }

        // Skip whitespace to reach next word
        while pos < len && chars[pos].is_whitespace() {
            pos += 1;
        }

        self.cursor_col = pos;
    }

    /// Deletes the word before the cursor (Option+Delete / Alt+Backspace).
    fn delete_word_backward(&mut self) {
        if self.cursor_row >= self.lines.len() {
            return;
        }

        if self.cursor_col == 0 {
            // At start of line, join with previous line
            if self.cursor_row > 0 {
                let current_line = self.lines.remove(self.cursor_row);
                self.cursor_row -= 1;
                self.cursor_col = self.lines[self.cursor_row].len();
                self.lines[self.cursor_row].push_str(&current_line);
            }
            self.update_links();
            return;
        }

        let line = &self.lines[self.cursor_row];
        let chars: Vec<char> = line.chars().collect();
        let mut pos = self.cursor_col;

        // Skip any whitespace immediately before cursor
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Skip non-whitespace characters (the word itself)
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Delete from pos to cursor_col
        let new_line: String = chars[..pos].iter().chain(chars[self.cursor_col..].iter()).collect();
        self.lines[self.cursor_row] = new_line;
        self.cursor_col = pos;
        self.update_links();
    }

    /// Handles a key event and returns the resulting action.
    pub fn handle_key(&mut self, key: KeyEvent) -> TextAreaAction {
        match key.code {
            // Ctrl+O opens the link at cursor
            KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(url) = self.link_at_cursor() {
                    return TextAreaAction::OpenLink(url.to_string());
                }
                TextAreaAction::None
            }

            // Word navigation (Emacs-style Alt+b/Alt+f for macOS Option+Arrow)
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::ALT) => {
                self.move_word_left();
                TextAreaAction::None
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::ALT) => {
                self.move_word_right();
                TextAreaAction::None
            }

            // Tab moves to next field
            KeyCode::Tab if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                TextAreaAction::NextField
            }

            // Shift+Tab moves to previous field
            KeyCode::BackTab => TextAreaAction::PrevField,

            // Enter inserts a newline
            KeyCode::Enter => {
                self.insert_newline();
                TextAreaAction::None
            }

            // Character input
            KeyCode::Char(c) => {
                self.insert_char(c);
                TextAreaAction::None
            }

            // Word deletion (Alt+Backspace)
            KeyCode::Backspace if key.modifiers.contains(KeyModifiers::ALT) => {
                self.delete_word_backward();
                TextAreaAction::None
            }

            // Backspace
            KeyCode::Backspace => {
                self.delete_backward();
                TextAreaAction::None
            }

            // Delete
            KeyCode::Delete => {
                self.delete_forward();
                TextAreaAction::None
            }

            // Word navigation (Alt+Arrow)
            KeyCode::Left if key.modifiers.contains(KeyModifiers::ALT) => {
                self.move_word_left();
                TextAreaAction::None
            }
            KeyCode::Right if key.modifiers.contains(KeyModifiers::ALT) => {
                self.move_word_right();
                TextAreaAction::None
            }
            KeyCode::Left => {
                self.move_left();
                TextAreaAction::None
            }
            KeyCode::Right => {
                self.move_right();
                TextAreaAction::None
            }
            KeyCode::Up => {
                self.move_up();
                TextAreaAction::None
            }
            KeyCode::Down => {
                self.move_down();
                TextAreaAction::None
            }
            KeyCode::Home => {
                self.move_home();
                TextAreaAction::None
            }
            KeyCode::End => {
                self.move_end();
                TextAreaAction::None
            }

            _ => TextAreaAction::None,
        }
    }

    /// Renders the textarea to the given area.
    pub fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, label: Option<&str>) {
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

        let title = label
            .map(|l| ratatui::text::Span::styled(format!(" {} ", l), label_style))
            .unwrap_or_default();

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, buf);

        // Render the textarea content with link highlighting
        self.render_content(inner, buf, focused);
    }

    /// Renders the textarea content with link highlighting.
    fn render_content(&self, area: Rect, buf: &mut Buffer, focused: bool) {
        // Show placeholder if empty
        if self.is_empty() {
            let placeholder = "Ctrl+O to open link under cursor";
            for (i, ch) in placeholder.chars().enumerate() {
                let x = area.x + i as u16;
                if x >= area.x + area.width {
                    break;
                }
                buf[(x, area.y)]
                    .set_char(ch)
                    .set_style(Style::default().fg(theme::TEXT_MUTED));
            }
            // Show cursor at start if focused
            if focused {
                buf[(area.x, area.y)]
                    .set_char(' ')
                    .set_style(Style::default().bg(theme::ACCENT).fg(Color::Black));
            }
            return;
        }

        let visible_height = area.height as usize;

        // Adjust scroll offset to keep cursor visible
        let scroll = if self.cursor_row < self.scroll_offset {
            self.cursor_row
        } else if self.cursor_row >= self.scroll_offset + visible_height {
            self.cursor_row - visible_height + 1
        } else {
            self.scroll_offset
        };

        for (display_row, line_idx) in (scroll..self.lines.len()).enumerate() {
            if display_row >= visible_height {
                break;
            }

            let y = area.y + display_row as u16;
            let line = &self.lines[line_idx];

            // Find links on this line
            let line_links: Vec<_> = self.links.iter().filter(|l| l.line == line_idx).collect();

            let mut x = area.x;
            let max_x = area.x + area.width;

            for (col, ch) in line.chars().enumerate() {
                if x >= max_x {
                    break;
                }

                // Determine style for this character
                let mut style = Style::default().fg(theme::TEXT_PRIMARY);

                // Check if this position is within a link
                for link in &line_links {
                    if col >= link.start && col < link.end {
                        style = Style::default()
                            .fg(theme::INFO)
                            .add_modifier(Modifier::UNDERLINED);
                        break;
                    }
                }

                // Apply cursor style if focused and at cursor position
                if focused && line_idx == self.cursor_row && col == self.cursor_col {
                    style = Style::default().bg(theme::ACCENT).fg(Color::Black);
                }

                buf[(x, y)].set_char(ch).set_style(style);
                x += 1;
            }

            // Show cursor at end of line if needed
            if focused && line_idx == self.cursor_row && self.cursor_col >= line.len() && x < max_x {
                buf[(x, y)]
                    .set_char(' ')
                    .set_style(Style::default().bg(theme::ACCENT).fg(Color::Black));
            }
        }
    }
}

impl Default for DescriptionTextArea {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_textarea() {
        let ta = DescriptionTextArea::new();
        assert!(ta.is_empty());
        assert!(ta.links().is_empty());
    }

    #[test]
    fn test_with_text() {
        let ta = DescriptionTextArea::with_text("Hello world");
        assert_eq!(ta.text(), "Hello world");
        assert!(!ta.is_empty());
    }

    #[test]
    fn test_detect_links() {
        let ta = DescriptionTextArea::with_text("Check https://example.com for info");
        let links = ta.links();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], "https://example.com");
    }

    #[test]
    fn test_detect_multiple_links() {
        let ta = DescriptionTextArea::with_text(
            "Visit https://one.com and http://two.com for more",
        );
        let links = ta.links();
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"https://one.com"));
        assert!(links.contains(&"http://two.com"));
    }

    #[test]
    fn test_multiline_links() {
        let ta = DescriptionTextArea::with_text(
            "First line https://first.com\nSecond line https://second.com",
        );
        let links = ta.links();
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_no_links() {
        let ta = DescriptionTextArea::with_text("Just plain text without any links");
        assert!(ta.links().is_empty());
    }

    #[test]
    fn test_set_text() {
        let mut ta = DescriptionTextArea::new();
        ta.set_text("New content with https://link.com");
        assert_eq!(ta.text(), "New content with https://link.com");
        assert_eq!(ta.links().len(), 1);
    }

    #[test]
    fn test_tab_moves_to_next_field() {
        let mut ta = DescriptionTextArea::new();
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(ta.handle_key(key), TextAreaAction::NextField);
    }

    #[test]
    fn test_backtab_moves_to_prev_field() {
        let mut ta = DescriptionTextArea::new();
        let key = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
        assert_eq!(ta.handle_key(key), TextAreaAction::PrevField);
    }

    #[test]
    fn test_insert_char() {
        let mut ta = DescriptionTextArea::new();
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        ta.handle_key(key);
        assert_eq!(ta.text(), "a");
    }

    #[test]
    fn test_insert_newline() {
        let mut ta = DescriptionTextArea::with_text("Hello");
        ta.cursor_col = 5; // End of line
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        ta.handle_key(key);
        assert_eq!(ta.text(), "Hello\n");
    }

    #[test]
    fn test_backspace() {
        let mut ta = DescriptionTextArea::with_text("Hello");
        ta.cursor_col = 5;
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        ta.handle_key(key);
        assert_eq!(ta.text(), "Hell");
    }

    #[test]
    fn test_move_word_left() {
        let mut ta = DescriptionTextArea::with_text("Hello world test");
        ta.cursor_col = 16; // At end

        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::ALT);
        ta.handle_key(key);
        assert_eq!(ta.cursor_col, 12); // Before "test"

        ta.handle_key(key);
        assert_eq!(ta.cursor_col, 6); // Before "world"

        ta.handle_key(key);
        assert_eq!(ta.cursor_col, 0); // Before "Hello"
    }

    #[test]
    fn test_move_word_right() {
        let mut ta = DescriptionTextArea::with_text("Hello world test");
        ta.cursor_col = 0;

        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::ALT);
        ta.handle_key(key);
        assert_eq!(ta.cursor_col, 6); // After "Hello "

        ta.handle_key(key);
        assert_eq!(ta.cursor_col, 12); // After "world "

        ta.handle_key(key);
        assert_eq!(ta.cursor_col, 16); // At end
    }

    #[test]
    fn test_delete_word_backward() {
        let mut ta = DescriptionTextArea::with_text("Hello world test");
        ta.cursor_col = 16; // At end

        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::ALT);
        ta.handle_key(key);
        assert_eq!(ta.text(), "Hello world ");
        assert_eq!(ta.cursor_col, 12);

        ta.handle_key(key);
        assert_eq!(ta.text(), "Hello ");
        assert_eq!(ta.cursor_col, 6);

        ta.handle_key(key);
        assert_eq!(ta.text(), "");
        assert_eq!(ta.cursor_col, 0);
    }
}
