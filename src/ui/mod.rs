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
//! - [`search`] - Search view for finding tasks
//! - [`task_detail`] - Task detail view
//! - [`calendar`] - Weekly calendar view

pub mod calendar;
pub mod date_picker;
mod debug;
pub mod description_textarea;
pub mod dialogs;
mod header;
mod help;
pub mod input;
mod layout;
pub mod search;
mod sidebar;
mod status_bar;
pub mod tag_input;
mod task_detail;
pub mod task_list;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, View};

/// Minimum terminal width required for the application to render properly.
pub const MIN_WIDTH: u16 = 100;

/// Minimum terminal height required for the application to render properly.
pub const MIN_HEIGHT: u16 = 20;

/// Main draw function - entry point for all UI rendering.
///
/// Routes to the appropriate view renderer based on the current view.
/// If a dialog is active, it renders on top of the main view.
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Check minimum terminal size
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_size_warning(frame, area);
        return;
    }

    // First render the base view
    match app.current_view {
        View::Main => layout::render_main_view(frame, app, frame.area()),
        View::Help => help::render_help(frame, app, frame.area()),
        View::DebugLogs => debug::render_debug_logs(frame, app, frame.area()),
        View::Search => search::render_search_with_context(
            frame,
            &app.input_buffer,
            app.input_cursor,
            &app.search_results,
            app.selected_search_index,
            frame.area(),
            Some(app.selected_project_name()),
        ),
        View::TaskDetail => task_detail::render_task_detail(frame, app, frame.area()),
        View::Calendar => calendar::render_calendar(frame, app, frame.area()),
    }

    // Then render any active dialog on top
    if let Some(ref dialog) = app.dialog {
        dialog.render(frame);
    }
}

/// Renders a warning message when the terminal is too small.
fn render_size_warning(frame: &mut Frame, area: Rect) {
    let width_ok = area.width >= MIN_WIDTH;
    let height_ok = area.height >= MIN_HEIGHT;

    let mut lines = vec![
        Line::from(Span::styled(
            "Terminal Too Small",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(format!(
            "Width:  {} (need {} minimum) {}",
            area.width,
            MIN_WIDTH,
            if width_ok { "✓" } else { "✗" }
        )),
        Line::from(format!(
            "Height: {} (need {} minimum) {}",
            area.height,
            MIN_HEIGHT,
            if height_ok { "✓" } else { "✗" }
        )),
        Line::from(""),
        Line::from("Please resize your terminal window."),
    ];

    // Add hint about current size
    if area.width >= 40 && area.height >= 8 {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Ratado requires a larger terminal for optimal display.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Calculate line count before consuming lines
    let line_count = lines.len() as u16;

    // Center vertically
    let y_offset = area.height.saturating_sub(line_count) / 2;
    let centered_area = Rect {
        x: area.x,
        y: area.y + y_offset,
        width: area.width,
        height: area.height.saturating_sub(y_offset),
    };

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, centered_area);
}
