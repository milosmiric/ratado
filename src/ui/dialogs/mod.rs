//! Dialog widgets for task operations.
//!
//! This module provides modal dialogs for user interactions like
//! adding tasks, editing tasks, and confirming deletions.
//!
//! ## Dialog Types
//!
//! - [`AddTaskDialog`] - Create or edit a task
//! - [`ConfirmDialog`] - Yes/No confirmation prompts
//! - [`DeleteProjectDialog`] - Project deletion with task handling options
//! - [`FilterSortDialog`] - Filter and sort selection
//! - [`MoveToProjectDialog`] - Move task to different project
//! - [`ProjectDialog`] - Create or edit a project
//!
//! ## Usage
//!
//! Dialogs are stored in the App state and rendered on top of the main view.
//! Each dialog handles its own key events and returns a [`DialogAction`]
//! indicating what happened.

mod add_task;
mod confirm;
mod delete_project;
mod filter_sort;
mod move_to_project;
mod project;
mod settings;

pub use add_task::AddTaskDialog;
pub use confirm::ConfirmDialog;
pub use delete_project::{DeleteProjectChoice, DeleteProjectDialog};
pub use filter_sort::FilterSortDialog;
pub use move_to_project::MoveToProjectDialog;
pub use project::ProjectDialog;
pub use settings::{SettingsDialog, SettingsOption};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    symbols::border,
    text::Span,
    widgets::{Block, Borders, Clear, Widget},
    Frame,
};

use super::theme;

/// Actions that can result from dialog interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogAction {
    /// No action needed, dialog continues
    None,
    /// User submitted/confirmed the dialog
    Submit,
    /// User cancelled the dialog
    Cancel,
}

/// All dialog types in the application.
#[derive(Debug)]
pub enum Dialog {
    /// Add or edit task dialog (boxed due to large size)
    AddTask(Box<AddTaskDialog>),
    /// Confirmation dialog (for delete, etc.)
    Confirm(ConfirmDialog),
    /// Delete project dialog with task handling options
    DeleteProject(DeleteProjectDialog),
    /// Filter and sort selection dialog
    FilterSort(FilterSortDialog),
    /// Move task to project dialog
    MoveToProject(MoveToProjectDialog),
    /// Add or edit project dialog
    Project(ProjectDialog),
    /// Settings dialog for app configuration
    Settings(SettingsDialog),
}

impl Dialog {
    /// Renders the dialog to the frame.
    pub fn render(&self, frame: &mut Frame) {
        match self {
            Dialog::AddTask(dialog) => dialog.render(frame),
            Dialog::Confirm(dialog) => dialog.render(frame),
            Dialog::DeleteProject(dialog) => dialog.render(frame),
            Dialog::FilterSort(dialog) => dialog.render(frame),
            Dialog::MoveToProject(dialog) => dialog.render(frame),
            Dialog::Project(dialog) => dialog.render(frame),
            Dialog::Settings(dialog) => dialog.render(frame),
        }
    }
}

/// Helper to center a dialog on screen.
///
/// Returns the centered area with the specified dimensions.
pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    area
}

/// Renders a dialog background (dims the main content).
pub fn render_dialog_background(area: Rect, buf: &mut Buffer) {
    // Clear the area first
    Clear.render(area, buf);

    // Fill with dim background using theme color
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            buf[(x, y)].set_style(Style::default().bg(theme::BG_DARK));
        }
    }
}

/// Renders a themed dialog box with rounded corners.
pub fn render_dialog_box(area: Rect, buf: &mut Buffer, title: &str) {
    let block = dialog_block(title, false);
    block.render(area, buf);
}

/// Creates a themed dialog block with optional destructive styling.
pub fn dialog_block(title: &str, destructive: bool) -> Block<'static> {
    let border_color = if destructive {
        theme::ERROR
    } else {
        theme::PRIMARY_LIGHT
    };

    let title_style = Style::default()
        .fg(border_color)
        .add_modifier(Modifier::BOLD);

    Block::default()
        .title(Span::styled(format!(" {} ", title), title_style))
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme::BG_ELEVATED))
}

/// Creates a themed block for dialog sections/fields.
pub fn field_block(label: &str, focused: bool) -> Block<'static> {
    let border_color = if focused {
        theme::PRIMARY_LIGHT
    } else {
        theme::BORDER
    };

    let label_style = if focused {
        Style::default()
            .fg(theme::PRIMARY_LIGHT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::TEXT_MUTED)
    };

    Block::default()
        .title(Span::styled(format!(" {} ", label), label_style))
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(border_color))
}

/// Style for selected/highlighted items in dialogs.
pub fn selected_style() -> Style {
    Style::default()
        .bg(theme::BG_SELECTION)
        .fg(theme::TEXT_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Style for unselected items in dialogs.
pub fn unselected_style() -> Style {
    Style::default().fg(theme::TEXT_SECONDARY)
}

/// Style for muted/hint text in dialogs.
pub fn hint_style() -> Style {
    Style::default().fg(theme::TEXT_MUTED)
}

/// Style for focused buttons.
pub fn button_focused_style() -> Style {
    Style::default()
        .bg(theme::PRIMARY)
        .fg(theme::TEXT_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Style for unfocused buttons.
pub fn button_style() -> Style {
    Style::default()
        .fg(theme::TEXT_SECONDARY)
}

/// Style for destructive/danger buttons when focused.
pub fn button_danger_style() -> Style {
    Style::default()
        .bg(theme::ERROR)
        .fg(theme::TEXT_PRIMARY)
        .add_modifier(Modifier::BOLD)
}
