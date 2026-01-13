# Phase 1: Project Foundation

**Goal:** Set up the Cargo project with proper structure, dependencies, and core data models.

**Prerequisites:** None (this is the starting phase)

**Outcome:** Project compiles, all modules in place, core models implemented with tests.

**Status:** ✅ COMPLETED

---

## Story 1.1: Initialize Cargo Project

**Priority:** Critical
**Estimate:** Small
**Status:** [x] Completed

### Description

Set up the Cargo project with proper structure and dependencies.

### Tasks

- [x] Create `Cargo.toml` with all dependencies from specification (section 6.2)
- [x] Create `src/main.rs` with minimal async entry point
- [x] Create `src/lib.rs` for library exports
- [x] Verify project compiles with `cargo build`

### Cargo.toml Reference

```toml
[package]
name = "ratado"
version = "0.1.0"
edition = "2024"

[dependencies]
turso = "0.4"
ratatui = { version = "0.30", features = ["crossterm"] }
crossterm = "0.29"
tokio = { version = "1.49", features = ["rt-multi-thread", "macros", "time"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.9"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.19", features = ["v7", "serde"] }
directories = "6.0"
notify-rust = "4.11"
clap = { version = "4.5", features = ["derive"] }
thiserror = "2.0"
tui-logger = "0.18"
log = "0.4"

[dev-dependencies]
insta = { version = "1.46", features = ["yaml"] }
pretty_assertions = "1.4"
```

### Acceptance Criteria

- [x] `cargo build` succeeds without errors
- [x] `cargo run` starts and exits cleanly
- [x] All dependencies resolve correctly

---

## Story 1.2: Create Module Structure

**Priority:** Critical
**Estimate:** Small
**Status:** [x] Completed

### Description

Set up the directory and module structure as defined in the specification.

### Tasks

- [x] Create `src/app.rs` with empty App struct placeholder
- [x] Create `src/ui/mod.rs` with submodule declarations
- [x] Create `src/models/mod.rs` with submodule declarations
- [x] Create `src/handlers/mod.rs` with submodule declarations
- [x] Create `src/storage/mod.rs` with submodule declarations
- [x] Create `src/notifications/mod.rs` placeholder
- [x] Create `src/utils/mod.rs` with submodule declarations
- [x] Wire up all modules in `lib.rs`

### Expected Structure

```
src/
├── main.rs
├── lib.rs
├── app.rs
├── ui/
│   └── mod.rs
├── models/
│   └── mod.rs
├── handlers/
│   └── mod.rs
├── storage/
│   └── mod.rs
├── notifications/
│   └── mod.rs
└── utils/
    └── mod.rs
```

### Acceptance Criteria

- [x] All modules compile without errors
- [x] Module tree matches specification architecture
- [x] `cargo check` passes

---

## Story 1.3: Implement Core Models

**Priority:** Critical
**Estimate:** Medium
**Status:** [x] Completed

### Description

Implement the Task, Project, and related data structures.

### Tasks

- [x] Create `src/models/task.rs`:
  - `Task` struct with all fields:
    - `id: String` (UUID v7)
    - `title: String`
    - `description: Option<String>`
    - `due_date: Option<DateTime<Utc>>`
    - `priority: Priority`
    - `status: TaskStatus`
    - `project_id: Option<String>`
    - `tags: Vec<String>`
    - `created_at: DateTime<Utc>`
    - `updated_at: DateTime<Utc>`
    - `completed_at: Option<DateTime<Utc>>`
  - `Priority` enum: `Low`, `Medium`, `High`, `Urgent`
  - `TaskStatus` enum: `Pending`, `InProgress`, `Completed`, `Archived`
  - Derive: `Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`
  - Implement `Default` for Task
  - Implement `PartialOrd`, `Ord` for Priority
  - Helper methods:
    - `Task::new(title: &str) -> Task`
    - `Task::is_overdue(&self) -> bool`
    - `Task::is_due_today(&self) -> bool`
    - `Task::is_due_this_week(&self) -> bool`
    - `Task::complete(&mut self)`
    - `Task::reopen(&mut self)`

- [x] Create `src/models/project.rs`:
  - `Project` struct:
    - `id: String`
    - `name: String`
    - `color: String` (hex color)
    - `icon: String`
    - `created_at: DateTime<Utc>`
  - `Project::new(name: &str) -> Project`
  - `Project::with_style(name: &str, color: &str, icon: &str) -> Project`

- [x] Create `src/models/filter.rs`:
  - `Filter` enum:
    - `All`
    - `Pending`
    - `InProgress`
    - `Completed`
    - `Archived`
    - `DueToday`
    - `DueThisWeek`
    - `Overdue`
    - `ByProject(String)`
    - `ByTag(String)`
    - `ByPriority(Priority)`
  - `SortOrder` enum:
    - `DueDateAsc`
    - `DueDateDesc`
    - `PriorityDesc`
    - `PriorityAsc`
    - `CreatedDesc`
    - `CreatedAsc`
    - `Alphabetical`
  - `Filter::apply(&self, tasks: &[Task]) -> Vec<&Task>`
  - `Filter::matches(&self, task: &Task) -> bool`
  - `SortOrder::apply(&self, tasks: &mut [&Task])`

- [x] Update `src/models/mod.rs` to export all types
- [x] Add unit tests for all models

### Test Cases

```rust
#[test]
fn test_task_is_overdue() {
    let mut task = Task::new("Test");
    task.due_date = Some(Utc::now() - Duration::hours(1));
    assert!(task.is_overdue());
}

#[test]
fn test_completed_task_not_overdue() {
    let mut task = Task::new("Test");
    task.due_date = Some(Utc::now() - Duration::hours(1));
    task.complete();
    assert!(!task.is_overdue());
}

#[test]
fn test_priority_ordering() {
    assert!(Priority::Urgent > Priority::High);
    assert!(Priority::High > Priority::Medium);
    assert!(Priority::Medium > Priority::Low);
}
```

### Acceptance Criteria

- [x] All model structs serialize/deserialize correctly with serde
- [x] `Task::is_overdue()` returns correct results
- [x] `Task::is_due_today()` returns correct results
- [x] Priority ordering works as expected
- [x] All unit tests pass

---

## Story 1.4: Implement Utility Modules

**Priority:** High
**Estimate:** Small
**Status:** [x] Completed

### Description

Create utility functions for date/time handling and other helpers.

### Tasks

- [x] Create `src/utils/datetime.rs`:
  - `format_relative_date(date: DateTime<Utc>) -> String`
    - Returns: "Today", "Tomorrow", "Yesterday", "Mon 15", "Jan 15"
  - `format_due_date(date: Option<DateTime<Utc>>) -> String`
    - Returns formatted due date or empty string
  - `is_same_day(a: DateTime<Utc>, b: DateTime<Utc>) -> bool`
  - `is_today(date: DateTime<Utc>) -> bool`
  - `is_this_week(date: DateTime<Utc>) -> bool`
  - `days_until(date: DateTime<Utc>) -> i64`
  - `now() -> DateTime<Utc>` (wrapper for testing)

- [x] Create `src/utils/ids.rs`:
  - `generate_id() -> String` (UUID v7 - time-ordered)

- [x] Update `src/utils/mod.rs` to export all utilities
- [x] Add unit tests

### Test Cases

```rust
#[test]
fn test_format_relative_date_today() {
    let today = Utc::now();
    assert_eq!(format_relative_date(today), "Today");
}

#[test]
fn test_format_relative_date_tomorrow() {
    let tomorrow = Utc::now() + Duration::days(1);
    assert_eq!(format_relative_date(tomorrow), "Tomorrow");
}

#[test]
fn test_days_until_positive() {
    let future = Utc::now() + Duration::days(5);
    assert_eq!(days_until(future), 5);
}
```

### Acceptance Criteria

- [x] Date formatting matches UI mockups
- [x] All utility functions have tests
- [x] `generate_id()` produces valid UUIDs (v7)

---

## Phase 1 Checklist

Before moving to Phase 2, ensure:

- [x] All 4 stories completed
- [x] `cargo build` succeeds
- [x] `cargo test` passes (all model and utility tests) - 29 unit tests + 29 doc tests
- [x] `cargo clippy` has no warnings
- [x] Module structure matches specification
- [x] All code is documented with rustdoc comments
