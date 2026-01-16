//! Command pattern implementation for decoupling input from actions.
//!
//! This module defines all possible commands that can be executed in the application.
//! Commands encapsulate actions and can be mapped from keyboard input, allowing for
//! flexible keybinding configuration and testable action handling.
//!
//! ## Architecture
//!
//! The command pattern provides several benefits:
//! - Decouples input handling from action execution
//! - Makes keybindings configurable
//! - Enables undo/redo functionality (future)
//! - Simplifies testing of individual actions
//!
//! ## Example
//!
//! ```rust,no_run
//! use ratado::handlers::commands::Command;
//! use ratado::app::App;
//!
//! # async fn example(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
//! // Execute a navigation command
//! let should_continue = Command::NavigateDown.execute(app).await?;
//!
//! // Execute a quit command
//! let should_continue = Command::Quit.execute(app).await?;
//! assert!(!should_continue); // Quit returns false to stop the loop
//! # Ok(())
//! # }
//! ```

use log::debug;
use tui_logger::TuiWidgetEvent;

use crate::app::{App, AppError, FocusPanel, InputMode, View};
use crate::models::{Filter, Priority, Task};
use crate::ui::dialogs::{AddTaskDialog, ConfirmDialog, DeleteProjectDialog, Dialog, FilterSortDialog, MoveToProjectDialog, ProjectDialog, SettingsDialog};
use crate::ui::search::search_tasks;

/// All possible commands that can be executed in the application.
///
/// Commands are the actions that the application can perform in response
/// to user input. Each command encapsulates a specific action and knows
/// how to modify the application state.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    // === Navigation ===
    /// Move selection up in the current list
    NavigateUp,
    /// Move selection down in the current list
    NavigateDown,
    /// Jump to the top of the current list
    NavigateTop,
    /// Jump to the bottom of the current list
    NavigateBottom,
    /// Scroll down by a page
    PageDown,
    /// Scroll up by a page
    PageUp,
    /// Switch focus between panels (sidebar/task list)
    SwitchPanel,
    /// Focus the sidebar panel
    FocusSidebar,
    /// Focus the task list panel
    FocusTaskList,

    // === Task Actions ===
    /// Start adding a new task
    AddTask,
    /// Edit the selected task
    EditTask,
    /// Delete the selected task
    DeleteTask,
    /// Toggle completion status of the selected task
    ToggleTaskStatus,
    /// Cycle through priority levels for the selected task
    CyclePriority,
    /// Move selected task to a different project
    MoveToProject,
    /// Edit tags on the selected task
    EditTags,
    /// Create a new project
    AddProject,
    /// Edit the selected project
    EditProject,
    /// Delete the selected project
    DeleteProject,

    // === Views ===
    /// Show the main task list view
    ShowMain,
    /// Show the help screen
    ShowHelp,
    /// Show the calendar view
    ShowCalendar,
    /// Enter search mode
    ShowSearch,
    /// Navigate up in search results
    SearchNavigateUp,
    /// Navigate down in search results
    SearchNavigateDown,
    /// Select the current search result
    SearchSelectTask,
    /// Toggle the debug logs view
    ShowDebugLogs,
    /// Show detailed view of selected task
    ShowTaskDetail,

    // === Calendar Navigation ===
    /// Move to previous day in calendar
    CalendarPrevDay,
    /// Move to next day in calendar
    CalendarNextDay,
    /// Move to previous week in calendar
    CalendarPrevWeek,
    /// Move to next week in calendar
    CalendarNextWeek,
    /// Jump to today in calendar
    CalendarToday,
    /// Select tasks for the current calendar day
    CalendarSelectDay,
    /// Toggle focus between day grid and task list
    CalendarToggleFocus,
    /// Move to previous task in calendar task list
    CalendarTaskUp,
    /// Move to next task in calendar task list
    CalendarTaskDown,
    /// Toggle selected task status in calendar
    CalendarToggleTask,
    /// Cycle priority of selected task in calendar
    CalendarCyclePriority,
    /// Edit selected task in calendar
    CalendarEditTask,
    /// Navigate to selected task in its project
    CalendarGoToTask,
    /// Toggle showing completed tasks in calendar
    CalendarToggleCompleted,

    // === Settings ===
    /// Show the settings dialog
    ShowSettings,

    // === Filters ===
    /// Set a specific filter
    SetFilter(Filter),
    /// Filter to show only tasks due today
    FilterToday,
    /// Filter to show only tasks due this week
    FilterThisWeek,
    /// Filter by priority level
    FilterByPriority(Priority),
    /// Open the filter/sort selection dialog
    ShowFilterSort,

    // === Input Mode ===
    /// Enter editing mode for text input
    EnterEditMode,
    /// Exit editing mode back to normal
    ExitEditMode,
    /// Submit the current input
    SubmitInput,
    /// Cancel the current input operation
    CancelInput,

    // === Text Editing ===
    /// Insert a character at cursor position
    InsertChar(char),
    /// Delete character before cursor (backspace)
    DeleteCharBackward,
    /// Delete character at cursor (delete)
    DeleteCharForward,
    /// Move cursor left
    MoveCursorLeft,
    /// Move cursor right
    MoveCursorRight,
    /// Move cursor to start of line
    MoveCursorStart,
    /// Move cursor to end of line
    MoveCursorEnd,

    // === Debug View (tui-logger) ===
    /// Send event to tui-logger widget
    LoggerEvent(TuiLoggerEvent),

    // === Application ===
    /// Quit the application
    Quit,
    /// Force quit without confirmation
    ForceQuit,
    /// Refresh data from database
    Refresh,
}

/// Events for the tui-logger debug view.
///
/// These map to [`TuiWidgetEvent`] for controlling the log viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiLoggerEvent {
    /// Toggle focus between target selector and logs
    SpaceBar,
    /// Scroll up through targets
    Up,
    /// Scroll down through targets
    Down,
    /// Move to previous page of logs
    PrevPage,
    /// Move to next page of logs
    NextPage,
    /// Decrease logging level for selected target
    Left,
    /// Increase logging level for selected target
    Right,
    /// Enable all log levels for selected target
    Plus,
    /// Disable all log levels for selected target
    Minus,
    /// Hide selected target
    Hide,
}

impl From<TuiLoggerEvent> for TuiWidgetEvent {
    fn from(event: TuiLoggerEvent) -> Self {
        match event {
            TuiLoggerEvent::SpaceBar => TuiWidgetEvent::SpaceKey,
            TuiLoggerEvent::Up => TuiWidgetEvent::UpKey,
            TuiLoggerEvent::Down => TuiWidgetEvent::DownKey,
            TuiLoggerEvent::PrevPage => TuiWidgetEvent::PrevPageKey,
            TuiLoggerEvent::NextPage => TuiWidgetEvent::NextPageKey,
            TuiLoggerEvent::Left => TuiWidgetEvent::LeftKey,
            TuiLoggerEvent::Right => TuiWidgetEvent::RightKey,
            TuiLoggerEvent::Plus => TuiWidgetEvent::PlusKey,
            TuiLoggerEvent::Minus => TuiWidgetEvent::MinusKey,
            TuiLoggerEvent::Hide => TuiWidgetEvent::HideKey,
        }
    }
}

impl Command {
    /// Executes the command, modifying the application state.
    ///
    /// # Arguments
    ///
    /// * `app` - The application state to modify
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the application should continue running,
    /// or `Ok(false)` if the application should quit.
    ///
    /// # Errors
    ///
    /// Returns an error if a database operation fails.
    pub async fn execute(self, app: &mut App) -> Result<bool, AppError> {
        debug!("Executing command: {:?}", self);

        match self {
            // === Navigation ===
            Command::NavigateUp => {
                match app.focus {
                    FocusPanel::TaskList => app.select_previous_task(),
                    FocusPanel::Sidebar => app.select_previous_project(),
                }
                Ok(true)
            }

            Command::NavigateDown => {
                match app.focus {
                    FocusPanel::TaskList => app.select_next_task(),
                    FocusPanel::Sidebar => app.select_next_project(),
                }
                Ok(true)
            }

            Command::NavigateTop => {
                if app.focus == FocusPanel::TaskList {
                    let count = app.visible_tasks().len();
                    if count > 0 {
                        app.selected_task_index = Some(0);
                    }
                } else {
                    app.selected_project_index = 0;
                }
                Ok(true)
            }

            Command::NavigateBottom => {
                if app.focus == FocusPanel::TaskList {
                    let count = app.visible_tasks().len();
                    if count > 0 {
                        app.selected_task_index = Some(count - 1);
                    }
                } else {
                    let count = app.projects.len() + 1; // +1 for "All Tasks"
                    app.selected_project_index = count - 1;
                }
                Ok(true)
            }

            Command::PageDown => {
                // Move down by 10 items or to the end
                if app.focus == FocusPanel::TaskList {
                    let count = app.visible_tasks().len();
                    if count > 0 {
                        let current = app.selected_task_index.unwrap_or(0);
                        app.selected_task_index = Some((current + 10).min(count - 1));
                    }
                }
                Ok(true)
            }

            Command::PageUp => {
                // Move up by 10 items or to the start
                if app.focus == FocusPanel::TaskList {
                    let count = app.visible_tasks().len();
                    if count > 0 {
                        let current = app.selected_task_index.unwrap_or(0);
                        app.selected_task_index = Some(current.saturating_sub(10));
                    }
                }
                Ok(true)
            }

            Command::SwitchPanel => {
                app.toggle_focus();
                Ok(true)
            }

            Command::FocusSidebar => {
                app.focus = FocusPanel::Sidebar;
                Ok(true)
            }

            Command::FocusTaskList => {
                app.focus = FocusPanel::TaskList;
                Ok(true)
            }

            // === Task Actions ===
            Command::AddTask => {
                // Open the add task dialog with available tags for autocomplete
                let mut dialog = AddTaskDialog::new().with_available_tags(app.tags.clone());

                // Set the project to the currently selected project (if not "All Tasks")
                if let Some(project) = app.selected_project() {
                    dialog.project_id = Some(project.id.clone());
                }

                app.dialog = Some(Dialog::AddTask(Box::new(dialog)));
                app.set_status("Tab between fields, Ctrl+Enter to save");
                Ok(true)
            }

            Command::EditTask => {
                if let Some(task) = app.selected_task().cloned() {
                    // Open the add task dialog in edit mode with available tags
                    let dialog = AddTaskDialog::from_task(&task).with_available_tags(app.tags.clone());
                    app.dialog = Some(Dialog::AddTask(Box::new(dialog)));
                    app.set_status("Tab between fields, Ctrl+Enter to save");
                }
                Ok(true)
            }

            Command::DeleteTask => {
                if let Some(task) = app.selected_task() {
                    // Open confirmation dialog
                    app.dialog = Some(Dialog::Confirm(ConfirmDialog::delete_task(&task.title)));
                }
                Ok(true)
            }

            Command::ToggleTaskStatus => {
                if let Some(task) = app.selected_task() {
                    let mut task = task.clone();
                    if task.status == crate::models::TaskStatus::Completed {
                        task.reopen();
                        app.set_status("Task reopened");
                    } else {
                        task.complete();
                        app.set_status("Task completed!");
                    }
                    app.db.update_task(&task).await?;
                    app.load_data().await?;
                }
                Ok(true)
            }

            Command::CyclePriority => {
                if let Some(task) = app.selected_task() {
                    let mut task = task.clone();
                    task.priority = match task.priority {
                        Priority::Low => Priority::Medium,
                        Priority::Medium => Priority::High,
                        Priority::High => Priority::Urgent,
                        Priority::Urgent => Priority::Low,
                    };
                    app.db.update_task(&task).await?;
                    app.load_data().await?;
                    app.set_status(format!("Priority: {:?}", task.priority));
                }
                Ok(true)
            }

            Command::MoveToProject => {
                if let Some(task) = app.selected_task() {
                    let dialog = MoveToProjectDialog::new(
                        app.projects.clone(),
                        task.id.clone(),
                        task.project_id.as_deref(),
                    );
                    app.dialog = Some(Dialog::MoveToProject(dialog));
                    app.set_status("Select project to move task to");
                }
                Ok(true)
            }

            Command::EditTags => {
                // Edit tags by opening the task dialog focused on tags field
                if let Some(task) = app.selected_task().cloned() {
                    let dialog = AddTaskDialog::from_task(&task).with_available_tags(app.tags.clone());
                    app.dialog = Some(Dialog::AddTask(Box::new(dialog)));
                    app.set_status("Tab to Tags field, Enter to add tags");
                }
                Ok(true)
            }

            Command::AddProject => {
                app.dialog = Some(Dialog::Project(ProjectDialog::new()));
                app.set_status("Enter project name, Tab to navigate");
                Ok(true)
            }

            Command::EditProject => {
                // Edit the currently selected project (not "All Tasks")
                if app.selected_project_index > 0 {
                    if let Some(project) = app.selected_project().cloned() {
                        app.dialog = Some(Dialog::Project(ProjectDialog::from_project(&project)));
                        app.set_status("Edit project, Tab to navigate");
                    }
                } else {
                    app.set_status("Select a project to edit (not 'All Tasks')");
                }
                Ok(true)
            }

            Command::DeleteProject => {
                // Delete the currently selected project (not "All Tasks" or "Inbox")
                if app.selected_project_index > 0 {
                    if let Some(project) = app.selected_project().cloned() {
                        if project.id == "inbox" {
                            app.set_status("Cannot delete the Inbox project");
                        } else {
                            let task_count = app.task_count_for_project(&project.id);
                            app.dialog = Some(Dialog::DeleteProject(
                                DeleteProjectDialog::new(
                                    project.id,
                                    project.name,
                                    task_count,
                                ),
                            ));
                        }
                    }
                } else {
                    app.set_status("Select a project to delete");
                }
                Ok(true)
            }

            // === Views ===
            Command::ShowMain => {
                app.current_view = View::Main;
                Ok(true)
            }

            Command::ShowHelp => {
                app.current_view = View::Help;
                Ok(true)
            }

            Command::ShowCalendar => {
                app.current_view = View::Calendar;
                // Reset calendar to today when opening
                app.calendar_state.goto_today();
                Ok(true)
            }

            // === Calendar Navigation ===
            Command::CalendarPrevDay => {
                app.calendar_state.prev_day();
                Ok(true)
            }

            Command::CalendarNextDay => {
                app.calendar_state.next_day();
                Ok(true)
            }

            Command::CalendarPrevWeek => {
                app.calendar_state.prev_week();
                Ok(true)
            }

            Command::CalendarNextWeek => {
                app.calendar_state.next_week();
                Ok(true)
            }

            Command::CalendarToday => {
                app.calendar_state.goto_today();
                Ok(true)
            }

            Command::CalendarSelectDay => {
                // Return to main view - filter could be applied for selected day
                app.current_view = View::Main;
                Ok(true)
            }

            Command::CalendarToggleFocus => {
                app.calendar_state.toggle_focus();
                Ok(true)
            }

            Command::CalendarTaskUp => {
                app.calendar_state.prev_task();
                Ok(true)
            }

            Command::CalendarTaskDown => {
                let task_count = crate::ui::calendar::get_task_count_for_selected_day(app);
                app.calendar_state.next_task(task_count);
                Ok(true)
            }

            Command::CalendarToggleTask => {
                let task_ids = crate::ui::calendar::get_tasks_for_selected_day(app);
                if let Some(task_id) = task_ids.get(app.calendar_state.selected_task_index)
                    && let Some(task) = app.tasks.iter().find(|t| &t.id == task_id)
                {
                    let mut task = task.clone();
                    if task.status == crate::models::TaskStatus::Completed {
                        task.reopen();
                        app.set_status("Task reopened");
                    } else {
                        task.complete();
                        app.set_status("Task completed!");
                    }
                    app.db.update_task(&task).await?;
                    app.load_data().await?;
                }
                Ok(true)
            }

            Command::CalendarCyclePriority => {
                let task_ids = crate::ui::calendar::get_tasks_for_selected_day(app);
                if let Some(task_id) = task_ids.get(app.calendar_state.selected_task_index)
                    && let Some(task) = app.tasks.iter().find(|t| &t.id == task_id)
                {
                    let mut task = task.clone();
                    task.priority = match task.priority {
                        Priority::Low => Priority::Medium,
                        Priority::Medium => Priority::High,
                        Priority::High => Priority::Urgent,
                        Priority::Urgent => Priority::Low,
                    };
                    app.db.update_task(&task).await?;
                    app.load_data().await?;
                    app.set_status(format!("Priority: {:?}", task.priority));
                }
                Ok(true)
            }

            Command::CalendarEditTask => {
                let task_ids = crate::ui::calendar::get_tasks_for_selected_day(app);
                if let Some(task_id) = task_ids.get(app.calendar_state.selected_task_index)
                    && let Some(task) = app.tasks.iter().find(|t| &t.id == task_id)
                {
                    let dialog = AddTaskDialog::from_task(task).with_available_tags(app.tags.clone());
                    app.dialog = Some(Dialog::AddTask(Box::new(dialog)));
                    app.set_status("Tab between fields, Ctrl+Enter to save");
                }
                Ok(true)
            }

            Command::CalendarGoToTask => {
                let task_ids = crate::ui::calendar::get_tasks_for_selected_day(app);
                if let Some(task_id) = task_ids.get(app.calendar_state.selected_task_index)
                    && let Some(task) = app.tasks.iter().find(|t| &t.id == task_id)
                {
                    let task_id = task.id.clone();
                    let project_id = task.project_id.clone();

                    // Switch to main view
                    app.current_view = View::Main;
                    app.focus = FocusPanel::TaskList;

                    // Select the project in the sidebar
                    if let Some(pid) = &project_id {
                        // Find the project index and select it
                        if let Some(idx) = app.projects.iter().position(|p| &p.id == pid) {
                            // Add 1 because index 0 is "All Tasks"
                            app.selected_project_index = idx + 1;
                        }
                    } else {
                        // No project - select "All Tasks"
                        app.selected_project_index = 0;
                    }

                    // Clear filter to ensure task is visible
                    app.filter = Filter::All;

                    // Find and select the task
                    if let Some(idx) = app.visible_tasks().iter().position(|t| t.id == task_id) {
                        app.selected_task_index = Some(idx);
                    }

                    // Reset calendar focus for next time
                    app.calendar_state.focus = crate::ui::calendar::CalendarFocus::DayGrid;
                }
                Ok(true)
            }

            Command::CalendarToggleCompleted => {
                app.calendar_state.toggle_show_completed();
                let status = if app.calendar_state.show_completed {
                    "Showing all tasks"
                } else {
                    "Showing active tasks only"
                };
                app.set_status(status);
                Ok(true)
            }

            Command::ShowSearch => {
                app.current_view = View::Search;
                app.input_mode = InputMode::Search;
                app.input_buffer.clear();
                app.input_cursor = 0;
                app.search_results.clear();
                app.selected_search_index = 0;
                let project_name = app.selected_project_name().to_string();
                if project_name != "All Tasks" {
                    app.set_status(format!("Searching in: {}", project_name));
                }
                Ok(true)
            }

            Command::SearchNavigateUp => {
                if !app.search_results.is_empty() {
                    app.selected_search_index = app.selected_search_index.saturating_sub(1);
                }
                Ok(true)
            }

            Command::SearchNavigateDown => {
                if !app.search_results.is_empty() {
                    app.selected_search_index =
                        (app.selected_search_index + 1).min(app.search_results.len() - 1);
                }
                Ok(true)
            }

            Command::SearchSelectTask => {
                if let Some(result) = app.search_results.get(app.selected_search_index) {
                    // Find the task index in visible_tasks and select it
                    let task_id = result.task.id.clone();
                    app.current_view = View::Main;
                    app.input_mode = InputMode::Normal;
                    app.input_buffer.clear();
                    app.input_cursor = 0;
                    // Focus the task list panel
                    app.focus = FocusPanel::TaskList;

                    // Try to find and select the task
                    if let Some(idx) = app
                        .visible_tasks()
                        .iter()
                        .position(|t| t.id == task_id)
                    {
                        app.selected_task_index = Some(idx);
                    } else {
                        // Task might not be visible due to filters, clear filter
                        app.filter = Filter::All;
                        if let Some(idx) = app
                            .visible_tasks()
                            .iter()
                            .position(|t| t.id == task_id)
                        {
                            app.selected_task_index = Some(idx);
                        }
                    }
                    app.set_status(format!("Selected: {}", result.task.title));
                }
                Ok(true)
            }

            Command::ShowDebugLogs => {
                app.current_view = if app.current_view == View::DebugLogs {
                    View::Main
                } else {
                    View::DebugLogs
                };
                Ok(true)
            }

            Command::ShowTaskDetail => {
                if app.selected_task().is_some() {
                    app.current_view = View::TaskDetail;
                }
                Ok(true)
            }

            // === Filters ===
            Command::SetFilter(filter) => {
                app.filter = filter;
                // Reset selection when filter changes
                let count = app.visible_tasks().len();
                app.selected_task_index = if count > 0 { Some(0) } else { None };
                Ok(true)
            }

            Command::ShowFilterSort => {
                // Use project-scoped tasks for filter counts
                let project_tasks: Vec<Task> = app.project_tasks().into_iter().cloned().collect();
                app.dialog = Some(Dialog::FilterSort(FilterSortDialog::new(
                    &app.filter,
                    &app.sort,
                    &project_tasks,
                )));
                Ok(true)
            }

            Command::ShowSettings => {
                app.dialog = Some(Dialog::Settings(SettingsDialog::new()));
                Ok(true)
            }

            Command::FilterToday => {
                app.filter = Filter::DueToday;
                let count = app.visible_tasks().len();
                app.selected_task_index = if count > 0 { Some(0) } else { None };
                app.set_status("Showing tasks due today");
                Ok(true)
            }

            Command::FilterThisWeek => {
                app.filter = Filter::DueThisWeek;
                let count = app.visible_tasks().len();
                app.selected_task_index = if count > 0 { Some(0) } else { None };
                app.set_status("Showing tasks due this week");
                Ok(true)
            }

            Command::FilterByPriority(priority) => {
                app.filter = Filter::ByPriority(priority);
                let count = app.visible_tasks().len();
                app.selected_task_index = if count > 0 { Some(0) } else { None };
                app.set_status(format!("Showing {:?} priority tasks", priority));
                Ok(true)
            }

            // === Input Mode ===
            Command::EnterEditMode => {
                app.input_mode = InputMode::Editing;
                Ok(true)
            }

            Command::ExitEditMode => {
                app.input_mode = InputMode::Normal;
                app.input_buffer.clear();
                app.input_cursor = 0;
                app.editing_task = None;
                app.clear_status();
                Ok(true)
            }

            Command::SubmitInput => {
                if app.input_mode == InputMode::Editing {
                    let title = app.input_buffer.trim().to_string();
                    if !title.is_empty() {
                        if let Some(mut task) = app.editing_task.take() {
                            // Editing existing task
                            task.title = title;
                            app.db.update_task(&task).await?;
                            app.set_status("Task updated");
                        } else {
                            // Creating new task
                            let task = crate::models::Task::new(&title);
                            app.db.insert_task(&task).await?;
                            app.set_status("Task added");
                        }
                        app.load_data().await?;
                    }
                }
                app.input_mode = InputMode::Normal;
                app.input_buffer.clear();
                app.input_cursor = 0;
                Ok(true)
            }

            Command::CancelInput => {
                // Return to main view if in search
                if app.current_view == View::Search {
                    app.current_view = View::Main;
                    app.search_results.clear();
                    app.selected_search_index = 0;
                }
                app.input_mode = InputMode::Normal;
                app.input_buffer.clear();
                app.input_cursor = 0;
                app.editing_task = None;
                app.clear_status();
                Ok(true)
            }

            // === Text Editing ===
            Command::InsertChar(c) => {
                app.input_buffer.insert(app.input_cursor, c);
                app.input_cursor += 1;
                // Update search results if in search mode (scoped to current project)
                if app.input_mode == InputMode::Search {
                    let project_tasks: Vec<Task> = app.project_tasks().into_iter().cloned().collect();
                    app.search_results = search_tasks(&app.input_buffer, &project_tasks);
                    app.selected_search_index = 0;
                }
                Ok(true)
            }

            Command::DeleteCharBackward => {
                if app.input_cursor > 0 {
                    app.input_cursor -= 1;
                    app.input_buffer.remove(app.input_cursor);
                    // Update search results if in search mode (scoped to current project)
                    if app.input_mode == InputMode::Search {
                        let project_tasks: Vec<Task> = app.project_tasks().into_iter().cloned().collect();
                        app.search_results = search_tasks(&app.input_buffer, &project_tasks);
                        app.selected_search_index = 0;
                    }
                }
                Ok(true)
            }

            Command::DeleteCharForward => {
                if app.input_cursor < app.input_buffer.len() {
                    app.input_buffer.remove(app.input_cursor);
                }
                Ok(true)
            }

            Command::MoveCursorLeft => {
                app.input_cursor = app.input_cursor.saturating_sub(1);
                Ok(true)
            }

            Command::MoveCursorRight => {
                if app.input_cursor < app.input_buffer.len() {
                    app.input_cursor += 1;
                }
                Ok(true)
            }

            Command::MoveCursorStart => {
                app.input_cursor = 0;
                Ok(true)
            }

            Command::MoveCursorEnd => {
                app.input_cursor = app.input_buffer.len();
                Ok(true)
            }

            // === Debug View ===
            Command::LoggerEvent(event) => {
                app.log_state.transition(TuiWidgetEvent::from(event));
                Ok(true)
            }

            // === Application ===
            Command::Quit => {
                app.should_quit = true;
                Ok(false)
            }

            Command::ForceQuit => {
                app.should_quit = true;
                Ok(false)
            }

            Command::Refresh => {
                app.load_data().await?;
                app.set_status("Data refreshed");
                Ok(true)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Task;
    use crate::storage::{run_migrations, Database};

    async fn setup_app() -> App {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();
        App::new(db).await.unwrap()
    }

    async fn setup_app_with_tasks() -> App {
        let mut app = setup_app().await;
        for i in 0..5 {
            let task = Task::new(&format!("Task {}", i));
            app.db.insert_task(&task).await.unwrap();
        }
        app.load_data().await.unwrap();
        app
    }

    #[tokio::test]
    async fn test_navigate_down() {
        let mut app = setup_app_with_tasks().await;
        app.selected_task_index = Some(0);

        Command::NavigateDown.execute(&mut app).await.unwrap();
        assert_eq!(app.selected_task_index, Some(1));
    }

    #[tokio::test]
    async fn test_navigate_up() {
        let mut app = setup_app_with_tasks().await;
        app.selected_task_index = Some(2);

        Command::NavigateUp.execute(&mut app).await.unwrap();
        assert_eq!(app.selected_task_index, Some(1));
    }

    #[tokio::test]
    async fn test_navigate_top() {
        let mut app = setup_app_with_tasks().await;
        app.selected_task_index = Some(3);

        Command::NavigateTop.execute(&mut app).await.unwrap();
        assert_eq!(app.selected_task_index, Some(0));
    }

    #[tokio::test]
    async fn test_navigate_bottom() {
        let mut app = setup_app_with_tasks().await;
        app.selected_task_index = Some(0);

        Command::NavigateBottom.execute(&mut app).await.unwrap();
        assert_eq!(app.selected_task_index, Some(4)); // 5 tasks, index 4
    }

    #[tokio::test]
    async fn test_switch_panel() {
        let mut app = setup_app().await;
        assert_eq!(app.focus, FocusPanel::TaskList);

        Command::SwitchPanel.execute(&mut app).await.unwrap();
        assert_eq!(app.focus, FocusPanel::Sidebar);

        Command::SwitchPanel.execute(&mut app).await.unwrap();
        assert_eq!(app.focus, FocusPanel::TaskList);
    }

    #[tokio::test]
    async fn test_quit_command() {
        let mut app = setup_app().await;
        assert!(!app.should_quit);

        let result = Command::Quit.execute(&mut app).await.unwrap();
        assert!(!result); // Returns false to stop loop
        assert!(app.should_quit);
    }

    #[tokio::test]
    async fn test_show_help() {
        let mut app = setup_app().await;
        assert_eq!(app.current_view, View::Main);

        Command::ShowHelp.execute(&mut app).await.unwrap();
        assert_eq!(app.current_view, View::Help);
    }

    #[tokio::test]
    async fn test_show_main() {
        let mut app = setup_app().await;
        app.current_view = View::Help;

        Command::ShowMain.execute(&mut app).await.unwrap();
        assert_eq!(app.current_view, View::Main);
    }

    #[tokio::test]
    async fn test_toggle_debug_logs() {
        let mut app = setup_app().await;
        assert_eq!(app.current_view, View::Main);

        Command::ShowDebugLogs.execute(&mut app).await.unwrap();
        assert_eq!(app.current_view, View::DebugLogs);

        Command::ShowDebugLogs.execute(&mut app).await.unwrap();
        assert_eq!(app.current_view, View::Main);
    }

    #[tokio::test]
    async fn test_toggle_task_status() {
        use crate::models::{Filter, TaskStatus};

        let mut app = setup_app().await;
        // Use All filter so completed tasks remain visible
        app.filter = Filter::All;

        let task = Task::new("Test task");
        app.db.insert_task(&task).await.unwrap();
        app.load_data().await.unwrap();
        app.selected_task_index = Some(0);

        // Initial status is Pending
        assert_eq!(app.tasks[0].status, TaskStatus::Pending);

        // Toggle to completed
        Command::ToggleTaskStatus.execute(&mut app).await.unwrap();
        assert_eq!(app.tasks[0].status, TaskStatus::Completed);

        // Toggle back to pending
        Command::ToggleTaskStatus.execute(&mut app).await.unwrap();
        assert_eq!(app.tasks[0].status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_add_task_opens_dialog() {
        use crate::ui::dialogs::Dialog;

        let mut app = setup_app().await;
        assert!(app.dialog.is_none());

        Command::AddTask.execute(&mut app).await.unwrap();

        // Dialog should be open
        assert!(app.dialog.is_some());
        assert!(matches!(app.dialog, Some(Dialog::AddTask(_))));
    }

    #[tokio::test]
    async fn test_edit_task_opens_dialog() {
        use crate::ui::dialogs::Dialog;

        let mut app = setup_app().await;
        let task = Task::new("Original title");
        app.db.insert_task(&task).await.unwrap();
        app.load_data().await.unwrap();
        app.selected_task_index = Some(0);

        Command::EditTask.execute(&mut app).await.unwrap();

        // Dialog should be open with task data
        assert!(app.dialog.is_some());
        if let Some(Dialog::AddTask(ref dialog)) = app.dialog {
            assert_eq!(dialog.title.value(), "Original title");
            assert!(dialog.is_editing());
        } else {
            panic!("Expected AddTask dialog");
        }
    }

    #[tokio::test]
    async fn test_delete_task_opens_confirm_dialog() {
        use crate::ui::dialogs::Dialog;

        let mut app = setup_app().await;
        let task = Task::new("Delete me");
        app.db.insert_task(&task).await.unwrap();
        app.load_data().await.unwrap();
        assert_eq!(app.tasks.len(), 1);
        app.selected_task_index = Some(0);

        Command::DeleteTask.execute(&mut app).await.unwrap();

        // Confirm dialog should be open, task not yet deleted
        assert!(app.dialog.is_some());
        assert!(matches!(app.dialog, Some(Dialog::Confirm(_))));
        assert_eq!(app.tasks.len(), 1); // Not deleted until confirmed
    }

    #[tokio::test]
    async fn test_cycle_priority() {
        let mut app = setup_app().await;
        let task = Task::new("Priority test");
        app.db.insert_task(&task).await.unwrap();
        app.load_data().await.unwrap();
        app.selected_task_index = Some(0);

        assert_eq!(app.tasks[0].priority, Priority::Medium); // Default

        Command::CyclePriority.execute(&mut app).await.unwrap();
        assert_eq!(app.tasks[0].priority, Priority::High);

        Command::CyclePriority.execute(&mut app).await.unwrap();
        assert_eq!(app.tasks[0].priority, Priority::Urgent);

        Command::CyclePriority.execute(&mut app).await.unwrap();
        assert_eq!(app.tasks[0].priority, Priority::Low);
    }

    #[tokio::test]
    async fn test_filter_today() {
        let mut app = setup_app().await;

        Command::FilterToday.execute(&mut app).await.unwrap();
        assert_eq!(app.filter, Filter::DueToday);
    }

    #[tokio::test]
    async fn test_show_filter_sort() {
        let mut app = setup_app().await;

        Command::ShowFilterSort.execute(&mut app).await.unwrap();
        assert!(matches!(app.dialog, Some(Dialog::FilterSort(_))));
    }

    #[tokio::test]
    async fn test_text_editing_commands() {
        let mut app = setup_app().await;
        app.input_mode = InputMode::Editing;

        // Insert characters
        Command::InsertChar('H').execute(&mut app).await.unwrap();
        Command::InsertChar('i').execute(&mut app).await.unwrap();
        assert_eq!(app.input_buffer, "Hi");
        assert_eq!(app.input_cursor, 2);

        // Move cursor left
        Command::MoveCursorLeft.execute(&mut app).await.unwrap();
        assert_eq!(app.input_cursor, 1);

        // Insert at cursor
        Command::InsertChar('o').execute(&mut app).await.unwrap();
        assert_eq!(app.input_buffer, "Hoi");
        assert_eq!(app.input_cursor, 2);

        // Backspace
        Command::DeleteCharBackward.execute(&mut app).await.unwrap();
        assert_eq!(app.input_buffer, "Hi");
        assert_eq!(app.input_cursor, 1);
    }

    #[tokio::test]
    async fn test_submit_new_task() {
        let mut app = setup_app().await;
        app.input_mode = InputMode::Editing;
        app.input_buffer = "New task from test".to_string();
        app.input_cursor = app.input_buffer.len();

        Command::SubmitInput.execute(&mut app).await.unwrap();

        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.input_buffer.is_empty());
        assert_eq!(app.tasks.len(), 1);
        assert_eq!(app.tasks[0].title, "New task from test");
    }

    #[tokio::test]
    async fn test_cancel_input() {
        let mut app = setup_app().await;
        app.input_mode = InputMode::Editing;
        app.input_buffer = "Some text".to_string();

        Command::CancelInput.execute(&mut app).await.unwrap();

        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.input_buffer.is_empty());
    }

    #[tokio::test]
    async fn test_refresh() {
        let mut app = setup_app().await;

        // Add a task directly to DB
        let task = Task::new("Direct insert");
        app.db.insert_task(&task).await.unwrap();

        // App doesn't know about it yet
        assert!(app.tasks.is_empty());

        // Refresh should load it
        Command::Refresh.execute(&mut app).await.unwrap();
        assert_eq!(app.tasks.len(), 1);
    }
}
