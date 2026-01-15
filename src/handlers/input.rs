//! Keyboard input to command mapping.
//!
//! This module provides the key mapping system that translates keyboard input
//! into application commands. The mapping is context-aware, taking into account
//! the current view and input mode.
//!
//! ## Keybinding Design
//!
//! The keybindings follow Vim conventions where possible:
//! - `j/k` for up/down navigation
//! - `h/l` for left/right panel switching
//! - `g/G` for top/bottom
//! - `Ctrl+d/u` for page down/up
//!
//! ## Example
//!
//! ```rust,no_run
//! use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
//! use ratado::handlers::input::map_key_to_command;
//! use ratado::handlers::commands::Command;
//! use ratado::app::App;
//!
//! # fn example(app: &App) {
//! let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
//! let cmd = map_key_to_command(key, app);
//! assert!(matches!(cmd, Some(Command::Quit)));
//! # }
//! ```

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, FocusPanel, InputMode, View};
use crate::handlers::commands::{Command, TuiLoggerEvent};
use crate::models::Priority;

/// Maps a keyboard event to a command based on the current application state.
///
/// The mapping considers:
/// - The current view (Main, Help, DebugLogs, etc.)
/// - The current input mode (Normal, Editing, Search)
///
/// # Arguments
///
/// * `key` - The keyboard event to map
/// * `app` - The current application state for context
///
/// # Returns
///
/// The corresponding command, or `None` if the key has no mapping in the
/// current context.
pub fn map_key_to_command(key: KeyEvent, app: &App) -> Option<Command> {
    // Global keybindings that work everywhere
    match key.code {
        // Force quit with Ctrl+C
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Command::ForceQuit);
        }
        // Toggle debug logs with F12 (works everywhere)
        KeyCode::F(12) => {
            return Some(Command::ShowDebugLogs);
        }
        _ => {}
    }

    // Handle view-specific bindings
    match app.current_view {
        View::Help => return map_help_view_key(key),
        View::DebugLogs => return map_debug_view_key(key),
        View::Calendar => return map_calendar_view_key(key),
        View::TaskDetail => return map_task_detail_view_key(key),
        View::Search => {
            // In search view, handle search-specific keys
            if app.input_mode == InputMode::Search {
                return map_search_mode_key(key);
            }
        }
        _ => {}
    }

    // Handle mode-specific bindings
    match app.input_mode {
        InputMode::Normal => map_normal_mode_key(key, app),
        InputMode::Editing => map_editing_mode_key(key),
        InputMode::Search => map_search_mode_key(key),
    }
}

/// Maps keys in the Help view.
///
/// Any key closes the help screen and returns to the main view.
fn map_help_view_key(key: KeyEvent) -> Option<Command> {
    match key.code {
        // Explicit escape goes back to main
        KeyCode::Esc => Some(Command::ShowMain),
        // Any other key also closes help
        _ => Some(Command::ShowMain),
    }
}

/// Maps keys in the Debug Logs view.
///
/// Handles tui-logger widget navigation and log level controls.
fn map_debug_view_key(key: KeyEvent) -> Option<Command> {
    match key.code {
        // Escape returns to main view
        KeyCode::Esc => Some(Command::ShowMain),

        // tui-logger navigation
        KeyCode::Char(' ') => Some(Command::LoggerEvent(TuiLoggerEvent::SpaceBar)),
        KeyCode::Up | KeyCode::Char('k') => Some(Command::LoggerEvent(TuiLoggerEvent::Up)),
        KeyCode::Down | KeyCode::Char('j') => Some(Command::LoggerEvent(TuiLoggerEvent::Down)),
        KeyCode::PageUp => Some(Command::LoggerEvent(TuiLoggerEvent::PrevPage)),
        KeyCode::PageDown => Some(Command::LoggerEvent(TuiLoggerEvent::NextPage)),
        KeyCode::Left | KeyCode::Char('h') => Some(Command::LoggerEvent(TuiLoggerEvent::Left)),
        KeyCode::Right | KeyCode::Char('l') => Some(Command::LoggerEvent(TuiLoggerEvent::Right)),
        KeyCode::Char('+') | KeyCode::Char('=') => {
            Some(Command::LoggerEvent(TuiLoggerEvent::Plus))
        }
        KeyCode::Char('-') => Some(Command::LoggerEvent(TuiLoggerEvent::Minus)),
        KeyCode::Char('H') => Some(Command::LoggerEvent(TuiLoggerEvent::Hide)),

        _ => None,
    }
}

/// Maps keys in the Calendar view.
///
/// Handles navigation between days and weeks.
fn map_calendar_view_key(key: KeyEvent) -> Option<Command> {
    match key.code {
        // Escape returns to main view
        KeyCode::Esc => Some(Command::ShowMain),

        // Day navigation
        KeyCode::Left | KeyCode::Char('h') => Some(Command::CalendarPrevDay),
        KeyCode::Right | KeyCode::Char('l') => Some(Command::CalendarNextDay),

        // Week navigation (using j/k for up/down since they're intuitive)
        KeyCode::Up | KeyCode::Char('k') => Some(Command::CalendarPrevWeek),
        KeyCode::Down | KeyCode::Char('j') => Some(Command::CalendarNextWeek),

        // Jump to today
        KeyCode::Char('t') => Some(Command::CalendarToday),

        // Select current day and return to main
        KeyCode::Enter => Some(Command::CalendarSelectDay),

        // Quit
        KeyCode::Char('q') => Some(Command::Quit),

        _ => None,
    }
}

/// Maps keys in the Task Detail view.
///
/// Handles quick actions on the displayed task.
fn map_task_detail_view_key(key: KeyEvent) -> Option<Command> {
    match key.code {
        // Escape returns to main view
        KeyCode::Esc => Some(Command::ShowMain),

        // Quick task actions
        KeyCode::Char(' ') => Some(Command::ToggleTaskStatus),
        KeyCode::Char('p') => Some(Command::CyclePriority),
        KeyCode::Char('e') | KeyCode::Enter => Some(Command::EditTask),
        KeyCode::Char('d') => Some(Command::DeleteTask),

        // Quit
        KeyCode::Char('q') => Some(Command::Quit),

        _ => None,
    }
}

/// Maps keys in Normal mode (main task list view).
///
/// This is where most keybindings are defined for navigating and
/// managing tasks.
fn map_normal_mode_key(key: KeyEvent, app: &App) -> Option<Command> {
    match key.code {
        // === Quit ===
        KeyCode::Char('q') => Some(Command::Quit),

        // === Navigation ===
        KeyCode::Char('j') | KeyCode::Down => Some(Command::NavigateDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Command::NavigateUp),
        KeyCode::Char('g') | KeyCode::Home => Some(Command::NavigateTop),
        KeyCode::Char('G') | KeyCode::End => Some(Command::NavigateBottom),

        // Page navigation with Ctrl+d/u
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Command::PageDown)
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Command::PageUp)
        }

        // === Panel Switching ===
        KeyCode::Tab => Some(Command::SwitchPanel),
        KeyCode::Char('h') | KeyCode::Left => Some(Command::FocusSidebar),
        KeyCode::Char('l') | KeyCode::Right => Some(Command::FocusTaskList),

        // === Context-sensitive actions (tasks or projects based on focus) ===
        KeyCode::Char('a') => {
            if app.focus == FocusPanel::Sidebar {
                Some(Command::AddProject)
            } else {
                Some(Command::AddTask)
            }
        }
        KeyCode::Char('e') | KeyCode::Enter => {
            if app.focus == FocusPanel::Sidebar {
                Some(Command::EditProject)
            } else {
                Some(Command::EditTask)
            }
        }
        KeyCode::Char('d') => {
            if app.focus == FocusPanel::Sidebar {
                Some(Command::DeleteProject)
            } else {
                Some(Command::DeleteTask)
            }
        }

        // === Task-specific actions ===
        KeyCode::Char(' ') => Some(Command::ToggleTaskStatus),
        KeyCode::Char('p') => Some(Command::CyclePriority),
        KeyCode::Char('t') => Some(Command::EditTags),
        KeyCode::Char('m') => Some(Command::MoveToProject),

        // === Views ===
        KeyCode::Char('?') => Some(Command::ShowHelp),
        KeyCode::Char('/') => Some(Command::ShowSearch),
        KeyCode::Char('c') => Some(Command::ShowCalendar),

        // === Quick Filters ===
        KeyCode::Char('T') => Some(Command::FilterToday),
        KeyCode::Char('W') => Some(Command::FilterThisWeek),

        // Priority filters (1-4 keys)
        KeyCode::Char('1') => Some(Command::FilterByPriority(Priority::Low)),
        KeyCode::Char('2') => Some(Command::FilterByPriority(Priority::Medium)),
        KeyCode::Char('3') => Some(Command::FilterByPriority(Priority::High)),
        KeyCode::Char('4') => Some(Command::FilterByPriority(Priority::Urgent)),

        // === Filter/Sort ===
        KeyCode::Char('f') => Some(Command::ShowFilterSort),

        // === Settings ===
        KeyCode::Char('S') => Some(Command::ShowSettings),

        // === Other ===
        KeyCode::Char('r') => Some(Command::Refresh),

        _ => None,
    }
}

/// Maps keys in Editing mode (text input for task title, etc.).
///
/// Standard text editing keys plus escape to cancel and enter to submit.
fn map_editing_mode_key(key: KeyEvent) -> Option<Command> {
    // Check for Ctrl modifiers first (Emacs-style shortcuts)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('a') => Some(Command::MoveCursorStart),
            KeyCode::Char('e') => Some(Command::MoveCursorEnd),
            _ => None,
        };
    }

    match key.code {
        // Exit editing
        KeyCode::Esc => Some(Command::CancelInput),
        KeyCode::Enter => Some(Command::SubmitInput),

        // Text editing
        KeyCode::Char(c) => Some(Command::InsertChar(c)),
        KeyCode::Backspace => Some(Command::DeleteCharBackward),
        KeyCode::Delete => Some(Command::DeleteCharForward),

        // Cursor movement
        KeyCode::Left => Some(Command::MoveCursorLeft),
        KeyCode::Right => Some(Command::MoveCursorRight),
        KeyCode::Home => Some(Command::MoveCursorStart),
        KeyCode::End => Some(Command::MoveCursorEnd),

        _ => None,
    }
}

/// Maps keys in Search mode.
///
/// Similar to editing mode but with search-specific behavior.
fn map_search_mode_key(key: KeyEvent) -> Option<Command> {
    // Check for Ctrl modifiers first (Emacs-style shortcuts)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('a') => Some(Command::MoveCursorStart),
            KeyCode::Char('e') => Some(Command::MoveCursorEnd),
            KeyCode::Char('n') => Some(Command::SearchNavigateDown),
            KeyCode::Char('p') => Some(Command::SearchNavigateUp),
            _ => None,
        };
    }

    match key.code {
        // Exit search
        KeyCode::Esc => Some(Command::CancelInput),
        // Select current search result
        KeyCode::Enter => Some(Command::SearchSelectTask),

        // Navigate search results
        KeyCode::Down => Some(Command::SearchNavigateDown),
        KeyCode::Up => Some(Command::SearchNavigateUp),

        // Text editing
        KeyCode::Char(c) => Some(Command::InsertChar(c)),
        KeyCode::Backspace => Some(Command::DeleteCharBackward),
        KeyCode::Delete => Some(Command::DeleteCharForward),

        // Cursor movement within search text
        KeyCode::Left => Some(Command::MoveCursorLeft),
        KeyCode::Right => Some(Command::MoveCursorRight),
        KeyCode::Home => Some(Command::MoveCursorStart),
        KeyCode::End => Some(Command::MoveCursorEnd),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{run_migrations, Database};

    async fn setup_app() -> App {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();
        App::new(db).await.unwrap()
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn key_with_mod(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    // === Normal Mode Tests ===

    #[tokio::test]
    async fn test_quit_keybinding() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('q')), &app);
        assert!(matches!(cmd, Some(Command::Quit)));
    }

    #[tokio::test]
    async fn test_force_quit_ctrl_c() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key_with_mod(KeyCode::Char('c'), KeyModifiers::CONTROL), &app);
        assert!(matches!(cmd, Some(Command::ForceQuit)));
    }

    #[tokio::test]
    async fn test_vim_navigation_j() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('j')), &app);
        assert!(matches!(cmd, Some(Command::NavigateDown)));
    }

    #[tokio::test]
    async fn test_vim_navigation_k() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('k')), &app);
        assert!(matches!(cmd, Some(Command::NavigateUp)));
    }

    #[tokio::test]
    async fn test_arrow_navigation_down() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Down), &app);
        assert!(matches!(cmd, Some(Command::NavigateDown)));
    }

    #[tokio::test]
    async fn test_arrow_navigation_up() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Up), &app);
        assert!(matches!(cmd, Some(Command::NavigateUp)));
    }

    #[tokio::test]
    async fn test_navigate_top_g() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('g')), &app);
        assert!(matches!(cmd, Some(Command::NavigateTop)));
    }

    #[tokio::test]
    async fn test_navigate_bottom_shift_g() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('G')), &app);
        assert!(matches!(cmd, Some(Command::NavigateBottom)));
    }

    #[tokio::test]
    async fn test_page_down_ctrl_d() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key_with_mod(KeyCode::Char('d'), KeyModifiers::CONTROL), &app);
        assert!(matches!(cmd, Some(Command::PageDown)));
    }

    #[tokio::test]
    async fn test_page_up_ctrl_u() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key_with_mod(KeyCode::Char('u'), KeyModifiers::CONTROL), &app);
        assert!(matches!(cmd, Some(Command::PageUp)));
    }

    #[tokio::test]
    async fn test_tab_switches_panel() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Tab), &app);
        assert!(matches!(cmd, Some(Command::SwitchPanel)));
    }

    #[tokio::test]
    async fn test_h_focuses_sidebar() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('h')), &app);
        assert!(matches!(cmd, Some(Command::FocusSidebar)));
    }

    #[tokio::test]
    async fn test_l_focuses_tasklist() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('l')), &app);
        assert!(matches!(cmd, Some(Command::FocusTaskList)));
    }

    #[tokio::test]
    async fn test_add_task_a() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('a')), &app);
        assert!(matches!(cmd, Some(Command::AddTask)));
    }

    #[tokio::test]
    async fn test_edit_task_e() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('e')), &app);
        assert!(matches!(cmd, Some(Command::EditTask)));
    }

    #[tokio::test]
    async fn test_edit_task_enter() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Enter), &app);
        assert!(matches!(cmd, Some(Command::EditTask)));
    }

    #[tokio::test]
    async fn test_delete_task_d() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('d')), &app);
        assert!(matches!(cmd, Some(Command::DeleteTask)));
    }

    #[tokio::test]
    async fn test_toggle_status_space() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char(' ')), &app);
        assert!(matches!(cmd, Some(Command::ToggleTaskStatus)));
    }

    #[tokio::test]
    async fn test_cycle_priority_p() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('p')), &app);
        assert!(matches!(cmd, Some(Command::CyclePriority)));
    }

    #[tokio::test]
    async fn test_show_help_question_mark() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('?')), &app);
        assert!(matches!(cmd, Some(Command::ShowHelp)));
    }

    #[tokio::test]
    async fn test_show_search_slash() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('/')), &app);
        assert!(matches!(cmd, Some(Command::ShowSearch)));
    }

    #[tokio::test]
    async fn test_show_calendar_c() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('c')), &app);
        assert!(matches!(cmd, Some(Command::ShowCalendar)));
    }

    #[tokio::test]
    async fn test_filter_today_shift_t() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('T')), &app);
        assert!(matches!(cmd, Some(Command::FilterToday)));
    }

    #[tokio::test]
    async fn test_filter_week_shift_w() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('W')), &app);
        assert!(matches!(cmd, Some(Command::FilterThisWeek)));
    }

    #[tokio::test]
    async fn test_priority_filters() {
        let app = setup_app().await;

        let cmd = map_key_to_command(key(KeyCode::Char('1')), &app);
        assert!(matches!(cmd, Some(Command::FilterByPriority(Priority::Low))));

        let cmd = map_key_to_command(key(KeyCode::Char('2')), &app);
        assert!(matches!(
            cmd,
            Some(Command::FilterByPriority(Priority::Medium))
        ));

        let cmd = map_key_to_command(key(KeyCode::Char('3')), &app);
        assert!(matches!(
            cmd,
            Some(Command::FilterByPriority(Priority::High))
        ));

        let cmd = map_key_to_command(key(KeyCode::Char('4')), &app);
        assert!(matches!(
            cmd,
            Some(Command::FilterByPriority(Priority::Urgent))
        ));
    }

    #[tokio::test]
    async fn test_refresh_r() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('r')), &app);
        assert!(matches!(cmd, Some(Command::Refresh)));
    }

    #[tokio::test]
    async fn test_show_filter_sort_f() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Char('f')), &app);
        assert!(matches!(cmd, Some(Command::ShowFilterSort)));
    }

    #[tokio::test]
    async fn test_esc_does_nothing_in_normal_mode() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::Esc), &app);
        assert!(cmd.is_none());
    }

    #[tokio::test]
    async fn test_f12_toggles_debug() {
        let app = setup_app().await;
        let cmd = map_key_to_command(key(KeyCode::F(12)), &app);
        assert!(matches!(cmd, Some(Command::ShowDebugLogs)));
    }

    // === Help View Tests ===

    #[tokio::test]
    async fn test_help_view_any_key_closes() {
        let mut app = setup_app().await;
        app.current_view = View::Help;

        let cmd = map_key_to_command(key(KeyCode::Char('x')), &app);
        assert!(matches!(cmd, Some(Command::ShowMain)));
    }

    #[tokio::test]
    async fn test_help_view_esc_closes() {
        let mut app = setup_app().await;
        app.current_view = View::Help;

        let cmd = map_key_to_command(key(KeyCode::Esc), &app);
        assert!(matches!(cmd, Some(Command::ShowMain)));
    }

    // === Debug View Tests ===

    #[tokio::test]
    async fn test_debug_view_esc_closes() {
        let mut app = setup_app().await;
        app.current_view = View::DebugLogs;

        let cmd = map_key_to_command(key(KeyCode::Esc), &app);
        assert!(matches!(cmd, Some(Command::ShowMain)));
    }

    #[tokio::test]
    async fn test_debug_view_navigation() {
        let mut app = setup_app().await;
        app.current_view = View::DebugLogs;

        // Space toggles focus
        let cmd = map_key_to_command(key(KeyCode::Char(' ')), &app);
        assert!(matches!(
            cmd,
            Some(Command::LoggerEvent(TuiLoggerEvent::SpaceBar))
        ));

        // j/k for navigation
        let cmd = map_key_to_command(key(KeyCode::Char('j')), &app);
        assert!(matches!(
            cmd,
            Some(Command::LoggerEvent(TuiLoggerEvent::Down))
        ));

        let cmd = map_key_to_command(key(KeyCode::Char('k')), &app);
        assert!(matches!(
            cmd,
            Some(Command::LoggerEvent(TuiLoggerEvent::Up))
        ));
    }

    // === Editing Mode Tests ===

    #[tokio::test]
    async fn test_editing_mode_chars() {
        let mut app = setup_app().await;
        app.input_mode = InputMode::Editing;

        let cmd = map_key_to_command(key(KeyCode::Char('x')), &app);
        assert!(matches!(cmd, Some(Command::InsertChar('x'))));
    }

    #[tokio::test]
    async fn test_editing_mode_backspace() {
        let mut app = setup_app().await;
        app.input_mode = InputMode::Editing;

        let cmd = map_key_to_command(key(KeyCode::Backspace), &app);
        assert!(matches!(cmd, Some(Command::DeleteCharBackward)));
    }

    #[tokio::test]
    async fn test_editing_mode_enter_submits() {
        let mut app = setup_app().await;
        app.input_mode = InputMode::Editing;

        let cmd = map_key_to_command(key(KeyCode::Enter), &app);
        assert!(matches!(cmd, Some(Command::SubmitInput)));
    }

    #[tokio::test]
    async fn test_editing_mode_esc_cancels() {
        let mut app = setup_app().await;
        app.input_mode = InputMode::Editing;

        let cmd = map_key_to_command(key(KeyCode::Esc), &app);
        assert!(matches!(cmd, Some(Command::CancelInput)));
    }

    #[tokio::test]
    async fn test_editing_mode_cursor_movement() {
        let mut app = setup_app().await;
        app.input_mode = InputMode::Editing;

        let cmd = map_key_to_command(key(KeyCode::Left), &app);
        assert!(matches!(cmd, Some(Command::MoveCursorLeft)));

        let cmd = map_key_to_command(key(KeyCode::Right), &app);
        assert!(matches!(cmd, Some(Command::MoveCursorRight)));

        let cmd = map_key_to_command(key(KeyCode::Home), &app);
        assert!(matches!(cmd, Some(Command::MoveCursorStart)));

        let cmd = map_key_to_command(key(KeyCode::End), &app);
        assert!(matches!(cmd, Some(Command::MoveCursorEnd)));
    }

    #[tokio::test]
    async fn test_editing_mode_emacs_keys() {
        let mut app = setup_app().await;
        app.input_mode = InputMode::Editing;

        let cmd = map_key_to_command(key_with_mod(KeyCode::Char('a'), KeyModifiers::CONTROL), &app);
        assert!(matches!(cmd, Some(Command::MoveCursorStart)));

        let cmd = map_key_to_command(key_with_mod(KeyCode::Char('e'), KeyModifiers::CONTROL), &app);
        assert!(matches!(cmd, Some(Command::MoveCursorEnd)));
    }

    // === Search Mode Tests ===

    #[tokio::test]
    async fn test_search_mode_chars() {
        let mut app = setup_app().await;
        app.input_mode = InputMode::Search;

        let cmd = map_key_to_command(key(KeyCode::Char('a')), &app);
        assert!(matches!(cmd, Some(Command::InsertChar('a'))));
    }

    #[tokio::test]
    async fn test_search_mode_esc_cancels() {
        let mut app = setup_app().await;
        app.input_mode = InputMode::Search;

        let cmd = map_key_to_command(key(KeyCode::Esc), &app);
        assert!(matches!(cmd, Some(Command::CancelInput)));
    }

    // === Calendar View Tests ===

    #[tokio::test]
    async fn test_calendar_view_esc_returns() {
        let mut app = setup_app().await;
        app.current_view = View::Calendar;

        let cmd = map_key_to_command(key(KeyCode::Esc), &app);
        assert!(matches!(cmd, Some(Command::ShowMain)));
    }

    #[tokio::test]
    async fn test_calendar_view_left_prev_day() {
        let mut app = setup_app().await;
        app.current_view = View::Calendar;

        let cmd = map_key_to_command(key(KeyCode::Left), &app);
        assert!(matches!(cmd, Some(Command::CalendarPrevDay)));
    }

    #[tokio::test]
    async fn test_calendar_view_right_next_day() {
        let mut app = setup_app().await;
        app.current_view = View::Calendar;

        let cmd = map_key_to_command(key(KeyCode::Right), &app);
        assert!(matches!(cmd, Some(Command::CalendarNextDay)));
    }

    #[tokio::test]
    async fn test_calendar_view_up_prev_week() {
        let mut app = setup_app().await;
        app.current_view = View::Calendar;

        let cmd = map_key_to_command(key(KeyCode::Up), &app);
        assert!(matches!(cmd, Some(Command::CalendarPrevWeek)));
    }

    #[tokio::test]
    async fn test_calendar_view_down_next_week() {
        let mut app = setup_app().await;
        app.current_view = View::Calendar;

        let cmd = map_key_to_command(key(KeyCode::Down), &app);
        assert!(matches!(cmd, Some(Command::CalendarNextWeek)));
    }

    #[tokio::test]
    async fn test_calendar_view_t_today() {
        let mut app = setup_app().await;
        app.current_view = View::Calendar;

        let cmd = map_key_to_command(key(KeyCode::Char('t')), &app);
        assert!(matches!(cmd, Some(Command::CalendarToday)));
    }

    #[tokio::test]
    async fn test_calendar_view_vim_navigation() {
        let mut app = setup_app().await;
        app.current_view = View::Calendar;

        let cmd = map_key_to_command(key(KeyCode::Char('h')), &app);
        assert!(matches!(cmd, Some(Command::CalendarPrevDay)));

        let cmd = map_key_to_command(key(KeyCode::Char('l')), &app);
        assert!(matches!(cmd, Some(Command::CalendarNextDay)));

        let cmd = map_key_to_command(key(KeyCode::Char('j')), &app);
        assert!(matches!(cmd, Some(Command::CalendarNextWeek)));

        let cmd = map_key_to_command(key(KeyCode::Char('k')), &app);
        assert!(matches!(cmd, Some(Command::CalendarPrevWeek)));
    }

    // === Task Detail View Tests ===

    #[tokio::test]
    async fn test_task_detail_view_esc_returns() {
        let mut app = setup_app().await;
        app.current_view = View::TaskDetail;

        let cmd = map_key_to_command(key(KeyCode::Esc), &app);
        assert!(matches!(cmd, Some(Command::ShowMain)));
    }

    #[tokio::test]
    async fn test_task_detail_view_space_toggles() {
        let mut app = setup_app().await;
        app.current_view = View::TaskDetail;

        let cmd = map_key_to_command(key(KeyCode::Char(' ')), &app);
        assert!(matches!(cmd, Some(Command::ToggleTaskStatus)));
    }

    #[tokio::test]
    async fn test_task_detail_view_p_cycles_priority() {
        let mut app = setup_app().await;
        app.current_view = View::TaskDetail;

        let cmd = map_key_to_command(key(KeyCode::Char('p')), &app);
        assert!(matches!(cmd, Some(Command::CyclePriority)));
    }

    #[tokio::test]
    async fn test_task_detail_view_e_edits() {
        let mut app = setup_app().await;
        app.current_view = View::TaskDetail;

        let cmd = map_key_to_command(key(KeyCode::Char('e')), &app);
        assert!(matches!(cmd, Some(Command::EditTask)));
    }

    #[tokio::test]
    async fn test_task_detail_view_d_deletes() {
        let mut app = setup_app().await;
        app.current_view = View::TaskDetail;

        let cmd = map_key_to_command(key(KeyCode::Char('d')), &app);
        assert!(matches!(cmd, Some(Command::DeleteTask)));
    }
}
