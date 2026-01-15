//! Application header widget.
//!
//! Displays the app title and status badges (overdue count, today's tasks).

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

/// Renders the application header.
pub fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![
        Span::styled(
            " Ratado ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            concat!("v", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        ),
    ];

    // Add overdue badge if there are overdue tasks
    let overdue = app.overdue_count();
    if overdue > 0 {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!(" Overdue: {} ", overdue),
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Add today's tasks badge
    let today = app.due_today_count();
    if today > 0 {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!(" Due Today: {} ", today),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow),
        ));
    }

    // Add total task count (dimmed)
    let total = app.total_task_count();
    spans.push(Span::raw("  "));
    spans.push(Span::styled(
        format!("{} tasks total", total),
        Style::default().fg(Color::DarkGray),
    ));

    let header = Paragraph::new(Line::from(spans))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(header, area);
}
