//! Move to project dialog.
//!
//! A popup dialog for selecting a project to move a task to.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::models::Project;
use crate::ui::dialogs::{centered_rect, DialogAction};

/// Dialog for selecting a project to move a task to.
#[derive(Debug, Clone)]
pub struct MoveToProjectDialog {
    /// Available projects to choose from
    pub projects: Vec<Project>,
    /// Currently selected project index
    pub selected_index: usize,
    /// ID of the task being moved
    pub task_id: String,
}

impl MoveToProjectDialog {
    /// Creates a new dialog with the given projects and task.
    ///
    /// The current project (if any) will be pre-selected.
    ///
    /// # Arguments
    ///
    /// * `projects` - List of available projects
    /// * `task_id` - ID of the task being moved
    /// * `current_project_id` - Current project ID of the task (if any)
    pub fn new(projects: Vec<Project>, task_id: String, current_project_id: Option<&str>) -> Self {
        // Find the index of the current project, or default to 0 (Inbox)
        let selected_index = current_project_id
            .and_then(|id| projects.iter().position(|p| p.id == id))
            .unwrap_or(0);

        Self {
            projects,
            selected_index,
            task_id,
        }
    }

    /// Returns the currently selected project.
    pub fn selected_project(&self) -> Option<&Project> {
        self.projects.get(self.selected_index)
    }

    /// Returns the ID of the selected project.
    pub fn selected_project_id(&self) -> Option<String> {
        self.selected_project().map(|p| p.id.clone())
    }

    /// Handles a key event and returns the resulting action.
    pub fn handle_key(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            // Cancel
            KeyCode::Esc | KeyCode::Char('q') => DialogAction::Cancel,

            // Confirm selection
            KeyCode::Enter => DialogAction::Submit,

            // Navigate up
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
                DialogAction::None
            }

            // Navigate down
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.projects.is_empty() {
                    self.selected_index = (self.selected_index + 1).min(self.projects.len() - 1);
                }
                DialogAction::None
            }

            // Jump to top
            KeyCode::Home | KeyCode::Char('g') => {
                self.selected_index = 0;
                DialogAction::None
            }

            // Jump to bottom
            KeyCode::End | KeyCode::Char('G') => {
                if !self.projects.is_empty() {
                    self.selected_index = self.projects.len() - 1;
                }
                DialogAction::None
            }

            _ => DialogAction::None,
        }
    }

    /// Renders the dialog to the frame.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Calculate dialog size based on number of projects
        let content_height = self.projects.len().min(15) as u16;
        let dialog_height = content_height + 5; // +5 for borders, title, and help text
        let dialog_width = 40.min(area.width.saturating_sub(4));
        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Render background dim effect
        frame.render_widget(Clear, area);
        let dim_block = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(dim_block, area);

        // Render dialog box
        let block = Block::default()
            .title(" Move to Project ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Render project list
        let mut lines: Vec<Line> = Vec::new();

        if self.projects.is_empty() {
            lines.push(Line::from(Span::styled(
                "No projects available",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for (i, project) in self.projects.iter().enumerate() {
                let is_selected = i == self.selected_index;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if is_selected { "▶ " } else { "  " };

                // Show project color indicator
                let color_indicator = "● ";
                let color = parse_hex_color(&project.color);

                lines.push(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(color_indicator, Style::default().fg(color)),
                    Span::styled(&project.name, style),
                ]));
            }
        }

        // Add help text
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "↑↓:select  Enter:move  Esc:cancel",
            Style::default().fg(Color::DarkGray),
        )));

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }
}

/// Parses a hex color string (e.g., "#3498db") to a Color.
fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::Gray;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);

    Color::Rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn sample_projects() -> Vec<Project> {
        vec![
            Project {
                id: "inbox".to_string(),
                name: "Inbox".to_string(),
                color: "#3498db".to_string(),
                icon: "📥".to_string(),
                created_at: chrono::Utc::now(),
            },
            Project {
                id: "work".to_string(),
                name: "Work".to_string(),
                color: "#e74c3c".to_string(),
                icon: "💼".to_string(),
                created_at: chrono::Utc::now(),
            },
            Project {
                id: "personal".to_string(),
                name: "Personal".to_string(),
                color: "#2ecc71".to_string(),
                icon: "🏠".to_string(),
                created_at: chrono::Utc::now(),
            },
        ]
    }

    #[test]
    fn test_new_dialog() {
        let projects = sample_projects();
        let dialog = MoveToProjectDialog::new(projects.clone(), "task-1".to_string(), None);
        assert_eq!(dialog.selected_index, 0);
        assert_eq!(dialog.task_id, "task-1");
    }

    #[test]
    fn test_new_dialog_with_current_project() {
        let projects = sample_projects();
        let dialog = MoveToProjectDialog::new(projects.clone(), "task-1".to_string(), Some("work"));
        assert_eq!(dialog.selected_index, 1); // "work" is at index 1
    }

    #[test]
    fn test_navigation_down() {
        let projects = sample_projects();
        let mut dialog = MoveToProjectDialog::new(projects, "task-1".to_string(), None);
        assert_eq!(dialog.selected_index, 0);

        dialog.handle_key(key(KeyCode::Down));
        assert_eq!(dialog.selected_index, 1);

        dialog.handle_key(key(KeyCode::Char('j')));
        assert_eq!(dialog.selected_index, 2);

        // Should not go beyond last item
        dialog.handle_key(key(KeyCode::Down));
        assert_eq!(dialog.selected_index, 2);
    }

    #[test]
    fn test_navigation_up() {
        let projects = sample_projects();
        let mut dialog = MoveToProjectDialog::new(projects, "task-1".to_string(), Some("personal"));
        assert_eq!(dialog.selected_index, 2);

        dialog.handle_key(key(KeyCode::Up));
        assert_eq!(dialog.selected_index, 1);

        dialog.handle_key(key(KeyCode::Char('k')));
        assert_eq!(dialog.selected_index, 0);

        // Should not go below 0
        dialog.handle_key(key(KeyCode::Up));
        assert_eq!(dialog.selected_index, 0);
    }

    #[test]
    fn test_escape_cancels() {
        let projects = sample_projects();
        let mut dialog = MoveToProjectDialog::new(projects, "task-1".to_string(), None);
        assert_eq!(dialog.handle_key(key(KeyCode::Esc)), DialogAction::Cancel);
    }

    #[test]
    fn test_enter_submits() {
        let projects = sample_projects();
        let mut dialog = MoveToProjectDialog::new(projects, "task-1".to_string(), None);
        assert_eq!(dialog.handle_key(key(KeyCode::Enter)), DialogAction::Submit);
    }

    #[test]
    fn test_selected_project() {
        let projects = sample_projects();
        let mut dialog = MoveToProjectDialog::new(projects, "task-1".to_string(), None);

        assert_eq!(dialog.selected_project().unwrap().id, "inbox");

        dialog.handle_key(key(KeyCode::Down));
        assert_eq!(dialog.selected_project().unwrap().id, "work");

        assert_eq!(dialog.selected_project_id(), Some("work".to_string()));
    }

    #[test]
    fn test_home_end_navigation() {
        let projects = sample_projects();
        let mut dialog = MoveToProjectDialog::new(projects, "task-1".to_string(), Some("work"));
        assert_eq!(dialog.selected_index, 1);

        dialog.handle_key(key(KeyCode::End));
        assert_eq!(dialog.selected_index, 2);

        dialog.handle_key(key(KeyCode::Home));
        assert_eq!(dialog.selected_index, 0);
    }
}
