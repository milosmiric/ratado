//! Project management dialog.
//!
//! This dialog handles creating new projects and editing existing ones.
//! Projects have a name, color, and icon.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

use super::{
    button_focused_style, button_style, centered_rect, dialog_block, field_block, DialogAction,
};
use crate::models::Project;
use crate::ui::input::TextInput;
use crate::ui::theme;

/// Preset colors for projects.
const PROJECT_COLORS: &[(&str, &str)] = &[
    ("#3498db", "Blue"),
    ("#e74c3c", "Red"),
    ("#2ecc71", "Green"),
    ("#f39c12", "Orange"),
    ("#9b59b6", "Purple"),
    ("#1abc9c", "Teal"),
    ("#e91e63", "Pink"),
    ("#607d8b", "Gray"),
    ("#f1c40f", "Yellow"),
    ("#00bcd4", "Cyan"),
    ("#8bc34a", "Lime"),
    ("#ff5722", "Coral"),
    ("#795548", "Brown"),
    ("#6366f1", "Indigo"),
    ("#ec4899", "Rose"),
    ("#64748b", "Slate"),
];

/// Number of items per row in color/icon selectors.
const ITEMS_PER_ROW: usize = 8;

/// Preset icons for projects.
const PROJECT_ICONS: &[&str] = &[
    "📁", "📋", "🏠", "💼", "📚", "🎯", "💡", "⭐",
    "🚀", "🔧", "🎨", "🎵", "❤️", "🌱", "🏗️", "📦",
];

/// The currently focused field in the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProjectField {
    #[default]
    Name,
    Color,
    Icon,
    Submit,
}

impl ProjectField {
    /// Returns the next field in tab order.
    fn next(self) -> Self {
        match self {
            Self::Name => Self::Color,
            Self::Color => Self::Icon,
            Self::Icon => Self::Submit,
            Self::Submit => Self::Name,
        }
    }

    /// Returns the previous field in tab order.
    fn prev(self) -> Self {
        match self {
            Self::Name => Self::Submit,
            Self::Color => Self::Name,
            Self::Icon => Self::Color,
            Self::Submit => Self::Icon,
        }
    }
}

/// Dialog for creating or editing a project.
#[derive(Debug, Clone)]
pub struct ProjectDialog {
    /// Name input field
    pub name: TextInput,
    /// Selected color index
    pub selected_color: usize,
    /// Selected icon index
    pub selected_icon: usize,
    /// Currently focused field
    pub focused_field: ProjectField,
    /// ID of project being edited (None for new project)
    pub editing_project_id: Option<String>,
    /// Title for the dialog
    dialog_title: String,
}

impl ProjectDialog {
    /// Creates a new dialog for adding a project.
    pub fn new() -> Self {
        Self {
            name: TextInput::new().with_placeholder("Project name..."),
            selected_color: 0,
            selected_icon: 0,
            focused_field: ProjectField::Name,
            editing_project_id: None,
            dialog_title: "New Project".to_string(),
        }
    }

    /// Creates a dialog pre-populated with an existing project for editing.
    pub fn from_project(project: &Project) -> Self {
        // Find color index
        let color_idx = PROJECT_COLORS
            .iter()
            .position(|(c, _)| *c == project.color)
            .unwrap_or(0);

        // Find icon index
        let icon_idx = PROJECT_ICONS
            .iter()
            .position(|i| *i == project.icon)
            .unwrap_or(0);

        Self {
            name: TextInput::with_value(&project.name),
            selected_color: color_idx,
            selected_icon: icon_idx,
            focused_field: ProjectField::Name,
            editing_project_id: Some(project.id.clone()),
            dialog_title: "Edit Project".to_string(),
        }
    }

    /// Returns whether this is editing an existing project.
    pub fn is_editing(&self) -> bool {
        self.editing_project_id.is_some()
    }

    /// Handles a key event and returns the resulting action.
    pub fn handle_key(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            // Cancel on Escape
            KeyCode::Esc => DialogAction::Cancel,

            // Submit on Enter when on Submit button, or Ctrl+Enter anywhere
            KeyCode::Enter if self.focused_field == ProjectField::Submit => DialogAction::Submit,
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => DialogAction::Submit,

            // Tab navigation
            KeyCode::Tab => {
                self.focused_field = self.focused_field.next();
                DialogAction::None
            }
            KeyCode::BackTab => {
                self.focused_field = self.focused_field.prev();
                DialogAction::None
            }

            // Field-specific handling
            _ => {
                match self.focused_field {
                    ProjectField::Name => self.handle_name_input(key),
                    ProjectField::Color => self.handle_color_input(key),
                    ProjectField::Icon => self.handle_icon_input(key),
                    ProjectField::Submit => {
                        // Enter handled above, other keys do nothing
                        DialogAction::None
                    }
                }
            }
        }
    }

    /// Handles input for the name field.
    fn handle_name_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Char(c) => self.name.insert(c),
            KeyCode::Backspace => self.name.delete_backward(),
            KeyCode::Delete => self.name.delete_forward(),
            KeyCode::Left => self.name.move_left(),
            KeyCode::Right => self.name.move_right(),
            KeyCode::Home => self.name.move_home(),
            KeyCode::End => self.name.move_end(),
            _ => {}
        }
        DialogAction::None
    }

    /// Handles input for the color field.
    fn handle_color_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                if self.selected_color > 0 {
                    self.selected_color -= 1;
                } else {
                    self.selected_color = PROJECT_COLORS.len() - 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.selected_color = (self.selected_color + 1) % PROJECT_COLORS.len();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_color >= ITEMS_PER_ROW {
                    self.selected_color -= ITEMS_PER_ROW;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_color + ITEMS_PER_ROW < PROJECT_COLORS.len() {
                    self.selected_color += ITEMS_PER_ROW;
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let idx = c.to_digit(10).unwrap() as usize;
                if idx > 0 && idx <= ITEMS_PER_ROW {
                    self.selected_color = idx - 1;
                }
            }
            _ => {}
        }
        DialogAction::None
    }

    /// Handles input for the icon field.
    fn handle_icon_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                if self.selected_icon > 0 {
                    self.selected_icon -= 1;
                } else {
                    self.selected_icon = PROJECT_ICONS.len() - 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.selected_icon = (self.selected_icon + 1) % PROJECT_ICONS.len();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_icon >= ITEMS_PER_ROW {
                    self.selected_icon -= ITEMS_PER_ROW;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_icon + ITEMS_PER_ROW < PROJECT_ICONS.len() {
                    self.selected_icon += ITEMS_PER_ROW;
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let idx = c.to_digit(10).unwrap() as usize;
                if idx > 0 && idx <= ITEMS_PER_ROW {
                    self.selected_icon = idx - 1;
                }
            }
            _ => {}
        }
        DialogAction::None
    }

    /// Creates a Project from the dialog fields.
    ///
    /// Returns None if the name is empty.
    pub fn to_project(&self) -> Option<Project> {
        let name = self.name.value().trim();
        if name.is_empty() {
            return None;
        }

        let (color, _) = PROJECT_COLORS[self.selected_color];
        let icon = PROJECT_ICONS[self.selected_icon];

        let project = if let Some(ref id) = self.editing_project_id {
            Project {
                id: id.clone(),
                name: name.to_string(),
                color: color.to_string(),
                icon: icon.to_string(),
                created_at: chrono::Utc::now(), // Will be ignored on update
            }
        } else {
            Project::with_style(name, color, icon)
        };

        Some(project)
    }

    /// Renders the dialog to the frame.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Dialog dimensions
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 18.min(area.height.saturating_sub(4));
        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Render dimmed background
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new("").style(Style::default().bg(theme::BG_DARK)),
            area,
        );

        // Render dialog box with themed styling
        let block = dialog_block(&self.dialog_title, false);
        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Layout the fields
        let chunks = Layout::vertical([
            Constraint::Length(3), // Name
            Constraint::Length(5), // Color (border + 2 rows + hint)
            Constraint::Length(5), // Icon (border + 2 rows + hint)
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Submit button
        ])
        .split(inner);

        // Render name field
        self.render_name_field(frame, chunks[0], self.focused_field == ProjectField::Name);

        // Render color selector
        self.render_color_selector(frame, chunks[1], self.focused_field == ProjectField::Color);

        // Render icon selector
        self.render_icon_selector(frame, chunks[2], self.focused_field == ProjectField::Icon);

        // Render submit button
        self.render_submit_button(frame, chunks[4], self.focused_field == ProjectField::Submit);
    }

    /// Renders the name input field.
    fn render_name_field(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let buf = frame.buffer_mut();
        self.name.render_to_buffer(area, buf, focused, Some("Name"));
    }

    /// Renders the color selector with colored circles in 2 rows of 8.
    fn render_color_selector(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let block = field_block("Color", focused);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Build two rows of color circles
        for row in 0..2 {
            let mut spans = Vec::new();
            for col in 0..ITEMS_PER_ROW {
                let i = row * ITEMS_PER_ROW + col;
                if i >= PROJECT_COLORS.len() {
                    break;
                }
                if col > 0 {
                    spans.push(Span::raw(" "));
                }

                let (color_hex, _) = PROJECT_COLORS[i];
                let color = parse_hex_color(color_hex);
                let style = if i == self.selected_color {
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default().fg(color)
                };

                spans.push(Span::styled(" ● ", style));
            }

            if inner.height > row as u16 {
                let row_area = Rect::new(inner.x, inner.y + row as u16, inner.width, 1);
                frame.render_widget(Paragraph::new(Line::from(spans)), row_area);
            }
        }

        // Hint line showing selected color name
        if inner.height > 2 {
            let (_, color_name) = PROJECT_COLORS[self.selected_color];
            let hint_area = Rect::new(inner.x, inner.y + 2, inner.width, 1);
            frame.render_widget(
                Paragraph::new(Span::styled(
                    color_name,
                    Style::default().fg(theme::TEXT_MUTED),
                )),
                hint_area,
            );
        }
    }

    /// Renders the icon selector with icons in 2 rows of 8.
    fn render_icon_selector(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let block = field_block("Icon", focused);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Build two rows of icons
        for row in 0..2 {
            let mut spans = Vec::new();
            for col in 0..ITEMS_PER_ROW {
                let i = row * ITEMS_PER_ROW + col;
                if i >= PROJECT_ICONS.len() {
                    break;
                }
                if col > 0 {
                    spans.push(Span::raw(" "));
                }

                let style = if i == self.selected_icon {
                    Style::default()
                        .fg(theme::PRIMARY_LIGHT)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default().fg(theme::TEXT_MUTED)
                };

                spans.push(Span::styled(format!(" {} ", PROJECT_ICONS[i]), style));
            }

            if inner.height > row as u16 {
                let row_area = Rect::new(inner.x, inner.y + row as u16, inner.width, 1);
                frame.render_widget(Paragraph::new(Line::from(spans)), row_area);
            }
        }

        // Hint line showing selected icon
        if inner.height > 2 {
            let icon = PROJECT_ICONS[self.selected_icon];
            let hint_area = Rect::new(inner.x, inner.y + 2, inner.width, 1);
            frame.render_widget(
                Paragraph::new(Span::styled(
                    format!("Selected: {}", icon),
                    Style::default().fg(theme::TEXT_MUTED),
                )),
                hint_area,
            );
        }
    }

    /// Renders the submit button.
    fn render_submit_button(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let text = if self.is_editing() {
            "[ Save Project ]"
        } else {
            "[ Create Project ]"
        };

        let style = if focused {
            button_focused_style()
        } else {
            button_style().fg(theme::SUCCESS)
        };

        // Center the button
        let button_width = text.len() as u16;
        let x_offset = (area.width.saturating_sub(button_width)) / 2;
        let button_area = Rect::new(area.x + x_offset, area.y, button_width, 1);

        frame.render_widget(Paragraph::new(text).style(style), button_area);
    }
}

impl Default for ProjectDialog {
    fn default() -> Self {
        Self::new()
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

    #[test]
    fn test_new_dialog() {
        let dialog = ProjectDialog::new();
        assert!(dialog.name.value().is_empty());
        assert!(dialog.editing_project_id.is_none());
        assert!(!dialog.is_editing());
    }

    #[test]
    fn test_from_project() {
        let project = Project::with_style("Work", "#e74c3c", "📋");

        let dialog = ProjectDialog::from_project(&project);
        assert_eq!(dialog.name.value(), "Work");
        assert_eq!(dialog.selected_color, 1); // Red
        assert_eq!(dialog.selected_icon, 1); // 📋
        assert!(dialog.is_editing());
    }

    #[test]
    fn test_to_project_empty_name() {
        let dialog = ProjectDialog::new();
        assert!(dialog.to_project().is_none());
    }

    #[test]
    fn test_to_project_with_name() {
        let mut dialog = ProjectDialog::new();
        dialog.name.set_value("Home");
        dialog.selected_color = 2; // Green
        dialog.selected_icon = 2; // 🏠

        let project = dialog.to_project().unwrap();
        assert_eq!(project.name, "Home");
        assert_eq!(project.color, "#2ecc71");
        assert_eq!(project.icon, "🏠");
    }

    #[test]
    fn test_field_navigation() {
        let field = ProjectField::Name;
        assert_eq!(field.next(), ProjectField::Color);
        assert_eq!(field.prev(), ProjectField::Submit);
    }

    #[test]
    fn test_handle_key_escape() {
        let mut dialog = ProjectDialog::new();
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(dialog.handle_key(key), DialogAction::Cancel);
    }

    #[test]
    fn test_handle_key_tab() {
        let mut dialog = ProjectDialog::new();
        assert_eq!(dialog.focused_field, ProjectField::Name);

        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        dialog.handle_key(key);
        assert_eq!(dialog.focused_field, ProjectField::Color);
    }

    #[test]
    fn test_parse_hex_color() {
        let color = parse_hex_color("#3498db");
        assert!(matches!(color, Color::Rgb(52, 152, 219)));
    }
}
