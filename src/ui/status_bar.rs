//! Status bar widget.
//!
//! Displays keybinding hints and status messages at the bottom of the screen.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, FocusPanel, InputMode};
use crate::models::Filter;

/// Renders the status bar.
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let content = match app.input_mode {
        InputMode::Normal => render_normal_mode_hints(app),
        InputMode::Editing => render_editing_mode_hints(),
        InputMode::Search => render_search_mode_hints(),
    };

    let status_bar = Paragraph::new(content);
    frame.render_widget(status_bar, area);
}

/// Renders hints for normal mode.
fn render_normal_mode_hints(app: &App) -> Line<'static> {
    // Show status message if present
    if let Some(ref msg) = app.status_message {
        return Line::from(vec![
            Span::styled(
                format!(" {} ", msg),
                Style::default().fg(Color::Green),
            ),
        ]);
    }

    // Context-sensitive hints based on focus
    match app.focus {
        FocusPanel::Sidebar => render_sidebar_hints(app),
        FocusPanel::TaskList => render_tasklist_hints(app),
    }
}

/// Renders hints when sidebar is focused.
fn render_sidebar_hints(_app: &App) -> Line<'static> {
    Line::from(vec![
        Span::styled(" a", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" New  ", Style::default().fg(Color::DarkGray)),
        Span::styled("e", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Edit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("d", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Delete  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Tasks  ", Style::default().fg(Color::DarkGray)),
        Span::styled("?", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Help", Style::default().fg(Color::DarkGray)),
    ])
}

/// Renders hints when task list is focused.
fn render_tasklist_hints(app: &App) -> Line<'static> {
    let mut spans = vec![
        Span::styled(" a", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Add  ", Style::default().fg(Color::DarkGray)),
        Span::styled("e", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Edit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Space", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Done  ", Style::default().fg(Color::DarkGray)),
        Span::styled("/", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Search  ", Style::default().fg(Color::DarkGray)),
        Span::styled("f", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Filter  ", Style::default().fg(Color::DarkGray)),
        Span::styled("?", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Help", Style::default().fg(Color::DarkGray)),
    ];

    // Add filter indicator if not default
    let filter_name = filter_display_name(&app.filter);
    if !filter_name.is_empty() {
        spans.push(Span::styled("  │ ", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled(
            format!("Filter: {}", filter_name),
            Style::default().fg(Color::Magenta),
        ));
    }

    Line::from(spans)
}

/// Returns a display name for the filter.
fn filter_display_name(filter: &Filter) -> String {
    match filter {
        Filter::Pending => String::new(), // Default, don't show
        Filter::All => "All".to_string(),
        Filter::Completed => "Completed".to_string(),
        Filter::InProgress => "In Progress".to_string(),
        Filter::Archived => "Archived".to_string(),
        Filter::DueToday => "Due Today".to_string(),
        Filter::DueThisWeek => "Due This Week".to_string(),
        Filter::Overdue => "Overdue".to_string(),
        Filter::ByProject(id) => format!("Project: {}", id),
        Filter::ByTag(tag) => format!("Tag: {}", tag),
        Filter::ByPriority(p) => format!("Priority: {:?}", p),
    }
}

/// Renders hints for editing mode.
fn render_editing_mode_hints() -> Line<'static> {
    Line::from(vec![
        Span::styled(" Editing: ", Style::default().fg(Color::Yellow)),
        Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Save  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
    ])
}

/// Renders hints for search mode.
fn render_search_mode_hints() -> Line<'static> {
    Line::from(vec![
        Span::styled(" Search: ", Style::default().fg(Color::Yellow)),
        Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Search  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
    ])
}
