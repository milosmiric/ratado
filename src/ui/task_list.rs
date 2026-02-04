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
use crate::models::{Filter, Priority, Project, Task, TaskStatus};
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

    // Handle empty state with context-aware artwork
    if tasks.is_empty() {
        render_empty_state(frame, block, area, app);
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

    // Store the list area and scroll offset for animation targeting
    // The block has no borders, so inner area == area with 1 row for title
    let list_content_area = Rect {
        x: area.x,
        y: area.y + 1, // Skip title line
        width: area.width,
        height: area.height.saturating_sub(1),
    };

    // ListState::offset() gives us the scroll position
    LAST_LIST_RENDER.with(|cell| {
        *cell.borrow_mut() = Some(ListRenderInfo {
            content_area: list_content_area,
            scroll_offset: state.offset(),
            task_ids: tasks.iter().map(|t| t.id.clone()).collect(),
        });
    });
}

/// Information about the last list render, used for animation targeting.
#[derive(Debug, Clone)]
pub struct ListRenderInfo {
    /// The content area of the list (excluding title)
    pub content_area: Rect,
    /// The scroll offset of the list
    pub scroll_offset: usize,
    /// Task IDs in visible order
    pub task_ids: Vec<String>,
}

impl ListRenderInfo {
    /// Calculates the screen rectangle for a task at the given index.
    ///
    /// Returns `None` if the task is not currently visible on screen.
    pub fn task_row_rect(&self, task_index: usize) -> Option<Rect> {
        if task_index < self.scroll_offset {
            return None;
        }
        let row_in_view = task_index - self.scroll_offset;
        if row_in_view as u16 >= self.content_area.height {
            return None;
        }
        Some(Rect {
            x: self.content_area.x,
            y: self.content_area.y + row_in_view as u16,
            width: self.content_area.width,
            height: 1,
        })
    }

    /// Finds the index and rect for a task by ID.
    pub fn find_task_rect(&self, task_id: &str) -> Option<Rect> {
        let index = self.task_ids.iter().position(|id| id == task_id)?;
        self.task_row_rect(index)
    }
}

std::thread_local! {
    static LAST_LIST_RENDER: std::cell::RefCell<Option<ListRenderInfo>> = const { std::cell::RefCell::new(None) };
}

/// Returns the last list render info for animation targeting.
pub fn take_last_render_info() -> Option<ListRenderInfo> {
    LAST_LIST_RENDER.with(|cell| cell.borrow_mut().take())
}

/// Detects the type of empty state and renders appropriate artwork.
fn render_empty_state(frame: &mut Frame, block: Block, area: Rect, app: &App) {
    let muted = Style::default().fg(theme::TEXT_MUTED);
    let bold_secondary = Style::default()
        .fg(theme::TEXT_SECONDARY)
        .add_modifier(Modifier::BOLD);
    let key_style = Style::default()
        .fg(theme::BG_DARK)
        .bg(theme::ACCENT)
        .add_modifier(Modifier::BOLD);
    let border = Style::default().fg(theme::BORDER_MUTED);
    let primary = Style::default()
        .fg(theme::PRIMARY_LIGHT)
        .add_modifier(Modifier::BOLD);
    let success = Style::default()
        .fg(theme::SUCCESS)
        .add_modifier(Modifier::BOLD);

    // Detect which empty state we're in
    let has_any_tasks = !app.tasks.is_empty();
    let all_completed = has_any_tasks
        && app
            .tasks
            .iter()
            .all(|t| t.status == TaskStatus::Completed || t.status == TaskStatus::Archived);
    let is_filtered = !matches!(app.filter, Filter::All | Filter::Pending);
    let is_project_view = app.selected_project_index > 0;

    let lines: Vec<Line> = if all_completed && !is_filtered {
        // All tasks completed - celebration
        vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("  ___  ", border)),
            Line::from(Span::styled(" |   | ", border)),
            Line::from(Span::styled(" | # | ", success)),
            Line::from(Span::styled(" |___| ", border)),
            Line::from(Span::styled("  /_\\  ", Style::default().fg(theme::ACCENT))),
            Line::from(""),
            Line::from(Span::styled("ALL DONE!", success)),
            Line::from(""),
            Line::from(Span::styled(
                "Every task is completed. Great work!",
                muted,
            )),
        ]
    } else if is_filtered {
        // Filter shows empty results
        vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("  (  )", border)),
            Line::from(Span::styled(" / __ \\", border)),
            Line::from(Span::styled("| /  \\|", border)),
            Line::from(Span::styled(" \\ -- /", border)),
            Line::from(Span::styled("  \\  / ", border)),
            Line::from(""),
            Line::from(Span::styled("Nothing here", bold_secondary)),
            Line::from(""),
            Line::from(Span::styled(
                "No tasks match the current filter",
                muted,
            )),
            Line::from(vec![
                Span::styled("Press ", muted),
                Span::styled(" f ", key_style),
                Span::styled(" to change filters", muted),
            ]),
        ]
    } else if is_project_view && !has_any_tasks {
        // Project is empty
        vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("  ┌───┐ ", border)),
            Line::from(Span::styled(" ┌┘   └┐", border)),
            Line::from(Span::styled(" │     │", border)),
            Line::from(Span::styled(" │     │", border)),
            Line::from(Span::styled(" └─────┘", border)),
            Line::from(""),
            Line::from(Span::styled("Empty project", bold_secondary)),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", muted),
                Span::styled(" a ", key_style),
                Span::styled(" to add a task", muted),
            ]),
        ]
    } else {
        // No tasks at all - rocket ship
        vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("    /\\    ", primary)),
            Line::from(Span::styled("   /  \\   ", primary)),
            Line::from(Span::styled("  | {} |  ", primary)),
            Line::from(Span::styled("  |    |  ", primary)),
            Line::from(Span::styled(" /|    |\\ ", primary)),
            Line::from(Span::styled("/ |____| \\", primary)),
            Line::from(Span::styled("  |    |  ", Style::default().fg(theme::ACCENT))),
            Line::from(Span::styled("  \\~~~~/ ", Style::default().fg(theme::WARNING))),
            Line::from(""),
            Line::from(Span::styled("Ready for liftoff!", bold_secondary)),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", muted),
                Span::styled(" a ", key_style),
                Span::styled(" to add your first task", muted),
            ]),
        ]
    };

    let empty = Paragraph::new(lines)
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
