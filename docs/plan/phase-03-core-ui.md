# Phase 3: Core UI

**Goal:** Implement the terminal UI infrastructure, main layout, and core widgets.

**Prerequisites:** Phase 2 (storage layer needed to load data)

**Outcome:** App launches, displays main layout with sidebar and task list, can quit cleanly.

---

## Story 3.1: Application State

**Priority:** Critical
**Estimate:** Medium
**Status:** [x] Complete

### Description

Implement the central App struct that manages all application state.

### Tasks

- [ ] Expand `src/app.rs` with full implementation:
  - `View` enum: `Main`, `TaskDetail`, `Calendar`, `Search`, `Help`, `DebugLogs`
  - `InputMode` enum: `Normal`, `Editing`, `Search`
  - `FocusPanel` enum: `Sidebar`, `TaskList`
  - Full `App` struct with fields:
    - `db: Database`
    - `tasks: Vec<Task>`
    - `projects: Vec<Project>`
    - `tags: Vec<Tag>`
    - `config: Config` (placeholder for now)
    - `current_view: View`
    - `input_mode: InputMode`
    - `focus: FocusPanel`
    - `selected_task_index: Option<usize>`
    - `selected_project_index: usize`
    - `filter: Filter`
    - `sort: SortOrder`
    - `input_buffer: String`
    - `log_state: TuiWidgetState`
    - `should_quit: bool`

- [ ] Implement constructor and data loading:
  - `App::new(db: Database) -> Result<App>`
  - `App::load_data(&mut self) -> Result<()>`
  - `App::refresh(&mut self) -> Result<()>`

- [ ] Implement state accessors:
  - `visible_tasks(&self) -> Vec<&Task>` - filtered and sorted
  - `selected_task(&self) -> Option<&Task>`
  - `selected_project(&self) -> Option<&Project>`
  - `task_count_for_project(&self, project_id: &str) -> usize`

### Code Sketch

```rust
use tui_logger::TuiWidgetState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Main,
    TaskDetail,
    Calendar,
    Search,
    Help,
    DebugLogs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    Sidebar,
    TaskList,
}

pub struct App {
    pub db: Database,
    pub tasks: Vec<Task>,
    pub projects: Vec<Project>,
    pub tags: Vec<Tag>,
    pub current_view: View,
    pub input_mode: InputMode,
    pub focus: FocusPanel,
    pub selected_task_index: Option<usize>,
    pub selected_project_index: usize,
    pub filter: Filter,
    pub sort: SortOrder,
    pub input_buffer: String,
    pub log_state: TuiWidgetState,
    pub should_quit: bool,
}

impl App {
    pub async fn new(db: Database) -> Result<Self, AppError> {
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
            filter: Filter::All,
            sort: SortOrder::DueDateAsc,
            input_buffer: String::new(),
            log_state: TuiWidgetState::default(),
            should_quit: false,
        };
        app.load_data().await?;
        Ok(app)
    }

    pub fn visible_tasks(&self) -> Vec<&Task> {
        let mut tasks: Vec<&Task> = self.tasks.iter()
            .filter(|t| self.filter.matches(t))
            .collect();
        self.sort.apply(&mut tasks);
        tasks
    }
}
```

### Acceptance Criteria

- [ ] App struct holds all necessary state
- [ ] State can be initialized from database
- [ ] `visible_tasks()` returns filtered and sorted list
- [ ] Selection tracking works correctly

---

## Story 3.2: Terminal Setup & Main Loop

**Priority:** Critical
**Estimate:** Medium
**Status:** [x] Complete

### Description

Set up terminal, event loop, and rendering infrastructure.

### Tasks

- [ ] Update `src/main.rs`:
  - Initialize tui-logger at startup
  - Set up terminal with crossterm (raw mode, alternate screen)
  - Create main async event loop
  - Handle graceful shutdown (restore terminal)
  - Install panic hook to restore terminal on crash

- [ ] Create basic render loop structure:
  - Clear screen
  - Draw UI
  - Flush

- [ ] Implement tick-based refresh (every 250ms for reminders)

### Code Sketch

```rust
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tui_logger::init_logger(log::LevelFilter::Debug)?;
    tui_logger::set_default_level(log::LevelFilter::Debug);

    // Setup panic hook
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        restore_terminal().unwrap();
        original_hook(panic);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app
    let db_path = Database::default_path()?;
    let db = Database::open(&db_path).await?;
    run_migrations(&db).await?;
    let mut app = App::new(db).await?;

    // Main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    restore_terminal()?;

    result
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    while !app.should_quit {
        // Draw
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle events
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                handlers::handle_key(app, key).await?;
            }
        }

        // Tick
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }

    Ok(())
}
```

### Acceptance Criteria

- [ ] Terminal enters TUI mode on start
- [ ] Terminal restores on quit (`q` or `Ctrl+C`)
- [ ] Terminal restores on panic (no corrupted terminal)
- [ ] Event loop processes input without blocking
- [ ] Logging works (can see logs in debug view later)

---

## Story 3.3: Main Layout

**Priority:** Critical
**Estimate:** Medium
**Status:** [x] Complete

### Description

Implement the main split-panel layout with sidebar and task list.

### Tasks

- [ ] Create `src/ui/mod.rs`:
  - `draw(frame: &mut Frame, app: &App)` - main entry point
  - Route to appropriate view based on `app.current_view`

- [ ] Create `src/ui/layout.rs`:
  - `render_main_view(frame: &mut Frame, app: &App, area: Rect)`
  - Use `Layout` to split:
    - Header (3 lines)
    - Main content (sidebar 25% | task list 75%)
    - Status bar (1 line)

- [ ] Create `src/ui/header.rs`:
  - `render_header(frame: &mut Frame, app: &App, area: Rect)`
  - Show: "Ratado v0.1.0"
  - Show: overdue count badge `[!N]`
  - Show: today's task count `[Today: N]`

- [ ] Create `src/ui/status_bar.rs`:
  - `render_status_bar(frame: &mut Frame, app: &App, area: Rect)`
  - Show keybinding hints
  - Show current filter

### Code Sketch

```rust
// src/ui/mod.rs
pub mod layout;
pub mod header;
pub mod status_bar;
pub mod sidebar;
pub mod task_list;

use ratatui::Frame;
use crate::app::{App, View};

pub fn draw(frame: &mut Frame, app: &App) {
    match app.current_view {
        View::Main => layout::render_main_view(frame, app, frame.area()),
        View::Help => { /* TODO */ }
        View::DebugLogs => { /* TODO */ }
        _ => layout::render_main_view(frame, app, frame.area()),
    }
}

// src/ui/layout.rs
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub fn render_main_view(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(1),  // Status bar
        ])
        .split(area);

    header::render_header(frame, app, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),  // Sidebar
            Constraint::Percentage(75),  // Task list
        ])
        .split(chunks[1]);

    sidebar::render_sidebar(frame, app, main_chunks[0]);
    task_list::render_task_list(frame, app, main_chunks[1]);

    status_bar::render_status_bar(frame, app, chunks[2]);
}
```

### Acceptance Criteria

- [ ] Layout displays header, sidebar, task list, status bar
- [ ] Layout matches UI mockup proportions
- [ ] Responsive to terminal resize
- [ ] Panels resize proportionally

---

## Story 3.4: Sidebar Widget

**Priority:** High
**Estimate:** Medium
**Status:** [x] Complete

### Description

Implement the projects and tags sidebar.

### Tasks

- [ ] Create `src/ui/sidebar.rs`:
  - `render_sidebar(frame: &mut Frame, app: &App, area: Rect)`
  - Split into projects section and tags section
  - `ProjectList` widget:
    - "All Tasks" option at top
    - List of projects with task counts
    - Selection highlighting when sidebar focused
    - Visual indicator `›●` for selected project
  - `TagList` widget:
    - List of tags with task counts
    - Selection highlighting

- [ ] Show task counts per project/tag
- [ ] Highlight focused panel

### Code Sketch

```rust
pub fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),    // Projects
            Constraint::Length(1), // Separator
            Constraint::Min(5),    // Tags
        ])
        .split(area);

    render_projects(frame, app, chunks[0]);
    render_tags(frame, app, chunks[2]);
}

fn render_projects(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("PROJECTS")
        .borders(Borders::RIGHT);

    let items: Vec<ListItem> = std::iter::once(("All", app.tasks.len()))
        .chain(app.projects.iter().map(|p| {
            (p.name.as_str(), app.task_count_for_project(&p.id))
        }))
        .enumerate()
        .map(|(i, (name, count))| {
            let selected = i == app.selected_project_index;
            let prefix = if selected { "›● " } else { "  " };
            let style = if selected && app.focus == FocusPanel::Sidebar {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("{}{} ({})", prefix, name, count)).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
```

### Acceptance Criteria

- [ ] Projects listed with accurate task counts
- [ ] Tags listed with accurate task counts
- [ ] Selection visible and styled correctly
- [ ] "All Tasks" option works
- [ ] Matches UI mockup styling

---

## Story 3.5: Task List Widget

**Priority:** Critical
**Estimate:** Medium
**Status:** [x] Complete

### Description

Implement the main task list display widget.

### Tasks

- [ ] Create `src/ui/task_list.rs`:
  - `render_task_list(frame: &mut Frame, app: &App, area: Rect)`
  - `TaskListWidget` - scrollable list of tasks
  - Task row rendering:
    - Status checkbox: `[ ]` pending, `[▸]` in progress, `[✓]` completed
    - Priority indicator: `!!` urgent, `!` high, ` ` medium/low
    - Title (truncated if needed)
    - Due date (relative format from utils)
    - Project badge (if not in project view)
  - Selection highlighting (blue background)
  - Color coding:
    - Red: overdue
    - Yellow: due today
    - Cyan: due this week
    - Dim/Gray: completed

- [ ] Implement scrolling when list exceeds viewport
- [ ] Handle empty state ("No tasks yet!")
- [ ] Add snapshot tests

### Code Sketch

```rust
pub fn render_task_list(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("TASKS")
        .borders(Borders::NONE);

    let tasks = app.visible_tasks();

    if tasks.is_empty() {
        let empty = Paragraph::new("No tasks yet!\n\nPress [a] to add your first task")
            .alignment(Alignment::Center)
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = tasks.iter()
        .enumerate()
        .map(|(i, task)| {
            let selected = Some(i) == app.selected_task_index;
            render_task_row(task, selected, app.focus == FocusPanel::TaskList)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Blue));

    let mut state = ListState::default();
    state.select(app.selected_task_index);

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_task_row(task: &Task, selected: bool, focused: bool) -> ListItem {
    let checkbox = match task.status {
        TaskStatus::Pending => "[ ]",
        TaskStatus::InProgress => "[▸]",
        TaskStatus::Completed | TaskStatus::Archived => "[✓]",
    };

    let priority = match task.priority {
        Priority::Urgent => "!! ",
        Priority::High => " ! ",
        _ => "   ",
    };

    let due = task.due_date
        .map(|d| format_relative_date(d))
        .unwrap_or_default();

    let style = if task.status == TaskStatus::Completed {
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)
    } else if task.is_overdue() {
        Style::default().fg(Color::Red)
    } else if task.is_due_today() {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let content = format!("{} {}{:<40} {}", checkbox, priority, task.title, due);
    ListItem::new(content).style(style)
}
```

### Test Cases (Snapshots)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_task_list_rendering() {
        let tasks = vec![
            Task { title: "First task".into(), priority: Priority::High, ..Default::default() },
            Task { title: "Second task".into(), ..Default::default() },
        ];
        // Render to buffer and snapshot
        let output = render_to_string(/* ... */);
        assert_snapshot!(output);
    }
}
```

### Acceptance Criteria

- [ ] Tasks display with all visual indicators
- [ ] Scrolling works with many tasks
- [ ] Selection clearly visible (blue highlight)
- [ ] Colors match specification
- [ ] Empty state shown when no tasks
- [ ] Snapshot tests pass

---

## Phase 3 Checklist

Before moving to Phase 4, ensure:

- [x] All 5 stories completed
- [x] App launches and displays UI
- [x] Layout matches mockups
- [x] Tasks load from database and display
- [x] Can quit with `q` (basic quit handler)
- [x] Terminal restores properly on exit
- [x] `cargo test` passes (including snapshots)
