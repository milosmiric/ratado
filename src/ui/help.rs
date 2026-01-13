//! Help screen widget.
//!
//! Displays a reference of all keybindings.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

/// Renders the help screen as an overlay.
pub fn render_help(frame: &mut Frame, _app: &App, area: Rect) {
    // Center the help popup
    let popup_area = centered_rect(60, 80, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Span::styled(
            " Help - Keybindings ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  NAVIGATION", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        keybinding_line("j / ↓", "Move down"),
        keybinding_line("k / ↑", "Move up"),
        keybinding_line("g / G", "Jump to top / bottom"),
        keybinding_line("h / ←", "Focus sidebar"),
        keybinding_line("l / →", "Focus task list"),
        keybinding_line("Tab", "Switch panel / sidebar section"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  TASKS (when task list focused)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        keybinding_line("a", "Add new task"),
        keybinding_line("e / Enter", "Edit selected task"),
        keybinding_line("d", "Delete selected task"),
        keybinding_line("Space", "Toggle task completion"),
        keybinding_line("p", "Cycle priority"),
        keybinding_line("t", "Edit tags"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  PROJECTS (when sidebar focused)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        keybinding_line("a", "Add new project"),
        keybinding_line("e / Enter", "Edit selected project"),
        keybinding_line("d", "Delete selected project"),
        keybinding_line("Tab", "Switch between Projects/Tags"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  FILTERS & SORT", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        keybinding_line("f", "Open filter/sort dialog"),
        keybinding_line("T", "Filter: Due today"),
        keybinding_line("W", "Filter: Due this week"),
        keybinding_line("1-4", "Filter by priority (1=Low, 4=Urgent)"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  GENERAL", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        keybinding_line("?", "Show this help"),
        keybinding_line("F12", "Toggle debug logs"),
        keybinding_line("r", "Refresh data"),
        keybinding_line("q", "Quit"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press any key to close", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, popup_area);
}

/// Creates a formatted keybinding line.
fn keybinding_line(key: &str, description: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw("    "),
        Span::styled(
            format!("{:12}", key),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(description.to_string(), Style::default().fg(Color::White)),
    ])
}

/// Creates a centered rectangle within the given area.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
