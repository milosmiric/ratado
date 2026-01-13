# Ratado - Terminal Task Manager Specification

## 1. Project Overview

### 1.1 Introduction

Ratado is a terminal-based task manager and reminder application built with Rust and the Ratatui framework. The name combines "Rata" (from Ratatui) with "do" (from todo), reflecting its technical foundation and purpose.

### 1.2 Goals

- **Fast and Lightweight**: Instant startup, minimal resource usage
- **Keyboard-Driven**: Full functionality accessible without a mouse
- **Offline-First**: All data stored locally, no network dependency
- **Vim-Inspired Navigation**: Familiar keybindings for terminal users
- **Reminder System**: Notify users of upcoming and overdue tasks

### 1.3 Target Users

- Developers and terminal power users
- Users who prefer keyboard-driven workflows
- Anyone seeking a distraction-free task management solution

---

## 2. Core Features

### 2.1 Task Management

| Feature | Description |
|---------|-------------|
| Create Task | Add new tasks with title, description, due date, priority |
| Edit Task | Modify any task attribute |
| Delete Task | Remove tasks with confirmation |
| Complete Task | Mark tasks as done (with timestamp) |
| Archive Task | Move completed tasks to archive |

### 2.2 Task Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | UUID | Auto | Unique identifier |
| `title` | String | Yes | Task name (max 100 chars) |
| `description` | String | No | Detailed notes (max 1000 chars) |
| `due_date` | DateTime | No | When task is due |
| `priority` | Enum | Yes | Low, Medium, High, Urgent |
| `status` | Enum | Yes | Pending, InProgress, Completed, Archived |
| `tags` | Vec<String> | No | Categorization labels |
| `created_at` | DateTime | Auto | Creation timestamp |
| `updated_at` | DateTime | Auto | Last modification timestamp |
| `completed_at` | DateTime | Auto | Completion timestamp |

### 2.3 Task Organization

- **Projects**: Group related tasks under named projects
- **Tags**: Flexible labeling system for cross-project categorization
- **Filters**: View tasks by status, priority, due date, project, or tag
- **Sorting**: Order by due date, priority, creation date, or alphabetically

### 2.4 Reminders

| Reminder Type | Description |
|---------------|-------------|
| Due Soon | Tasks due within configurable time window (default: 24 hours) |
| Overdue | Tasks past their due date |
| Daily Digest | Summary of tasks for the day (optional) |

Reminder delivery methods:
- In-app notification banner
- Terminal bell
- Desktop notification (via `notify-rust` crate)

---

## 3. User Interface

### 3.1 Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│ Ratado                                            [!3] [Today: 5]   │
├─────────────────┬───────────────────────────────────────────────────┤
│                 │                                                   │
│  PROJECTS       │  TASKS                                            │
│  ─────────      │  ─────                                            │
│  > Inbox    (3) │  [ ] !! Buy groceries           Due: Today        │
│    Work     (5) │  [>]  ! Finish report           Due: Tomorrow     │
│    Personal (2) │  [ ]    Call dentist            Due: Friday       │
│    Shopping (1) │  [x]    Send email              Done: Yesterday   │
│                 │                                                   │
│  TAGS           │                                                   │
│  ────           │                                                   │
│    #urgent  (2) │                                                   │
│    #home    (3) │                                                   │
│                 │                                                   │
├─────────────────┴───────────────────────────────────────────────────┤
│ [a]dd [e]dit [d]elete [Space]toggle [/]search [?]help    Filter: All│
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 Views

| View | Description | Shortcut |
|------|-------------|----------|
| Main | Split panel with projects/tags and task list | Default |
| Task Detail | Full task information and edit mode | `Enter` |
| Calendar | Monthly view with task indicators | `c` |
| Search | Filter tasks by text query | `/` |
| Help | Keybinding reference | `?` |
| Debug Logs | tui-logger widget for debugging (dev mode) | `F12` |

### 3.3 Visual Indicators

| Symbol | Meaning |
|--------|---------|
| `[ ]` | Pending task |
| `[>]` | In progress |
| `[x]` | Completed |
| `!!` | Urgent priority |
| `!` | High priority |
| (none) | Medium/Low priority |
| Red text | Overdue |
| Yellow text | Due today |
| Cyan text | Due this week |

### 3.4 Color Scheme

Support for multiple themes with sensible defaults:

| Element | Default Color |
|---------|---------------|
| Background | Terminal default |
| Text | White |
| Selection | Blue background |
| Urgent | Red |
| High Priority | Yellow |
| Completed | Gray/Dim |
| Overdue | Red (bold) |
| Borders | Gray |

---

## 4. Keybindings

### 4.1 Global

| Key | Action |
|-----|--------|
| `q` / `Ctrl+c` | Quit application |
| `?` | Toggle help panel |
| `/` | Enter search mode |
| `Esc` | Cancel / Close popup / Exit mode |
| `Tab` | Switch focus between panels |
| `1-4` | Quick filter by priority |
| `r` | Refresh view |

### 4.2 Navigation (Vim-style)

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g` / `Home` | Go to first item |
| `G` / `End` | Go to last item |
| `h` / `←` | Collapse / Previous panel |
| `l` / `→` | Expand / Next panel |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |

### 4.3 Task Actions

| Key | Action |
|-----|--------|
| `a` | Add new task |
| `e` / `Enter` | Edit selected task |
| `d` | Delete task (with confirmation) |
| `Space` | Toggle task status (Pending ↔ Completed) |
| `p` | Cycle priority |
| `t` | Add/edit tags |
| `m` | Move to project |
| `y` | Yank (copy) task |
| `P` | Paste task |

### 4.4 Views and Filters

| Key | Action |
|-----|--------|
| `c` | Calendar view |
| `f` | Filter menu |
| `s` | Sort menu |
| `A` | Toggle show archived |
| `T` | Show today's tasks only |
| `W` | Show this week's tasks |

---

## 5. Data Model

### 5.1 Database

Ratado uses **Turso Database** - an in-process SQL database written in pure Rust with SQLite compatibility.

**Why Turso:**
- Pure Rust implementation (no C dependencies)
- SQLite-compatible SQL dialect and file format
- Native async I/O support
- Vector search capabilities (for future AI features)
- Cross-platform (Linux, macOS, Windows)

**Storage location** (XDG compliant):
```
~/.config/ratado/
├── ratado.db        # Turso database file
├── config.toml      # User preferences
└── themes/
    └── custom.toml  # User-defined themes
```

### 5.2 Database Schema

```sql
-- Tasks table
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    due_date TIMESTAMP,
    priority TEXT NOT NULL DEFAULT 'medium'
        CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_progress', 'completed', 'archived')),
    project_id TEXT REFERENCES projects(id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);

-- Projects table
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT DEFAULT '#4A90D9',
    icon TEXT DEFAULT 'folder',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Tags table
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

-- Task-Tags junction table
CREATE TABLE task_tags (
    task_id TEXT REFERENCES tasks(id) ON DELETE CASCADE,
    tag_id TEXT REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, tag_id)
);

-- Indexes for common queries
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_due_date ON tasks(due_date);
CREATE INDEX idx_tasks_priority ON tasks(priority);
CREATE INDEX idx_tasks_project ON tasks(project_id);
```

### 5.3 Database Usage Example

```rust
use turso::Builder;

// Initialize database
let db = Builder::new_local("~/.config/ratado/ratado.db")
    .build()
    .await?;
let conn = db.connect()?;

// Query tasks due today
let tasks = conn.query(
    "SELECT * FROM tasks
     WHERE status = 'pending'
       AND date(due_date) = date('now')
     ORDER BY priority DESC",
    ()
).await?;

// Insert new task
conn.execute(
    "INSERT INTO tasks (id, title, priority) VALUES (?, ?, ?)",
    (uuid::Uuid::new_v4().to_string(), "Buy groceries", "high")
).await?;
```

### 5.4 Configuration Schema

Configuration stored in TOML format at `~/.config/ratado/config.toml`:

```toml
[general]
theme = "default"
default_priority = "medium"
date_format = "%Y-%m-%d"
time_format = "%H:%M"
week_start = "monday"

[display]
show_completed_tasks = true
auto_archive_days = 7

[notifications]
enabled = true
sound = true
desktop = true
reminder_window_hours = 24
```

---

## 6. Technical Architecture

### 6.1 Project Structure

```
ratado/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point
│   ├── app.rs               # Application state
│   ├── lib.rs               # Library exports
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── layout.rs        # Main layout composition
│   │   ├── task_list.rs     # Task list widget
│   │   ├── project_panel.rs # Projects/tags sidebar
│   │   ├── task_detail.rs   # Task detail view
│   │   ├── calendar.rs      # Calendar view
│   │   ├── search.rs        # Search interface
│   │   ├── help.rs          # Help overlay
│   │   └── input.rs         # Text input components
│   ├── models/
│   │   ├── mod.rs
│   │   ├── task.rs          # Task struct and methods
│   │   ├── project.rs       # Project struct
│   │   └── config.rs        # Configuration struct
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── input.rs         # Keyboard event handling
│   │   └── commands.rs      # Command execution
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── database.rs      # Turso database operations
│   │   └── migrations.rs    # Schema migrations
│   ├── notifications/
│   │   ├── mod.rs
│   │   └── reminder.rs      # Reminder system
│   └── utils/
│       ├── mod.rs
│       └── datetime.rs      # Date/time helpers
├── tests/
│   ├── integration/
│   └── unit/
└── docs/
    └── SPECIFICATION.md
```

### 6.2 Dependencies

| Crate | Purpose |
|-------|---------|
| `turso` | Embedded SQL database (pure Rust, SQLite-compatible) |
| `ratatui` | Terminal UI framework |
| `crossterm` | Cross-platform terminal manipulation |
| `tokio` | Async runtime (required by Turso and reminders) |
| `serde` / `toml` | Configuration serialization |
| `chrono` | Date and time handling |
| `uuid` | Unique identifiers |
| `directories` | XDG path resolution |
| `notify-rust` | Desktop notifications |
| `clap` | Command-line argument parsing |
| `thiserror` | Error handling |
| `tui-logger` | In-app logging widget for debugging |
| `log` | Logging facade (used by tui-logger) |

**Cargo.toml reference** (versions as of January 2026):

```toml
[package]
name = "ratado"
version = "0.1.0"
edition = "2024"

[dependencies]
# Database
turso = "0.4"

# TUI
ratatui = { version = "0.30", features = ["crossterm"] }
crossterm = "0.29"

# Async runtime
tokio = { version = "1.49", features = ["rt-multi-thread", "macros", "time"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.11", features = ["v4", "serde"] }
directories = "6.0"

# Notifications
notify-rust = "4.11"

# CLI
clap = { version = "4.5", features = ["derive"] }

# Error handling
thiserror = "2.0"

# Logging
tui-logger = "0.17"
log = "0.4"

[dev-dependencies]
insta = { version = "1.42", features = ["yaml"] }
pretty_assertions = "1.4"
ratatui-testlib = "0.1"
```

> **Note:** Always verify latest versions at [lib.rs](https://lib.rs) or [crates.io](https://crates.io) before starting development. Run `cargo update` periodically to get patch updates.

### 6.3 Application Loop

```
┌─────────────┐
│   Start     │
└──────┬──────┘
       ▼
┌─────────────┐
│ Load Config │
│ Load Data   │
└──────┬──────┘
       ▼
┌─────────────┐     ┌─────────────┐
│   Event     │◄────│  Reminder   │
│   Loop      │     │   Thread    │
└──────┬──────┘     └─────────────┘
       │
   ┌───┴───┐
   ▼       ▼
┌─────┐ ┌─────┐
│Input│ │Timer│
└──┬──┘ └──┬──┘
   │       │
   └───┬───┘
       ▼
┌─────────────┐
│   Update    │
│   State     │
└──────┬──────┘
       ▼
┌─────────────┐
│   Render    │
│   UI        │
└──────┬──────┘
       │
       ▼
   [Continue]
```

### 6.4 State Management

The application uses a central `App` struct to manage state:

```rust
pub struct App {
    pub tasks: Vec<Task>,
    pub projects: Vec<Project>,
    pub config: Config,
    pub current_view: View,
    pub selected_task: Option<usize>,
    pub selected_project: Option<String>,
    pub filter: Filter,
    pub sort: SortOrder,
    pub input_mode: InputMode,
    pub should_quit: bool,
}
```

### 6.5 Logging

Ratado uses `tui-logger` for in-app debugging with a dedicated log viewer widget.

**Initialization:**

```rust
use log::LevelFilter;

fn init_logging() {
    tui_logger::init_logger(LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(LevelFilter::Debug);

    // Optional: filter from RUST_LOG environment variable
    tui_logger::set_env_filter_from_env(Some("RATADO_LOG"));
}
```

**Usage throughout the codebase:**

```rust
use log::{info, debug, warn, error};

// In handlers
info!("Task created: {}", task.title);
debug!("Database query took {:?}", duration);
warn!("Task {} is overdue", task.id);
error!("Failed to save task: {}", err);
```

**Debug View Widget (F12):**

```rust
use tui_logger::{TuiLoggerWidget, TuiWidgetState};

// In App struct
pub struct App {
    // ...
    pub log_state: TuiWidgetState,
}

// Rendering the debug view
fn render_debug_view(f: &mut Frame, app: &App, area: Rect) {
    let widget = TuiLoggerWidget::default()
        .block(Block::bordered().title("Debug Logs"))
        .state(&app.log_state);
    f.render_widget(widget, area);
}
```

**Log Widget Keybindings (when in debug view):**

| Key | Action |
|-----|--------|
| `h` | Toggle target selector |
| `f` | Focus selected target only |
| `↑/↓` | Navigate log targets |
| `←/→` | Adjust display level |
| `PageUp/Down` | Scroll log history |
| `Space` | Toggle inactive targets |
| `Esc` | Exit debug view |

---

## 7. Command-Line Interface

### 7.1 Commands

```bash
# Launch interactive TUI
ratado

# Quick add task from command line
ratado add "Buy milk" --due tomorrow --priority high

# List tasks (non-interactive)
ratado list
ratado list --today
ratado list --project work

# Complete a task
ratado complete <task-id>

# Export/Import
ratado export --format json > backup.json
ratado import backup.json
```

### 7.2 Arguments

| Flag | Description |
|------|-------------|
| `--config <path>` | Use custom config file |
| `--no-notifications` | Disable desktop notifications |
| `--version` | Show version |
| `--help` | Show help |

---

## 8. Future Considerations

These features are out of scope for the initial version but may be considered later:

- **Recurring Tasks**: Support for repeating tasks
- **Subtasks**: Hierarchical task structure
- **Time Tracking**: Track time spent on tasks
- **Sync**: Optional cloud synchronization
- **Plugins**: Extension system for custom functionality
- **Multiple Lists**: Separate todo lists (work, personal)
- **Natural Language Input**: "Buy milk tomorrow at 5pm"
- **Undo/Redo**: Command history with reversal

---

## 9. Success Criteria

### 9.1 Minimum Viable Product (MVP)

- [ ] Create, read, update, delete tasks
- [ ] Organize tasks by project
- [ ] Filter and sort task list
- [ ] Persist data to local storage
- [ ] Basic reminder notifications
- [ ] Vim-style keyboard navigation
- [ ] Responsive terminal UI

### 9.2 Performance Targets

| Metric | Target |
|--------|--------|
| Startup time | < 100ms |
| Input latency | < 16ms |
| Memory usage | < 50MB |
| Binary size | < 10MB |

---

## 10. Appendix

### 10.1 Glossary

| Term | Definition |
|------|------------|
| Task | A single actionable item |
| Project | A collection of related tasks |
| Tag | A label for cross-cutting categorization |
| Archive | Storage for completed tasks |
| Filter | Criteria for displaying subset of tasks |

### 10.2 References

- [Turso Database Documentation](https://docs.turso.tech/introduction)
- [Turso Database GitHub](https://github.com/tursodatabase/turso)
- [Ratatui Documentation](https://ratatui.rs/)
- [Crossterm Documentation](https://docs.rs/crossterm/)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
