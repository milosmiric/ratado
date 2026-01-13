//! User interface module.
//!
//! This module contains all Ratatui widgets and views for the terminal UI.
//! It handles rendering the application state to the terminal.
//!
//! ## Submodules
//!
//! - [`layout`] - Screen layout and panel management
//! - [`header`] - Application header with title and badges
//! - [`sidebar`] - Projects and tags sidebar
//! - [`task_list`] - Main task list display
//! - [`status_bar`] - Bottom status bar with keybindings
//! - [`help`] - Help screen with keybinding reference
//! - [`debug`] - Debug log viewer
//! - [`dialogs`] - Modal dialogs for task operations
//! - [`input`] - Text input widget
//! - [`tag_input`] - Tag input widget with autocomplete
//! - [`date_picker`] - Calendar date picker widget
//! - [`description_textarea`] - Multi-line textarea with link support

pub mod date_picker;
mod debug;
pub mod description_textarea;
pub mod dialogs;
mod header;
mod help;
pub mod input;
mod layout;
mod sidebar;
mod status_bar;
pub mod tag_input;
pub mod task_list;

use ratatui::Frame;

use crate::app::{App, View};

/// Main draw function - entry point for all UI rendering.
///
/// Routes to the appropriate view renderer based on the current view.
/// If a dialog is active, it renders on top of the main view.
pub fn draw(frame: &mut Frame, app: &App) {
    // First render the base view
    match app.current_view {
        View::Main => layout::render_main_view(frame, app, frame.area()),
        View::Help => help::render_help(frame, app, frame.area()),
        View::DebugLogs => debug::render_debug_logs(frame, app, frame.area()),
        // Other views fall back to main for now
        _ => layout::render_main_view(frame, app, frame.area()),
    }

    // Then render any active dialog on top
    if let Some(ref dialog) = app.dialog {
        dialog.render(frame);
    }
}
