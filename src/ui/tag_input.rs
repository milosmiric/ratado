//! Tag input widget with autocomplete.
//!
//! This module provides a [`TagInput`] widget for entering tags with
//! autocomplete suggestions based on existing tags in the database.
//!
//! ## Features
//!
//! - Comma or Enter to add a tag
//! - Backspace on empty input removes the last tag
//! - Tab to accept autocomplete suggestion
//! - Shows existing tags as chips/badges
//!
//! ## Example
//!
//! ```rust,no_run
//! use ratado::ui::tag_input::TagInput;
//!
//! let mut input = TagInput::new();
//! input.add_tag("work".to_string());
//! input.add_tag("urgent".to_string());
//! assert_eq!(input.tags(), &["work", "urgent"]);
//! ```

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::storage::Tag;
use crate::ui::input::TextInput;

/// Default placeholder text for tag input.
const DEFAULT_PLACEHOLDER: &str = "Type tag, press Enter to add";

/// A tag input widget with autocomplete support.
///
/// `TagInput` provides a multi-value input field for tags:
/// - Text input for typing new tags
/// - Autocomplete suggestions from existing tags
/// - Visual display of selected tags as chips
/// - Easy removal of tags via backspace
#[derive(Debug, Clone)]
pub struct TagInput {
    /// The text input for typing new tags
    input: TextInput,
    /// Currently selected tags
    tags: Vec<String>,
    /// Autocomplete suggestions based on current input
    suggestions: Vec<String>,
    /// Index of selected suggestion (if any)
    selected_suggestion: Option<usize>,
    /// Placeholder text shown when empty
    placeholder: String,
}

impl TagInput {
    /// Creates a new empty tag input.
    pub fn new() -> Self {
        Self {
            input: TextInput::new(),
            tags: Vec::new(),
            suggestions: Vec::new(),
            selected_suggestion: None,
            placeholder: DEFAULT_PLACEHOLDER.to_string(),
        }
    }

    /// Creates a tag input with initial tags.
    pub fn with_tags(tags: Vec<String>) -> Self {
        Self {
            input: TextInput::new(),
            tags,
            suggestions: Vec::new(),
            selected_suggestion: None,
            placeholder: DEFAULT_PLACEHOLDER.to_string(),
        }
    }

    /// Returns the current list of tags.
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Returns the tags as a cloned vector.
    pub fn tags_vec(&self) -> Vec<String> {
        self.tags.clone()
    }

    /// Adds a tag if it's not already present.
    pub fn add_tag(&mut self, tag: String) {
        let tag = tag.trim().to_string();
        if !tag.is_empty() && !self.tags.iter().any(|t| t.eq_ignore_ascii_case(&tag)) {
            self.tags.push(tag);
        }
    }

    /// Removes the last tag.
    pub fn remove_last_tag(&mut self) {
        self.tags.pop();
    }

    /// Removes a tag by name.
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// Returns the current input value.
    pub fn input_value(&self) -> &str {
        self.input.value()
    }

    /// Returns the currently selected suggestion, if any.
    pub fn selected_suggestion(&self) -> Option<&String> {
        self.selected_suggestion
            .and_then(|idx| self.suggestions.get(idx))
    }

    /// Updates autocomplete suggestions based on current input and available tags.
    pub fn update_suggestions(&mut self, all_tags: &[Tag]) {
        let query = self.input.value().to_lowercase();

        if query.is_empty() {
            self.suggestions.clear();
            self.selected_suggestion = None;
            return;
        }

        self.suggestions = all_tags
            .iter()
            .filter(|t| t.name.to_lowercase().contains(&query))
            .filter(|t| !self.tags.iter().any(|existing| existing.eq_ignore_ascii_case(&t.name)))
            .map(|t| t.name.clone())
            .take(5)
            .collect();

        self.selected_suggestion = if self.suggestions.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    /// Handles a key event and updates state accordingly.
    ///
    /// Returns `true` if the event was consumed.
    pub fn handle_key(&mut self, key: KeyEvent, all_tags: &[Tag]) -> bool {
        match key.code {
            // Comma or Enter adds the current input as a tag
            KeyCode::Char(',') | KeyCode::Enter => {
                let tag = self.input.value().trim().to_string();
                if !tag.is_empty() {
                    self.add_tag(tag);
                    self.input.clear();
                    self.suggestions.clear();
                    self.selected_suggestion = None;
                }
                true
            }

            // Backspace on empty input removes the last tag
            KeyCode::Backspace if self.input.value().is_empty() => {
                self.remove_last_tag();
                true
            }

            // Regular backspace
            KeyCode::Backspace => {
                self.input.delete_backward();
                self.update_suggestions(all_tags);
                true
            }

            // Tab accepts the selected suggestion
            KeyCode::Tab => {
                if let Some(suggestion) = self.selected_suggestion() {
                    let tag = suggestion.clone();
                    self.add_tag(tag);
                    self.input.clear();
                    self.suggestions.clear();
                    self.selected_suggestion = None;
                    return true;
                }
                false // Allow tab to move to next field if no suggestion
            }

            // Up/Down navigate suggestions
            KeyCode::Up => {
                if !self.suggestions.is_empty() {
                    self.selected_suggestion = Some(
                        self.selected_suggestion
                            .map(|i| if i == 0 { self.suggestions.len() - 1 } else { i - 1 })
                            .unwrap_or(0),
                    );
                    true
                } else {
                    false
                }
            }

            KeyCode::Down => {
                if !self.suggestions.is_empty() {
                    self.selected_suggestion = Some(
                        self.selected_suggestion
                            .map(|i| (i + 1) % self.suggestions.len())
                            .unwrap_or(0),
                    );
                    true
                } else {
                    false
                }
            }

            // Regular character input
            KeyCode::Char(c) => {
                self.input.insert(c);
                self.update_suggestions(all_tags);
                true
            }

            // Delete key
            KeyCode::Delete => {
                self.input.delete_forward();
                self.update_suggestions(all_tags);
                true
            }

            // Cursor movement
            KeyCode::Left => {
                self.input.move_left();
                true
            }
            KeyCode::Right => {
                self.input.move_right();
                true
            }
            KeyCode::Home => {
                self.input.move_home();
                true
            }
            KeyCode::End => {
                self.input.move_end();
                true
            }

            _ => false,
        }
    }

    /// Renders the tag input to a buffer.
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
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        if let Some(label) = label {
            block = block.title(format!(" {} ", label));
        }

        let inner = block.inner(area);
        block.render(area, buf);

        // Build the content line with tags and input
        let mut spans: Vec<Span> = Vec::new();

        // Render existing tags as chips
        for tag in &self.tags {
            spans.push(Span::styled(
                format!(" #{} ", tag),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(" "));
        }

        // Render the input text or placeholder
        let input_text = if self.input.value().is_empty() && self.tags.is_empty() {
            Span::styled(&self.placeholder, Style::default().fg(Color::DarkGray))
        } else if self.input.value().is_empty() {
            Span::raw("")
        } else {
            Span::styled(
                self.input.value().to_string(),
                if focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Gray)
                },
            )
        };
        spans.push(input_text);

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        paragraph.render(inner, buf);

        // Render cursor if focused and there's room
        if focused && inner.width > 0 {
            // Calculate cursor position (after all tags)
            let tags_width: usize = self.tags.iter().map(|t| t.len() + 4).sum(); // " #tag "
            let cursor_x = inner.x + tags_width as u16 + self.input.cursor() as u16;

            if cursor_x < inner.x + inner.width {
                let cursor_char = self.input.value().chars().nth(self.input.cursor()).unwrap_or(' ');
                buf[(cursor_x, inner.y)]
                    .set_char(cursor_char)
                    .set_style(Style::default().bg(Color::Yellow).fg(Color::Black));
            }
        }
    }

    /// Renders autocomplete suggestions below the input area.
    pub fn render_suggestions(&self, frame: &mut ratatui::Frame, area: Rect) {
        if self.suggestions.is_empty() {
            return;
        }

        let suggestion_height = self.suggestions.len() as u16 + 2; // +2 for borders
        let suggestion_area = Rect::new(
            area.x,
            area.y + area.height,
            area.width.min(30),
            suggestion_height.min(7),
        );

        // Check if we have room below
        let frame_area = frame.area();
        if suggestion_area.y + suggestion_area.height > frame_area.height {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(suggestion_area);

        // Render block background
        frame.render_widget(ratatui::widgets::Clear, suggestion_area);
        frame.render_widget(block, suggestion_area);

        // Render suggestions
        let mut y = inner.y;
        for (i, suggestion) in self.suggestions.iter().enumerate() {
            let style = if Some(i) == self.selected_suggestion {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let text = format!(" #{}", suggestion);
            let paragraph = Paragraph::new(text).style(style);
            let line_area = Rect::new(inner.x, y, inner.width, 1);
            frame.render_widget(paragraph, line_area);
            y += 1;
        }
    }
}

impl Default for TagInput {
    fn default() -> Self {
        Self::new()
    }
}

/// A renderable version of TagInput for use as a Widget.
pub struct TagInputWidget<'a> {
    input: &'a TagInput,
    focused: bool,
    label: Option<&'a str>,
}

impl<'a> TagInputWidget<'a> {
    /// Creates a new tag input widget.
    pub fn new(input: &'a TagInput) -> Self {
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

impl Widget for TagInputWidget<'_> {
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
        let input = TagInput::new();
        assert!(input.tags().is_empty());
        assert!(input.input_value().is_empty());
    }

    #[test]
    fn test_add_tag() {
        let mut input = TagInput::new();
        input.add_tag("work".to_string());
        input.add_tag("urgent".to_string());
        assert_eq!(input.tags(), &["work", "urgent"]);
    }

    #[test]
    fn test_add_duplicate_tag() {
        let mut input = TagInput::new();
        input.add_tag("work".to_string());
        input.add_tag("Work".to_string()); // Same tag, different case
        assert_eq!(input.tags().len(), 1);
    }

    #[test]
    fn test_add_empty_tag() {
        let mut input = TagInput::new();
        input.add_tag("".to_string());
        input.add_tag("   ".to_string());
        assert!(input.tags().is_empty());
    }

    #[test]
    fn test_remove_last_tag() {
        let mut input = TagInput::with_tags(vec!["a".to_string(), "b".to_string()]);
        input.remove_last_tag();
        assert_eq!(input.tags(), &["a"]);
    }

    #[test]
    fn test_remove_tag_by_name() {
        let mut input = TagInput::with_tags(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        input.remove_tag("b");
        assert_eq!(input.tags(), &["a", "c"]);
    }

    #[test]
    fn test_update_suggestions() {
        let all_tags = vec![
            Tag { id: "1".to_string(), name: "work".to_string() },
            Tag { id: "2".to_string(), name: "workout".to_string() },
            Tag { id: "3".to_string(), name: "personal".to_string() },
        ];

        let mut input = TagInput::new();
        input.input.set_value("wor");
        input.update_suggestions(&all_tags);

        assert_eq!(input.suggestions.len(), 2);
        assert!(input.suggestions.contains(&"work".to_string()));
        assert!(input.suggestions.contains(&"workout".to_string()));
    }

    #[test]
    fn test_suggestions_exclude_already_selected() {
        let all_tags = vec![
            Tag { id: "1".to_string(), name: "work".to_string() },
            Tag { id: "2".to_string(), name: "workout".to_string() },
        ];

        let mut input = TagInput::with_tags(vec!["work".to_string()]);
        input.input.set_value("wor");
        input.update_suggestions(&all_tags);

        assert_eq!(input.suggestions.len(), 1);
        assert!(input.suggestions.contains(&"workout".to_string()));
    }
}
