//! Add/Edit task dialog.
//!
//! This dialog handles both creating new tasks and editing existing ones.
//! Includes calendar picker integration for date selection, tag input
//! with autocomplete, and a multi-line description with clickable links.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::models::{Priority, Task};
use crate::storage::Tag;
use crate::ui::date_picker::{DatePicker, DatePickerAction};
use crate::ui::description_textarea::{DescriptionTextArea, TextAreaAction};
use crate::ui::dialogs::{centered_rect, DialogAction};
use crate::ui::input::TextInput;
use crate::ui::tag_input::TagInput;

/// The currently focused field in the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AddTaskField {
    #[default]
    Title,
    Description,
    DueDate,
    Priority,
    Tags,
    Submit,
}

impl AddTaskField {
    /// Returns the next field in tab order.
    fn next(self) -> Self {
        match self {
            Self::Title => Self::Description,
            Self::Description => Self::DueDate,
            Self::DueDate => Self::Priority,
            Self::Priority => Self::Tags,
            Self::Tags => Self::Submit,
            Self::Submit => Self::Title,
        }
    }

    /// Returns the previous field in tab order.
    fn prev(self) -> Self {
        match self {
            Self::Title => Self::Submit,
            Self::Description => Self::Title,
            Self::DueDate => Self::Description,
            Self::Priority => Self::DueDate,
            Self::Tags => Self::Priority,
            Self::Submit => Self::Tags,
        }
    }
}

/// Dialog for adding or editing a task.
#[derive(Debug, Clone)]
pub struct AddTaskDialog {
    /// Title input field
    pub title: TextInput,
    /// Description textarea (multi-line with link support)
    pub description: DescriptionTextArea,
    /// Due date as text (parsed on submit)
    pub due_date: TextInput,
    /// Selected priority
    pub priority: Priority,
    /// Project ID to assign
    pub project_id: Option<String>,
    /// Tag input field
    pub tags: TagInput,
    /// All available tags for autocomplete
    all_tags: Vec<Tag>,
    /// Currently focused field
    pub focused_field: AddTaskField,
    /// ID of task being edited (None for new task)
    pub editing_task_id: Option<String>,
    /// Title for the dialog
    dialog_title: String,
    /// Calendar picker (shown when user presses 'c' on due date field)
    date_picker: Option<DatePicker>,
    /// Status message to show (e.g., "Link opened")
    pub status_message: Option<String>,
}

impl AddTaskDialog {
    /// Creates a new dialog for adding a task.
    pub fn new() -> Self {
        Self {
            title: TextInput::new().with_placeholder("Task title..."),
            description: DescriptionTextArea::new(),
            due_date: TextInput::new().with_placeholder("today, +1d, mon, c=calendar"),
            priority: Priority::Medium,
            project_id: None,
            tags: TagInput::new(),
            all_tags: Vec::new(),
            focused_field: AddTaskField::Title,
            editing_task_id: None,
            dialog_title: "Add Task".to_string(),
            date_picker: None,
            status_message: None,
        }
    }

    /// Creates a dialog pre-populated with an existing task for editing.
    pub fn from_task(task: &Task) -> Self {
        let due_date_str = task
            .due_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();

        Self {
            title: TextInput::with_value(&task.title),
            description: DescriptionTextArea::with_text(task.description.as_deref().unwrap_or("")),
            due_date: TextInput::with_value(due_date_str),
            priority: task.priority,
            project_id: task.project_id.clone(),
            tags: TagInput::with_tags(task.tags.clone()),
            all_tags: Vec::new(),
            focused_field: AddTaskField::Title,
            editing_task_id: Some(task.id.clone()),
            dialog_title: "Edit Task".to_string(),
            date_picker: None,
            status_message: None,
        }
    }

    /// Sets the available tags for autocomplete.
    pub fn with_available_tags(mut self, tags: Vec<Tag>) -> Self {
        self.all_tags = tags;
        self
    }

    /// Returns whether this is editing an existing task.
    pub fn is_editing(&self) -> bool {
        self.editing_task_id.is_some()
    }

    /// Handles a key event and returns the resulting action.
    pub fn handle_key(&mut self, key: KeyEvent) -> DialogAction {
        // If date picker is active, route input to it
        if let Some(ref mut picker) = self.date_picker {
            match picker.handle_key(key) {
                DatePickerAction::Select => {
                    // User selected a date, populate the due date field
                    let selected = picker.selected();
                    self.due_date.set_value(selected.format("%Y-%m-%d").to_string());
                    self.date_picker = None;
                    return DialogAction::None;
                }
                DatePickerAction::Cancel => {
                    // User cancelled, just close the picker
                    self.date_picker = None;
                    return DialogAction::None;
                }
                DatePickerAction::None => {
                    return DialogAction::None;
                }
            }
        }

        match key.code {
            // Cancel on Escape
            KeyCode::Esc => DialogAction::Cancel,

            // Submit on Enter when on Submit button, or Ctrl+Enter anywhere
            KeyCode::Enter if self.focused_field == AddTaskField::Submit => DialogAction::Submit,
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => DialogAction::Submit,

            // Tab navigation - but check if TagInput wants to handle it first
            KeyCode::Tab => {
                // If on Tags field with suggestions, let TagInput accept the suggestion
                if self.focused_field == AddTaskField::Tags && self.tags.selected_suggestion().is_some() {
                    let all_tags = self.all_tags.clone();
                    self.tags.handle_key(key, &all_tags);
                    DialogAction::None
                } else {
                    self.focused_field = self.focused_field.next();
                    DialogAction::None
                }
            }
            KeyCode::BackTab => {
                self.focused_field = self.focused_field.prev();
                DialogAction::None
            }

            // Field-specific handling
            _ => {
                match self.focused_field {
                    AddTaskField::Title => self.handle_text_input(&mut self.title.clone(), key),
                    AddTaskField::Description => self.handle_description_input(key),
                    AddTaskField::DueDate => self.handle_due_date_input(key),
                    AddTaskField::Priority => self.handle_priority_input(key),
                    AddTaskField::Tags => self.handle_tags_input(key),
                    AddTaskField::Submit => {
                        // Enter handled above, other keys do nothing
                        DialogAction::None
                    }
                }
            }
        }
    }

    /// Handles input for the description field (multi-line textarea with link support).
    fn handle_description_input(&mut self, key: KeyEvent) -> DialogAction {
        match self.description.handle_key(key) {
            TextAreaAction::None => DialogAction::None,
            TextAreaAction::OpenLink(url) => {
                // Open the link in the default browser
                if open::that(&url).is_ok() {
                    self.status_message = Some(format!("Opened: {}", url));
                } else {
                    self.status_message = Some("Failed to open link".to_string());
                }
                DialogAction::None
            }
            TextAreaAction::NextField => {
                self.focused_field = self.focused_field.next();
                DialogAction::None
            }
            TextAreaAction::PrevField => {
                self.focused_field = self.focused_field.prev();
                DialogAction::None
            }
        }
    }

    /// Handles input for the tags field.
    fn handle_tags_input(&mut self, key: KeyEvent) -> DialogAction {
        // Let the tag input handle the key
        let all_tags = self.all_tags.clone();
        self.tags.handle_key(key, &all_tags);
        DialogAction::None
    }

    /// Handles input for the due date field, including calendar picker trigger.
    fn handle_due_date_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            // 'c' opens the calendar picker
            KeyCode::Char('c') => {
                // Initialize calendar with current due date if one is already parsed
                let initial_date = parse_due_date(self.due_date.value().trim());
                self.date_picker = Some(if let Some(dt) = initial_date {
                    DatePicker::with_date(dt.date_naive())
                } else {
                    DatePicker::new()
                });
                DialogAction::None
            }
            // Regular text input for other keys
            KeyCode::Char(c) => {
                self.due_date.insert(c);
                DialogAction::None
            }
            KeyCode::Backspace => {
                self.due_date.delete_backward();
                DialogAction::None
            }
            KeyCode::Delete => {
                self.due_date.delete_forward();
                DialogAction::None
            }
            KeyCode::Left => {
                self.due_date.move_left();
                DialogAction::None
            }
            KeyCode::Right => {
                self.due_date.move_right();
                DialogAction::None
            }
            KeyCode::Home => {
                self.due_date.move_home();
                DialogAction::None
            }
            KeyCode::End => {
                self.due_date.move_end();
                DialogAction::None
            }
            _ => DialogAction::None,
        }
    }

    /// Handles text input for the current field.
    fn handle_text_input(&mut self, _input: &mut TextInput, key: KeyEvent) -> DialogAction {
        let input = match self.focused_field {
            AddTaskField::Title => &mut self.title,
            AddTaskField::DueDate => &mut self.due_date,
            _ => return DialogAction::None,
        };

        match key.code {
            // Word navigation (Emacs-style Alt+b/Alt+f for macOS Option+Arrow)
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::ALT) => input.move_word_left(),
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::ALT) => input.move_word_right(),
            KeyCode::Char(c) => input.insert(c),
            // Word deletion (Alt+Backspace)
            KeyCode::Backspace if key.modifiers.contains(KeyModifiers::ALT) => input.delete_word_backward(),
            KeyCode::Backspace => input.delete_backward(),
            KeyCode::Delete => input.delete_forward(),
            // Word navigation (Alt+Arrow)
            KeyCode::Left if key.modifiers.contains(KeyModifiers::ALT) => input.move_word_left(),
            KeyCode::Right if key.modifiers.contains(KeyModifiers::ALT) => input.move_word_right(),
            KeyCode::Left => input.move_left(),
            KeyCode::Right => input.move_right(),
            KeyCode::Home => input.move_home(),
            KeyCode::End => input.move_end(),
            _ => {}
        }
        DialogAction::None
    }

    /// Handles priority selection.
    fn handle_priority_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                self.priority = match self.priority {
                    Priority::Low => Priority::Urgent,
                    Priority::Medium => Priority::Low,
                    Priority::High => Priority::Medium,
                    Priority::Urgent => Priority::High,
                };
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.priority = match self.priority {
                    Priority::Low => Priority::Medium,
                    Priority::Medium => Priority::High,
                    Priority::High => Priority::Urgent,
                    Priority::Urgent => Priority::Low,
                };
            }
            KeyCode::Char('1') => self.priority = Priority::Low,
            KeyCode::Char('2') => self.priority = Priority::Medium,
            KeyCode::Char('3') => self.priority = Priority::High,
            KeyCode::Char('4') => self.priority = Priority::Urgent,
            _ => {}
        }
        DialogAction::None
    }

    /// Creates a Task from the dialog fields.
    ///
    /// Returns None if the title is empty.
    pub fn to_task(&self) -> Option<Task> {
        let title = self.title.value().trim();
        if title.is_empty() {
            return None;
        }

        let mut task = if let Some(ref id) = self.editing_task_id {
            let mut t = Task::new(title);
            t.id = id.clone();
            t
        } else {
            Task::new(title)
        };

        // Set description
        let desc = self.description.text();
        let desc = desc.trim();
        task.description = if desc.is_empty() {
            None
        } else {
            Some(desc.to_string())
        };

        // Parse due date
        task.due_date = parse_due_date(self.due_date.value().trim());

        // Set priority
        task.priority = self.priority;

        // Set project
        task.project_id = self.project_id.clone();

        // Set tags
        task.tags = self.tags.tags_vec();

        Some(task)
    }

    /// Renders the dialog to the frame.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Dialog dimensions - increased height for textarea
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 28.min(area.height.saturating_sub(4));
        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Render background dim effect
        let dim_block = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(Clear, area);
        frame.render_widget(dim_block, area);

        // Render dialog box
        let block = Block::default()
            .title(format!(" {} ", self.dialog_title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Layout the fields
        let chunks = Layout::vertical([
            Constraint::Length(3),  // Title
            Constraint::Length(10), // Description (textarea - 8 rows + border)
            Constraint::Length(3),  // Due date
            Constraint::Length(3),  // Priority
            Constraint::Length(3),  // Tags
            Constraint::Length(1),  // Status message
            Constraint::Length(1),  // Submit button
        ])
        .split(inner);

        // Render title field
        self.render_text_field(
            frame,
            chunks[0],
            "Title",
            &self.title,
            self.focused_field == AddTaskField::Title,
        );

        // Render description textarea
        self.render_description_field(frame, chunks[1], self.focused_field == AddTaskField::Description);

        // Render due date field
        self.render_text_field(
            frame,
            chunks[2],
            "Due Date",
            &self.due_date,
            self.focused_field == AddTaskField::DueDate,
        );

        // Render priority selector
        self.render_priority_selector(frame, chunks[3], self.focused_field == AddTaskField::Priority);

        // Render tags field
        self.render_tags_field(frame, chunks[4], self.focused_field == AddTaskField::Tags);

        // Render status message if any
        if let Some(ref msg) = self.status_message {
            let status = Paragraph::new(msg.as_str())
                .style(Style::default().fg(Color::Green));
            frame.render_widget(status, chunks[5]);
        }

        // Render submit button
        self.render_submit_button(frame, chunks[6], self.focused_field == AddTaskField::Submit);

        // Render date picker on top if active
        if let Some(ref picker) = self.date_picker {
            picker.render(frame);
        }
    }

    /// Renders the description textarea.
    fn render_description_field(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let buf = frame.buffer_mut();
        self.description.render(area, buf, focused, Some("Description (Ctrl+O to open link)"));
    }

    /// Renders a text input field.
    fn render_text_field(
        &self,
        frame: &mut Frame,
        area: Rect,
        label: &str,
        input: &TextInput,
        focused: bool,
    ) {
        let buf = frame.buffer_mut();
        input.render_to_buffer(area, buf, focused, Some(label));
    }

    /// Renders the tags input field.
    fn render_tags_field(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let buf = frame.buffer_mut();
        self.tags.render_to_buffer(area, buf, focused, Some("Tags"));

        // Render autocomplete suggestions if focused and there are suggestions
        if focused {
            self.tags.render_suggestions(frame, area);
        }
    }

    /// Renders the priority selector.
    fn render_priority_selector(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .title(" Priority ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Priority options
        let priorities = [
            (Priority::Low, "Low"),
            (Priority::Medium, "Medium"),
            (Priority::High, "High"),
            (Priority::Urgent, "Urgent"),
        ];

        let mut spans = Vec::new();
        for (i, (p, name)) in priorities.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("  "));
            }

            let style = if *p == self.priority {
                Style::default()
                    .fg(priority_color(*p))
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(Color::Gray)
            };

            spans.push(Span::styled(format!(" {} ", name), style));
        }

        let line = Line::from(spans);
        frame.render_widget(Paragraph::new(line), inner);
    }

    /// Renders the submit button.
    fn render_submit_button(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let text = if self.is_editing() {
            "[ Save Changes ]"
        } else {
            "[ Create Task ]"
        };

        let style = if focused {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };

        // Center the button
        let button_width = text.len() as u16;
        let x_offset = (area.width.saturating_sub(button_width)) / 2;
        let button_area = Rect::new(area.x + x_offset, area.y, button_width, 1);

        frame.render_widget(Paragraph::new(text).style(style), button_area);
    }
}

impl Default for AddTaskDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the display color for a priority level.
fn priority_color(priority: Priority) -> Color {
    match priority {
        Priority::Low => Color::Gray,
        Priority::Medium => Color::Blue,
        Priority::High => Color::Yellow,
        Priority::Urgent => Color::Red,
    }
}

/// Parses a due date string into a DateTime.
///
/// Supports formats:
/// - "YYYY-MM-DD" or "YYYY/MM/DD"
/// - "MM/DD" or "DD/MM" (assumes current year)
/// - "today", "tomorrow", "yesterday"
/// - "+1d", "+3d", "+1w", "+2w" (relative days/weeks)
/// - "mon", "tue", "wed", "thu", "fri", "sat", "sun" (next occurrence)
/// - "next week", "next month"
fn parse_due_date(input: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    use chrono::{Datelike, Duration, Local, NaiveDate, TimeZone, Utc, Weekday};

    let input = input.to_lowercase().trim().to_string();

    if input.is_empty() {
        return None;
    }

    // Use local timezone for "today" reference
    let today = Local::now().date_naive();

    // Helper to create UTC datetime at end of day in local timezone
    let to_datetime = |date: NaiveDate| {
        // Create end of day (23:59:59) in local timezone, then convert to UTC
        let local_eod = date.and_hms_opt(23, 59, 59).unwrap();
        Local.from_local_datetime(&local_eod)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| Utc.from_utc_datetime(&local_eod))
    };

    // Keywords
    match input.as_str() {
        "today" | "tod" => return Some(to_datetime(today)),
        "tomorrow" | "tom" => return Some(to_datetime(today + Duration::days(1))),
        "yesterday" => return Some(to_datetime(today - Duration::days(1))),
        "next week" => return Some(to_datetime(today + Duration::days(7))),
        "next month" => return Some(to_datetime(today + Duration::days(30))),
        _ => {}
    }

    // Relative days: +1d, +3d, +1w, +2w, etc.
    if let Some(rest) = input.strip_prefix('+') {
        if let Some(days_str) = rest.strip_suffix('d')
            && let Ok(days) = days_str.parse::<i64>() {
                return Some(to_datetime(today + Duration::days(days)));
            }
        if let Some(weeks_str) = rest.strip_suffix('w')
            && let Ok(weeks) = weeks_str.parse::<i64>() {
                return Some(to_datetime(today + Duration::weeks(weeks)));
            }
    }

    // Weekday names: mon, tue, wed, thu, fri, sat, sun
    let weekday = match input.as_str() {
        "mon" | "monday" => Some(Weekday::Mon),
        "tue" | "tuesday" => Some(Weekday::Tue),
        "wed" | "wednesday" => Some(Weekday::Wed),
        "thu" | "thursday" => Some(Weekday::Thu),
        "fri" | "friday" => Some(Weekday::Fri),
        "sat" | "saturday" => Some(Weekday::Sat),
        "sun" | "sunday" => Some(Weekday::Sun),
        _ => None,
    };

    if let Some(target_weekday) = weekday {
        let current_weekday = today.weekday();
        let days_ahead = (target_weekday.num_days_from_monday() as i64
            - current_weekday.num_days_from_monday() as i64
            + 7) % 7;
        // If it's the same day, go to next week
        let days_ahead = if days_ahead == 0 { 7 } else { days_ahead };
        return Some(to_datetime(today + Duration::days(days_ahead)));
    }

    // Try parsing as YYYY-MM-DD
    if let Ok(date) = NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
        return Some(to_datetime(date));
    }

    // Try parsing as YYYY/MM/DD
    if let Ok(date) = NaiveDate::parse_from_str(&input, "%Y/%m/%d") {
        return Some(to_datetime(date));
    }

    // Try parsing as MM/DD (assume current year)
    if let Ok(date) = NaiveDate::parse_from_str(&format!("{}/{}", today.year(), input), "%Y/%m/%d") {
        // If date is in the past, use next year
        let date = if date < today {
            NaiveDate::from_ymd_opt(today.year() + 1, date.month(), date.day()).unwrap_or(date)
        } else {
            date
        };
        return Some(to_datetime(date));
    }

    // Try parsing as DD/MM (assume current year) - European format
    if input.contains('/') {
        let parts: Vec<&str> = input.split('/').collect();
        if parts.len() == 2
            && let (Ok(day), Ok(month)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                && day <= 31 && month <= 12
                    && let Some(date) = NaiveDate::from_ymd_opt(today.year(), month, day) {
                        let date = if date < today {
                            NaiveDate::from_ymd_opt(today.year() + 1, month, day).unwrap_or(date)
                        } else {
                            date
                        };
                        return Some(to_datetime(date));
                    }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dialog() {
        let dialog = AddTaskDialog::new();
        assert!(dialog.title.value().is_empty());
        assert!(dialog.editing_task_id.is_none());
        assert!(!dialog.is_editing());
    }

    #[test]
    fn test_from_task() {
        let mut task = Task::new("Test Task");
        task.description = Some("A description".to_string());
        task.priority = Priority::High;

        let dialog = AddTaskDialog::from_task(&task);
        assert_eq!(dialog.title.value(), "Test Task");
        assert_eq!(dialog.description.text(), "A description");
        assert_eq!(dialog.priority, Priority::High);
        assert!(dialog.is_editing());
    }

    #[test]
    fn test_to_task_empty_title() {
        let dialog = AddTaskDialog::new();
        assert!(dialog.to_task().is_none());
    }

    #[test]
    fn test_to_task_with_title() {
        let mut dialog = AddTaskDialog::new();
        dialog.title.set_value("My Task");
        dialog.priority = Priority::Urgent;

        let task = dialog.to_task().unwrap();
        assert_eq!(task.title, "My Task");
        assert_eq!(task.priority, Priority::Urgent);
    }

    #[test]
    fn test_parse_due_date_today() {
        let date = parse_due_date("today");
        assert!(date.is_some());
    }

    #[test]
    fn test_parse_due_date_iso() {
        let date = parse_due_date("2025-12-31");
        assert!(date.is_some());
        let d = date.unwrap();
        assert_eq!(d.format("%Y-%m-%d").to_string(), "2025-12-31");
    }

    #[test]
    fn test_parse_due_date_empty() {
        assert!(parse_due_date("").is_none());
    }

    #[test]
    fn test_field_navigation() {
        let field = AddTaskField::Title;
        assert_eq!(field.next(), AddTaskField::Description);
        assert_eq!(field.prev(), AddTaskField::Submit);
    }

    #[test]
    fn test_handle_key_escape() {
        let mut dialog = AddTaskDialog::new();
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(dialog.handle_key(key), DialogAction::Cancel);
    }

    #[test]
    fn test_handle_key_tab() {
        let mut dialog = AddTaskDialog::new();
        assert_eq!(dialog.focused_field, AddTaskField::Title);

        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        dialog.handle_key(key);
        assert_eq!(dialog.focused_field, AddTaskField::Description);
    }
}
