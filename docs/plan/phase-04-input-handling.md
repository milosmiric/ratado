# Phase 4: Input Handling

**Goal:** Implement the event system, command pattern, and keybindings.

**Prerequisites:** Phase 3 (UI must be rendering)

**Outcome:** All keyboard navigation works, commands execute, app responds to input.

---

## Story 4.1: Event System

**Priority:** Critical
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Set up the event handling infrastructure with support for keys, ticks, and resize.

### Tasks

- [ ] Create `src/handlers/events.rs`:
  - `AppEvent` enum:
    - `Key(KeyEvent)`
    - `Mouse(MouseEvent)` (optional)
    - `Tick`
    - `Resize(u16, u16)`
  - `EventHandler` struct with channel-based event stream
  - Spawn background task to poll crossterm events
  - Generate tick events at regular intervals

- [ ] Integrate with main loop

### Code Sketch

```rust
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    Resize(u16, u16),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
    _tx: mpsc::UnboundedSender<AppEvent>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let _tx = tx.clone();

        tokio::spawn(async move {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate.saturating_sub(last_tick.elapsed());

                if event::poll(timeout).unwrap() {
                    match event::read().unwrap() {
                        CrosstermEvent::Key(key) => {
                            tx.send(AppEvent::Key(key)).unwrap();
                        }
                        CrosstermEvent::Mouse(mouse) => {
                            tx.send(AppEvent::Mouse(mouse)).unwrap();
                        }
                        CrosstermEvent::Resize(w, h) => {
                            tx.send(AppEvent::Resize(w, h)).unwrap();
                        }
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    tx.send(AppEvent::Tick).unwrap();
                    last_tick = Instant::now();
                }
            }
        });

        Self { rx, _tx }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
```

### Acceptance Criteria

- [ ] Key events captured correctly
- [ ] Tick events fire at regular intervals (~250ms)
- [ ] Resize events handled
- [ ] No blocking on input
- [ ] Clean shutdown when app quits

---

## Story 4.2: Command Pattern

**Priority:** Critical
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Implement the command pattern for decoupling input from actions.

### Tasks

- [ ] Create `src/handlers/commands.rs`:
  - `Command` enum with all possible actions
  - `Command::execute(&self, app: &mut App) -> Result<bool>` (returns should_continue)

- [ ] Define all commands:
  ```rust
  pub enum Command {
      // Navigation
      NavigateUp,
      NavigateDown,
      NavigateTop,
      NavigateBottom,
      PageUp,
      PageDown,
      SwitchPanel,

      // Task Actions
      AddTask,
      EditTask,
      DeleteTask,
      ToggleTaskStatus,
      CyclePriority,
      MoveToProject,
      EditTags,

      // Views
      ShowMain,
      ShowHelp,
      ShowCalendar,
      ShowSearch,
      ShowDebugLogs,
      ShowTaskDetail,

      // Filters
      SetFilter(Filter),
      SetSort(SortOrder),
      ClearFilter,
      FilterToday,
      FilterThisWeek,
      FilterByPriority(Priority),

      // Input Mode
      EnterEditMode,
      ExitEditMode,
      SubmitInput,
      CancelInput,

      // App
      Quit,
      Refresh,
      ForceQuit,
  }
  ```

- [ ] Implement `execute` for each command
- [ ] Add unit tests for all commands

### Code Sketch

```rust
impl Command {
    pub async fn execute(self, app: &mut App) -> Result<bool, AppError> {
        match self {
            // Navigation
            Command::NavigateUp => {
                if let Some(idx) = app.selected_task_index {
                    if idx > 0 {
                        app.selected_task_index = Some(idx - 1);
                    }
                }
                Ok(true)
            }
            Command::NavigateDown => {
                let max = app.visible_tasks().len().saturating_sub(1);
                if let Some(idx) = app.selected_task_index {
                    if idx < max {
                        app.selected_task_index = Some(idx + 1);
                    }
                } else if max > 0 {
                    app.selected_task_index = Some(0);
                }
                Ok(true)
            }
            Command::SwitchPanel => {
                app.focus = match app.focus {
                    FocusPanel::Sidebar => FocusPanel::TaskList,
                    FocusPanel::TaskList => FocusPanel::Sidebar,
                };
                Ok(true)
            }

            // Task Actions
            Command::ToggleTaskStatus => {
                if let Some(task) = app.selected_task_mut() {
                    if task.status == TaskStatus::Completed {
                        task.reopen();
                    } else {
                        task.complete();
                    }
                    app.db.update_task(task).await?;
                }
                Ok(true)
            }

            // Views
            Command::ShowHelp => {
                app.current_view = View::Help;
                Ok(true)
            }
            Command::ShowDebugLogs => {
                app.current_view = View::DebugLogs;
                Ok(true)
            }

            // App
            Command::Quit => {
                app.should_quit = true;
                Ok(false)
            }
            Command::Refresh => {
                app.load_data().await?;
                Ok(true)
            }

            _ => Ok(true),
        }
    }
}
```

### Test Cases

```rust
#[tokio::test]
async fn test_navigate_down() {
    let mut app = create_test_app().await;
    app.tasks = vec![Task::new("A"), Task::new("B"), Task::new("C")];
    app.selected_task_index = Some(0);

    Command::NavigateDown.execute(&mut app).await.unwrap();
    assert_eq!(app.selected_task_index, Some(1));
}

#[tokio::test]
async fn test_toggle_task_status() {
    let mut app = create_test_app().await;
    let task = Task::new("Test");
    app.tasks.push(task);
    app.selected_task_index = Some(0);

    assert_eq!(app.tasks[0].status, TaskStatus::Pending);
    Command::ToggleTaskStatus.execute(&mut app).await.unwrap();
    assert_eq!(app.tasks[0].status, TaskStatus::Completed);
}

#[tokio::test]
async fn test_quit_command() {
    let mut app = create_test_app().await;
    assert!(!app.should_quit);

    let result = Command::Quit.execute(&mut app).await.unwrap();
    assert!(!result); // returns false to stop loop
    assert!(app.should_quit);
}
```

### Acceptance Criteria

- [ ] All commands implemented
- [ ] Commands modify app state correctly
- [ ] Database updates happen where needed
- [ ] Unit tests pass for all commands

---

## Story 4.3: Key Mapping

**Priority:** Critical
**Estimate:** Small
**Status:** [ ] Not Started

### Description

Map keyboard input to commands based on current mode and view.

### Tasks

- [ ] Create `src/handlers/input.rs`:
  - `map_key_to_command(key: KeyEvent, app: &App) -> Option<Command>`
  - Handle different modes: Normal, Editing, Search
  - Handle different views: Main, Help, DebugLogs, etc.

- [ ] Implement all keybindings from specification:

  **Normal Mode (Main View):**
  | Key | Command |
  |-----|---------|
  | `q` | Quit |
  | `?` | ShowHelp |
  | `/` | ShowSearch |
  | `j` / `↓` | NavigateDown |
  | `k` / `↑` | NavigateUp |
  | `g` / `Home` | NavigateTop |
  | `G` / `End` | NavigateBottom |
  | `Ctrl+d` | PageDown |
  | `Ctrl+u` | PageUp |
  | `Tab` | SwitchPanel |
  | `h` / `←` | (sidebar collapse or panel switch) |
  | `l` / `→` | (expand or panel switch) |
  | `a` | AddTask |
  | `e` / `Enter` | EditTask or ShowTaskDetail |
  | `d` | DeleteTask |
  | `Space` | ToggleTaskStatus |
  | `p` | CyclePriority |
  | `t` | EditTags |
  | `m` | MoveToProject |
  | `c` | ShowCalendar |
  | `F12` | ShowDebugLogs |
  | `T` | FilterToday |
  | `W` | FilterThisWeek |
  | `1-4` | FilterByPriority |
  | `r` | Refresh |

  **Help View:**
  | Key | Command |
  |-----|---------|
  | Any key | ShowMain |
  | `Esc` | ShowMain |

  **Debug Logs View:**
  | Key | Command |
  |-----|---------|
  | `Esc` / `F12` | ShowMain |
  | (tui-logger keys handled separately) |

- [ ] Add unit tests for all mappings

### Code Sketch

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn map_key_to_command(key: KeyEvent, app: &App) -> Option<Command> {
    // Handle view-specific bindings first
    match app.current_view {
        View::Help => return Some(Command::ShowMain),
        View::DebugLogs => {
            if key.code == KeyCode::Esc || key.code == KeyCode::F(12) {
                return Some(Command::ShowMain);
            }
            // Let tui-logger handle other keys
            return None;
        }
        _ => {}
    }

    // Handle mode-specific bindings
    match app.input_mode {
        InputMode::Editing => map_editing_key(key),
        InputMode::Search => map_search_key(key),
        InputMode::Normal => map_normal_key(key, app),
    }
}

fn map_normal_key(key: KeyEvent, app: &App) -> Option<Command> {
    match key.code {
        // Quit
        KeyCode::Char('q') => Some(Command::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Command::ForceQuit)
        }

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => Some(Command::NavigateDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Command::NavigateUp),
        KeyCode::Char('g') | KeyCode::Home => Some(Command::NavigateTop),
        KeyCode::Char('G') | KeyCode::End => Some(Command::NavigateBottom),
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Command::PageDown)
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Command::PageUp)
        }
        KeyCode::Tab => Some(Command::SwitchPanel),

        // Task actions
        KeyCode::Char('a') => Some(Command::AddTask),
        KeyCode::Char('e') | KeyCode::Enter => Some(Command::EditTask),
        KeyCode::Char('d') => Some(Command::DeleteTask),
        KeyCode::Char(' ') => Some(Command::ToggleTaskStatus),
        KeyCode::Char('p') => Some(Command::CyclePriority),

        // Views
        KeyCode::Char('?') => Some(Command::ShowHelp),
        KeyCode::Char('/') => Some(Command::ShowSearch),
        KeyCode::Char('c') => Some(Command::ShowCalendar),
        KeyCode::F(12) => Some(Command::ShowDebugLogs),

        // Filters
        KeyCode::Char('T') => Some(Command::FilterToday),
        KeyCode::Char('W') => Some(Command::FilterThisWeek),
        KeyCode::Char('1') => Some(Command::FilterByPriority(Priority::Low)),
        KeyCode::Char('2') => Some(Command::FilterByPriority(Priority::Medium)),
        KeyCode::Char('3') => Some(Command::FilterByPriority(Priority::High)),
        KeyCode::Char('4') => Some(Command::FilterByPriority(Priority::Urgent)),

        // Other
        KeyCode::Char('r') => Some(Command::Refresh),
        KeyCode::Esc => Some(Command::ClearFilter),

        _ => None,
    }
}
```

### Test Cases

```rust
#[test]
fn test_quit_keybinding() {
    let app = App::default();
    let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
    let cmd = map_key_to_command(key, &app);
    assert!(matches!(cmd, Some(Command::Quit)));
}

#[test]
fn test_vim_navigation_j() {
    let app = App::default();
    let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
    let cmd = map_key_to_command(key, &app);
    assert!(matches!(cmd, Some(Command::NavigateDown)));
}

#[test]
fn test_help_view_any_key_closes() {
    let mut app = App::default();
    app.current_view = View::Help;
    let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
    let cmd = map_key_to_command(key, &app);
    assert!(matches!(cmd, Some(Command::ShowMain)));
}
```

### Acceptance Criteria

- [ ] All keybindings from spec implemented
- [ ] Mode-specific bindings work correctly
- [ ] View-specific bindings work correctly
- [ ] Vim navigation (j/k/h/l) works
- [ ] Arrow keys work as alternatives
- [ ] Unit tests pass

---

## Story 4.4: Input Handler Integration

**Priority:** High
**Estimate:** Small
**Status:** [ ] Not Started

### Description

Wire up event handling in the main loop.

### Tasks

- [ ] Create `src/handlers/mod.rs` orchestration:
  - `handle_event(app: &mut App, event: AppEvent) -> Result<bool>`
  - Route to appropriate handler based on event type
  - For Key events: map to command and execute
  - For Tick events: call `app.on_tick()`
  - For Resize events: (ratatui handles automatically)

- [ ] Update main loop to use event handler
- [ ] Handle tui-logger events for debug view
- [ ] Ensure quit command exits cleanly

### Code Sketch

```rust
// src/handlers/mod.rs
pub mod commands;
pub mod events;
pub mod input;

use crate::app::App;
use events::AppEvent;

pub async fn handle_event(app: &mut App, event: AppEvent) -> Result<bool, AppError> {
    match event {
        AppEvent::Key(key) => {
            // Special handling for debug view (tui-logger)
            if app.current_view == View::DebugLogs {
                // Convert to tui-logger event
                if let Some(tui_event) = key_to_tui_logger_event(key) {
                    app.log_state.transition(tui_event);
                    return Ok(true);
                }
            }

            // Map key to command and execute
            if let Some(cmd) = input::map_key_to_command(key, app) {
                log::debug!("Executing command: {:?}", cmd);
                return cmd.execute(app).await;
            }
            Ok(true)
        }
        AppEvent::Tick => {
            app.on_tick();
            Ok(true)
        }
        AppEvent::Resize(_, _) => {
            // Ratatui handles resize automatically
            Ok(true)
        }
        AppEvent::Mouse(_) => {
            // Optional: handle mouse events
            Ok(true)
        }
    }
}

// In main.rs
async fn run_app(terminal: &mut Terminal<impl Backend>, app: &mut App) -> Result<()> {
    let mut events = EventHandler::new(Duration::from_millis(250));

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Some(event) = events.next().await {
            if !handlers::handle_event(app, event).await? {
                break;
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
```

### Acceptance Criteria

- [ ] All key presses trigger appropriate actions
- [ ] Navigation (j/k, arrows) works
- [ ] Quit works correctly (q, Ctrl+C)
- [ ] Help view opens and closes
- [ ] Debug logs view opens and closes
- [ ] tui-logger responds to keys in debug view

---

## Phase 4 Checklist

Before moving to Phase 5, ensure:

- [ ] All 4 stories completed
- [ ] All keybindings work as specified
- [ ] Navigation works (up/down/top/bottom)
- [ ] Panel switching works (Tab)
- [ ] Help view toggles with `?`
- [ ] Debug view toggles with `F12`
- [ ] Quit works with `q`
- [ ] All unit tests pass
