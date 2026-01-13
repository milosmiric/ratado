# Ratado Testing Strategy

This document outlines the TDD approach and testing best practices for Ratado.

---

## 1. Architecture for Testability

The key to testing TUI apps is **separation of concerns**. We use a layered architecture:

```
┌─────────────────────────────────────────────────────────────────┐
│                        UI Layer (View)                         │
│              Ratatui widgets, rendering, layout                 │
├─────────────────────────────────────────────────────────────────┤
│                    Handler Layer (Controller)                   │
│              Event handling, command execution                  │
├─────────────────────────────────────────────────────────────────┤
│                    Domain Layer (Model)                         │
│              Task, Project, business logic                      │
├─────────────────────────────────────────────────────────────────┤
│                    Storage Layer                                │
│              Turso database operations                          │
└─────────────────────────────────────────────────────────────────┘
```

**Rules:**
- UI layer has NO business logic - only rendering
- Domain layer has NO dependencies on UI or storage
- Handlers orchestrate between layers
- Storage is injected via traits (for mocking)

---

## 2. Testing Pyramid

```
        /\
       /  \        E2E Tests (few)
      /    \       - Full app with PTY harness
     /──────\
    /        \     Integration Tests (some)
   /          \    - Database operations
  /            \   - Multi-component flows
 /──────────────\
/                \ Unit Tests (many)
/                  \ - Models, pure functions
/                    \ - Isolated handlers
/______________________\ - Widget rendering
```

| Layer | Tools | Speed | Coverage Goal |
|-------|-------|-------|---------------|
| Unit | `#[test]`, `TestBackend` | Fast | 80%+ |
| Integration | `tokio::test`, real DB | Medium | Key flows |
| E2E | `ratatui-testlib` | Slow | Critical paths |

---

## 3. Unit Testing

### 3.1 Testing Models (Domain Layer)

Test business logic in isolation - no database, no UI.

```rust
// src/models/task.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_is_overdue() {
        let task = Task {
            due_date: Some(Utc::now() - Duration::hours(1)),
            status: TaskStatus::Pending,
            ..Default::default()
        };
        assert!(task.is_overdue());
    }

    #[test]
    fn test_completed_task_not_overdue() {
        let task = Task {
            due_date: Some(Utc::now() - Duration::hours(1)),
            status: TaskStatus::Completed,
            ..Default::default()
        };
        assert!(!task.is_overdue()); // Completed tasks aren't overdue
    }

    #[test]
    fn test_task_priority_ordering() {
        let urgent = Task { priority: Priority::Urgent, ..Default::default() };
        let low = Task { priority: Priority::Low, ..Default::default() };
        assert!(urgent > low);
    }
}
```

### 3.2 Testing Handlers (Command Pattern)

Use the **Command/Action pattern** - map inputs to commands, test commands independently.

```rust
// src/handlers/commands.rs

pub enum Command {
    AddTask(String),
    CompleteTask(TaskId),
    DeleteTask(TaskId),
    SetFilter(Filter),
    NavigateUp,
    NavigateDown,
    Quit,
}

impl Command {
    /// Apply command to app state, return whether to continue
    pub fn execute(self, app: &mut App) -> bool {
        match self {
            Command::AddTask(title) => {
                app.tasks.push(Task::new(title));
                true
            }
            Command::CompleteTask(id) => {
                if let Some(task) = app.find_task_mut(id) {
                    task.complete();
                }
                true
            }
            Command::Quit => false,
            // ...
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_task_command() {
        let mut app = App::default();
        let cmd = Command::AddTask("Test task".into());

        let should_continue = cmd.execute(&mut app);

        assert!(should_continue);
        assert_eq!(app.tasks.len(), 1);
        assert_eq!(app.tasks[0].title, "Test task");
    }

    #[test]
    fn test_complete_task_command() {
        let mut app = App::default();
        app.tasks.push(Task::new("Test"));
        let task_id = app.tasks[0].id.clone();

        Command::CompleteTask(task_id).execute(&mut app);

        assert_eq!(app.tasks[0].status, TaskStatus::Completed);
        assert!(app.tasks[0].completed_at.is_some());
    }

    #[test]
    fn test_quit_command_returns_false() {
        let mut app = App::default();
        assert!(!Command::Quit.execute(&mut app));
    }
}
```

### 3.3 Testing Input Mapping

```rust
// src/handlers/input.rs

pub fn map_key_to_command(key: KeyEvent, mode: InputMode) -> Option<Command> {
    match mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => Some(Command::Quit),
            KeyCode::Char('a') => Some(Command::EnterAddMode),
            KeyCode::Char('j') | KeyCode::Down => Some(Command::NavigateDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Command::NavigateUp),
            KeyCode::Char(' ') => Some(Command::ToggleSelected),
            _ => None,
        },
        InputMode::Editing => match key.code {
            KeyCode::Esc => Some(Command::ExitEditMode),
            KeyCode::Enter => Some(Command::SubmitEdit),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_quit_keybinding() {
        let cmd = map_key_to_command(key(KeyCode::Char('q')), InputMode::Normal);
        assert!(matches!(cmd, Some(Command::Quit)));
    }

    #[test]
    fn test_vim_navigation() {
        let j = map_key_to_command(key(KeyCode::Char('j')), InputMode::Normal);
        let k = map_key_to_command(key(KeyCode::Char('k')), InputMode::Normal);

        assert!(matches!(j, Some(Command::NavigateDown)));
        assert!(matches!(k, Some(Command::NavigateUp)));
    }

    #[test]
    fn test_escape_in_edit_mode() {
        let cmd = map_key_to_command(key(KeyCode::Esc), InputMode::Editing);
        assert!(matches!(cmd, Some(Command::ExitEditMode)));
    }
}
```

### 3.4 Testing Widget Rendering (Snapshot Tests)

Use Ratatui's `TestBackend` with `insta` for snapshot testing.

```rust
// src/ui/task_list.rs

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};
    use insta::assert_snapshot;

    fn render_to_string(widget: impl Widget, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| f.render_widget(widget, f.area())).unwrap();

        let buffer = terminal.backend().buffer();
        buffer_to_string(buffer)
    }

    #[test]
    fn test_task_list_rendering() {
        let tasks = vec![
            Task { title: "First task".into(), priority: Priority::High, ..Default::default() },
            Task { title: "Second task".into(), priority: Priority::Low, ..Default::default() },
        ];
        let widget = TaskListWidget::new(&tasks, Some(0));

        let output = render_to_string(widget, 50, 10);
        assert_snapshot!(output);
    }

    #[test]
    fn test_empty_task_list() {
        let widget = TaskListWidget::new(&[], None);
        let output = render_to_string(widget, 50, 10);
        assert_snapshot!(output);
    }

    #[test]
    fn test_task_with_overdue_styling() {
        let tasks = vec![Task {
            title: "Overdue task".into(),
            due_date: Some(Utc::now() - Duration::days(1)),
            ..Default::default()
        }];
        let widget = TaskListWidget::new(&tasks, Some(0));

        let output = render_to_string(widget, 50, 10);
        assert_snapshot!(output);
    }
}
```

---

## 4. Integration Testing

### 4.1 Database Tests

Test storage layer with real Turso database (in-memory for speed).

```rust
// tests/integration/storage_test.rs

use ratado::storage::Database;
use ratado::models::{Task, Priority};

async fn setup_test_db() -> Database {
    // Use in-memory database for tests
    Database::open_in_memory().await.unwrap()
}

#[tokio::test]
async fn test_create_and_retrieve_task() {
    let db = setup_test_db().await;

    let task = Task::new("Test task");
    db.insert_task(&task).await.unwrap();

    let retrieved = db.get_task(&task.id).await.unwrap();
    assert_eq!(retrieved.title, "Test task");
}

#[tokio::test]
async fn test_query_tasks_by_status() {
    let db = setup_test_db().await;

    db.insert_task(&Task::new("Pending 1")).await.unwrap();
    db.insert_task(&Task::new("Pending 2")).await.unwrap();

    let mut completed = Task::new("Completed");
    completed.complete();
    db.insert_task(&completed).await.unwrap();

    let pending = db.query_tasks_by_status(TaskStatus::Pending).await.unwrap();
    assert_eq!(pending.len(), 2);
}

#[tokio::test]
async fn test_update_task() {
    let db = setup_test_db().await;

    let mut task = Task::new("Original");
    db.insert_task(&task).await.unwrap();

    task.title = "Updated".into();
    task.priority = Priority::Urgent;
    db.update_task(&task).await.unwrap();

    let retrieved = db.get_task(&task.id).await.unwrap();
    assert_eq!(retrieved.title, "Updated");
    assert_eq!(retrieved.priority, Priority::Urgent);
}
```

### 4.2 App State Integration

Test complete flows through the app.

```rust
// tests/integration/app_flow_test.rs

#[tokio::test]
async fn test_add_task_flow() {
    let db = Database::open_in_memory().await.unwrap();
    let mut app = App::new(db);

    // Simulate: press 'a', type title, press Enter
    app.handle_command(Command::EnterAddMode);
    app.input_buffer = "New task".into();
    app.handle_command(Command::SubmitEdit);

    assert_eq!(app.tasks.len(), 1);
    assert_eq!(app.tasks[0].title, "New task");
    assert_eq!(app.input_mode, InputMode::Normal);
}

#[tokio::test]
async fn test_filter_flow() {
    let db = Database::open_in_memory().await.unwrap();
    let mut app = App::new(db);

    // Add mixed tasks
    app.handle_command(Command::AddTask("Task 1".into()));
    app.handle_command(Command::AddTask("Task 2".into()));
    app.handle_command(Command::CompleteTask(app.tasks[0].id.clone()));

    // Filter to pending only
    app.handle_command(Command::SetFilter(Filter::Pending));

    assert_eq!(app.visible_tasks().len(), 1);
    assert_eq!(app.visible_tasks()[0].title, "Task 2");
}
```

---

## 5. End-to-End Testing

Use `ratatui-testlib` for full PTY-based testing.

```rust
// tests/e2e/app_test.rs

use ratatui_testlib::{TuiTestHarness, Result};

#[test]
fn test_app_startup() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;
    harness.spawn("cargo run")?;

    // Wait for app to render
    harness.wait_for(|state| {
        state.contents().contains("Ratado")
    })?;

    // Verify initial UI elements
    assert!(harness.screen_contents().contains("PROJECTS"));
    assert!(harness.screen_contents().contains("TASKS"));
    Ok(())
}

#[test]
fn test_add_task_e2e() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;
    harness.spawn("cargo run")?;

    harness.wait_for(|s| s.contents().contains("Ratado"))?;

    // Press 'a' to add task
    harness.send_key('a')?;
    harness.wait_for(|s| s.contents().contains("Add New Task"))?;

    // Type task title
    harness.send_text("Buy groceries")?;
    harness.send_key('\n')?; // Enter

    // Verify task appears
    harness.wait_for(|s| s.contents().contains("Buy groceries"))?;

    Ok(())
}

#[test]
fn test_quit_with_q() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;
    harness.spawn("cargo run")?;

    harness.wait_for(|s| s.contents().contains("Ratado"))?;
    harness.send_key('q')?;

    // App should exit
    harness.wait_for_exit()?;
    Ok(())
}
```

---

## 6. Test Organization

```
ratado/
├── src/
│   ├── models/
│   │   ├── task.rs        # Unit tests inline with #[cfg(test)]
│   │   └── project.rs
│   ├── handlers/
│   │   ├── commands.rs    # Unit tests inline
│   │   └── input.rs
│   └── ui/
│       └── task_list.rs   # Snapshot tests inline
├── tests/
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── storage_test.rs
│   │   └── app_flow_test.rs
│   └── e2e/
│       ├── mod.rs
│       └── app_test.rs
└── snapshots/             # insta snapshot files
    └── ratado__ui__task_list__tests__task_list_rendering.snap
```

---

## 7. TDD Workflow

### Red-Green-Refactor Cycle

```
1. RED:    Write a failing test for new behavior
2. GREEN:  Write minimum code to pass the test
3. REFACTOR: Clean up while keeping tests green
```

### Example: Adding "Due Today" Filter

```rust
// Step 1: RED - Write failing test
#[test]
fn test_due_today_filter() {
    let today = Task {
        title: "Due today".into(),
        due_date: Some(today_midnight()),
        ..Default::default()
    };
    let tomorrow = Task {
        title: "Due tomorrow".into(),
        due_date: Some(tomorrow_midnight()),
        ..Default::default()
    };

    let tasks = vec![today.clone(), tomorrow];
    let filtered = Filter::DueToday.apply(&tasks);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].title, "Due today");
}

// Step 2: GREEN - Implement filter
impl Filter {
    pub fn apply<'a>(&self, tasks: &'a [Task]) -> Vec<&'a Task> {
        match self {
            Filter::DueToday => tasks.iter()
                .filter(|t| t.is_due_today())
                .collect(),
            // ...
        }
    }
}

// Step 3: REFACTOR - Extract helper if needed
impl Task {
    pub fn is_due_today(&self) -> bool {
        self.due_date
            .map(|d| d.date_naive() == Utc::now().date_naive())
            .unwrap_or(false)
    }
}
```

---

## 8. Testing Dependencies

Add to `Cargo.toml`:

```toml
[dev-dependencies]
# Snapshot testing
insta = { version = "1.40", features = ["yaml"] }

# Async test support
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }

# E2E testing (optional)
ratatui-testlib = "0.1"

# Test utilities
pretty_assertions = "1.4"
```

---

## 9. CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run unit tests
        run: cargo test --lib

      - name: Run integration tests
        run: cargo test --test '*'

      - name: Check snapshots
        run: cargo insta test --review=false

      - name: Clippy
        run: cargo clippy -- -D warnings
```

---

## 10. References

- [Ratatui Best Practices Discussion](https://github.com/ratatui/ratatui/discussions/220)
- [ratatui-testlib Documentation](https://docs.rs/ratatui-testlib/latest/ratatui_testlib/)
- [insta Snapshot Testing](https://insta.rs/)
- [Ratatui Templates](https://github.com/ratatui/templates)
