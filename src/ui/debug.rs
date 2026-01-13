//! Debug log viewer.
//!
//! Displays application logs using tui-logger.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders},
    Frame,
};
use tui_logger::TuiLoggerWidget;

use crate::app::App;

/// Renders the debug log viewer.
pub fn render_debug_logs(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Debug Logs (F12 to close) ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    // Use tui-logger widget
    let logger = TuiLoggerWidget::default()
        .block(block)
        .style_error(Style::default().fg(Color::Red))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_info(Style::default().fg(Color::Green))
        .style_debug(Style::default().fg(Color::Cyan))
        .style_trace(Style::default().fg(Color::DarkGray));

    // Note: TuiLoggerWidget uses its own state, not app.log_state
    // The log_state field is for potential future use with TuiLoggerSmartWidget
    let _ = &app.log_state; // Acknowledge the field exists

    frame.render_widget(logger, area);
}
