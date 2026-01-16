//! Application header widget.
//!
//! Displays a distinctive branded header with progress visualization
//! and key statistics. Designed to make an immediate visual impression.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme::{self, icons};

/// Renders the application header with innovative design.
///
/// Features:
/// - Distinctive ASCII brand mark
/// - Visual progress bar showing completion
/// - Color-coded status indicators
pub fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    // Calculate stats
    let total = app.total_task_count();
    let completed = app.completed_count();
    let overdue = app.overdue_count();
    let today = app.due_today_count();
    let in_progress = app.in_progress_count();

    // Split header into brand section and stats section
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),  // Brand (◆═══ RATADO ═══◆)
            Constraint::Fill(1),     // Stats and progress
        ])
        .split(area);

    // ═══════════════════════════════════════════════════════════════════════
    // BRAND SECTION - Distinctive logo
    // ═══════════════════════════════════════════════════════════════════════
    let brand = Line::from(vec![
        Span::styled(" ◆", Style::default().fg(theme::PRIMARY)),
        Span::styled("═══", Style::default().fg(theme::PRIMARY_DARK)),
        Span::styled(" RATADO ", Style::default()
            .fg(theme::PRIMARY_LIGHT)
            .add_modifier(Modifier::BOLD)),
        Span::styled("═══", Style::default().fg(theme::PRIMARY_DARK)),
        Span::styled("◆ ", Style::default().fg(theme::PRIMARY)),
    ]);

    let brand_widget = Paragraph::new(brand).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::BORDER)),
    );
    frame.render_widget(brand_widget, chunks[0]);

    // ═══════════════════════════════════════════════════════════════════════
    // STATS SECTION - Progress bar and indicators
    // ═══════════════════════════════════════════════════════════════════════
    let mut stats_spans = Vec::new();

    // Progress bar
    let progress = if total > 0 {
        completed as f32 / total as f32
    } else {
        0.0
    };

    stats_spans.push(Span::styled(" ", Style::default()));
    stats_spans.extend(render_progress_bar(progress, 16));
    stats_spans.push(Span::styled(
        format!(" {}% ", (progress * 100.0) as u8),
        Style::default().fg(if progress >= 1.0 { theme::SUCCESS } else { theme::TEXT_SECONDARY }),
    ));

    // Separator
    stats_spans.push(Span::styled(
        format!(" {} ", icons::LINE_VERTICAL),
        Style::default().fg(theme::BORDER),
    ));

    // Overdue - urgent indicator
    if overdue > 0 {
        stats_spans.push(Span::styled(
            format!("{} ", icons::PRIORITY_URGENT),
            Style::default().fg(theme::ERROR),
        ));
        stats_spans.push(Span::styled(
            format!("{} overdue  ", overdue),
            Style::default().fg(theme::ERROR).add_modifier(Modifier::BOLD),
        ));
    }

    // Today
    if today > 0 {
        stats_spans.push(Span::styled(
            format!("{} ", icons::CIRCLE),
            Style::default().fg(theme::ACCENT),
        ));
        stats_spans.push(Span::styled(
            format!("{} today  ", today),
            Style::default().fg(theme::ACCENT),
        ));
    }

    // In progress
    if in_progress > 0 {
        stats_spans.push(Span::styled(
            format!("{} ", icons::CHECKBOX_PROGRESS),
            Style::default().fg(theme::STATUS_IN_PROGRESS),
        ));
        stats_spans.push(Span::styled(
            format!("{} active  ", in_progress),
            Style::default().fg(theme::STATUS_IN_PROGRESS),
        ));
    }

    // Total
    stats_spans.push(Span::styled(
        format!("{}/{} tasks", completed, total),
        Style::default().fg(theme::TEXT_MUTED),
    ));

    let stats_widget = Paragraph::new(Line::from(stats_spans)).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::BORDER)),
    );
    frame.render_widget(stats_widget, chunks[1]);
}

/// Renders a visual progress bar using Unicode blocks.
fn render_progress_bar(progress: f32, width: usize) -> Vec<Span<'static>> {
    let filled = (progress * width as f32) as usize;
    let partial = ((progress * width as f32).fract() * 8.0) as usize;
    let empty = width.saturating_sub(filled).saturating_sub(if partial > 0 { 1 } else { 0 });

    // Gradient colors for the progress bar
    let bar_color = if progress >= 1.0 {
        theme::SUCCESS
    } else if progress >= 0.7 {
        theme::INFO
    } else if progress >= 0.3 {
        theme::PRIMARY_LIGHT
    } else {
        theme::TEXT_MUTED
    };

    let mut spans = vec![
        Span::styled("▐", Style::default().fg(theme::BORDER)),
    ];

    // Filled portion
    if filled > 0 {
        spans.push(Span::styled(
            "█".repeat(filled),
            Style::default().fg(bar_color),
        ));
    }

    // Partial block (░▒▓█)
    if partial > 0 && filled < width {
        let partial_char = match partial {
            1 => "▏",
            2 => "▎",
            3 => "▍",
            4 => "▌",
            5 => "▋",
            6 => "▊",
            7 => "▉",
            _ => "█",
        };
        spans.push(Span::styled(
            partial_char,
            Style::default().fg(bar_color),
        ));
    }

    // Empty portion
    if empty > 0 {
        spans.push(Span::styled(
            "░".repeat(empty),
            Style::default().fg(theme::BORDER_MUTED),
        ));
    }

    spans.push(Span::styled("▌", Style::default().fg(theme::BORDER)));

    spans
}
