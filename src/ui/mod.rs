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

mod debug;
mod header;
mod help;
mod layout;
mod sidebar;
mod status_bar;
mod task_list;

use ratatui::Frame;

use crate::app::{App, View};

/// Main draw function - entry point for all UI rendering.
///
/// Routes to the appropriate view renderer based on the current view.
pub fn draw(frame: &mut Frame, app: &App) {
    match app.current_view {
        View::Main => layout::render_main_view(frame, app, frame.area()),
        View::Help => help::render_help(frame, app, frame.area()),
        View::DebugLogs => debug::render_debug_logs(frame, app, frame.area()),
        // Other views fall back to main for now
        _ => layout::render_main_view(frame, app, frame.area()),
    }
}
