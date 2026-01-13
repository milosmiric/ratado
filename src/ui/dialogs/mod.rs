//! Dialog widgets for task operations.
//!
//! This module provides modal dialogs for user interactions like
//! adding tasks, editing tasks, and confirming deletions.
//!
//! ## Dialog Types
//!
//! - [`AddTaskDialog`] - Create or edit a task
//! - [`ConfirmDialog`] - Yes/No confirmation prompts
//! - [`FilterSortDialog`] - Filter and sort selection
//!
//! ## Usage
//!
//! Dialogs are stored in the App state and rendered on top of the main view.
//! Each dialog handles its own key events and returns a [`DialogAction`]
//! indicating what happened.

mod add_task;
mod confirm;
mod filter_sort;

pub use add_task::AddTaskDialog;
pub use confirm::ConfirmDialog;
pub use filter_sort::FilterSortDialog;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Widget},
    Frame,
};

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
    /// Add or edit task dialog
    AddTask(AddTaskDialog),
    /// Confirmation dialog (for delete, etc.)
    Confirm(ConfirmDialog),
    /// Filter and sort selection dialog
    FilterSort(FilterSortDialog),
}

impl Dialog {
    /// Renders the dialog to the frame.
    pub fn render(&self, frame: &mut Frame) {
        match self {
            Dialog::AddTask(dialog) => dialog.render(frame),
            Dialog::Confirm(dialog) => dialog.render(frame),
            Dialog::FilterSort(dialog) => dialog.render(frame),
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

    // Fill with dim background
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            buf[(x, y)].set_style(Style::default().bg(Color::Black));
        }
    }
}

/// Renders a dialog box with border.
pub fn render_dialog_box(area: Rect, buf: &mut Buffer, title: &str) {
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    block.render(area, buf);
}
