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

use crate::app::{App, InputMode};

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

    // Default keybinding hints
    Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Quit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("a", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Add  ", Style::default().fg(Color::DarkGray)),
        Span::styled("f", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Filter/Sort  ", Style::default().fg(Color::DarkGray)),
        Span::styled("?", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Help", Style::default().fg(Color::DarkGray)),
    ])
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
