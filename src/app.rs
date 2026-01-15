//! Application state management.
//!
//! This module contains the central [`App`] struct that manages all application
//! state. Following the Elm architecture pattern, the App struct holds the
//! complete state of the application and is updated in response to events.
//!
//! ## Architecture
//!
//! The application follows a unidirectional data flow:
//! 1. User input generates events
//! 2. Events are processed by handlers that update the App state
//! 3. The UI renders based on the current App state
//!
//! ## Example
//!
//! ```rust,no_run
//! use ratado::app::App;
//! use ratado::storage::{Database, run_migrations};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let db = Database::open_in_memory().await?;
//! run_migrations(&db).await?;
//! let app = App::new(db).await?;
//! # Ok(())
//! # }
//! ```

use std::time::Instant;

use thiserror::Error;
use tui_logger::TuiWidgetState;

use crate::models::{Filter, Priority, Project, SortOrder, Task, TaskStatus};
use crate::storage::{Database, StorageError, Tag};
use crate::ui::calendar::CalendarState;
use crate::ui::dialogs::Dialog;
use crate::ui::search::SearchResult;

/// How long status messages are displayed before auto-clearing (in seconds).
const STATUS_MESSAGE_TIMEOUT_SECS: u64 = 3;

/// Errors that can occur in the application.
#[derive(Error, Debug)]
pub enum AppError {
    /// Storage/database error
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

/// Result type for application operations.
pub type Result<T> = std::result::Result<T, AppError>;

/// The current view being displayed.
///
/// Determines which screen/layout is shown to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    /// Main view with sidebar and task list
    #[default]
    Main,
    /// Detailed view of a single task
    TaskDetail,
    /// Calendar view showing tasks by date
    Calendar,
    /// Search results view
    Search,
    /// Help/keybindings view
    Help,
    /// Debug log viewer (F12)
    DebugLogs,
}

/// Input mode determines how keyboard input is interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Normal mode - keys trigger commands
    #[default]
    Normal,
    /// Editing mode - keys are typed into input field
    Editing,
    /// Search mode - keys are typed into search field
    Search,
}

/// Which panel currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusPanel {
    /// Projects/tags sidebar
    Sidebar,
    /// Main task list
    #[default]
    TaskList,
}


/// Central application state.
///
/// The `App` struct holds all state for the Ratado application. It follows
/// the single source of truth pattern where all UI state, data, and
/// configuration is managed in one place.
///
/// # Thread Safety
///
/// The App struct is designed to be used from a single thread (the main
/// event loop). Database operations are async but the App itself is not
/// shared between threads.
pub struct App {
    /// Database connection
    pub db: Database,

    /// All tasks loaded from database
    pub tasks: Vec<Task>,

    /// All projects loaded from database
    pub projects: Vec<Project>,

    /// All tags loaded from database
    pub tags: Vec<Tag>,

    /// Current view being displayed
    pub current_view: View,

    /// Current input mode
    pub input_mode: InputMode,

    /// Which panel has focus
    pub focus: FocusPanel,

    /// Index of selected task in the filtered list (None if no tasks)
    pub selected_task_index: Option<usize>,

    /// Index of selected project in sidebar (0 = "All Tasks")
    pub selected_project_index: usize,

    /// Current filter applied to task list
    pub filter: Filter,

    /// Current sort order for task list
    pub sort: SortOrder,

    /// Text input buffer for editing/search modes
    pub input_buffer: String,

    /// Cursor position in input buffer
    pub input_cursor: usize,

    /// State for tui-logger widget
    pub log_state: TuiWidgetState,

    /// Whether the application should exit
    pub should_quit: bool,

    /// Status message to display (temporary)
    pub status_message: Option<String>,

    /// When the status message was set (for auto-clearing)
    status_message_set_at: Option<Instant>,

    /// Task being edited (for edit mode)
    pub editing_task: Option<Task>,

    /// Currently active dialog (if any)
    pub dialog: Option<Dialog>,

    /// Search results (populated when in search view)
    pub search_results: Vec<SearchResult>,

    /// Selected index in search results
    pub selected_search_index: usize,

    /// Calendar view state
    pub calendar_state: CalendarState,
}

impl App {
    /// Creates a new App instance with the given database.
    ///
    /// Loads initial data from the database and sets up default state.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection to use
    ///
    /// # Returns
    ///
    /// A new App instance with data loaded from the database.
    ///
    /// # Errors
    ///
    /// Returns an error if data cannot be loaded from the database.
    pub async fn new(db: Database) -> Result<Self> {
        let mut app = Self {
            db,
            tasks: Vec::new(),
            projects: Vec::new(),
            tags: Vec::new(),
            current_view: View::Main,
            input_mode: InputMode::Normal,
            focus: FocusPanel::TaskList,
            selected_task_index: None,
            selected_project_index: 0,
            filter: Filter::Pending,
            sort: SortOrder::DueDateAsc,
            input_buffer: String::new(),
            input_cursor: 0,
            log_state: TuiWidgetState::default(),
            should_quit: false,
            status_message: None,
            status_message_set_at: None,
            editing_task: None,
            dialog: None,
            search_results: Vec::new(),
            selected_search_index: 0,
            calendar_state: CalendarState::new(),
        };
        app.load_data().await?;
        Ok(app)
    }

    /// Loads all data from the database.
    ///
    /// Fetches tasks, projects, and tags from the database and updates
    /// the app state. Also resets selection if needed.
    pub async fn load_data(&mut self) -> Result<()> {
        self.tasks = self.db.get_all_tasks().await?;
        self.projects = self.db.get_all_projects().await?;
        self.tags = self.db.get_all_tags().await?;

        // Reset task selection if out of bounds
        let visible_count = self.visible_tasks().len();
        if visible_count == 0 {
            self.selected_task_index = None;
        } else if let Some(idx) = self.selected_task_index {
            if idx >= visible_count {
                self.selected_task_index = Some(visible_count.saturating_sub(1));
            }
        } else {
            self.selected_task_index = Some(0);
        }

        Ok(())
    }

    /// Refreshes data from the database.
    ///
    /// Call this after modifying data to sync the UI with the database.
    pub async fn refresh(&mut self) -> Result<()> {
        self.load_data().await
    }

    /// Returns the list of tasks after applying current filter and sort.
    ///
    /// This is the list that should be displayed in the task list UI.
    pub fn visible_tasks(&self) -> Vec<&Task> {
        // Apply project filter if a specific project is selected
        let project_filter = if self.selected_project_index == 0 {
            None // "All Tasks"
        } else {
            self.projects
                .get(self.selected_project_index - 1)
                .map(|p| p.id.clone())
        };

        let mut tasks: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| {
                // Apply project filter
                if let Some(ref proj_id) = project_filter
                    && t.project_id.as_ref() != Some(proj_id)
                {
                    return false;
                }
                // Apply status/date filter
                self.filter.matches(t)
            })
            .collect();

        self.sort.apply(&mut tasks);
        tasks
    }

    /// Returns the currently selected task, if any.
    pub fn selected_task(&self) -> Option<&Task> {
        let tasks = self.visible_tasks();
        self.selected_task_index.and_then(|idx| tasks.get(idx).copied())
    }

    /// Returns tasks filtered by the currently selected project.
    ///
    /// This is useful for search and filter counts that should be
    /// scoped to the current project context.
    pub fn project_tasks(&self) -> Vec<&Task> {
        if self.selected_project_index == 0 {
            // "All Tasks" - return all tasks
            self.tasks.iter().collect()
        } else if let Some(project) = self.projects.get(self.selected_project_index - 1) {
            // Specific project selected
            self.tasks
                .iter()
                .filter(|t| t.project_id.as_ref() == Some(&project.id))
                .collect()
        } else {
            self.tasks.iter().collect()
        }
    }

    /// Returns the name of the currently selected project for display.
    pub fn selected_project_name(&self) -> &str {
        if self.selected_project_index == 0 {
            "All Tasks"
        } else if let Some(project) = self.projects.get(self.selected_project_index - 1) {
            &project.name
        } else {
            "All Tasks"
        }
    }

    /// Returns the currently selected project, if any.
    ///
    /// Returns None if "All Tasks" is selected (index 0).
    pub fn selected_project(&self) -> Option<&Project> {
        if self.selected_project_index == 0 {
            None
        } else {
            self.projects.get(self.selected_project_index - 1)
        }
    }

    /// Returns the task count for a specific project.
    pub fn task_count_for_project(&self, project_id: &str) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.project_id.as_ref().map(|id| id == project_id).unwrap_or(false))
            .filter(|t| t.status != TaskStatus::Archived)
            .count()
    }

    /// Returns the count of all non-archived tasks.
    pub fn total_task_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.status != TaskStatus::Archived)
            .count()
    }

    /// Returns the count of overdue tasks.
    pub fn overdue_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.is_overdue()).count()
    }

    /// Returns the count of tasks due today.
    pub fn due_today_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.is_due_today() && t.status != TaskStatus::Completed)
            .count()
    }

    /// Returns the task count for a specific tag.
    pub fn task_count_for_tag(&self, tag_name: &str) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.tags.contains(&tag_name.to_string()))
            .filter(|t| t.status != TaskStatus::Archived)
            .count()
    }

    /// Moves task selection up.
    pub fn select_previous_task(&mut self) {
        let count = self.visible_tasks().len();
        if count == 0 {
            self.selected_task_index = None;
            return;
        }

        self.selected_task_index = Some(match self.selected_task_index {
            Some(0) => count - 1, // Wrap to end
            Some(i) => i - 1,
            None => 0,
        });
    }

    /// Moves task selection down.
    pub fn select_next_task(&mut self) {
        let count = self.visible_tasks().len();
        if count == 0 {
            self.selected_task_index = None;
            return;
        }

        self.selected_task_index = Some(match self.selected_task_index {
            Some(i) if i >= count - 1 => 0, // Wrap to start
            Some(i) => i + 1,
            None => 0,
        });
    }

    /// Moves project selection up.
    pub fn select_previous_project(&mut self) {
        let count = self.projects.len() + 1; // +1 for "All Tasks"
        if self.selected_project_index == 0 {
            self.selected_project_index = count - 1;
        } else {
            self.selected_project_index -= 1;
        }
        self.update_task_selection();
    }

    /// Moves project selection down.
    pub fn select_next_project(&mut self) {
        let count = self.projects.len() + 1; // +1 for "All Tasks"
        self.selected_project_index = (self.selected_project_index + 1) % count;
        self.update_task_selection();
    }

    /// Updates task selection based on current filters.
    fn update_task_selection(&mut self) {
        let count = self.visible_tasks().len();
        self.selected_task_index = if count > 0 { Some(0) } else { None };
    }

    /// Switches focus between panels.
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPanel::Sidebar => FocusPanel::TaskList,
            FocusPanel::TaskList => FocusPanel::Sidebar,
        };
    }

    /// Sets a temporary status message that auto-clears after a timeout.
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
        self.status_message_set_at = Some(Instant::now());
    }

    /// Clears the status message.
    pub fn clear_status(&mut self) {
        self.status_message = None;
        self.status_message_set_at = None;
    }

    /// Called on each tick of the event loop.
    ///
    /// Used for time-based updates like clearing status messages.
    pub fn on_tick(&mut self) {
        // Auto-clear status message after timeout
        if let Some(set_at) = self.status_message_set_at
            && set_at.elapsed().as_secs() >= STATUS_MESSAGE_TIMEOUT_SECS {
                self.clear_status();
            }
    }

    /// Cycles through filter options.
    pub fn cycle_filter(&mut self) {
        self.filter = match self.filter {
            Filter::All => Filter::Pending,
            Filter::Pending => Filter::Completed,
            Filter::Completed => Filter::DueToday,
            Filter::DueToday => Filter::Overdue,
            Filter::Overdue => Filter::All,
            _ => Filter::All,
        };
        // Reset selection
        let count = self.visible_tasks().len();
        self.selected_task_index = if count > 0 { Some(0) } else { None };
    }

    /// Cycles through sort options.
    pub fn cycle_sort(&mut self) {
        self.sort = match self.sort {
            SortOrder::DueDateAsc => SortOrder::PriorityDesc,
            SortOrder::PriorityDesc => SortOrder::CreatedDesc,
            SortOrder::CreatedDesc => SortOrder::Alphabetical,
            SortOrder::Alphabetical => SortOrder::DueDateAsc,
            _ => SortOrder::DueDateAsc,
        };
    }

    /// Returns the display name for the current filter.
    pub fn filter_name(&self) -> &'static str {
        match self.filter {
            Filter::All => "All",
            Filter::Pending => "Pending",
            Filter::InProgress => "In Progress",
            Filter::Completed => "Completed",
            Filter::Archived => "Archived",
            Filter::DueToday => "Due Today",
            Filter::DueThisWeek => "This Week",
            Filter::Overdue => "Overdue",
            Filter::ByProject(_) => "Project",
            Filter::ByTag(_) => "Tag",
            Filter::ByPriority(Priority::Low) => "Low Priority",
            Filter::ByPriority(Priority::Medium) => "Medium Priority",
            Filter::ByPriority(Priority::High) => "High Priority",
            Filter::ByPriority(Priority::Urgent) => "Urgent",
        }
    }

    /// Returns the display name for the current sort order.
    pub fn sort_name(&self) -> &'static str {
        match self.sort {
            SortOrder::DueDateAsc => "Due Date ↑",
            SortOrder::DueDateDesc => "Due Date ↓",
            SortOrder::PriorityDesc => "Priority ↓",
            SortOrder::PriorityAsc => "Priority ↑",
            SortOrder::CreatedDesc => "Newest",
            SortOrder::CreatedAsc => "Oldest",
            SortOrder::Alphabetical => "A-Z",
        }
    }
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("tasks", &self.tasks.len())
            .field("projects", &self.projects.len())
            .field("current_view", &self.current_view)
            .field("input_mode", &self.input_mode)
            .field("focus", &self.focus)
            .field("should_quit", &self.should_quit)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::run_migrations;

    async fn setup_app() -> App {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();
        App::new(db).await.unwrap()
    }

    #[tokio::test]
    async fn test_app_new() {
        let app = setup_app().await;
        assert!(!app.should_quit);
        assert_eq!(app.current_view, View::Main);
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.focus, FocusPanel::TaskList);
    }

    #[tokio::test]
    async fn test_visible_tasks_empty() {
        let app = setup_app().await;
        let visible = app.visible_tasks();
        assert!(visible.is_empty());
    }

    #[tokio::test]
    async fn test_visible_tasks_with_data() {
        let mut app = setup_app().await;

        // Add some tasks
        let task1 = Task::new("Task 1");
        let task2 = Task::new("Task 2");
        app.db.insert_task(&task1).await.unwrap();
        app.db.insert_task(&task2).await.unwrap();
        app.load_data().await.unwrap();

        let visible = app.visible_tasks();
        assert_eq!(visible.len(), 2);
    }

    #[tokio::test]
    async fn test_select_next_previous_task() {
        let mut app = setup_app().await;

        // Add tasks
        for i in 0..3 {
            let task = Task::new(&format!("Task {}", i));
            app.db.insert_task(&task).await.unwrap();
        }
        app.load_data().await.unwrap();

        assert_eq!(app.selected_task_index, Some(0));

        app.select_next_task();
        assert_eq!(app.selected_task_index, Some(1));

        app.select_next_task();
        assert_eq!(app.selected_task_index, Some(2));

        app.select_next_task(); // Wrap
        assert_eq!(app.selected_task_index, Some(0));

        app.select_previous_task(); // Wrap back
        assert_eq!(app.selected_task_index, Some(2));
    }

    #[tokio::test]
    async fn test_toggle_focus() {
        let mut app = setup_app().await;
        assert_eq!(app.focus, FocusPanel::TaskList);

        app.toggle_focus();
        assert_eq!(app.focus, FocusPanel::Sidebar);

        app.toggle_focus();
        assert_eq!(app.focus, FocusPanel::TaskList);
    }

    #[tokio::test]
    async fn test_cycle_filter() {
        let mut app = setup_app().await;
        assert_eq!(app.filter, Filter::Pending);

        app.cycle_filter();
        assert_eq!(app.filter, Filter::Completed);

        app.cycle_filter();
        assert_eq!(app.filter, Filter::DueToday);
    }

    #[tokio::test]
    async fn test_project_selection() {
        let mut app = setup_app().await;

        // Should have Inbox project from migrations
        assert!(!app.projects.is_empty());
        assert_eq!(app.selected_project_index, 0); // "All Tasks"

        app.select_next_project();
        assert_eq!(app.selected_project_index, 1); // Inbox

        app.select_previous_project();
        assert_eq!(app.selected_project_index, 0); // Back to All
    }

    #[tokio::test]
    async fn test_task_counts() {
        let mut app = setup_app().await;

        let mut task1 = Task::new("Task 1");
        task1.project_id = Some("inbox".to_string());
        app.db.insert_task(&task1).await.unwrap();

        let task2 = Task::new("Task 2");
        app.db.insert_task(&task2).await.unwrap();

        app.load_data().await.unwrap();

        assert_eq!(app.total_task_count(), 2);
        assert_eq!(app.task_count_for_project("inbox"), 1);
    }
}
