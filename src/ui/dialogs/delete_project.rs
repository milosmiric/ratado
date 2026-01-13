//! Delete project dialog with task handling options.
//!
//! When deleting a project, users can choose to either delete all tasks
//! in the project or move them to Inbox.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::ui::dialogs::{centered_rect, DialogAction};

/// Action to take with tasks when deleting a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeleteProjectChoice {
    /// Move tasks to Inbox (safe option)
    #[default]
    MoveToInbox,
    /// Delete all tasks in the project
    DeleteTasks,
    /// Cancel the deletion
    Cancel,
}

/// Dialog for confirming project deletion with task handling choice.
#[derive(Debug)]
pub struct DeleteProjectDialog {
    /// ID of the project to delete
    pub project_id: String,
    /// Name of the project (for display)
    pub project_name: String,
    /// Number of tasks in the project
    pub task_count: usize,
    /// Currently selected choice
    pub selected: DeleteProjectChoice,
}

impl DeleteProjectDialog {
    /// Creates a new delete project dialog.
    pub fn new(project_id: String, project_name: String, task_count: usize) -> Self {
        Self {
            project_id,
            project_name,
            task_count,
            selected: DeleteProjectChoice::MoveToInbox,
        }
    }

    /// Returns the selected choice.
    pub fn choice(&self) -> DeleteProjectChoice {
        self.selected
    }

    /// Handles a key event and returns the resulting action.
    pub fn handle_key(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            // Direct shortcuts
            KeyCode::Char('m') | KeyCode::Char('M') => {
                self.selected = DeleteProjectChoice::MoveToInbox;
                DialogAction::Submit
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.selected = DeleteProjectChoice::DeleteTasks;
                DialogAction::Submit
            }
            KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('C') => {
                self.selected = DeleteProjectChoice::Cancel;
                DialogAction::Cancel
            }

            // Navigate between options with arrows/vim keys
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = match self.selected {
                    DeleteProjectChoice::MoveToInbox => DeleteProjectChoice::Cancel,
                    DeleteProjectChoice::DeleteTasks => DeleteProjectChoice::MoveToInbox,
                    DeleteProjectChoice::Cancel => DeleteProjectChoice::DeleteTasks,
                };
                DialogAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = match self.selected {
                    DeleteProjectChoice::MoveToInbox => DeleteProjectChoice::DeleteTasks,
                    DeleteProjectChoice::DeleteTasks => DeleteProjectChoice::Cancel,
                    DeleteProjectChoice::Cancel => DeleteProjectChoice::MoveToInbox,
                };
                DialogAction::None
            }

            // Confirm with Enter
            KeyCode::Enter => {
                if self.selected == DeleteProjectChoice::Cancel {
                    DialogAction::Cancel
                } else {
                    DialogAction::Submit
                }
            }

            _ => DialogAction::None,
        }
    }

    /// Renders the dialog to the frame.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Dialog dimensions
        let dialog_width = 55.min(area.width.saturating_sub(4));
        let dialog_height = 14.min(area.height.saturating_sub(4));
        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Render background
        frame.render_widget(Clear, area);
        let dim_block = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(dim_block, area);

        // Dialog box
        let block = Block::default()
            .title(" Delete Project ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Layout
        let chunks = Layout::vertical([
            Constraint::Length(2), // Message
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Option 1
            Constraint::Length(1), // Option 2
            Constraint::Length(1), // Option 3
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Hint
        ])
        .split(inner);

        // Message
        let task_text = if self.task_count == 1 {
            "1 task".to_string()
        } else {
            format!("{} tasks", self.task_count)
        };

        let message = format!(
            "Delete project \"{}\"?\nThis project has {}.",
            self.project_name, task_text
        );
        let message_widget = Paragraph::new(message)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(message_widget, chunks[0]);

        // Options
        let options = [
            (DeleteProjectChoice::MoveToInbox, "Move tasks to Inbox", "m", Color::Green),
            (DeleteProjectChoice::DeleteTasks, "Delete all tasks", "d", Color::Red),
            (DeleteProjectChoice::Cancel, "Cancel", "c", Color::Gray),
        ];

        for (i, (choice, label, key, color)) in options.iter().enumerate() {
            let is_selected = self.selected == *choice;

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(*color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let key_style = Style::default()
                .fg(*color)
                .add_modifier(Modifier::BOLD);

            let prefix = if is_selected { ">" } else { " " };

            let line = Line::from(vec![
                Span::styled(format!(" {} ", prefix), style),
                Span::styled(format!("[{}] ", key), key_style),
                Span::styled(format!("{} ", label), style),
            ]);

            let option = Paragraph::new(line).alignment(Alignment::Center);
            frame.render_widget(option, chunks[2 + i]);
        }

        // Hint
        let hint = Paragraph::new("j/k to navigate, Enter to confirm")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[6]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_new_dialog() {
        let dialog = DeleteProjectDialog::new(
            "proj-1".to_string(),
            "Work".to_string(),
            5,
        );
        assert_eq!(dialog.project_id, "proj-1");
        assert_eq!(dialog.project_name, "Work");
        assert_eq!(dialog.task_count, 5);
        assert_eq!(dialog.selected, DeleteProjectChoice::MoveToInbox);
    }

    #[test]
    fn test_move_shortcut() {
        let mut dialog = DeleteProjectDialog::new(
            "proj-1".to_string(),
            "Work".to_string(),
            5,
        );
        let key = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE);
        let action = dialog.handle_key(key);
        assert_eq!(action, DialogAction::Submit);
        assert_eq!(dialog.selected, DeleteProjectChoice::MoveToInbox);
    }

    #[test]
    fn test_delete_shortcut() {
        let mut dialog = DeleteProjectDialog::new(
            "proj-1".to_string(),
            "Work".to_string(),
            5,
        );
        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        let action = dialog.handle_key(key);
        assert_eq!(action, DialogAction::Submit);
        assert_eq!(dialog.selected, DeleteProjectChoice::DeleteTasks);
    }

    #[test]
    fn test_cancel_shortcut() {
        let mut dialog = DeleteProjectDialog::new(
            "proj-1".to_string(),
            "Work".to_string(),
            5,
        );
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = dialog.handle_key(key);
        assert_eq!(action, DialogAction::Cancel);
    }

    #[test]
    fn test_navigation() {
        let mut dialog = DeleteProjectDialog::new(
            "proj-1".to_string(),
            "Work".to_string(),
            5,
        );
        assert_eq!(dialog.selected, DeleteProjectChoice::MoveToInbox);

        // Navigate down
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        dialog.handle_key(key);
        assert_eq!(dialog.selected, DeleteProjectChoice::DeleteTasks);

        // Navigate down again
        dialog.handle_key(key);
        assert_eq!(dialog.selected, DeleteProjectChoice::Cancel);

        // Navigate up
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        dialog.handle_key(key);
        assert_eq!(dialog.selected, DeleteProjectChoice::DeleteTasks);
    }
}
