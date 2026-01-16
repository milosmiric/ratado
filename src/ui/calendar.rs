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
use crate::models::{Priority, Task, TaskStatus};
use super::theme;

/// State for the calendar view.
///
/// Tracks the currently displayed week and selected day.
#[derive(Debug, Clone)]
pub struct CalendarState {
    /// The Monday of the currently displayed week
    pub week_start: NaiveDate,
    /// The currently selected day (0-6, Monday-Sunday)
    pub selected_day: usize,
}

impl Default for CalendarState {
    fn default() -> Self {
        let today = Local::now().date_naive();
        let week_start = today - Duration::days(today.weekday().num_days_from_monday() as i64);
        let selected_day = today.weekday().num_days_from_monday() as usize;
        Self {
            week_start,
            selected_day,
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
    render_help_line(frame, chunks[4]);
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

    // Get tasks for the selected date
    let tasks: Vec<&Task> = app
        .tasks
        .iter()
        .filter(|t| {
            t.due_date
                .map(|d| d.with_timezone(&Local).date_naive() == selected_date)
                .unwrap_or(false)
        })
        .collect();

    // Block title with date
    let date_str = if selected_date == today {
        "Today".to_string()
    } else if selected_date == today + Duration::days(1) {
        "Tomorrow".to_string()
    } else if selected_date == today - Duration::days(1) {
        "Yesterday".to_string()
    } else {
        selected_date.format("%A, %B %d").to_string()
    };

    let block = Block::default()
        .title(Span::styled(
            format!(" Tasks for {} ", date_str),
            Style::default().fg(theme::TEXT_PRIMARY),
        ))
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(theme::BORDER));

    if tasks.is_empty() {
        let empty_msg = Paragraph::new("No tasks due on this day")
            .style(Style::default().fg(theme::TEXT_MUTED))
            .alignment(Alignment::Center)
            .block(block);
        frame.render_widget(empty_msg, area);
        return;
    }

    // Build task list
    let items: Vec<ListItem> = tasks
        .iter()
        .map(|task| {
            let checkbox = match task.status {
                TaskStatus::Pending => "[ ]",
                TaskStatus::InProgress => "[▸]",
                TaskStatus::Completed | TaskStatus::Archived => "[✓]",
            };

            let priority_indicator = match task.priority {
                Priority::Urgent => ("!!", theme::PRIORITY_URGENT),
                Priority::High => ("! ", theme::PRIORITY_HIGH),
                Priority::Medium => ("  ", theme::TEXT_PRIMARY),
                Priority::Low => ("↓ ", theme::PRIORITY_LOW),
            };

            let title_style = if task.status == TaskStatus::Completed {
                Style::default().fg(theme::TEXT_COMPLETED)
            } else if task.is_overdue() {
                Style::default().fg(theme::ERROR)
            } else {
                Style::default().fg(theme::TEXT_PRIMARY)
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", checkbox), title_style),
                Span::styled(
                    priority_indicator.0,
                    Style::default().fg(priority_indicator.1),
                ),
                Span::styled(task.title.clone(), title_style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

/// Renders the help line.
fn render_help_line(frame: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled("[←/→]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
        Span::styled(" Day  ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled("[↑/↓]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
        Span::styled(" Week  ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled("[t]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
        Span::styled(" Today  ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled("[Esc]", Style::default().fg(theme::PRIMARY_LIGHT).add_modifier(Modifier::BOLD)),
        Span::styled(" Back", Style::default().fg(theme::TEXT_MUTED)),
    ]);

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
