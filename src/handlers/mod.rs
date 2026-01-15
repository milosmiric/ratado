//! Input handling module.
//!
//! This module handles keyboard input, commands, and keybindings.
//! It processes user input events and updates the application state.
//!
//! ## Architecture
//!
//! The handler system follows the command pattern:
//!
//! 1. [`EventHandler`] generates [`AppEvent`]s from terminal input
//! 2. [`handle_event`] routes events to appropriate handlers
//! 3. [`map_key_to_command`] maps keys to [`Command`]s based on context
//! 4. [`Command::execute`] modifies the application state
//!
//! ## Submodules
//!
//! - [`events`] - Event polling and distribution
//! - [`commands`] - Command definitions and execution
//! - [`input`] - Keyboard-to-command mapping
//!
//! ## Example
//!
//! ```rust,no_run
//! use std::time::Duration;
//! use ratado::handlers::{EventHandler, AppEvent, handle_event};
//! use ratado::app::App;
//!
//! # async fn example(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
//! let mut events = EventHandler::new(Duration::from_millis(250));
//!
//! while let Some(event) = events.next().await {
//!     if !handle_event(app, event).await? {
//!         break; // Quit command issued
//!     }
//! }
//! # Ok(())
//! # }
//! ```

pub mod commands;
pub mod events;
pub mod input;

// Re-export commonly used types
pub use commands::Command;
pub use events::{AppEvent, EventHandler};
pub use input::map_key_to_command;

use crossterm::event::KeyEvent;
use log::debug;

use crate::app::{App, AppError};
use crate::ui::dialogs::{DeleteProjectChoice, Dialog, DialogAction, SettingsOption};

/// Handles an application event and updates state accordingly.
///
/// This is the main entry point for event processing. It routes
/// different event types to their appropriate handlers.
///
/// # Arguments
///
/// * `app` - The application state to update
/// * `event` - The event to process
///
/// # Returns
///
/// Returns `Ok(true)` if the application should continue running,
/// or `Ok(false)` if the application should quit.
///
/// # Errors
///
/// Returns an error if a command execution fails (e.g., database error).
pub async fn handle_event(app: &mut App, event: AppEvent) -> Result<bool, AppError> {
    match event {
        AppEvent::Key(key) => {
            debug!("Key event: {:?}", key);

            // If a dialog is active, route events to it first
            if app.dialog.is_some() {
                return handle_dialog_key(app, key).await;
            }

            // Map the key to a command based on current context
            if let Some(cmd) = map_key_to_command(key, app) {
                return cmd.execute(app).await;
            }

            // Key has no mapping in current context, ignore
            Ok(true)
        }

        AppEvent::Tick => {
            // Timer tick for time-based updates
            app.on_tick();
            Ok(true)
        }

        AppEvent::Resize(_width, _height) => {
            // Ratatui handles resize automatically when terminal.draw() is called
            // We could add custom resize handling here if needed
            Ok(true)
        }

        AppEvent::Mouse(_mouse) => {
            // Optional: handle mouse events
            // For now, we ignore mouse input
            Ok(true)
        }
    }
}

/// Handles key events when a dialog is active.
async fn handle_dialog_key(app: &mut App, key: KeyEvent) -> Result<bool, AppError> {
    // Take the dialog out to work with it
    let dialog = app.dialog.take();

    match dialog {
        Some(Dialog::AddTask(mut add_dialog)) => {
            let action = add_dialog.handle_key(key);
            match action {
                DialogAction::Submit => {
                    // Create or update the task
                    if let Some(task) = add_dialog.to_task() {
                        if add_dialog.is_editing() {
                            app.db.update_task(&task).await?;
                            app.set_status("Task updated");
                        } else {
                            app.db.insert_task(&task).await?;
                            app.set_status("Task created");
                        }
                        app.load_data().await?;
                    }
                    // Dialog closed, don't put it back
                }
                DialogAction::Cancel => {
                    app.clear_status();
                    // Dialog closed, don't put it back
                }
                DialogAction::None => {
                    // Keep the dialog open
                    app.dialog = Some(Dialog::AddTask(add_dialog));
                }
            }
        }
        Some(Dialog::Confirm(mut confirm_dialog)) => {
            let action = confirm_dialog.handle_key(key);
            match action {
                DialogAction::Submit => {
                    // Confirmation accepted - execute the pending delete
                    // The task ID was stored when the dialog was created
                    // For now we handle this by checking if we have a selected task
                    if let Some(task) = app.selected_task().cloned() {
                        app.db.delete_task(&task.id).await?;
                        app.load_data().await?;
                        app.set_status("Task deleted");
                    }
                    // Dialog closed
                }
                DialogAction::Cancel => {
                    app.clear_status();
                    // Dialog closed
                }
                DialogAction::None => {
                    // Keep the dialog open
                    app.dialog = Some(Dialog::Confirm(confirm_dialog));
                }
            }
        }
        Some(Dialog::FilterSort(mut filter_dialog)) => {
            let action = filter_dialog.handle_key(key);
            match action {
                DialogAction::Submit => {
                    // Apply the selected filter and sort
                    app.filter = filter_dialog.selected_filter();
                    app.sort = filter_dialog.selected_sort();
                    // Reset selection for new filter
                    let count = app.visible_tasks().len();
                    app.selected_task_index = if count > 0 { Some(0) } else { None };
                    app.set_status(format!("Filter: {:?}, Sort: {:?}", app.filter, app.sort));
                    // Dialog closed
                }
                DialogAction::Cancel => {
                    // Dialog closed without applying changes
                }
                DialogAction::None => {
                    // Keep the dialog open
                    app.dialog = Some(Dialog::FilterSort(filter_dialog));
                }
            }
        }
        Some(Dialog::DeleteProject(mut delete_dialog)) => {
            let action = delete_dialog.handle_key(key);
            match action {
                DialogAction::Submit => {
                    let project_id = delete_dialog.project_id.clone();
                    match delete_dialog.choice() {
                        DeleteProjectChoice::MoveToInbox => {
                            // Move all tasks to inbox, then delete project
                            app.db.move_tasks_to_inbox(&project_id).await?;
                            app.db.delete_project(&project_id).await?;
                            app.set_status("Project deleted, tasks moved to Inbox");
                        }
                        DeleteProjectChoice::DeleteTasks => {
                            // Delete all tasks in project, then delete project
                            app.db.delete_tasks_by_project(&project_id).await?;
                            app.db.delete_project(&project_id).await?;
                            app.set_status("Project and tasks deleted");
                        }
                        DeleteProjectChoice::Cancel => {
                            // Shouldn't reach here, but handle anyway
                            app.clear_status();
                        }
                    }
                    app.load_data().await?;
                    // Reset project selection to "All Tasks"
                    app.selected_project_index = 0;
                    // Dialog closed
                }
                DialogAction::Cancel => {
                    app.clear_status();
                    // Dialog closed
                }
                DialogAction::None => {
                    // Keep the dialog open
                    app.dialog = Some(Dialog::DeleteProject(delete_dialog));
                }
            }
        }
        Some(Dialog::Project(mut project_dialog)) => {
            let action = project_dialog.handle_key(key);
            match action {
                DialogAction::Submit => {
                    // Create or update the project
                    if let Some(project) = project_dialog.to_project() {
                        if project_dialog.is_editing() {
                            app.db.update_project(&project).await?;
                            app.set_status("Project updated");
                        } else {
                            app.db.insert_project(&project).await?;
                            app.set_status("Project created");
                        }
                        app.load_data().await?;
                    }
                    // Dialog closed
                }
                DialogAction::Cancel => {
                    app.clear_status();
                    // Dialog closed
                }
                DialogAction::None => {
                    // Keep the dialog open
                    app.dialog = Some(Dialog::Project(project_dialog));
                }
            }
        }
        Some(Dialog::MoveToProject(mut move_dialog)) => {
            let action = move_dialog.handle_key(key);
            match action {
                DialogAction::Submit => {
                    // Move the task to the selected project
                    if let Some(project_id) = move_dialog.selected_project_id() {
                        let task_id = move_dialog.task_id.clone();
                        if let Some(task) = app.tasks.iter_mut().find(|t| t.id == task_id) {
                            task.project_id = Some(project_id.clone());
                            task.updated_at = chrono::Utc::now();
                            app.db.update_task(task).await?;
                            let project_name = move_dialog
                                .selected_project()
                                .map(|p| p.name.as_str())
                                .unwrap_or("Unknown");
                            app.set_status(format!("Task moved to {}", project_name));
                            app.load_data().await?;
                        }
                    }
                    // Dialog closed
                }
                DialogAction::Cancel => {
                    app.clear_status();
                    // Dialog closed
                }
                DialogAction::None => {
                    // Keep the dialog open
                    app.dialog = Some(Dialog::MoveToProject(move_dialog));
                }
            }
        }
        Some(Dialog::Settings(mut settings_dialog)) => {
            let action = settings_dialog.handle_key(key);
            match action {
                DialogAction::Submit => {
                    // Execute the confirmed action
                    if let Some(option) = settings_dialog.confirmed_option() {
                        match option {
                            SettingsOption::DeleteCompletedTasks => {
                                let count = app.db.delete_completed_tasks().await?;
                                app.load_data().await?;
                                app.set_status(format!("Deleted {} completed task(s)", count));
                            }
                            SettingsOption::ResetDatabase => {
                                // Delete all tasks first
                                let task_count = app.db.delete_all_tasks().await?;
                                // Delete all projects except Inbox
                                let project_count = app.db.delete_all_projects_except_inbox().await?;
                                app.load_data().await?;
                                // Reset selection
                                app.selected_task_index = None;
                                app.selected_project_index = 0;
                                app.set_status(format!(
                                    "Database reset: deleted {} task(s) and {} project(s)",
                                    task_count, project_count
                                ));
                            }
                        }
                    }
                    // Dialog closed
                }
                DialogAction::Cancel => {
                    app.clear_status();
                    // Dialog closed
                }
                DialogAction::None => {
                    // Keep the dialog open
                    app.dialog = Some(Dialog::Settings(settings_dialog));
                }
            }
        }
        None => {
            // No dialog was active (shouldn't happen)
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::View;
    use crate::storage::{run_migrations, Database};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    async fn setup_app() -> App {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();
        App::new(db).await.unwrap()
    }

    #[tokio::test]
    async fn test_handle_key_event_quit() {
        let mut app = setup_app().await;
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

        let result = handle_event(&mut app, AppEvent::Key(key)).await.unwrap();
        assert!(!result); // Should return false to quit
        assert!(app.should_quit);
    }

    #[tokio::test]
    async fn test_handle_key_event_navigation() {
        let mut app = setup_app().await;

        // Add some tasks first
        let task = crate::models::Task::new("Test task");
        app.db.insert_task(&task).await.unwrap();
        app.load_data().await.unwrap();
        app.selected_task_index = Some(0);

        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let result = handle_event(&mut app, AppEvent::Key(key)).await.unwrap();

        assert!(result); // Should continue
    }

    #[tokio::test]
    async fn test_handle_tick_event() {
        let mut app = setup_app().await;

        let result = handle_event(&mut app, AppEvent::Tick).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_handle_resize_event() {
        let mut app = setup_app().await;

        let result = handle_event(&mut app, AppEvent::Resize(100, 50))
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_handle_help_view_close() {
        let mut app = setup_app().await;
        app.current_view = View::Help;

        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let result = handle_event(&mut app, AppEvent::Key(key)).await.unwrap();

        assert!(result);
        assert_eq!(app.current_view, View::Main);
    }

    #[tokio::test]
    async fn test_handle_unmapped_key() {
        let mut app = setup_app().await;

        // F5 is not mapped to any command
        let key = KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE);
        let result = handle_event(&mut app, AppEvent::Key(key)).await.unwrap();

        assert!(result); // Should continue, just ignore the key
    }

    #[tokio::test]
    async fn test_handle_force_quit() {
        let mut app = setup_app().await;
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

        let result = handle_event(&mut app, AppEvent::Key(key)).await.unwrap();
        assert!(!result);
        assert!(app.should_quit);
    }
}
