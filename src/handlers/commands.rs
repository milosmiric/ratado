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
use crate::models::{Filter, Priority};

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

    // === Views ===
    /// Show the main task list view
    ShowMain,
    /// Show the help screen
    ShowHelp,
    /// Show the calendar view
    ShowCalendar,
    /// Enter search mode
    ShowSearch,
    /// Toggle the debug logs view
    ShowDebugLogs,
    /// Show detailed view of selected task
    ShowTaskDetail,

    // === Filters ===
    /// Set a specific filter
    SetFilter(Filter),
    /// Clear the current filter (show all)
    ClearFilter,
    /// Filter to show only tasks due today
    FilterToday,
    /// Filter to show only tasks due this week
    FilterThisWeek,
    /// Filter by priority level
    FilterByPriority(Priority),
    /// Cycle through filter options
    CycleFilter,
    /// Cycle through sort options
    CycleSort,

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
                app.input_mode = InputMode::Editing;
                app.input_buffer.clear();
                app.input_cursor = 0;
                app.editing_task = None;
                app.set_status("Enter task title...");
                Ok(true)
            }

            Command::EditTask => {
                if let Some(task) = app.selected_task().cloned() {
                    app.input_mode = InputMode::Editing;
                    app.input_buffer = task.title.clone();
                    app.input_cursor = task.title.len();
                    app.editing_task = Some(task);
                    app.set_status("Editing task...");
                }
                Ok(true)
            }

            Command::DeleteTask => {
                if let Some(task) = app.selected_task() {
                    let task_id = task.id.clone();
                    app.db.delete_task(&task_id).await?;
                    app.load_data().await?;
                    app.set_status("Task deleted");
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
                // TODO: Implement project picker modal
                app.set_status("Move to project: Not yet implemented");
                Ok(true)
            }

            Command::EditTags => {
                // TODO: Implement tag editor modal
                app.set_status("Edit tags: Not yet implemented");
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
                Ok(true)
            }

            Command::ShowSearch => {
                app.current_view = View::Search;
                app.input_mode = InputMode::Search;
                app.input_buffer.clear();
                app.input_cursor = 0;
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

            Command::ClearFilter => {
                app.filter = Filter::All;
                let count = app.visible_tasks().len();
                app.selected_task_index = if count > 0 { Some(0) } else { None };
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

            Command::CycleFilter => {
                app.cycle_filter();
                Ok(true)
            }

            Command::CycleSort => {
                app.cycle_sort();
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
                Ok(true)
            }

            Command::DeleteCharBackward => {
                if app.input_cursor > 0 {
                    app.input_cursor -= 1;
                    app.input_buffer.remove(app.input_cursor);
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
    async fn test_add_task_enters_edit_mode() {
        let mut app = setup_app().await;
        assert_eq!(app.input_mode, InputMode::Normal);

        Command::AddTask.execute(&mut app).await.unwrap();
        assert_eq!(app.input_mode, InputMode::Editing);
        assert!(app.input_buffer.is_empty());
        assert!(app.editing_task.is_none());
    }

    #[tokio::test]
    async fn test_edit_task() {
        let mut app = setup_app().await;
        let task = Task::new("Original title");
        app.db.insert_task(&task).await.unwrap();
        app.load_data().await.unwrap();
        app.selected_task_index = Some(0);

        Command::EditTask.execute(&mut app).await.unwrap();
        assert_eq!(app.input_mode, InputMode::Editing);
        assert_eq!(app.input_buffer, "Original title");
        assert!(app.editing_task.is_some());
    }

    #[tokio::test]
    async fn test_delete_task() {
        let mut app = setup_app().await;
        let task = Task::new("Delete me");
        app.db.insert_task(&task).await.unwrap();
        app.load_data().await.unwrap();
        assert_eq!(app.tasks.len(), 1);
        app.selected_task_index = Some(0);

        Command::DeleteTask.execute(&mut app).await.unwrap();
        assert_eq!(app.tasks.len(), 0);
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
    async fn test_clear_filter() {
        let mut app = setup_app().await;
        app.filter = Filter::DueToday;

        Command::ClearFilter.execute(&mut app).await.unwrap();
        assert_eq!(app.filter, Filter::All);
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
