//! Application header widget.
//!
//! Displays the app title with a distinctive brand design and status badges.
//! The header uses the Ratado color palette to create a memorable visual identity.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme::{self, icons};

/// Renders the application header with brand styling.
///
/// The header features:
/// - A distinctive logo/title with gradient-like styling
/// - Status badges for overdue and today's tasks
/// - Task statistics
pub fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    // Build the header content
    let mut spans = vec![
        // Decorative diamond
        Span::styled(
            format!(" {} ", icons::DIAMOND),
            Style::default().fg(theme::PRIMARY),
        ),
        // Logo text with brand color
        Span::styled(
            "Ratado",
            Style::default()
                .fg(theme::PRIMARY_LIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        // Version - subtle
        Span::styled(
            format!(" {}", concat!("v", env!("CARGO_PKG_VERSION"))),
            Style::default().fg(theme::TEXT_MUTED),
        ),
    ];

    // Separator
    spans.push(Span::styled(
        format!("  {}  ", icons::LINE_VERTICAL),
        Style::default().fg(theme::BORDER),
    ));

    // Add overdue badge if there are overdue tasks
    let overdue = app.overdue_count();
    if overdue > 0 {
        spans.push(Span::styled(
            icons::WARNING_ICON,
            Style::default().fg(theme::ERROR),
        ));
        spans.push(Span::styled(
            format!(" {} overdue", overdue),
            Style::default()
                .fg(theme::ERROR)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw("  "));
    }

    // Add today's tasks badge
    let today = app.due_today_count();
    if today > 0 {
        spans.push(Span::styled(
            icons::CIRCLE,
            Style::default().fg(theme::ACCENT),
        ));
        spans.push(Span::styled(
            format!(" {} today", today),
            Style::default().fg(theme::ACCENT),
        ));
        spans.push(Span::raw("  "));
    }

    // In-progress count
    let in_progress = app.in_progress_count();
    if in_progress > 0 {
        spans.push(Span::styled(
            icons::CHECKBOX_PROGRESS,
            Style::default().fg(theme::STATUS_IN_PROGRESS),
        ));
        spans.push(Span::styled(
            format!(" {} active", in_progress),
            Style::default().fg(theme::STATUS_IN_PROGRESS),
        ));
        spans.push(Span::raw("  "));
    }

    // Add total task count (muted)
    let total = app.total_task_count();
    spans.push(Span::styled(
        format!("{} total", total),
        Style::default().fg(theme::TEXT_MUTED),
    ));

    let header = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::BORDER)),
    );

    frame.render_widget(header, area);
}
