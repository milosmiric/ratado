//! Main layout for the application.
//!
//! Defines the split-panel layout with header, sidebar, task list, and status bar.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::app::App;
use super::{header, sidebar, status_bar, task_list};

/// Renders the main view with all panels.
pub fn render_main_view(frame: &mut Frame, app: &App, area: Rect) {
    // Split into header, main content, and status bar
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(1),  // Status bar
        ])
        .split(area);

    // Render header
    header::render_header(frame, app, vertical_chunks[0]);

    // Split main content into sidebar and task list
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),  // Sidebar
            Constraint::Percentage(75),  // Task list
        ])
        .split(vertical_chunks[1]);

    // Render sidebar and task list
    sidebar::render_sidebar(frame, app, horizontal_chunks[0]);
    task_list::render_task_list(frame, app, horizontal_chunks[1]);

    // Render status bar
    status_bar::render_status_bar(frame, app, vertical_chunks[2]);
}
