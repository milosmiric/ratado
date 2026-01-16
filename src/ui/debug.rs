//! Debug log viewer.
//!
//! Displays application logs using tui-logger.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    symbols::border,
    text::Span,
    widgets::{Block, Borders},
    Frame,
};
use tui_logger::TuiLoggerWidget;

use crate::app::App;
use super::theme;

/// Renders the debug log viewer.
pub fn render_debug_logs(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Debug Logs (F12 to close) ",
            Style::default()
                .fg(theme::WARNING)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(theme::WARNING))
        .style(Style::default().bg(theme::BG_ELEVATED));

    // Use tui-logger widget
    let logger = TuiLoggerWidget::default()
        .block(block)
        .style_error(Style::default().fg(theme::ERROR))
        .style_warn(Style::default().fg(theme::WARNING))
        .style_info(Style::default().fg(theme::SUCCESS))
        .style_debug(Style::default().fg(theme::INFO))
        .style_trace(Style::default().fg(theme::TEXT_MUTED));

    // Note: TuiLoggerWidget uses its own state, not app.log_state
    // The log_state field is for potential future use with TuiLoggerSmartWidget
    let _ = &app.log_state; // Acknowledge the field exists

    frame.render_widget(logger, area);
}
