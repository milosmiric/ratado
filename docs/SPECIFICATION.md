# Ratado - Terminal Task Manager Specification

## 1. Project Overview

### 1.1 Introduction

Ratado is a terminal-based task manager built with Rust and the Ratatui framework. The name combines "Rata" (from Ratatui) with "do" (from todo), reflecting its technical foundation and purpose.

### 1.2 Goals

- **Fast and Lightweight**: Instant startup, minimal resource usage
- **Keyboard-Driven**: Full functionality accessible without a mouse
- **Offline-First**: All data stored locally, no network dependency
- **Vim-Inspired Navigation**: Familiar keybindings for terminal users

### 1.3 Target Users

- Developers and terminal power users
- Users who prefer keyboard-driven workflows
- Anyone seeking a distraction-free task management solution

---

## 2. Core Features

### 2.1 Task Management

| Feature | Description | Status |
|---------|-------------|--------|
| Create Task | Add new tasks with title, description, due date, priority | ✓ Implemented |
| Edit Task | Modify any task attribute | ✓ Implemented |
| Delete Task | Remove tasks with confirmation | ✓ Implemented |
| Complete Task | Mark tasks as done (with timestamp) | ✓ Implemented |
| Reopen Task | Mark completed tasks as pending | ✓ Implemented |
| Move Task | Move task to different project | ✓ Implemented |

### 2.2 Task Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | UUID v7 | Auto | Unique identifier (time-ordered) |
| `title` | String | Yes | Task name |
| `description` | String | No | Detailed notes with link detection |
| `due_date` | DateTime | No | When task is due (UTC) |
| `priority` | Enum | Yes | Low, Medium, High, Urgent |
| `status` | Enum | Yes | Pending, InProgress, Completed, Archived |
| `project_id` | UUID | No | Associated project |
| `tags` | Vec<String> | No | Categorization labels |
| `created_at` | DateTime | Auto | Creation timestamp |
| `updated_at` | DateTime | Auto | Last modification timestamp |
| `completed_at` | DateTime | Auto | Completion timestamp (set when completed) |

### 2.3 Task Organization

- **Projects**: Group related tasks under named projects with custom colors and icons
- **Tags**: Flexible labeling system for cross-project categorization with autocomplete
- **Filters**: View tasks by status, priority, due date, project, or tag
- **Sorting**: Order by due date, priority, creation date, or alphabetically

### 2.4 Project Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | UUID v7 | Auto | Unique identifier |
| `name` | String | Yes | Project name |
| `color` | String | Yes | Hex color code (default: #3498db) |
| `icon` | String | Yes | Emoji icon (default: 📁) |
| `created_at` | DateTime | Auto | Creation timestamp |

### 2.5 Default Project

- **Inbox**: A default project that cannot be deleted, used for unassigned tasks

---

## 3. User Interface

### 3.1 Layout

```
Ratado v0.1.0   [Overdue: 2] [Due Today: 3]  12 tasks total
──────────────────────────────────────────────────────────────────────
PROJECTS          │ TASKS  [Pending]  [Due Date ↑]
                  │
▸ All Tasks (12)  │ ▶ [ ] !! Fix production bug                  Yesterday
  Inbox (2)       │   [ ] !! Review pull request #42             Yesterday
  Work (6)        │   [ ]  ! Deploy v2.0 release                     Today
  Personal (4)    │   [ ]    Update API documentation                Today
                  │   [ ]    Team standup meeting                    Today
                  │   [ ]    Write unit tests              @Work  #backend
                  │   [ ]    Refactor auth module          @Work  #backend
                  │   [ ]  ↓ Clean up old branches                  Friday
                  │   [ ]    Buy groceries                @Personal #home
                  │   [ ]    Schedule dentist appointment       @Personal
                  │   [ ]    Read "Clean Code" chapter 5  @Personal #books
                  │   [ ]  ↓ Organize desk                  @Inbox  #home
                  │
──────────────────────────────────────────────────────────────────────
a Add  e Edit  Space Done  / Search  c Calendar  f Filter  ? Help
```

### 3.2 Views

| View | Description | Access |
|------|-------------|--------|
| Main | Split view with sidebar and task list | Default |
| Task Detail | Full-screen single task display | `Enter` on task |
| Calendar | Weekly calendar with tasks by due date | `c` |
| Search | Full-text search with results | `/` |
| Help | Keybindings reference | `?` |
| Settings | Data management options | `S` |
| Debug Logs | tui-logger debug view | `F12` |

### 3.3 Panels

**Sidebar Panel:**
- Projects section with task counts
- Tags section with task counts
- Tab to switch between Projects/Tags
- "All Tasks" pseudo-project at top

**Task List Panel:**
- Displays filtered and sorted tasks
- Shows: checkbox, priority indicator, title, project, tags, date
- Completed tasks show both due date and completion date
- Color-coded by priority and due status

**Status Bar:**
- Task statistics (pending, completed counts)
- Current filter indicator

### 3.4 Visual Indicators

**Task Status:**
- `[ ]` - Pending
- `[▸]` - In Progress
- `[✓]` - Completed/Archived

**Priority:**
- `!!` (Red, Bold) - Urgent
- ` !` (Yellow) - High
- `  ` (Default) - Medium
- ` ↓` (Gray) - Low

**Due Date Colors:**
- Red - Overdue
- Yellow - Due today
- Cyan - Due this week
- Gray - Future/Completed

---

## 4. Keyboard Interface

### 4.1 Input Modes

| Mode | Description |
|------|-------------|
| Normal | Navigate and execute commands |
| Editing | Text input in dialogs |
| Search | Search input with result navigation |

### 4.2 Global Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+c` | Force quit |
| `F12` | Toggle debug logs |

### 4.3 Normal Mode - Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g` / `Home` | Jump to top |
| `G` / `End` | Jump to bottom |
| `Ctrl+d` | Page down (10 items) |
| `Ctrl+u` | Page up (10 items) |
| `Tab` | Switch panel (Sidebar ↔ Task List) |
| `h` / `←` | Focus sidebar |
| `l` / `→` | Focus task list |

### 4.4 Normal Mode - Actions

| Key | Context | Action |
|-----|---------|--------|
| `a` | Task List | Add new task |
| `a` | Sidebar | Add new project |
| `e` / `Enter` | Task List | Edit selected task |
| `e` / `Enter` | Sidebar | Edit selected project |
| `d` | Task List | Delete selected task |
| `d` | Sidebar | Delete selected project |
| `Space` | Task List | Toggle task completion |
| `p` | Task List | Cycle priority |
| `t` | Task List | Edit tags |
| `m` | Task List | Move to project |

### 4.5 Normal Mode - Views & Filters

| Key | Action |
|-----|--------|
| `?` | Show help |
| `/` | Open search |
| `c` | Open calendar |
| `S` | Open settings |
| `f` | Open filter/sort dialog |
| `T` | Filter: Due today |
| `W` | Filter: Due this week |
| `1` | Filter: Low priority |
| `2` | Filter: Medium priority |
| `3` | Filter: High priority |
| `4` | Filter: Urgent priority |
| `r` | Refresh data |
| `q` | Quit |

### 4.6 Editing Mode

| Key | Action |
|-----|--------|
| `Esc` | Cancel input |
| `Enter` | Submit input |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character at cursor |
| `←` / `→` | Move cursor |
| `Home` | Move to start |
| `End` | Move to end |
| `Ctrl+a` | Move to start |
| `Ctrl+e` | Move to end |

### 4.7 Search Mode

| Key | Action |
|-----|--------|
| `Esc` | Cancel search |
| `Enter` | Select result |
| `↑` / `↓` | Navigate results |
| `Ctrl+p` | Previous result |
| `Ctrl+n` | Next result |
| Text input | Filter results |

### 4.8 Calendar View

| Key | Action |
|-----|--------|
| `Esc` | Return to main |
| `h` / `←` | Previous day |
| `l` / `→` | Next day |
| `k` / `↑` | Previous week |
| `j` / `↓` | Next week |
| `t` | Jump to today |
| `Enter` | Select day |
| `q` | Quit |

### 4.9 Task Detail View

| Key | Action |
|-----|--------|
| `Esc` | Return to main |
| `Space` | Toggle completion |
| `p` | Cycle priority |
| `e` / `Enter` | Edit task |
| `d` | Delete task |
| `q` | Quit |

---

## 5. Dialogs

### 5.1 Add/Edit Task Dialog

**Fields:**
- Title (single-line, required)
- Description (multi-line textarea with link detection)
- Due Date (text input with date picker, formats: "today", "tomorrow", "YYYY-MM-DD")
- Priority (cycle through options)
- Project (select from list)
- Tags (autocomplete from existing tags)

**Navigation:**
- `Tab` / `Shift+Tab` - Move between fields
- `Ctrl+Enter` - Save task
- `Esc` - Cancel

### 5.2 Confirmation Dialog

- Yes/No prompt for destructive actions
- `y` / `n` - Quick response
- `←` / `→` - Toggle selection
- `Enter` - Confirm selection
- `Esc` - Cancel

### 5.3 Filter/Sort Dialog

**Sections:**
- Filters (with task counts)
- Sort options

**Navigation:**
- `Tab` - Switch sections
- `j` / `k` - Navigate options
- `Enter` - Apply selection
- `Esc` - Cancel

### 5.4 Project Dialog

**Fields:**
- Name (required)
- Color (hex code)
- Icon (emoji)

### 5.5 Delete Project Dialog

**Options:**
- Move tasks to Inbox
- Delete all tasks
- Cancel

### 5.6 Move to Project Dialog

- List of available projects
- Navigate and select destination

### 5.7 Settings Dialog

**Options:**
- Delete all completed tasks
- Reset database (delete everything)

**Confirmation required for all actions.**

---

## 6. Filtering & Sorting

### 6.1 Filter Types

| Filter | Description |
|--------|-------------|
| All | All tasks regardless of status |
| Pending | Tasks not yet completed (default) |
| In Progress | Tasks currently being worked on |
| Completed | Finished tasks |
| Archived | Archived tasks |
| Due Today | Tasks due today |
| Due This Week | Tasks due within 7 days |
| Overdue | Tasks past due date |
| By Priority | Tasks of specific priority level |
| By Project | Tasks in specific project |
| By Tag | Tasks with specific tag |

### 6.2 Sort Orders

| Sort | Description |
|------|-------------|
| Due Date ↑ | Earliest due first (nulls last) |
| Due Date ↓ | Latest due first (nulls first) |
| Priority ↓ | Urgent first |
| Priority ↑ | Low first |
| Created ↓ | Newest first |
| Created ↑ | Oldest first |
| Alphabetical | A-Z by title |

---

## 7. Data Storage

### 7.1 Database

- **Type**: Turso (SQLite-compatible, pure Rust, async)
- **Location**: `~/.config/ratado/ratado.db`
- **Migrations**: Auto-run on startup

### 7.2 Schema

**tasks table:**
```sql
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    due_date TEXT,
    priority TEXT NOT NULL,
    status TEXT NOT NULL,
    project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT
);
```

**projects table:**
```sql
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    icon TEXT NOT NULL,
    created_at TEXT NOT NULL
);
```

**tags table:**
```sql
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);
```

**task_tags table:**
```sql
CREATE TABLE task_tags (
    task_id TEXT REFERENCES tasks(id) ON DELETE CASCADE,
    tag_id TEXT REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, tag_id)
);
```

### 7.3 Data Integrity

- Foreign key constraints enabled
- Orphaned tag cleanup on task deletion/update
- Cascade delete for task-tag associations

---

## 8. Technical Architecture

### 8.1 Project Structure

```
src/
├── main.rs              # Entry point, event loop
├── app.rs               # Central App state struct
├── lib.rs               # Library exports
├── ui/                  # UI components
│   ├── mod.rs           # Main draw function
│   ├── layout.rs        # Main view layout
│   ├── sidebar.rs       # Projects/Tags panel
│   ├── task_list.rs     # Task list rendering
│   ├── task_detail.rs   # Task detail view
│   ├── calendar.rs      # Weekly calendar view
│   ├── search.rs        # Search view
│   ├── help.rs          # Help screen
│   ├── input.rs         # Text input widget
│   ├── tag_input.rs     # Tag input with autocomplete
│   ├── date_picker.rs   # Calendar date picker
│   ├── description_textarea.rs  # Multi-line text input
│   └── dialogs/         # Dialog components
│       ├── mod.rs
│       ├── add_task.rs
│       ├── confirm.rs
│       ├── filter_sort.rs
│       ├── project.rs
│       ├── delete_project.rs
│       ├── move_to_project.rs
│       └── settings.rs
├── models/              # Data models
│   ├── mod.rs
│   ├── task.rs          # Task struct
│   ├── project.rs       # Project struct
│   └── filter.rs        # Filter & SortOrder enums
├── handlers/            # Event handling
│   ├── mod.rs           # Main event handler
│   ├── commands.rs      # Command enum & execution
│   ├── events.rs        # Event types
│   └── input.rs         # Key-to-command mapping
├── storage/             # Database layer
│   ├── mod.rs
│   ├── database.rs      # Connection management
│   ├── migrations.rs    # Schema migrations
│   ├── tasks.rs         # Task CRUD
│   ├── projects.rs      # Project CRUD
│   └── tags.rs          # Tag CRUD
└── utils/               # Utilities
    ├── mod.rs
    ├── datetime.rs      # Date formatting
    └── ids.rs           # UUID generation
```

### 8.2 Application State

Central `App` struct manages:
- Database connection
- Loaded tasks, projects, tags
- Current view and input mode
- Focus panel (Sidebar/TaskList)
- Selection indices
- Filter and sort preferences
- Input buffer and cursor
- Active dialog
- Status messages

### 8.3 Event Loop

```
loop {
    1. Render UI based on App state
    2. Wait for event (keyboard, tick, resize)
    3. Handle event → update App state
    4. Check quit flag
}
```

**Tick Rate**: 250ms

### 8.4 Command Pattern

1. Key event received
2. Map to Command based on context (view, mode, focus)
3. Execute Command (modifies App state)
4. Return continue/quit signal

---

## 9. Dependencies

| Crate | Purpose |
|-------|---------|
| ratatui | Terminal UI framework |
| crossterm | Terminal manipulation |
| turso | SQLite database |
| tokio | Async runtime |
| chrono | Date/time handling |
| uuid | Unique identifiers |
| serde | Serialization |
| thiserror | Error handling |
| log | Logging facade |
| tui-logger | In-app log viewer |
| directories | Config directory resolution |

---

## 10. Future Considerations

The following features are **not currently implemented** but could be added:

- Notifications/reminders system
- Configuration file for settings
- Custom keybinding configuration
- Themes and color customization
- Recurring tasks
- Task dependencies
- Cloud sync
- Export/import functionality
- Mouse support

---

## 11. Version History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2025 | Initial release |
