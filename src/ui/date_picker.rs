//! Date picker calendar widget.
//!
//! A visual calendar widget for selecting dates using keyboard navigation.

use chrono::{Datelike, Duration, NaiveDate, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// A calendar-based date picker widget.
#[derive(Debug, Clone)]
pub struct DatePicker {
    /// Currently selected date
    selected: NaiveDate,
    /// Currently viewed month (for navigation)
    view_month: NaiveDate,
}

impl DatePicker {
    /// Creates a new date picker with today's date selected.
    pub fn new() -> Self {
        let today = Utc::now().date_naive();
        Self {
            selected: today,
            view_month: today,
        }
    }

    /// Creates a date picker with a specific date selected.
    pub fn with_date(date: NaiveDate) -> Self {
        Self {
            selected: date,
            view_month: date,
        }
    }

    /// Returns the currently selected date.
    pub fn selected(&self) -> NaiveDate {
        self.selected
    }

    /// Sets the selected date.
    pub fn set_date(&mut self, date: NaiveDate) {
        self.selected = date;
        self.view_month = date;
    }

    /// Handles a key event and returns true if the picker should close.
    pub fn handle_key(&mut self, key: KeyEvent) -> DatePickerAction {
        match key.code {
            // Navigation
            KeyCode::Left | KeyCode::Char('h') => {
                self.selected -= Duration::days(1);
                self.ensure_view_contains_selected();
                DatePickerAction::None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.selected += Duration::days(1);
                self.ensure_view_contains_selected();
                DatePickerAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected -= Duration::days(7);
                self.ensure_view_contains_selected();
                DatePickerAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected += Duration::days(7);
                self.ensure_view_contains_selected();
                DatePickerAction::None
            }

            // Month navigation
            KeyCode::PageUp | KeyCode::Char('H') => {
                self.prev_month();
                DatePickerAction::None
            }
            KeyCode::PageDown | KeyCode::Char('L') => {
                self.next_month();
                DatePickerAction::None
            }

            // Quick jumps
            KeyCode::Char('t') => {
                self.selected = Utc::now().date_naive();
                self.ensure_view_contains_selected();
                DatePickerAction::None
            }

            // Confirm/Cancel
            KeyCode::Enter => DatePickerAction::Select,
            KeyCode::Esc | KeyCode::Char('q') => DatePickerAction::Cancel,

            _ => DatePickerAction::None,
        }
    }

    /// Move view to previous month.
    fn prev_month(&mut self) {
        let year = self.view_month.year();
        let month = self.view_month.month();
        self.view_month = if month == 1 {
            NaiveDate::from_ymd_opt(year - 1, 12, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month - 1, 1).unwrap()
        };
        // Also move selected to be in the visible month
        self.selected = NaiveDate::from_ymd_opt(
            self.view_month.year(),
            self.view_month.month(),
            self.selected.day().min(days_in_month(self.view_month)),
        )
        .unwrap();
    }

    /// Move view to next month.
    fn next_month(&mut self) {
        let year = self.view_month.year();
        let month = self.view_month.month();
        self.view_month = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        };
        // Also move selected to be in the visible month
        self.selected = NaiveDate::from_ymd_opt(
            self.view_month.year(),
            self.view_month.month(),
            self.selected.day().min(days_in_month(self.view_month)),
        )
        .unwrap();
    }

    /// Ensure the view month contains the selected date.
    fn ensure_view_contains_selected(&mut self) {
        if self.selected.year() != self.view_month.year()
            || self.selected.month() != self.view_month.month()
        {
            self.view_month = NaiveDate::from_ymd_opt(
                self.selected.year(),
                self.selected.month(),
                1,
            )
            .unwrap();
        }
    }

    /// Renders the date picker as a popup.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Calculate popup size and position (bigger for better readability)
        let popup_width = 32;
        let popup_height = 15;
        let popup_area = centered_rect(popup_width, popup_height, area);

        // Clear background
        frame.render_widget(Clear, area);
        let dim = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(dim, area);

        // Draw popup border
        let block = Block::default()
            .title(" Select Date ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Layout
        let chunks = Layout::vertical([
            Constraint::Length(2), // Month/Year header with spacing
            Constraint::Length(1), // Day names
            Constraint::Length(1), // Spacer
            Constraint::Min(6),    // Calendar grid
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Help text
        ])
        .split(inner);

        // Month/Year header with navigation hints
        let month_name = month_name(self.view_month.month());
        let header = format!(
            "◀  {} {}  ▶",
            month_name,
            self.view_month.year()
        );
        let header_para = Paragraph::new(header)
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(header_para, chunks[0]);

        // Day names (Monday to Sunday)
        let day_names = " Mo  Tu  We  Th  Fr  Sa  Su";
        let day_names_para = Paragraph::new(day_names)
            .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(day_names_para, chunks[1]);

        // Calendar grid
        let grid = self.build_calendar_grid();
        let grid_para = Paragraph::new(grid).alignment(Alignment::Center);
        frame.render_widget(grid_para, chunks[3]);

        // Help text
        let help = "←↓↑→:nav  PgUp/Dn:month  t:today  Enter:ok";
        let help_para = Paragraph::new(help)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(help_para, chunks[5]);
    }

    /// Builds the calendar grid as styled lines (Monday to Sunday).
    fn build_calendar_grid(&self) -> Vec<Line<'static>> {
        let today = Utc::now().date_naive();
        let first_of_month = NaiveDate::from_ymd_opt(
            self.view_month.year(),
            self.view_month.month(),
            1,
        )
        .unwrap();
        let days_in_month = days_in_month(first_of_month);
        // Use Monday as first day of week (0 = Monday, 6 = Sunday)
        let start_weekday = first_of_month.weekday().num_days_from_monday() as usize;

        let mut lines = Vec::new();
        let mut current_line: Vec<Span> = Vec::new();

        // Add empty spaces for days before the 1st
        for _ in 0..start_weekday {
            current_line.push(Span::raw("    "));
        }

        for day in 1..=days_in_month {
            let date = NaiveDate::from_ymd_opt(
                self.view_month.year(),
                self.view_month.month(),
                day,
            )
            .unwrap();

            let is_selected = date == self.selected;
            let is_today = date == today;
            // Weekend check: Saturday = 5, Sunday = 6 (from Monday)
            let is_weekend = date.weekday().num_days_from_monday() >= 5;

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_today {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_weekend {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            current_line.push(Span::styled(format!(" {:2} ", day), style));

            // If we've reached Sunday (7 items), start a new line
            if current_line.len() == 7 {
                lines.push(Line::from(current_line));
                current_line = Vec::new();
            }
        }

        // Add remaining days if any
        if !current_line.is_empty() {
            while current_line.len() < 7 {
                current_line.push(Span::raw("    "));
            }
            lines.push(Line::from(current_line));
        }

        // Pad to 6 lines for consistent height
        while lines.len() < 6 {
            lines.push(Line::from(""));
        }

        lines
    }
}

impl Default for DatePicker {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of handling a key in the date picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatePickerAction {
    /// No action, continue showing picker
    None,
    /// User selected a date
    Select,
    /// User cancelled
    Cancel,
}

/// Returns the number of days in the month containing the given date.
fn days_in_month(date: NaiveDate) -> u32 {
    let year = date.year();
    let month = date.month();
    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    }
    .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
    .num_days() as u32
}

/// Returns the name of a month (1-12).
fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

/// Centers a rect within another rect.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_new_picker() {
        let picker = DatePicker::new();
        assert_eq!(picker.selected(), Utc::now().date_naive());
    }

    #[test]
    fn test_navigation_right() {
        let mut picker = DatePicker::with_date(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
        picker.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(picker.selected(), NaiveDate::from_ymd_opt(2025, 1, 16).unwrap());
    }

    #[test]
    fn test_navigation_down() {
        let mut picker = DatePicker::with_date(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
        picker.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(picker.selected(), NaiveDate::from_ymd_opt(2025, 1, 22).unwrap());
    }

    #[test]
    fn test_month_navigation() {
        let mut picker = DatePicker::with_date(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
        picker.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        assert_eq!(picker.view_month.month(), 2);
    }

    #[test]
    fn test_enter_selects() {
        let mut picker = DatePicker::new();
        let action = picker.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DatePickerAction::Select);
    }

    #[test]
    fn test_escape_cancels() {
        let mut picker = DatePicker::new();
        let action = picker.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(action, DatePickerAction::Cancel);
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()), 31);
        assert_eq!(days_in_month(NaiveDate::from_ymd_opt(2025, 2, 1).unwrap()), 28);
        assert_eq!(days_in_month(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()), 29); // Leap year
        assert_eq!(days_in_month(NaiveDate::from_ymd_opt(2025, 4, 1).unwrap()), 30);
    }
}
