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

use log::debug;

use crate::app::{App, AppError};

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
