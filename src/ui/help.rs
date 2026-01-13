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
        keybinding_line("h / ←", "Focus sidebar"),
        keybinding_line("l / →", "Focus task list"),
        keybinding_line("Tab", "Switch panel focus"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  VIEWS", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        keybinding_line("f", "Cycle filter"),
        keybinding_line("s", "Cycle sort order"),
        keybinding_line("?", "Show this help"),
        keybinding_line("F12", "Toggle debug logs"),
        keybinding_line("Esc", "Close overlay / Cancel"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  TASKS", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        keybinding_line("a", "Add new task"),
        keybinding_line("e", "Edit selected task"),
        keybinding_line("d", "Delete selected task"),
        keybinding_line("Space", "Toggle task completion"),
        keybinding_line("Enter", "View task details"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  GENERAL", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        keybinding_line("q", "Quit"),
        keybinding_line("Ctrl+C", "Force quit"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press Esc or ? to close", Style::default().fg(Color::DarkGray)),
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
