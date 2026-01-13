//! Task list widget.
//!
//! Displays the main list of tasks with status, priority, title, and due date.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, FocusPanel};
use crate::models::{Priority, Task, TaskStatus};
use crate::utils::format_relative_date;

/// Renders the task list.
pub fn render_task_list(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::TaskList;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Build title with filter and sort info
    let title = format!(
        " TASKS  [{}]  [{}] ",
        app.filter_name(),
        app.sort_name()
    );

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(if is_focused { Color::Cyan } else { Color::White })
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::NONE)
        .border_style(border_style);

    let tasks = app.visible_tasks();

    // Handle empty state
    if tasks.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No tasks yet!",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press [a] to add your first task",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let empty = Paragraph::new(empty_text)
            .alignment(Alignment::Center)
            .block(block);

        frame.render_widget(empty, area);
        return;
    }

    // Build task list items
    let items: Vec<ListItem> = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let selected = Some(i) == app.selected_task_index;
            render_task_row(task, selected, is_focused, area.width)
        })
        .collect();

    // Create list with selection state
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(30, 60, 90))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    // Render with state for selection highlighting
    let mut state = ListState::default();
    state.select(app.selected_task_index);

    frame.render_stateful_widget(list, area, &mut state);
}

/// Renders a single task row.
fn render_task_row(task: &Task, selected: bool, focused: bool, width: u16) -> ListItem<'static> {
    // Checkbox based on status
    let checkbox = match task.status {
        TaskStatus::Pending => "[ ]",
        TaskStatus::InProgress => "[▸]",
        TaskStatus::Completed | TaskStatus::Archived => "[✓]",
    };

    // Priority indicator
    let priority = match task.priority {
        Priority::Urgent => "!!",
        Priority::High => " !",
        Priority::Medium => "  ",
        Priority::Low => "  ",
    };

    // Priority color
    let priority_style = match task.priority {
        Priority::Urgent => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        Priority::High => Style::default().fg(Color::Yellow),
        Priority::Medium => Style::default(),
        Priority::Low => Style::default().fg(Color::DarkGray),
    };

    // Due date
    let due_str = task
        .due_date
        .map(format_relative_date)
        .unwrap_or_default();

    // Calculate available width for title
    // Format: "  [ ] !! Title...                  Due Date"
    let fixed_width = 3 + 3 + 3 + 2 + due_str.len() + 2; // spaces + checkbox + priority + spacing + due
    let title_width = (width as usize).saturating_sub(fixed_width).max(10);

    // Truncate title if needed
    let title = if task.title.len() > title_width {
        format!("{}...", &task.title[..title_width.saturating_sub(3)])
    } else {
        task.title.clone()
    };

    // Base style based on task state
    let base_style = if task.status == TaskStatus::Completed || task.status == TaskStatus::Archived {
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM)
    } else if task.is_overdue() {
        Style::default().fg(Color::Red)
    } else if task.is_due_today() {
        Style::default().fg(Color::Yellow)
    } else if task.is_due_this_week() {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    // Selection highlight
    let row_style = if selected && focused {
        base_style.add_modifier(Modifier::BOLD)
    } else {
        base_style
    };

    // Build the line with spans
    let spans = vec![
        Span::styled(format!("  {} ", checkbox), row_style),
        Span::styled(format!("{} ", priority), priority_style),
        Span::styled(format!("{:<width$}", title, width = title_width), row_style),
        Span::styled(format!("  {}", due_str), row_style.fg(Color::DarkGray)),
    ];

    ListItem::new(Line::from(spans))
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
        let _item = render_task_row(&task, false, false, 80);
        let _item = render_task_row(&task, true, true, 40);
        let _item = render_task_row(&task, false, false, 20);
    }

    #[test]
    fn test_overdue_task_style() {
        let mut task = Task::new("Overdue task");
        task.due_date = Some(Utc::now() - Duration::days(1));

        // Should be rendered without panic
        let _item = render_task_row(&task, false, false, 80);
    }

    #[test]
    fn test_completed_task_style() {
        let mut task = Task::new("Completed task");
        task.complete();

        // Should be rendered without panic
        let _item = render_task_row(&task, false, false, 80);
    }
}
