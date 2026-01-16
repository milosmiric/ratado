//! Task detail view widget.
//!
//! Displays a full view of a single task with all its fields and
//! supports quick actions like toggling status and cycling priority.

use chrono::{DateTime, Local, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::models::{Priority, Task, TaskStatus};
use crate::utils::format_relative_date;
use super::theme;

/// Renders the task detail view.
///
/// Shows a full-screen view of the selected task with all its fields
/// and available actions.
pub fn render_task_detail(frame: &mut Frame, app: &App, area: Rect) {
    // Get the selected task
    let task = match app.selected_task() {
        Some(t) => t,
        None => {
            // If no task is selected, show a message
            let msg = Paragraph::new("No task selected")
                .style(Style::default().fg(theme::TEXT_MUTED))
                .block(
                    Block::default()
                        .title(" Task Detail ")
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED)
                        .border_style(Style::default().fg(theme::BORDER))
                        .style(Style::default().bg(theme::BG_ELEVATED)),
                );
            frame.render_widget(msg, area);
            return;
        }
    };

    // Main block
    let block = Block::default()
        .title(Span::styled(
            " Task Detail ",
            Style::default()
                .fg(theme::PRIMARY_LIGHT)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(theme::PRIMARY_LIGHT))
        .style(Style::default().bg(theme::BG_ELEVATED));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout for the content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Status & Priority row
            Constraint::Length(3), // Due date & Project row
            Constraint::Length(2), // Tags
            Constraint::Length(1), // Spacer
            Constraint::Min(5),    // Description
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Timestamps
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Help line
        ])
        .split(inner);

    // Title
    render_title(frame, task, chunks[0]);

    // Status & Priority
    render_status_priority(frame, task, chunks[2]);

    // Due date & Project
    render_due_project(frame, task, app, chunks[3]);

    // Tags
    render_tags(frame, task, chunks[4]);

    // Description
    render_description(frame, task, chunks[6]);

    // Timestamps
    render_timestamps(frame, task, chunks[8]);

    // Help line
    render_help_line(frame, chunks[10]);
}

/// Renders the task title.
fn render_title(frame: &mut Frame, task: &Task, area: Rect) {
    let style = if task.status == TaskStatus::Completed {
        Style::default().fg(theme::TEXT_COMPLETED)
    } else {
        Style::default()
            .fg(theme::TEXT_PRIMARY)
            .add_modifier(Modifier::BOLD)
    };

    let title = Paragraph::new(task.title.clone())
        .style(style)
        .wrap(Wrap { trim: true });

    frame.render_widget(title, area);
}

/// Renders status and priority on one row.
fn render_status_priority(frame: &mut Frame, task: &Task, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Status
    let (status_icon, status_text, status_color) = match task.status {
        TaskStatus::Pending => ("○", "Pending", theme::STATUS_PENDING),
        TaskStatus::InProgress => ("◐", "In Progress", theme::STATUS_IN_PROGRESS),
        TaskStatus::Completed => ("●", "Completed", theme::STATUS_COMPLETED),
        TaskStatus::Archived => ("◌", "Archived", theme::STATUS_ARCHIVED),
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled(
            format!("{} {}", status_icon, status_text),
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(status, chunks[0]);

    // Priority
    let (priority_icon, priority_text, priority_color) = match task.priority {
        Priority::Urgent => ("!!", "Urgent", theme::PRIORITY_URGENT),
        Priority::High => ("!", "High", theme::PRIORITY_HIGH),
        Priority::Medium => ("−", "Medium", theme::TEXT_PRIMARY),
        Priority::Low => ("↓", "Low", theme::PRIORITY_LOW),
    };

    let priority = Paragraph::new(Line::from(vec![
        Span::styled("Priority: ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled(
            format!("{} {}", priority_icon, priority_text),
            Style::default()
                .fg(priority_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(priority, chunks[1]);
}

/// Renders due date and project.
fn render_due_project(frame: &mut Frame, task: &Task, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Due date
    let due_text = match &task.due_date {
        Some(date) => {
            let formatted = format_relative_date(*date);
            let full_date = date.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string();
            let color = if task.is_overdue() {
                theme::DUE_OVERDUE
            } else if task.is_due_today() {
                theme::DUE_TODAY
            } else if task.is_due_this_week() {
                theme::DUE_WEEK
            } else {
                theme::TEXT_PRIMARY
            };
            Line::from(vec![
                Span::styled("Due: ", Style::default().fg(theme::TEXT_MUTED)),
                Span::styled(
                    format!("{} ({})", formatted, full_date),
                    Style::default().fg(color),
                ),
            ])
        }
        None => Line::from(vec![
            Span::styled("Due: ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("Not set", Style::default().fg(theme::TEXT_MUTED)),
        ]),
    };
    frame.render_widget(Paragraph::new(due_text), chunks[0]);

    // Project
    let project_name = task
        .project_id
        .as_ref()
        .and_then(|pid| app.projects.iter().find(|p| &p.id == pid))
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "None".to_string());

    let project = Paragraph::new(Line::from(vec![
        Span::styled("Project: ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled(
            format!("@{}", project_name),
            Style::default().fg(theme::PROJECT),
        ),
    ]));
    frame.render_widget(project, chunks[1]);
}

/// Renders task tags.
fn render_tags(frame: &mut Frame, task: &Task, area: Rect) {
    let tags_line = if task.tags.is_empty() {
        Line::from(vec![
            Span::styled("Tags: ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("None", Style::default().fg(theme::TEXT_MUTED)),
        ])
    } else {
        let mut spans = vec![Span::styled("Tags: ", Style::default().fg(theme::TEXT_MUTED))];
        for (i, tag) in task.tags.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            spans.push(Span::styled(
                format!("#{}", tag),
                Style::default().fg(theme::TAG),
            ));
        }
        Line::from(spans)
    };

    frame.render_widget(Paragraph::new(tags_line), area);
}

/// Renders the task description.
fn render_description(frame: &mut Frame, task: &Task, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Description ",
            Style::default().fg(theme::TEXT_SECONDARY),
        ))
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(theme::BORDER));

    let content = task.description.as_deref().unwrap_or("No description");

    let style = if task.description.is_some() {
        Style::default().fg(theme::TEXT_PRIMARY)
    } else {
        Style::default().fg(theme::TEXT_MUTED)
    };

    let description = Paragraph::new(content)
        .style(style)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(description, area);
}

/// Renders created/updated timestamps.
fn render_timestamps(frame: &mut Frame, task: &Task, area: Rect) {
    let created = format_timestamp(task.created_at);
    let updated = format_timestamp(task.updated_at);

    let line = Line::from(vec![
        Span::styled("Created: ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled(created, Style::default().fg(theme::TEXT_MUTED)),
        Span::styled("  │  ", Style::default().fg(theme::BORDER)),
        Span::styled("Updated: ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled(updated, Style::default().fg(theme::TEXT_MUTED)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

/// Formats a timestamp for display.
fn format_timestamp(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

/// Renders the help line showing available actions.
fn render_help_line(frame: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled("[Space]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
        Span::styled(" Toggle  ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled("[p]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
        Span::styled(" Priority  ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled("[e]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
        Span::styled(" Edit  ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled("[d]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
        Span::styled(" Delete  ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled("[Esc]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
        Span::styled(" Back", Style::default().fg(theme::TEXT_MUTED)),
    ]);

    frame.render_widget(Paragraph::new(help), area);
}
