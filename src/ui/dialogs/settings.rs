//! Settings dialog for application configuration.
//!
//! This dialog provides options for managing application data:
//! - Reset database (delete all data)
//! - Delete all completed tasks

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

use super::{
    button_danger_style, button_focused_style, centered_rect, dialog_block, hint_style,
    DialogAction,
};
use crate::ui::theme;

/// Available settings options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsOption {
    /// Delete all completed tasks from all projects
    DeleteCompletedTasks,
    /// Reset entire database to default state
    ResetDatabase,
}

impl SettingsOption {
    /// Returns the display label for this option.
    fn label(&self) -> &'static str {
        match self {
            SettingsOption::DeleteCompletedTasks => "Delete all completed tasks",
            SettingsOption::ResetDatabase => "Reset database (delete everything)",
        }
    }

    /// Returns whether this option is destructive.
    fn is_destructive(&self) -> bool {
        true // Both options are destructive
    }
}

/// The settings dialog state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DialogState {
    /// Selecting an option from the menu
    Menu,
    /// Confirming a destructive action
    Confirming,
}

/// Settings dialog for application configuration.
#[derive(Debug)]
pub struct SettingsDialog {
    /// Currently selected option index
    selected_index: usize,
    /// Available options
    options: Vec<SettingsOption>,
    /// Current dialog state
    state: DialogState,
    /// The option being confirmed (if in confirming state)
    confirming_option: Option<SettingsOption>,
    /// Whether "Yes" is selected in confirmation
    confirm_selected_yes: bool,
}

impl Default for SettingsDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsDialog {
    /// Creates a new settings dialog.
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            options: vec![
                SettingsOption::DeleteCompletedTasks,
                SettingsOption::ResetDatabase,
            ],
            state: DialogState::Menu,
            confirming_option: None,
            confirm_selected_yes: false,
        }
    }

    /// Returns the currently selected option.
    pub fn selected_option(&self) -> Option<SettingsOption> {
        self.options.get(self.selected_index).copied()
    }

    /// Returns the confirmed option (after user confirms).
    pub fn confirmed_option(&self) -> Option<SettingsOption> {
        self.confirming_option
    }

    /// Handles a key event and returns the resulting action.
    pub fn handle_key(&mut self, key: KeyEvent) -> DialogAction {
        match self.state {
            DialogState::Menu => self.handle_menu_key(key),
            DialogState::Confirming => self.handle_confirm_key(key),
        }
    }

    /// Handles key events in menu state.
    fn handle_menu_key(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            // Cancel
            KeyCode::Esc => DialogAction::Cancel,

            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                DialogAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.options.len() - 1 {
                    self.selected_index += 1;
                }
                DialogAction::None
            }

            // Select option
            KeyCode::Enter => {
                if let Some(option) = self.selected_option() {
                    if option.is_destructive() {
                        // Show confirmation for destructive actions
                        self.state = DialogState::Confirming;
                        self.confirming_option = Some(option);
                        self.confirm_selected_yes = false;
                    } else {
                        return DialogAction::Submit;
                    }
                }
                DialogAction::None
            }

            _ => DialogAction::None,
        }
    }

    /// Handles key events in confirmation state.
    fn handle_confirm_key(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            // Direct yes/no with y/n keys
            KeyCode::Char('y') | KeyCode::Char('Y') => DialogAction::Submit,
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                // Go back to menu
                self.state = DialogState::Menu;
                self.confirming_option = None;
                DialogAction::None
            }

            // Toggle selection
            KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::Char('h') | KeyCode::Char('l') => {
                self.confirm_selected_yes = !self.confirm_selected_yes;
                DialogAction::None
            }

            // Confirm selection
            KeyCode::Enter => {
                if self.confirm_selected_yes {
                    DialogAction::Submit
                } else {
                    // Go back to menu
                    self.state = DialogState::Menu;
                    self.confirming_option = None;
                    DialogAction::None
                }
            }

            _ => DialogAction::None,
        }
    }

    /// Renders the dialog to the frame.
    pub fn render(&self, frame: &mut Frame) {
        match self.state {
            DialogState::Menu => self.render_menu(frame),
            DialogState::Confirming => self.render_confirmation(frame),
        }
    }

    /// Renders the menu state.
    fn render_menu(&self, frame: &mut Frame) {
        let area = frame.area();

        // Dialog dimensions - fixed size for consistency
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 9.min(area.height.saturating_sub(4));
        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Render dimmed background
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new("").style(Style::default().bg(theme::BG_DARK)),
            area,
        );

        // Dialog box with themed styling
        let block = dialog_block("Settings", false);
        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Layout - simpler structure
        let chunks = Layout::vertical([
            Constraint::Length(1), // Title/spacer
            Constraint::Length(1), // Option 1
            Constraint::Length(1), // Option 2
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Hint
        ])
        .split(inner);

        // Section title
        let title = Paragraph::new("Data Management")
            .style(Style::default().fg(theme::WARNING).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Render options
        for (i, option) in self.options.iter().enumerate() {
            let is_selected = i == self.selected_index;
            let prefix = if is_selected { " ▸ " } else { "   " };

            let style = if is_selected {
                Style::default().fg(theme::ERROR).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT_PRIMARY)
            };

            let line = Paragraph::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(option.label(), style),
            ]));
            frame.render_widget(line, chunks[1 + i]);
        }

        // Hint
        let hint = Paragraph::new("↑/↓ navigate • Enter select • Esc close")
            .style(hint_style())
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[4]);
    }

    /// Renders the confirmation state.
    fn render_confirmation(&self, frame: &mut Frame) {
        let area = frame.area();

        // Dialog dimensions - taller to fit multi-line messages
        let dialog_width = 55.min(area.width.saturating_sub(4));
        let dialog_height = 11.min(area.height.saturating_sub(4));
        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Render dimmed background
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new("").style(Style::default().bg(theme::BG_DARK)),
            area,
        );

        // Dialog box with destructive styling
        let block = dialog_block("⚠ Confirm", true);
        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Layout with fixed heights
        let chunks = Layout::vertical([
            Constraint::Length(1), // Question
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Warning message
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Buttons
            Constraint::Length(1), // Hint
        ])
        .split(inner);

        // Question (first line, bold)
        let question = match self.confirming_option {
            Some(SettingsOption::DeleteCompletedTasks) => "Delete all completed tasks?",
            Some(SettingsOption::ResetDatabase) => "Reset entire database?",
            None => "",
        };
        let question_paragraph = Paragraph::new(question)
            .style(Style::default().fg(theme::TEXT_PRIMARY).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(question_paragraph, chunks[0]);

        // Warning details
        let warning = match self.confirming_option {
            Some(SettingsOption::DeleteCompletedTasks) => {
                vec![
                    Line::from(Span::styled("All completed tasks will be permanently removed.", Style::default().fg(theme::WARNING))),
                    Line::from(""),
                    Line::from(Span::styled("This action cannot be undone.", Style::default().fg(theme::ERROR))),
                ]
            }
            Some(SettingsOption::ResetDatabase) => {
                vec![
                    Line::from(Span::styled("All tasks, projects, and tags will be deleted.", Style::default().fg(theme::WARNING))),
                    Line::from(Span::styled("Only the Inbox project will remain.", Style::default().fg(theme::WARNING))),
                    Line::from(Span::styled("This action cannot be undone.", Style::default().fg(theme::ERROR))),
                ]
            }
            None => vec![],
        };
        let warning_paragraph = Paragraph::new(warning).alignment(Alignment::Center);
        frame.render_widget(warning_paragraph, chunks[2]);

        // Buttons
        let yes_style = if self.confirm_selected_yes {
            button_danger_style()
        } else {
            hint_style()
        };

        let no_style = if !self.confirm_selected_yes {
            button_focused_style()
        } else {
            hint_style()
        };

        let buttons = Line::from(vec![
            Span::styled(" Delete ", yes_style),
            Span::raw("    "),
            Span::styled(" Cancel ", no_style),
        ]);

        let button_paragraph = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(button_paragraph, chunks[4]);

        // Hint
        let hint = Paragraph::new("y/n or ←/→ select • Enter confirm")
            .style(hint_style())
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[5]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_new_dialog() {
        let dialog = SettingsDialog::new();
        assert_eq!(dialog.selected_index, 0);
        assert_eq!(dialog.state, DialogState::Menu);
    }

    #[test]
    fn test_navigation() {
        let mut dialog = SettingsDialog::new();
        assert_eq!(dialog.selected_index, 0);

        dialog.handle_key(key(KeyCode::Down));
        assert_eq!(dialog.selected_index, 1);

        dialog.handle_key(key(KeyCode::Up));
        assert_eq!(dialog.selected_index, 0);
    }

    #[test]
    fn test_enter_confirmation() {
        let mut dialog = SettingsDialog::new();

        // Press enter on first option
        let action = dialog.handle_key(key(KeyCode::Enter));
        assert_eq!(action, DialogAction::None);
        assert_eq!(dialog.state, DialogState::Confirming);
    }

    #[test]
    fn test_cancel_confirmation() {
        let mut dialog = SettingsDialog::new();

        // Enter confirmation
        dialog.handle_key(key(KeyCode::Enter));
        assert_eq!(dialog.state, DialogState::Confirming);

        // Press 'n' to cancel
        let action = dialog.handle_key(key(KeyCode::Char('n')));
        assert_eq!(action, DialogAction::None);
        assert_eq!(dialog.state, DialogState::Menu);
    }

    #[test]
    fn test_confirm_with_y() {
        let mut dialog = SettingsDialog::new();

        // Enter confirmation
        dialog.handle_key(key(KeyCode::Enter));

        // Press 'y' to confirm
        let action = dialog.handle_key(key(KeyCode::Char('y')));
        assert_eq!(action, DialogAction::Submit);
    }

    #[test]
    fn test_escape_closes() {
        let mut dialog = SettingsDialog::new();
        let action = dialog.handle_key(key(KeyCode::Esc));
        assert_eq!(action, DialogAction::Cancel);
    }
}
