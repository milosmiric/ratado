//! Status bar widget.
//!
//! Displays keybinding hints and status messages with modern styling.
//! The status bar provides context-sensitive help and feedback.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, FocusPanel, InputMode};
use crate::models::Filter;
use super::theme::{self, icons};

/// Renders the status bar with themed styling.
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let content = match app.input_mode {
        InputMode::Normal => render_normal_mode_hints(app),
        InputMode::Editing => render_editing_mode_hints(),
        InputMode::Search => render_search_mode_hints(),
    };

    let status_bar = Paragraph::new(content);
    frame.render_widget(status_bar, area);
}

/// Creates a styled keybinding hint.
fn key_hint(key: &str, label: &str) -> Vec<Span<'static>> {
    vec![
        Span::styled(
            format!(" {}", key),
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {} ", label), Style::default().fg(theme::TEXT_MUTED)),
    ]
}

/// Renders hints for normal mode.
fn render_normal_mode_hints(app: &App) -> Line<'static> {
    // Show status message if present
    if let Some(ref msg) = app.status_message {
        return Line::from(vec![
            Span::styled(
                format!(" {} ", icons::CHECK),
                Style::default().fg(theme::SUCCESS),
            ),
            Span::styled(
                format!("{} ", msg),
                Style::default().fg(theme::SUCCESS),
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
    let mut spans = Vec::new();
    spans.extend(key_hint("a", "New"));
    spans.extend(key_hint("e", "Edit"));
    spans.extend(key_hint("d", "Delete"));
    spans.extend(key_hint("c", "Calendar"));
    spans.extend(key_hint("Tab", "Tasks"));
    spans.extend(key_hint("?", "Help"));
    Line::from(spans)
}

/// Renders hints when task list is focused.
fn render_tasklist_hints(app: &App) -> Line<'static> {
    let mut spans = Vec::new();
    spans.extend(key_hint("a", "Add"));
    spans.extend(key_hint("e", "Edit"));
    spans.extend(key_hint("d", "Delete"));
    spans.extend(key_hint("Space", "Done"));
    spans.extend(key_hint("/", "Search"));
    spans.extend(key_hint("f", "Filter"));
    spans.extend(key_hint("?", "Help"));

    // Add filter indicator if not default
    let filter_name = filter_display_name(&app.filter);
    if !filter_name.is_empty() {
        spans.push(Span::styled(
            format!(" {} ", icons::LINE_VERTICAL),
            Style::default().fg(theme::BORDER),
        ));
        spans.push(Span::styled(
            format!("{} {}", icons::BULLET, filter_name),
            Style::default().fg(theme::INFO),
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
    let mut spans = vec![
        Span::styled(
            format!(" {} Editing ", icons::BULLET),
            Style::default()
                .fg(theme::WARNING)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{} ", icons::LINE_VERTICAL),
            Style::default().fg(theme::BORDER),
        ),
    ];
    spans.extend(key_hint("Enter", "Save"));
    spans.extend(key_hint("Esc", "Cancel"));
    Line::from(spans)
}

/// Renders hints for search mode.
fn render_search_mode_hints() -> Line<'static> {
    let mut spans = vec![
        Span::styled(
            format!(" {} Search ", icons::BULLET),
            Style::default()
                .fg(theme::INFO)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{} ", icons::LINE_VERTICAL),
            Style::default().fg(theme::BORDER),
        ),
    ];
    spans.extend(key_hint("Enter", "Go"));
    spans.extend(key_hint("↑/↓", "Navigate"));
    spans.extend(key_hint("Esc", "Cancel"));
    Line::from(spans)
}
