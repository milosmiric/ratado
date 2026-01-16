//! Task list widget.
//!
//! Displays the main list of tasks with a modern, distinctive visual design.
//! Features semantic coloring, clear visual hierarchy, and smooth selection states.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, FocusPanel};
use crate::models::{Priority, Project, Task, TaskStatus};
use crate::utils::format_relative_date;
use super::theme::{self, icons};

/// Renders the task list with modern styling.
pub fn render_task_list(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::TaskList;

    // Title style indicates focus
    let title_style = if is_focused {
        Style::default()
            .fg(theme::PRIMARY_LIGHT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::TEXT_MUTED)
    };

    let filter_style = Style::default().fg(theme::TEXT_MUTED);

    let title = Line::from(vec![
        Span::styled(" Tasks ", title_style),
        Span::styled(
            format!("{} {} ", icons::DOT, app.filter_name()),
            filter_style,
        ),
        Span::styled(
            format!("{} {} ", icons::DOT, app.sort_name()),
            filter_style,
        ),
    ]);

    let block = Block::default()
        .title(title)
        .borders(Borders::NONE);

    let tasks = app.visible_tasks();

    // Handle empty state with inspiring design
    if tasks.is_empty() {
        render_empty_state(frame, block, area);
        return;
    }

    // Determine if we should show project names (only in "All Tasks" view)
    let show_projects = app.selected_project_index == 0;

    // Build task list items
    let items: Vec<ListItem> = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let selected = Some(i) == app.selected_task_index;
            let project_name = if show_projects {
                task.project_id
                    .as_ref()
                    .and_then(|pid| app.projects.iter().find(|p| &p.id == pid))
            } else {
                None
            };
            render_task_row(task, selected, is_focused, area.width, project_name)
        })
        .collect();

    // Create list with selection state - spotlight effect
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(theme::BG_SELECTION)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▌ ");

    // Render with state for selection highlighting
    let mut state = ListState::default();
    state.select(app.selected_task_index);

    frame.render_stateful_widget(list, area, &mut state);
}

/// Renders an inspiring empty state with visual design.
fn render_empty_state(frame: &mut Frame, block: Block, area: Rect) {
    let empty_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "┌─────────────────────────┐",
            Style::default().fg(theme::BORDER_MUTED),
        )),
        Line::from(vec![
            Span::styled("│     ", Style::default().fg(theme::BORDER_MUTED)),
            Span::styled("✦ ✦ ✦", Style::default().fg(theme::PRIMARY)),
            Span::styled("     │", Style::default().fg(theme::BORDER_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("│   ", Style::default().fg(theme::BORDER_MUTED)),
            Span::styled("Ready to go!", Style::default().fg(theme::TEXT_SECONDARY).add_modifier(Modifier::BOLD)),
            Span::styled("   │", Style::default().fg(theme::BORDER_MUTED)),
        ]),
        Line::from(Span::styled(
            "└─────────────────────────┘",
            Style::default().fg(theme::BORDER_MUTED),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled(" a ", Style::default().fg(theme::BG_DARK).bg(theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(" to add your first task", Style::default().fg(theme::TEXT_MUTED)),
        ]),
    ];

    let empty = Paragraph::new(empty_text)
        .alignment(Alignment::Center)
        .block(block);

    frame.render_widget(empty, area);
}

/// Renders a single task row with modern styling.
fn render_task_row(
    task: &Task,
    selected: bool,
    focused: bool,
    width: u16,
    project: Option<&Project>,
) -> ListItem<'static> {
    // Status indicator with themed icons
    let (status_icon, status_style) = match task.status {
        TaskStatus::Pending => (
            icons::CHECKBOX_EMPTY,
            Style::default().fg(theme::STATUS_PENDING),
        ),
        TaskStatus::InProgress => (
            icons::CHECKBOX_PROGRESS,
            Style::default().fg(theme::STATUS_IN_PROGRESS),
        ),
        TaskStatus::Completed => (
            icons::CHECKBOX_DONE,
            Style::default().fg(theme::STATUS_COMPLETED),
        ),
        TaskStatus::Archived => (
            icons::CHECKBOX_ARCHIVED,
            Style::default().fg(theme::STATUS_ARCHIVED),
        ),
    };

    // Priority indicator with themed styling
    let (priority_icon, priority_style) = match task.priority {
        Priority::Urgent => (
            icons::PRIORITY_URGENT,
            Style::default()
                .fg(theme::PRIORITY_URGENT)
                .add_modifier(Modifier::BOLD),
        ),
        Priority::High => (icons::PRIORITY_HIGH, Style::default().fg(theme::PRIORITY_HIGH)),
        Priority::Medium => (" ", Style::default()),
        Priority::Low => (icons::PRIORITY_LOW, Style::default().fg(theme::PRIORITY_LOW)),
    };

    // Date string - for completed tasks show both due date and completion date
    let date_str = if task.status == TaskStatus::Completed || task.status == TaskStatus::Archived {
        let done_str = task
            .completed_at
            .map(|d| format!("{} {}", icons::CHECK, format_relative_date(d)));
        let due_str = task.due_date.map(format_relative_date);

        match (due_str, done_str) {
            (Some(due), Some(done)) => format!("{} {} {}", due, icons::DOT, done),
            (None, Some(done)) => done,
            (Some(due), None) => due,
            (None, None) => String::new(),
        }
    } else {
        task.due_date.map(format_relative_date).unwrap_or_default()
    };

    // Format tags string
    let tags_str = render_tags(&task.tags, 20);

    // Format project string (shown in "All Tasks" view)
    let project_str = project
        .map(|p| format!("{}{}", icons::PROJECT_PREFIX, p.name))
        .unwrap_or_default();

    // Calculate available width for title
    let fixed_width = 3 + 2 + 2 + project_str.len() + 1 + tags_str.len() + 2 + date_str.len() + 4;
    let title_width = (width as usize).saturating_sub(fixed_width).max(10);

    // Truncate title if needed
    let title = if task.title.len() > title_width {
        format!("{}...", &task.title[..title_width.saturating_sub(3)])
    } else {
        task.title.clone()
    };

    // Title style based on task state
    let title_style = if task.status == TaskStatus::Completed || task.status == TaskStatus::Archived
    {
        // Completed tasks: readable gray
        Style::default().fg(theme::TEXT_COMPLETED)
    } else if task.is_overdue() {
        Style::default().fg(theme::DUE_OVERDUE)
    } else if task.is_due_today() {
        Style::default().fg(theme::DUE_TODAY)
    } else if task.is_due_this_week() {
        Style::default().fg(theme::DUE_WEEK)
    } else {
        Style::default().fg(theme::TEXT_PRIMARY)
    };

    // Selection emphasis
    let title_style = if selected && focused {
        title_style.add_modifier(Modifier::BOLD)
    } else {
        title_style
    };

    // Date style
    let date_style = if task.status == TaskStatus::Completed || task.status == TaskStatus::Archived
    {
        Style::default().fg(theme::TEXT_COMPLETED)
    } else if task.is_overdue() {
        Style::default().fg(theme::DUE_OVERDUE)
    } else if task.is_due_today() {
        Style::default().fg(theme::DUE_TODAY)
    } else {
        Style::default().fg(theme::TEXT_MUTED)
    };

    // Build the line with spans
    let mut spans = vec![
        Span::styled(format!(" {} ", status_icon), status_style),
        Span::styled(format!("{} ", priority_icon), priority_style),
        Span::styled(format!("{:<width$}", title, width = title_width), title_style),
    ];

    // Add project name if present (shown in "All Tasks" view)
    if !project_str.is_empty() {
        spans.push(Span::styled(
            format!(" {}", project_str),
            Style::default().fg(theme::PROJECT),
        ));
    }

    // Add tags if present
    if !tags_str.is_empty() {
        spans.push(Span::styled(
            format!(" {}", tags_str),
            Style::default().fg(theme::TAG),
        ));
    }

    // Add date
    if !date_str.is_empty() {
        spans.push(Span::styled(format!("  {}", date_str), date_style));
    }

    ListItem::new(Line::from(spans))
}

/// Renders tags as a formatted string with themed prefix.
fn render_tags(tags: &[String], max_width: usize) -> String {
    if tags.is_empty() {
        return String::new();
    }

    let tag_str: String = tags
        .iter()
        .map(|t| format!("{}{}", icons::TAG_PREFIX, t))
        .collect::<Vec<_>>()
        .join(" ");

    if tag_str.len() > max_width {
        format!("{}...", &tag_str[..max_width.saturating_sub(3)])
    } else {
        tag_str
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_task_row_truncation() {
        let mut task = Task::new("This is a very long task title that should be truncated");
        task.priority = Priority::High;

        // Just verify it doesn't panic with various widths
        let _item = render_task_row(&task, false, false, 80, None);
        let _item = render_task_row(&task, true, true, 40, None);
        let _item = render_task_row(&task, false, false, 20, None);
    }

    #[test]
    fn test_overdue_task_style() {
        let mut task = Task::new("Overdue task");
        task.due_date = Some(Utc::now() - Duration::days(1));

        // Should be rendered without panic
        let _item = render_task_row(&task, false, false, 80, None);
    }

    #[test]
    fn test_completed_task_style() {
        let mut task = Task::new("Completed task");
        task.complete();

        // Should be rendered without panic
        let _item = render_task_row(&task, false, false, 80, None);
    }

    #[test]
    fn test_task_row_with_project() {
        let mut task = Task::new("Task in project");
        task.project_id = Some("proj-1".to_string());

        let project = Project::new("Work");

        // Should render with project name
        let _item = render_task_row(&task, false, false, 80, Some(&project));
    }
}
