//! Weekly calendar view widget.
//!
//! Displays a week view with tasks organized by their due dates.
//! Users can navigate between weeks and select days to see tasks.

use chrono::{Datelike, Duration, Local, NaiveDate};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::models::{Priority, SortOrder, Task, TaskStatus};
use crate::utils::format_relative_date;
use super::theme::{self, icons};

/// Focus state for the calendar view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CalendarFocus {
    /// Focus on the day grid (navigate days/weeks)
    #[default]
    DayGrid,
    /// Focus on the task list for the selected day
    TaskList,
}

/// State for the calendar view.
///
/// Tracks the currently displayed week and selected day.
#[derive(Debug, Clone)]
pub struct CalendarState {
    /// The Monday of the currently displayed week
    pub week_start: NaiveDate,
    /// The currently selected day (0-6, Monday-Sunday)
    pub selected_day: usize,
    /// Current focus (day grid or task list)
    pub focus: CalendarFocus,
    /// Selected task index in the task list (when focused on tasks)
    pub selected_task_index: usize,
    /// Whether to show completed tasks in the task list
    pub show_completed: bool,
}

impl Default for CalendarState {
    fn default() -> Self {
        let today = Local::now().date_naive();
        let week_start = today - Duration::days(today.weekday().num_days_from_monday() as i64);
        let selected_day = today.weekday().num_days_from_monday() as usize;
        Self {
            week_start,
            selected_day,
            focus: CalendarFocus::DayGrid,
            selected_task_index: 0,
            show_completed: false,
        }
    }
}

impl CalendarState {
    /// Creates a new calendar state centered on today.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the date of the selected day.
    pub fn selected_date(&self) -> NaiveDate {
        self.week_start + Duration::days(self.selected_day as i64)
    }

    /// Moves selection to the previous day.
    pub fn prev_day(&mut self) {
        if self.selected_day == 0 {
            self.prev_week();
            self.selected_day = 6;
        } else {
            self.selected_day -= 1;
        }
    }

    /// Moves selection to the next day.
    pub fn next_day(&mut self) {
        if self.selected_day == 6 {
            self.next_week();
            self.selected_day = 0;
        } else {
            self.selected_day += 1;
        }
    }

    /// Moves to the previous week.
    pub fn prev_week(&mut self) {
        self.week_start -= Duration::days(7);
    }

    /// Moves to the next week.
    pub fn next_week(&mut self) {
        self.week_start += Duration::days(7);
    }

    /// Jumps to today.
    pub fn goto_today(&mut self) {
        let today = Local::now().date_naive();
        self.week_start = today - Duration::days(today.weekday().num_days_from_monday() as i64);
        self.selected_day = today.weekday().num_days_from_monday() as usize;
    }

    /// Toggles focus between day grid and task list.
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            CalendarFocus::DayGrid => CalendarFocus::TaskList,
            CalendarFocus::TaskList => CalendarFocus::DayGrid,
        };
        // Reset task selection when switching to task list
        if self.focus == CalendarFocus::TaskList {
            self.selected_task_index = 0;
        }
    }

    /// Moves to the previous task in the list.
    pub fn prev_task(&mut self) {
        self.selected_task_index = self.selected_task_index.saturating_sub(1);
    }

    /// Moves to the next task in the list (capped by task_count).
    pub fn next_task(&mut self, task_count: usize) {
        if task_count > 0 && self.selected_task_index < task_count - 1 {
            self.selected_task_index += 1;
        }
    }

    /// Resets task selection (call when day changes).
    pub fn reset_task_selection(&mut self) {
        self.selected_task_index = 0;
    }

    /// Toggles showing completed tasks.
    pub fn toggle_show_completed(&mut self) {
        self.show_completed = !self.show_completed;
        // Reset selection when filter changes
        self.selected_task_index = 0;
    }
}

/// Renders the weekly calendar view.
pub fn render_calendar(frame: &mut Frame, app: &App, area: Rect) {
    let state = &app.calendar_state;

    let block = Block::default()
        .title(Span::styled(
            " Weekly Calendar ",
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

    // Layout: header, week grid, task list, help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Week header (month/year)
            Constraint::Length(5), // Day cards
            Constraint::Length(1), // Spacer
            Constraint::Min(5),    // Tasks for selected day
            Constraint::Length(2), // Help line
        ])
        .split(inner);

    // Week header
    render_week_header(frame, state, chunks[0]);

    // Day cards
    render_day_cards(frame, state, app, chunks[1]);

    // Tasks for selected day
    render_day_tasks(frame, state, app, chunks[3]);

    // Help line
    render_help_line(frame, state, chunks[4]);
}

/// Renders the week header showing the date range.
fn render_week_header(frame: &mut Frame, state: &CalendarState, area: Rect) {
    let week_end = state.week_start + Duration::days(6);
    let today = Local::now().date_naive();

    // Format: "January 2024" or "Dec 2023 - Jan 2024" if spanning months
    let header = if state.week_start.month() == week_end.month() {
        format!(
            "{} {}",
            state.week_start.format("%B"),
            state.week_start.year()
        )
    } else if state.week_start.year() == week_end.year() {
        format!(
            "{} - {} {}",
            state.week_start.format("%b"),
            week_end.format("%b"),
            week_end.year()
        )
    } else {
        format!(
            "{} {} - {} {}",
            state.week_start.format("%b"),
            state.week_start.year(),
            week_end.format("%b"),
            week_end.year()
        )
    };

    // Add indicator if this week contains today
    let contains_today = today >= state.week_start && today <= week_end;
    let header_style = if contains_today {
        Style::default()
            .fg(theme::PRIMARY_LIGHT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::TEXT_PRIMARY)
    };

    let header_widget = Paragraph::new(header)
        .style(header_style)
        .alignment(Alignment::Center);

    frame.render_widget(header_widget, area);
}

/// Renders the 7 day cards for the week.
fn render_day_cards(frame: &mut Frame, state: &CalendarState, app: &App, area: Rect) {
    let today = Local::now().date_naive();

    // Create 7 equal columns for each day
    let day_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 7),
            Constraint::Ratio(1, 7),
            Constraint::Ratio(1, 7),
            Constraint::Ratio(1, 7),
            Constraint::Ratio(1, 7),
            Constraint::Ratio(1, 7),
            Constraint::Ratio(1, 7),
        ])
        .split(area);

    let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    for (i, (chunk, day_name)) in day_chunks.iter().zip(day_names.iter()).enumerate() {
        let date = state.week_start + Duration::days(i as i64);
        let is_selected = i == state.selected_day;
        let is_today = date == today;
        let is_weekend = i >= 5;

        // Count tasks for this day
        let task_count = count_tasks_for_date(app, date);
        let has_overdue = has_overdue_tasks_for_date(app, date);

        render_day_card(
            frame,
            *chunk,
            day_name,
            date.day(),
            is_selected,
            is_today,
            is_weekend,
            task_count,
            has_overdue,
        );
    }
}

/// Renders a single day card.
#[allow(clippy::too_many_arguments)]
fn render_day_card(
    frame: &mut Frame,
    area: Rect,
    day_name: &str,
    day_num: u32,
    is_selected: bool,
    is_today: bool,
    is_weekend: bool,
    task_count: usize,
    has_overdue: bool,
) {
    // Determine styling
    let (border_color, bg_color) = if is_selected {
        (theme::PRIMARY_LIGHT, Some(theme::BG_SELECTION))
    } else if is_today {
        (theme::ACCENT, None)
    } else if is_weekend {
        (theme::BORDER_MUTED, None)
    } else {
        (theme::BORDER, None)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Fill background if selected
    if let Some(bg) = bg_color {
        let bg_widget = Paragraph::new("").style(Style::default().bg(bg));
        frame.render_widget(bg_widget, inner);
    }

    // Day content layout
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Day name
            Constraint::Length(1), // Day number
            Constraint::Length(1), // Task indicator
        ])
        .split(inner);

    // Day name
    let name_style = if is_weekend {
        Style::default().fg(theme::TEXT_MUTED)
    } else {
        Style::default().fg(theme::TEXT_SECONDARY)
    };
    let name_widget = Paragraph::new(day_name)
        .style(name_style)
        .alignment(Alignment::Center);
    frame.render_widget(name_widget, content_chunks[0]);

    // Day number
    let num_style = if is_today {
        Style::default()
            .fg(theme::ACCENT)
            .add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default()
            .fg(theme::PRIMARY_LIGHT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::TEXT_PRIMARY)
    };
    let num_widget = Paragraph::new(format!("{}", day_num))
        .style(num_style)
        .alignment(Alignment::Center);
    frame.render_widget(num_widget, content_chunks[1]);

    // Task indicator
    if task_count > 0 {
        let indicator_color = if has_overdue {
            theme::ERROR
        } else {
            theme::SUCCESS
        };
        let indicator = if task_count <= 3 {
            "●".repeat(task_count)
        } else {
            format!("●{}", task_count)
        };
        let indicator_widget = Paragraph::new(indicator)
            .style(Style::default().fg(indicator_color))
            .alignment(Alignment::Center);
        frame.render_widget(indicator_widget, content_chunks[2]);
    }
}

/// Gets the IDs of tasks due on the selected day, filtered and sorted.
///
/// Returns a vector of task IDs for the currently selected day in the calendar.
/// Respects the show_completed filter and applies default sort order.
pub fn get_tasks_for_selected_day(app: &App) -> Vec<String> {
    let selected_date = app.calendar_state.selected_date();
    let show_completed = app.calendar_state.show_completed;

    // Filter tasks for the selected date
    let mut tasks: Vec<&Task> = app
        .tasks
        .iter()
        .filter(|t| {
            // Must have due date on the selected day
            let is_on_date = t
                .due_date
                .map(|d| d.with_timezone(&Local).date_naive() == selected_date)
                .unwrap_or(false);

            // Apply completed filter
            let passes_completed_filter = show_completed
                || (t.status != TaskStatus::Completed && t.status != TaskStatus::Archived);

            is_on_date && passes_completed_filter
        })
        .collect();

    // Sort using default sort order (DueDateAsc, which for same-day tasks falls back to priority)
    SortOrder::default().apply(&mut tasks);

    tasks.iter().map(|t| t.id.clone()).collect()
}

/// Gets the count of tasks for the selected day.
pub fn get_task_count_for_selected_day(app: &App) -> usize {
    get_tasks_for_selected_day(app).len()
}

/// Counts tasks due on a specific date.
fn count_tasks_for_date(app: &App, date: NaiveDate) -> usize {
    app.tasks
        .iter()
        .filter(|t| {
            t.status != TaskStatus::Archived
                && t.due_date
                    .map(|d| d.with_timezone(&Local).date_naive() == date)
                    .unwrap_or(false)
        })
        .count()
}

/// Checks if there are overdue tasks for a specific date.
fn has_overdue_tasks_for_date(app: &App, date: NaiveDate) -> bool {
    let today = Local::now().date_naive();
    if date >= today {
        return false;
    }

    app.tasks.iter().any(|t| {
        t.status != TaskStatus::Completed
            && t.status != TaskStatus::Archived
            && t.due_date
                .map(|d| d.with_timezone(&Local).date_naive() == date)
                .unwrap_or(false)
    })
}

/// Renders the task list for the selected day.
fn render_day_tasks(frame: &mut Frame, state: &CalendarState, app: &App, area: Rect) {
    let selected_date = state.selected_date();
    let today = Local::now().date_naive();
    let is_focused = state.focus == CalendarFocus::TaskList;

    // Get filtered and sorted task IDs
    let task_ids = get_tasks_for_selected_day(app);

    // Get task references in order
    let tasks: Vec<&Task> = task_ids
        .iter()
        .filter_map(|id| app.tasks.iter().find(|t| &t.id == id))
        .collect();

    // Block title with date and filter status
    let date_str = if selected_date == today {
        "Today".to_string()
    } else if selected_date == today + Duration::days(1) {
        "Tomorrow".to_string()
    } else if selected_date == today - Duration::days(1) {
        "Yesterday".to_string()
    } else {
        selected_date.format("%A, %B %d").to_string()
    };

    let filter_indicator = if state.show_completed {
        format!(" {} All ", icons::DOT)
    } else {
        format!(" {} Active ", icons::DOT)
    };

    // Highlight border when task list is focused
    let border_color = if is_focused {
        theme::PRIMARY_LIGHT
    } else {
        theme::BORDER
    };

    let title_style = if is_focused {
        Style::default()
            .fg(theme::PRIMARY_LIGHT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::TEXT_PRIMARY)
    };

    let title = Line::from(vec![
        Span::styled(format!(" {} ", date_str), title_style),
        Span::styled(filter_indicator, Style::default().fg(theme::TEXT_MUTED)),
    ]);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(border_color));

    if tasks.is_empty() {
        let empty_msg = if is_focused {
            "No tasks - press Tab to return to calendar"
        } else {
            "No tasks due on this day"
        };
        let msg = Paragraph::new(empty_msg)
            .style(Style::default().fg(theme::TEXT_MUTED))
            .alignment(Alignment::Center)
            .block(block);
        frame.render_widget(msg, area);
        return;
    }

    // Calculate available width for title
    let inner_width = area.width.saturating_sub(4); // Border padding

    // Build task list with selection highlighting - matching main task list style
    let items: Vec<ListItem> = tasks
        .iter()
        .enumerate()
        .map(|(idx, task)| {
            let is_selected = is_focused && idx == state.selected_task_index;

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

            // Completion date for completed tasks
            let completed_str = if task.status == TaskStatus::Completed || task.status == TaskStatus::Archived {
                task.completed_at
                    .map(|d| format!(" {} {}", icons::CHECK, format_relative_date(d)))
            } else {
                None
            };

            // Format tags string
            let tags_str = render_tags(&task.tags, 15);

            // Format project string
            let project_str = task
                .project_id
                .as_ref()
                .and_then(|pid| app.projects.iter().find(|p| &p.id == pid))
                .map(|p| format!("{}{}", icons::PROJECT_PREFIX, p.name));

            // Calculate available width for title
            let meta_width = project_str.as_ref().map(|s| s.len() + 1).unwrap_or(0)
                + if !tags_str.is_empty() { tags_str.len() + 1 } else { 0 }
                + completed_str.as_ref().map(|s| s.len() + 1).unwrap_or(0);
            let fixed_width = 3 + 2 + meta_width + 2; // status + priority + meta + padding
            let title_width = (inner_width as usize).saturating_sub(fixed_width).max(10);

            // Truncate title if needed
            let title = if task.title.len() > title_width {
                format!("{}...", &task.title[..title_width.saturating_sub(3)])
            } else {
                task.title.clone()
            };

            // Title style based on task state
            let base_title_style = if task.status == TaskStatus::Completed || task.status == TaskStatus::Archived {
                Style::default().fg(theme::TEXT_COMPLETED)
            } else if task.is_overdue() {
                Style::default().fg(theme::DUE_OVERDUE)
            } else {
                Style::default().fg(theme::TEXT_PRIMARY)
            };

            // Apply selection background
            let (title_style, status_style, priority_style, project_style, tag_style, completed_style) = if is_selected {
                (
                    base_title_style.bg(theme::BG_SELECTION).add_modifier(Modifier::BOLD),
                    status_style.bg(theme::BG_SELECTION),
                    priority_style.bg(theme::BG_SELECTION),
                    Style::default().fg(theme::PROJECT).bg(theme::BG_SELECTION),
                    Style::default().fg(theme::TAG).bg(theme::BG_SELECTION),
                    Style::default().fg(theme::TEXT_COMPLETED).bg(theme::BG_SELECTION),
                )
            } else {
                (
                    base_title_style,
                    status_style,
                    priority_style,
                    Style::default().fg(theme::PROJECT),
                    Style::default().fg(theme::TAG),
                    Style::default().fg(theme::TEXT_COMPLETED),
                )
            };

            // Build the line with spans
            let mut spans = vec![
                Span::styled(format!(" {} ", status_icon), status_style),
                Span::styled(format!("{} ", priority_icon), priority_style),
                Span::styled(format!("{:<width$}", title, width = title_width), title_style),
            ];

            // Add project name if present
            if let Some(ref project) = project_str {
                spans.push(Span::styled(format!(" {}", project), project_style));
            }

            // Add tags if present
            if !tags_str.is_empty() {
                spans.push(Span::styled(format!(" {}", tags_str), tag_style));
            }

            // Add completion date if present
            if let Some(ref completed) = completed_str {
                spans.push(Span::styled(completed.clone(), completed_style));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
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

/// Renders the help line based on current focus.
fn render_help_line(frame: &mut Frame, state: &CalendarState, area: Rect) {
    let help = match state.focus {
        CalendarFocus::DayGrid => Line::from(vec![
            Span::styled("[←/→]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled(" Day  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("[↑/↓]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled(" Week  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("[t]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled(" Today  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("[f]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled(" Filter  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("[Tab]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled(" Tasks  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("[Esc]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled(" Back", Style::default().fg(theme::TEXT_MUTED)),
        ]),
        CalendarFocus::TaskList => Line::from(vec![
            Span::styled("[j/k]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled(" Nav  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("[Space]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled(" Done  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("[p]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled(" Priority  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("[e]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled(" Edit  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("[f]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled(" Filter  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("[Enter]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled(" Go  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("[Tab]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled(" Cal", Style::default().fg(theme::TEXT_MUTED)),
        ]),
    };

    frame.render_widget(Paragraph::new(help), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Weekday;

    #[test]
    fn test_calendar_state_new() {
        let state = CalendarState::new();
        let today = Local::now().date_naive();

        // Week start should be a Monday
        assert_eq!(state.week_start.weekday(), Weekday::Mon);

        // Selected day should be today's weekday index
        assert_eq!(
            state.selected_day,
            today.weekday().num_days_from_monday() as usize
        );
    }

    #[test]
    fn test_calendar_state_selected_date() {
        let state = CalendarState::new();
        let today = Local::now().date_naive();

        // Selected date should be today
        assert_eq!(state.selected_date(), today);
    }

    #[test]
    fn test_calendar_state_next_day() {
        let mut state = CalendarState::new();
        let initial_date = state.selected_date();

        state.next_day();

        assert_eq!(state.selected_date(), initial_date + Duration::days(1));
    }

    #[test]
    fn test_calendar_state_prev_day() {
        let mut state = CalendarState::new();
        let initial_date = state.selected_date();

        state.prev_day();

        assert_eq!(state.selected_date(), initial_date - Duration::days(1));
    }

    #[test]
    fn test_calendar_state_next_week() {
        let mut state = CalendarState::new();
        let initial_week_start = state.week_start;

        state.next_week();

        assert_eq!(state.week_start, initial_week_start + Duration::days(7));
    }

    #[test]
    fn test_calendar_state_prev_week() {
        let mut state = CalendarState::new();
        let initial_week_start = state.week_start;

        state.prev_week();

        assert_eq!(state.week_start, initial_week_start - Duration::days(7));
    }

    #[test]
    fn test_calendar_state_goto_today() {
        let mut state = CalendarState::new();
        let today = Local::now().date_naive();

        // Move away from today
        state.next_week();
        state.next_week();
        state.selected_day = 0;

        // Jump back to today
        state.goto_today();

        assert_eq!(state.selected_date(), today);
    }

    #[test]
    fn test_calendar_state_day_wrap_forward() {
        let mut state = CalendarState::new();
        state.selected_day = 6; // Sunday
        let initial_week_start = state.week_start;

        state.next_day();

        // Should wrap to Monday of next week
        assert_eq!(state.selected_day, 0);
        assert_eq!(state.week_start, initial_week_start + Duration::days(7));
    }

    #[test]
    fn test_calendar_state_day_wrap_backward() {
        let mut state = CalendarState::new();
        state.selected_day = 0; // Monday
        let initial_week_start = state.week_start;

        state.prev_day();

        // Should wrap to Sunday of previous week
        assert_eq!(state.selected_day, 6);
        assert_eq!(state.week_start, initial_week_start - Duration::days(7));
    }
}
