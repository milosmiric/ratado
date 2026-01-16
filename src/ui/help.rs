//! Help screen widget.
//!
//! Displays a reference of all keybindings.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme;

/// Renders the help screen as an overlay.
pub fn render_help(frame: &mut Frame, _app: &App, area: Rect) {
    // Center the help popup
    let popup_area = centered_rect(60, 80, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    // Themed block with rounded borders
    let block = Block::default()
        .title(Span::styled(
            " Help - Keybindings ",
            Style::default()
                .fg(theme::PRIMARY_LIGHT)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(theme::PRIMARY_LIGHT))
        .style(Style::default().bg(theme::BG_ELEVATED));

    let help_text = vec![
        Line::from(""),
        section_header("NAVIGATION"),
        Line::from(""),
        keybinding_line("j / ↓", "Move down"),
        keybinding_line("k / ↑", "Move up"),
        keybinding_line("g / G", "Jump to top / bottom"),
        keybinding_line("h / ←", "Focus sidebar"),
        keybinding_line("l / →", "Focus task list"),
        keybinding_line("Tab", "Switch panel / sidebar section"),
        Line::from(""),
        section_header("TASKS (when task list focused)"),
        Line::from(""),
        keybinding_line("a", "Add new task"),
        keybinding_line("e / Enter", "Edit selected task"),
        keybinding_line("d", "Delete selected task"),
        keybinding_line("Space", "Toggle task completion"),
        keybinding_line("p", "Cycle priority"),
        keybinding_line("t", "Edit tags"),
        Line::from(""),
        section_header("PROJECTS (when sidebar focused)"),
        Line::from(""),
        keybinding_line("a", "Add new project"),
        keybinding_line("e / Enter", "Edit selected project"),
        keybinding_line("d", "Delete selected project"),
        keybinding_line("Tab", "Switch between Projects/Tags"),
        Line::from(""),
        section_header("FILTERS & SORT"),
        Line::from(""),
        keybinding_line("f", "Open filter/sort dialog"),
        keybinding_line("T", "Filter: Due today"),
        keybinding_line("W", "Filter: Due this week"),
        keybinding_line("1-4", "Filter by priority (1=Low, 4=Urgent)"),
        Line::from(""),
        section_header("VIEWS"),
        Line::from(""),
        keybinding_line("/", "Search tasks"),
        keybinding_line("c", "Weekly calendar"),
        keybinding_line("v", "Task detail view"),
        Line::from(""),
        section_header("GENERAL"),
        Line::from(""),
        keybinding_line("?", "Show this help"),
        keybinding_line("F12", "Toggle debug logs"),
        keybinding_line("S", "Settings"),
        keybinding_line("r", "Refresh data"),
        keybinding_line("q", "Quit"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Ratado v0.1.0", Style::default().fg(theme::TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  Created by Miloš Mirić", Style::default().fg(theme::TEXT_MUTED)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press any key to close", Style::default().fg(theme::TEXT_MUTED)),
        ]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, popup_area);
}

/// Creates a section header line.
fn section_header(title: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {}", title),
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

/// Creates a formatted keybinding line.
fn keybinding_line(key: &str, description: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw("    "),
        Span::styled(
            format!("{:12}", key),
            Style::default()
                .fg(theme::PRIMARY_LIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(description.to_string(), Style::default().fg(theme::TEXT_PRIMARY)),
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
