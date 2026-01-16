//! Main layout for the application.
//!
//! Defines the split-panel layout with header, sidebar, task list, and status bar.
//!
//! ## Layout Structure
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │ Header (3 lines)                                │
//! ├──────────────┬──────────────────────────────────┤
//! │ Sidebar      │ Task List                        │
//! │ (20-35 cols) │ (remaining space)                │
//! │              │                                  │
//! ├──────────────┴──────────────────────────────────┤
//! │ Status Bar (1 line)                             │
//! └─────────────────────────────────────────────────┘
//! ```

use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    Frame,
};

use crate::app::App;
use super::{header, sidebar, status_bar, task_list};

/// Minimum width for the sidebar panel.
const SIDEBAR_MIN_WIDTH: u16 = 20;

/// Maximum width for the sidebar panel.
const SIDEBAR_MAX_WIDTH: u16 = 35;

/// Renders the main view with all panels.
pub fn render_main_view(frame: &mut Frame, app: &App, area: Rect) {
    // Vertical layout: Header | Content | Status Bar
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Header - fixed height
            Constraint::Fill(1),     // Main content - fills remaining space
            Constraint::Length(1),   // Status bar - fixed height
        ])
        .split(area);

    // Render header
    header::render_header(frame, app, vertical_chunks[0]);

    // Horizontal layout: Sidebar | Task List
    // Sidebar has min/max constraints, task list fills remaining space
    let sidebar_width = calculate_sidebar_width(vertical_chunks[1].width);
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .flex(Flex::Start)
        .constraints([
            Constraint::Length(sidebar_width),  // Sidebar - constrained width
            Constraint::Fill(1),                // Task list - fills remaining
        ])
        .split(vertical_chunks[1]);

    // Render sidebar and task list
    sidebar::render_sidebar(frame, app, horizontal_chunks[0]);
    task_list::render_task_list(frame, app, horizontal_chunks[1]);

    // Render status bar
    status_bar::render_status_bar(frame, app, vertical_chunks[2]);
}

/// Calculates the sidebar width based on available space.
///
/// Returns a width between `SIDEBAR_MIN_WIDTH` and `SIDEBAR_MAX_WIDTH`,
/// targeting approximately 25% of the total width.
fn calculate_sidebar_width(total_width: u16) -> u16 {
    let target = total_width / 4; // 25% of total width
    target.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH)
}
