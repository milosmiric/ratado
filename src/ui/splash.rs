//! Startup splash screen.
//!
//! Displays a large "RATADO" ASCII art title with subtitle.
//! The splash is animated with a coalesce → hold → fade sequence via tachyonfx.

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::theme;

/// ASCII art for "RATADO" using block characters.
const LOGO: &[&str] = &[
    "█▀█  █▀█ ▀█▀ █▀█ █▀▄ █▀█",
    "█▀▄  █▀█  █  █▀█ █ █ █ █",
    "▀ ▀  ▀ ▀  ▀  ▀ ▀ ▀▀  ▀▀▀",
];

/// Renders the splash screen content (ASCII art title + subtitle).
///
/// The actual animation is handled by the effect system in `effects.rs`.
/// This function renders the static content that effects will animate.
///
/// # Arguments
///
/// * `frame` - The frame to render into
/// * `area` - The available area for the splash screen
pub fn render_splash(frame: &mut Frame, area: Rect) {
    let logo_height = LOGO.len() as u16;
    // logo + gap + subtitle + version
    let total_height: u16 = logo_height + 1 + 1 + 1;
    let vertical_offset = area.height.saturating_sub(total_height) / 2;

    let [_, content_area, _] = Layout::vertical([
        Constraint::Length(vertical_offset),
        Constraint::Length(total_height),
        Constraint::Min(0),
    ])
    .areas(area);

    let [logo_area, _, subtitle_area, version_area] = Layout::vertical([
        Constraint::Length(logo_height),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(content_area);

    // Render ASCII logo
    let logo_lines: Vec<Line> = LOGO
        .iter()
        .map(|line| {
            Line::from(Span::styled(
                *line,
                Style::default()
                    .fg(theme::PRIMARY_LIGHT)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let logo = Paragraph::new(logo_lines).alignment(Alignment::Center);
    frame.render_widget(logo, logo_area);

    // Subtitle
    let subtitle = Paragraph::new(Line::from(Span::styled(
        "Terminal Task Manager",
        Style::default().fg(theme::TEXT_MUTED),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(subtitle, subtitle_area);

    // Version
    let version = Paragraph::new(Line::from(Span::styled(
        format!("v{}", env!("CARGO_PKG_VERSION")),
        Style::default().fg(theme::TEXT_DISABLED),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(version, version_area);
}
