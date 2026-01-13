# Phase 5: Task Operations

**Goal:** Implement all task CRUD operations with dialogs.

**Prerequisites:** Phase 4 (input handling must work)

**Outcome:** Users can add, edit, delete, toggle, and modify tasks.

---

## Story 5.1: Add Task Dialog

**Priority:** Critical
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Implement the task creation dialog/modal.

### Tasks

- [ ] Create `src/ui/input.rs`:
  - `TextInput` widget for text entry
  - Cursor position tracking
  - Basic editing: insert, delete, backspace
  - `TextInput::value()` - get current text
  - `TextInput::handle_key(key)` - process input

- [ ] Create `src/ui/dialogs/mod.rs`:
  - Module for all dialog widgets

- [ ] Create `src/ui/dialogs/add_task.rs`:
  - `AddTaskDialog` struct holding form state:
    - `title: TextInput`
    - `description: TextInput`
    - `due_date: Option<DateTime>`
    - `priority: Priority`
    - `project_id: Option<String>`
    - `focused_field: usize`
  - `render(frame, area)` - draw centered modal
  - `handle_key(key)` - process input for dialog
  - Tab to move between fields
  - Enter to submit
  - Esc to cancel

- [ ] Update App state:
  - `dialog: Option<Dialog>` enum for active dialog
  - Handle dialog input in event handler

- [ ] Wire AddTask command to open dialog

### Code Sketch

```rust
// src/ui/input.rs
pub struct TextInput {
    value: String,
    cursor: usize,
}

impl TextInput {
    pub fn new() -> Self {
        Self { value: String::new(), cursor: 0 }
    }

    pub fn with_value(value: String) -> Self {
        let cursor = value.len();
        Self { value, cursor }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor, c);
                self.cursor += 1;
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.value.remove(self.cursor);
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                }
            }
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                self.cursor = (self.cursor + 1).min(self.value.len());
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.value.len(),
            _ => {}
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, focused: bool) {
        let style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        // Render with cursor indicator
    }
}

// src/ui/dialogs/add_task.rs
pub struct AddTaskDialog {
    pub title: TextInput,
    pub description: TextInput,
    pub due_date: String,  // text input, parse on submit
    pub priority: Priority,
    pub project_id: Option<String>,
    pub focused_field: usize,
}

impl AddTaskDialog {
    pub fn new() -> Self {
        Self {
            title: TextInput::new(),
            description: TextInput::new(),
            due_date: String::new(),
            priority: Priority::Medium,
            project_id: None,
            focused_field: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Cancel,
            KeyCode::Enter if self.focused_field == 4 => DialogAction::Submit,
            KeyCode::Tab => {
                self.focused_field = (self.focused_field + 1) % 5;
                DialogAction::None
            }
            KeyCode::BackTab => {
                self.focused_field = self.focused_field.checked_sub(1).unwrap_or(4);
                DialogAction::None
            }
            _ => {
                match self.focused_field {
                    0 => self.title.handle_key(key),
                    1 => self.description.handle_key(key),
                    2 => { /* due date input */ }
                    3 => { /* priority selector */ }
                    4 => { /* project selector */ }
                    _ => {}
                }
                DialogAction::None
            }
        }
    }

    pub fn to_task(&self) -> Option<Task> {
        if self.title.value().is_empty() {
            return None;
        }
        let mut task = Task::new(self.title.value());
        task.description = if self.description.value().is_empty() {
            None
        } else {
            Some(self.description.value().to_string())
        };
        task.priority = self.priority;
        task.project_id = self.project_id.clone();
        // Parse due_date
        Some(task)
    }
}

pub enum DialogAction {
    None,
    Submit,
    Cancel,
}
```

### Acceptance Criteria

- [ ] Dialog displays centered over main view
- [ ] Title field editable with cursor
- [ ] Tab moves between fields
- [ ] Enter submits (creates task in database)
- [ ] Esc cancels (closes without saving)
- [ ] Task appears in list after creation
- [ ] Priority selector works
- [ ] Project selector shows available projects

---

## Story 5.2: Edit Task

**Priority:** High
**Estimate:** Small
**Status:** [ ] Not Started

### Description

Enable editing existing tasks using the same dialog.

### Tasks

- [ ] Add `AddTaskDialog::from_task(task: &Task)`:
  - Pre-populate all fields from existing task
  - Store original task ID for update

- [ ] Modify AddTask command:
  - If task selected, open in edit mode
  - Otherwise open blank for new task

- [ ] On submit in edit mode:
  - Update existing task instead of insert
  - Update `updated_at` timestamp

### Code Sketch

```rust
impl AddTaskDialog {
    pub fn from_task(task: &Task) -> Self {
        Self {
            title: TextInput::with_value(task.title.clone()),
            description: TextInput::with_value(
                task.description.clone().unwrap_or_default()
            ),
            due_date: task.due_date.map(|d| d.to_string()).unwrap_or_default(),
            priority: task.priority,
            project_id: task.project_id.clone(),
            focused_field: 0,
            editing_task_id: Some(task.id.clone()),  // New field
        }
    }
}

// In command execution
Command::EditTask => {
    if let Some(task) = app.selected_task() {
        app.dialog = Some(Dialog::AddTask(AddTaskDialog::from_task(task)));
    }
    Ok(true)
}
```

### Acceptance Criteria

- [ ] `e` on selected task opens edit dialog
- [ ] All fields pre-populated with current values
- [ ] Changes persist to database on submit
- [ ] `updated_at` timestamp updated
- [ ] UI updates after edit

---

## Story 5.3: Delete Task

**Priority:** High
**Estimate:** Small
**Status:** [ ] Not Started

### Description

Implement task deletion with confirmation dialog.

### Tasks

- [ ] Create `src/ui/dialogs/confirm.rs`:
  - `ConfirmDialog` struct:
    - `title: String`
    - `message: String`
    - `confirm_text: String` (default "Yes")
    - `cancel_text: String` (default "No")
    - `selected: bool` (which button focused)
  - Render centered modal with buttons
  - `y` or Enter on Yes = confirm
  - `n` or Enter on No or Esc = cancel

- [ ] Wire DeleteTask command:
  - Open confirm dialog with task title
  - On confirm: delete from database
  - On cancel: close dialog

- [ ] Update selection after delete:
  - Move to next task, or previous if at end

### Code Sketch

```rust
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub on_confirm: Box<dyn FnOnce(&mut App) -> Result<()>>,
    pub selected_yes: bool,
}

impl ConfirmDialog {
    pub fn delete_task(task: &Task) -> Self {
        let task_id = task.id.clone();
        Self {
            title: "Delete Task?".to_string(),
            message: format!("\"{}\"\\n\\nThis action cannot be undone.", task.title),
            on_confirm: Box::new(move |app| {
                app.db.delete_task(&task_id).await?;
                app.load_data().await?;
                Ok(())
            }),
            selected_yes: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Char('y') => DialogAction::Confirm,
            KeyCode::Char('n') | KeyCode::Esc => DialogAction::Cancel,
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                self.selected_yes = !self.selected_yes;
                DialogAction::None
            }
            KeyCode::Enter => {
                if self.selected_yes {
                    DialogAction::Confirm
                } else {
                    DialogAction::Cancel
                }
            }
            _ => DialogAction::None,
        }
    }
}
```

### Acceptance Criteria

- [ ] `d` on selected task opens confirmation
- [ ] Confirmation shows task title
- [ ] `y` or selecting Yes deletes task
- [ ] `n` or Esc cancels
- [ ] Task removed from database
- [ ] UI updates (list refreshes)
- [ ] Selection moves to adjacent task

---

## Story 5.4: Toggle Task Status

**Priority:** Critical
**Estimate:** Small
**Status:** [ ] Not Started

### Description

Implement quick status toggle with spacebar.

### Tasks

- [ ] Implement ToggleTaskStatus command fully:
  - Pending → Completed (set `completed_at = now()`)
  - InProgress → Completed
  - Completed → Pending (clear `completed_at`)
  - Archived tasks: no toggle (or toggle to Completed)

- [ ] Update database immediately
- [ ] Log the action
- [ ] Visual feedback (task row updates)

### Code Sketch

```rust
Command::ToggleTaskStatus => {
    if let Some(idx) = app.selected_task_index {
        let visible = app.visible_tasks();
        if let Some(&task) = visible.get(idx) {
            // Find the actual task in app.tasks
            if let Some(task) = app.tasks.iter_mut().find(|t| t.id == task.id) {
                match task.status {
                    TaskStatus::Completed => {
                        task.status = TaskStatus::Pending;
                        task.completed_at = None;
                        log::info!("Task reopened: {}", task.title);
                    }
                    _ => {
                        task.status = TaskStatus::Completed;
                        task.completed_at = Some(Utc::now());
                        log::info!("Task completed: {}", task.title);
                    }
                }
                task.updated_at = Utc::now();
                app.db.update_task(task).await?;
            }
        }
    }
    Ok(true)
}
```

### Acceptance Criteria

- [ ] Spacebar toggles between Pending ↔ Completed
- [ ] `completed_at` timestamp set/cleared appropriately
- [ ] Database updated immediately
- [ ] UI reflects change (checkbox, styling)
- [ ] Works on any visible task

---

## Story 5.5: Cycle Priority

**Priority:** Medium
**Estimate:** Small
**Status:** [ ] Not Started

### Description

Implement priority cycling with `p` key.

### Tasks

- [ ] Implement CyclePriority command:
  - Low → Medium → High → Urgent → Low (cycle)
  - Update database
  - Log the change

- [ ] Visual feedback:
  - Priority indicator updates immediately
  - Color may change based on priority

### Code Sketch

```rust
impl Priority {
    pub fn cycle(&self) -> Priority {
        match self {
            Priority::Low => Priority::Medium,
            Priority::Medium => Priority::High,
            Priority::High => Priority::Urgent,
            Priority::Urgent => Priority::Low,
        }
    }
}

Command::CyclePriority => {
    if let Some(idx) = app.selected_task_index {
        let task_id = app.visible_tasks().get(idx).map(|t| t.id.clone());
        if let Some(id) = task_id {
            if let Some(task) = app.tasks.iter_mut().find(|t| t.id == id) {
                task.priority = task.priority.cycle();
                task.updated_at = Utc::now();
                log::debug!("Priority changed to {:?}: {}", task.priority, task.title);
                app.db.update_task(task).await?;
            }
        }
    }
    Ok(true)
}
```

### Acceptance Criteria

- [ ] `p` cycles through all four priorities
- [ ] Visual indicator updates (`!!`, `!`, or blank)
- [ ] Database updated
- [ ] If sorted by priority, list may reorder

---

## Phase 5 Checklist

Before moving to Phase 6, ensure:

- [ ] All 5 stories completed
- [ ] Can add new tasks via dialog
- [ ] Can edit existing tasks
- [ ] Can delete tasks with confirmation
- [ ] Spacebar toggles completion
- [ ] `p` cycles priority
- [ ] All changes persist to database
- [ ] UI updates reflect changes
- [ ] All unit/integration tests pass

---

## MVP Milestone

**After completing Phase 5, you have a Minimum Viable Product:**

- Launch TUI application
- View tasks in list
- Navigate with keyboard
- Add new tasks
- Edit tasks
- Delete tasks
- Toggle completion
- Change priority
- Data persists to database

Phases 6-12 add polish and additional features.
