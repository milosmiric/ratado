//! Confirmation dialog for destructive actions.
//!
//! This dialog presents a yes/no choice to the user, typically used
//! for confirming deletions or other irreversible actions.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::ui::dialogs::{centered_rect, DialogAction};

/// A yes/no confirmation dialog.
#[derive(Debug)]
pub struct ConfirmDialog {
    /// Dialog title
    pub title: String,
    /// Message to display
    pub message: String,
    /// Text for confirm button
    pub confirm_text: String,
    /// Text for cancel button
    pub cancel_text: String,
    /// Whether "Yes" is currently selected (false = "No" selected)
    pub selected_yes: bool,
    /// Whether this is a destructive action (affects styling)
    pub destructive: bool,
}

impl ConfirmDialog {
    /// Creates a new confirmation dialog.
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_text: "Yes".to_string(),
            cancel_text: "No".to_string(),
            selected_yes: false, // Default to "No" for safety
            destructive: false,
        }
    }

    /// Creates a delete confirmation dialog for a task.
    pub fn delete_task(task_title: &str) -> Self {
        Self {
            title: "Delete Task?".to_string(),
            message: format!(
                "\"{}\"\n\nThis action cannot be undone.",
                task_title
            ),
            confirm_text: "Delete".to_string(),
            cancel_text: "Cancel".to_string(),
            selected_yes: false,
            destructive: true,
        }
    }

    /// Sets the confirm button text.
    pub fn with_confirm_text(mut self, text: impl Into<String>) -> Self {
        self.confirm_text = text.into();
        self
    }

    /// Sets the cancel button text.
    pub fn with_cancel_text(mut self, text: impl Into<String>) -> Self {
        self.cancel_text = text.into();
        self
    }

    /// Marks this as a destructive action (red confirm button).
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    /// Handles a key event and returns the resulting action.
    pub fn handle_key(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            // Direct yes/no with y/n keys
            KeyCode::Char('y') | KeyCode::Char('Y') => DialogAction::Submit,
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => DialogAction::Cancel,

            // Toggle selection with arrow keys or tab
            KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::Char('h') | KeyCode::Char('l') => {
                self.selected_yes = !self.selected_yes;
                DialogAction::None
            }

            // Confirm selection with Enter
            KeyCode::Enter => {
                if self.selected_yes {
                    DialogAction::Submit
                } else {
                    DialogAction::Cancel
                }
            }

            _ => DialogAction::None,
        }
    }

    /// Renders the dialog to the frame.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Dialog dimensions
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 10.min(area.height.saturating_sub(4));
        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Render background
        frame.render_widget(Clear, area);
        let dim_block = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(dim_block, area);

        // Dialog box
        let border_color = if self.destructive {
            Color::Red
        } else {
            Color::Cyan
        };

        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Layout
        let chunks = Layout::vertical([
            Constraint::Min(3),    // Message
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Buttons
            Constraint::Length(1), // Hint
        ])
        .split(inner);

        // Message
        let message = Paragraph::new(self.message.as_str())
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(message, chunks[0]);

        // Buttons
        let yes_style = if self.selected_yes {
            if self.destructive {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(Color::Gray)
        };

        let no_style = if !self.selected_yes {
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let buttons = Line::from(vec![
            Span::styled(format!(" {} ", self.confirm_text), yes_style),
            Span::raw("    "),
            Span::styled(format!(" {} ", self.cancel_text), no_style),
        ]);

        let button_paragraph = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(button_paragraph, chunks[2]);

        // Hint
        let hint = Paragraph::new("y/n or ←/→ to select, Enter to confirm")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_new_dialog() {
        let dialog = ConfirmDialog::new("Test", "Are you sure?");
        assert_eq!(dialog.title, "Test");
        assert_eq!(dialog.message, "Are you sure?");
        assert!(!dialog.selected_yes);
    }

    #[test]
    fn test_delete_task_dialog() {
        let dialog = ConfirmDialog::delete_task("My Task");
        assert!(dialog.title.contains("Delete"));
        assert!(dialog.message.contains("My Task"));
        assert!(dialog.destructive);
    }

    #[test]
    fn test_handle_key_yes() {
        let mut dialog = ConfirmDialog::new("Test", "Message");
        let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        assert_eq!(dialog.handle_key(key), DialogAction::Submit);
    }

    #[test]
    fn test_handle_key_no() {
        let mut dialog = ConfirmDialog::new("Test", "Message");
        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        assert_eq!(dialog.handle_key(key), DialogAction::Cancel);
    }

    #[test]
    fn test_handle_key_escape() {
        let mut dialog = ConfirmDialog::new("Test", "Message");
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(dialog.handle_key(key), DialogAction::Cancel);
    }

    #[test]
    fn test_handle_key_toggle() {
        let mut dialog = ConfirmDialog::new("Test", "Message");
        assert!(!dialog.selected_yes);

        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        dialog.handle_key(key);
        assert!(dialog.selected_yes);

        dialog.handle_key(key);
        assert!(!dialog.selected_yes);
    }

    #[test]
    fn test_handle_key_enter_yes() {
        let mut dialog = ConfirmDialog::new("Test", "Message");
        dialog.selected_yes = true;

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(dialog.handle_key(key), DialogAction::Submit);
    }

    #[test]
    fn test_handle_key_enter_no() {
        let mut dialog = ConfirmDialog::new("Test", "Message");
        dialog.selected_yes = false;

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(dialog.handle_key(key), DialogAction::Cancel);
    }
}
