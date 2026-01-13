# Phase 12: Testing & Polish

**Goal:** Ensure comprehensive test coverage and handle edge cases.

**Prerequisites:** All previous phases

**Outcome:** Production-ready application with good test coverage.

---

## Story 12.1: Unit Test Coverage

**Priority:** High
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Review and add tests for all core logic.

### Tasks

- [ ] Review test coverage for models:
  - `Task` - all methods tested
  - `Priority` - ordering, cycling
  - `TaskStatus` - transitions
  - `Filter` - all filter types
  - `SortOrder` - all sort types

- [ ] Review test coverage for commands:
  - Navigation commands
  - Task CRUD commands
  - Filter/sort commands
  - View commands

- [ ] Review test coverage for utilities:
  - Date formatting
  - Date parsing
  - ID generation

- [ ] Add missing tests
- [ ] Aim for 80%+ coverage on core logic

### Test Checklist

```rust
// Models
#[cfg(test)]
mod task_tests {
    // Creation
    fn test_task_new()
    fn test_task_default()

    // Status methods
    fn test_task_complete()
    fn test_task_reopen()
    fn test_task_is_overdue_when_past_due()
    fn test_task_is_overdue_when_completed()
    fn test_task_is_due_today()
    fn test_task_is_due_this_week()
    fn test_task_not_overdue_without_due_date()

    // Priority
    fn test_priority_ordering()
    fn test_priority_cycle()
}

// Commands
#[cfg(test)]
mod command_tests {
    // Navigation
    fn test_navigate_down()
    fn test_navigate_down_at_bottom()
    fn test_navigate_up()
    fn test_navigate_up_at_top()
    fn test_navigate_top()
    fn test_navigate_bottom()
    fn test_switch_panel()

    // Task operations
    fn test_toggle_task_pending_to_completed()
    fn test_toggle_task_completed_to_pending()
    fn test_cycle_priority()
    fn test_delete_task()

    // Filters
    fn test_set_filter()
    fn test_clear_filter()
}

// Utilities
#[cfg(test)]
mod datetime_tests {
    fn test_format_relative_today()
    fn test_format_relative_tomorrow()
    fn test_format_relative_yesterday()
    fn test_format_relative_this_week()
    fn test_format_relative_future()
    fn test_is_same_day()
    fn test_days_until_positive()
    fn test_days_until_negative()
}
```

### Acceptance Criteria

- [ ] All models have comprehensive tests
- [ ] All commands have tests
- [ ] All utilities have tests
- [ ] `cargo test` passes
- [ ] No panics in tests

---

## Story 12.2: Integration Tests

**Priority:** High
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Add integration tests for database and full workflows.

### Tasks

- [ ] Database integration tests:
  - Task CRUD operations
  - Project CRUD operations
  - Tag operations
  - Complex queries (filter + sort)
  - Migration idempotency

- [ ] Workflow integration tests:
  - Add task → appears in list
  - Complete task → status updates
  - Delete task → removed from list
  - Filter → correct tasks shown
  - Search → finds matching tasks

- [ ] Use in-memory database for speed

### Code Sketch

```rust
// tests/integration/workflow_test.rs

async fn setup() -> App {
    let db = Database::open_in_memory().await.unwrap();
    run_migrations(&db).await.unwrap();
    App::new(db).await.unwrap()
}

#[tokio::test]
async fn test_full_task_lifecycle() {
    let mut app = setup().await;

    // Initially empty
    assert!(app.tasks.is_empty());

    // Add task
    let task = Task::new("Test task");
    app.db.insert_task(&task).await.unwrap();
    app.load_data().await.unwrap();

    assert_eq!(app.tasks.len(), 1);
    assert_eq!(app.tasks[0].title, "Test task");
    assert_eq!(app.tasks[0].status, TaskStatus::Pending);

    // Complete task
    app.selected_task_index = Some(0);
    Command::ToggleTaskStatus.execute(&mut app).await.unwrap();

    assert_eq!(app.tasks[0].status, TaskStatus::Completed);
    assert!(app.tasks[0].completed_at.is_some());

    // Delete task
    let task_id = app.tasks[0].id.clone();
    app.db.delete_task(&task_id).await.unwrap();
    app.load_data().await.unwrap();

    assert!(app.tasks.is_empty());
}

#[tokio::test]
async fn test_filter_workflow() {
    let mut app = setup().await;

    // Add tasks with different statuses
    let mut task1 = Task::new("Pending task");
    let mut task2 = Task::new("Completed task");
    task2.complete();

    app.db.insert_task(&task1).await.unwrap();
    app.db.insert_task(&task2).await.unwrap();
    app.load_data().await.unwrap();

    // All tasks visible initially
    assert_eq!(app.visible_tasks().len(), 2);

    // Filter to pending
    app.filter = Filter::Pending;
    assert_eq!(app.visible_tasks().len(), 1);
    assert_eq!(app.visible_tasks()[0].title, "Pending task");

    // Filter to completed
    app.filter = Filter::Completed;
    assert_eq!(app.visible_tasks().len(), 1);
    assert_eq!(app.visible_tasks()[0].title, "Completed task");
}

#[tokio::test]
async fn test_project_workflow() {
    let mut app = setup().await;

    // Create project
    let project = Project::new("Work");
    app.db.insert_project(&project).await.unwrap();

    // Add task to project
    let mut task = Task::new("Work task");
    task.project_id = Some(project.id.clone());
    app.db.insert_task(&task).await.unwrap();

    // Add task without project
    let task2 = Task::new("Personal task");
    app.db.insert_task(&task2).await.unwrap();

    app.load_data().await.unwrap();

    // Filter by project
    app.filter = Filter::ByProject(project.id.clone());
    assert_eq!(app.visible_tasks().len(), 1);
    assert_eq!(app.visible_tasks()[0].title, "Work task");
}
```

### Acceptance Criteria

- [ ] All database operations tested
- [ ] Key workflows tested end-to-end
- [ ] Tests run quickly (in-memory DB)
- [ ] All integration tests pass

---

## Story 12.3: UI Snapshot Tests

**Priority:** Medium
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Add snapshot tests for UI components using insta.

### Tasks

- [ ] Set up insta snapshot testing
- [ ] Create test helpers for rendering widgets
- [ ] Add snapshots for:
  - Task list (various states)
  - Sidebar
  - Header
  - Status bar
  - Help overlay
  - Empty state

- [ ] Review and approve initial snapshots
- [ ] Add CI check for snapshot changes

### Code Sketch

```rust
// src/ui/task_list.rs
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};
    use insta::assert_snapshot;

    fn render_widget<W: Widget>(widget: W, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| f.render_widget(widget, f.area())).unwrap();

        let buffer = terminal.backend().buffer();
        let mut output = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = buffer.get(x, y);
                output.push_str(cell.symbol());
            }
            output.push('\n');
        }
        output
    }

    #[test]
    fn test_task_list_empty() {
        let app = App::default();
        let widget = TaskListWidget::new(&app.visible_tasks(), None);
        let output = render_widget(widget, 60, 10);
        assert_snapshot!(output);
    }

    #[test]
    fn test_task_list_with_tasks() {
        let tasks = vec![
            Task {
                title: "High priority task".into(),
                priority: Priority::High,
                status: TaskStatus::Pending,
                ..Default::default()
            },
            Task {
                title: "Completed task".into(),
                status: TaskStatus::Completed,
                ..Default::default()
            },
        ];

        let output = render_task_list_to_string(&tasks, Some(0), 60, 10);
        assert_snapshot!(output);
    }

    #[test]
    fn test_task_list_overdue() {
        let tasks = vec![Task {
            title: "Overdue task".into(),
            due_date: Some(Utc::now() - Duration::days(1)),
            status: TaskStatus::Pending,
            ..Default::default()
        }];

        let output = render_task_list_to_string(&tasks, Some(0), 60, 10);
        assert_snapshot!(output);
    }
}
```

### Acceptance Criteria

- [ ] Snapshots capture expected UI
- [ ] Tests catch unintended UI changes
- [ ] Easy to update snapshots when intended
- [ ] CI fails on unexpected snapshot changes

---

## Story 12.4: Error Handling & Edge Cases

**Priority:** High
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Review and improve error handling throughout the application.

### Tasks

- [ ] Audit all `unwrap()` and `expect()` calls:
  - Replace with proper error handling
  - Use `?` operator where appropriate
  - Log errors with context

- [ ] User-friendly error messages:
  - Database errors → "Failed to save task. Please try again."
  - File errors → "Cannot access config file: {path}"
  - Network errors (if any) → clear message

- [ ] Edge cases to handle:
  - Empty task title (validation)
  - Very long task titles (truncation)
  - Many tasks (performance)
  - Unicode in task titles
  - Special characters in search
  - Corrupt database file
  - Missing config directory permissions
  - Terminal too small

- [ ] Graceful degradation:
  - Desktop notifications fail → just log, continue
  - Theme file missing → use default
  - Config parse error → use defaults, warn user

### Code Sketch

```rust
// Error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] StorageError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Task validation failed: {0}")]
    Validation(String),
}

// Validation
impl Task {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.title.trim().is_empty() {
            return Err(AppError::Validation("Task title cannot be empty".to_string()));
        }
        if self.title.len() > 100 {
            return Err(AppError::Validation("Task title too long (max 100 chars)".to_string()));
        }
        Ok(())
    }
}

// Graceful error handling in main
async fn run_app() -> Result<(), AppError> {
    // Load config with fallback
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to load config, using defaults: {}", e);
            Config::default()
        }
    };

    // Initialize database
    let db_path = Database::default_path()?;
    let db = Database::open(&db_path).await.map_err(|e| {
        log::error!("Failed to open database: {}", e);
        e
    })?;

    // ... rest of app
}

// Terminal size check
fn check_terminal_size(size: Rect) -> Result<(), AppError> {
    if size.width < 40 || size.height < 10 {
        return Err(AppError::Validation(
            "Terminal too small. Minimum size: 40x10".to_string()
        ));
    }
    Ok(())
}
```

### Acceptance Criteria

- [ ] No panics on common error conditions
- [ ] User sees helpful error messages
- [ ] Errors logged for debugging
- [ ] App recovers gracefully where possible
- [ ] Edge cases handled properly

---

## Phase 12 Checklist

Before considering project complete:

- [ ] All 4 stories completed
- [ ] Unit test coverage > 80% for core logic
- [ ] All integration tests pass
- [ ] Snapshot tests in place
- [ ] No `unwrap()` in production code paths
- [ ] Error messages are user-friendly
- [ ] Edge cases handled
- [ ] `cargo clippy` clean
- [ ] `cargo fmt` applied
- [ ] Documentation complete

---

## Final Release Checklist

- [ ] All phases completed
- [ ] All tests pass
- [ ] Documentation complete
- [ ] README written
- [ ] Version set in Cargo.toml
- [ ] License file added
- [ ] Git tags created
- [ ] Consider publishing to crates.io
