# Phase 8: Additional Views

**Goal:** Implement help overlay, task detail view, and calendar view.

**Prerequisites:** Phase 5 (basic operations)

**Outcome:** Users have access to help, detailed task view, and calendar.

---

## Story 8.1: Help Overlay

**Priority:** High
**Estimate:** Small
**Status:** [x] Complete

### Description

Implement the help overlay showing all keybindings.

### Tasks

- [x] Create `src/ui/help.rs`:
  - Full-screen or large centered overlay
  - Organized keybinding reference
  - Categories: Navigation, Task Actions, Views, Filters
  - Two-column layout for efficiency

- [x] Toggle with `?` key
- [x] Dismiss with any key press

### Code Sketch

```rust
pub fn render_help(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let help_text = vec![
        "",
        "  NAVIGATION                    TASK ACTIONS",
        "  ──────────                    ────────────",
        "  j / ↓      Move down          a          Add task",
        "  k / ↑      Move up            e / Enter  Edit task",
        "  g / Home   First item         d          Delete task",
        "  G / End    Last item          Space      Toggle done",
        "  Tab        Switch panel       p          Cycle priority",
        "  Ctrl+d     Page down          t          Edit tags",
        "  Ctrl+u     Page up            m          Move to project",
        "",
        "  VIEWS                         FILTERS",
        "  ─────                         ───────",
        "  ?          This help          f          Filter menu",
        "  /          Search             s          Sort menu",
        "  c          Calendar           T          Today only",
        "  F12        Debug logs         W          This week",
        "                                1-4        By priority",
        "  GENERAL                       A          Toggle archived",
        "  ───────                       0 / Esc    Clear filter",
        "  q          Quit",
        "  r          Refresh",
        "",
        "                    Press any key to close",
    ];

    let paragraph = Paragraph::new(help_text.join("\n"))
        .block(block)
        .alignment(Alignment::Left);

    // Render centered
    let popup_area = centered_rect(70, 80, area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

// In ui/mod.rs draw function
View::Help => {
    layout::render_main_view(frame, app, frame.area());
    help::render_help(frame, frame.area());
}

// In input handler - any key closes help
if app.current_view == View::Help {
    return Some(Command::ShowMain);
}
```

### Acceptance Criteria

- [x] `?` opens help overlay
- [x] All keybindings listed and organized
- [x] Layout is readable and clear
- [x] Any key press closes help
- [x] Main view visible behind (dimmed optional)

---

## Story 8.2: Task Detail View

**Priority:** Medium
**Estimate:** Medium
**Status:** [x] Complete

### Description

Implement full task detail view for viewing/editing.

### Tasks

- [x] Create `src/ui/task_detail.rs`:
  - Full panel or large view
  - Display all task fields:
    - Title (large)
    - Status with radio-style selector
    - Priority with radio-style selector
    - Due date
    - Project
    - Tags
    - Description (full, scrollable)
    - Created/Updated timestamps
  - Edit capability inline

- [x] Navigation:
  - Enter on task opens detail view
  - Tab to move between fields
  - Enter to edit focused field
  - Esc to return to list

- [x] Actions from detail view:
  - `e` to edit (opens dialog)
  - `d` to delete
  - Space to toggle status
  - `p` to cycle priority

### Code Sketch

```rust
pub struct TaskDetailView {
    pub task: Task,
    pub focused_field: DetailField,
    pub description_scroll: u16,
}

#[derive(Clone, Copy)]
pub enum DetailField {
    Status,
    Priority,
    DueDate,
    Project,
    Tags,
    Description,
}

impl TaskDetailView {
    pub fn new(task: Task) -> Self {
        Self {
            task,
            focused_field: DetailField::Status,
            description_scroll: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> DetailAction {
        match key.code {
            KeyCode::Esc => DetailAction::Close,
            KeyCode::Tab => {
                self.focused_field = self.focused_field.next();
                DetailAction::None
            }
            KeyCode::Char(' ') => {
                // Toggle status
                DetailAction::ToggleStatus
            }
            KeyCode::Char('p') => DetailAction::CyclePriority,
            KeyCode::Char('e') => DetailAction::Edit,
            KeyCode::Char('d') => DetailAction::Delete,
            KeyCode::Down if self.focused_field == DetailField::Description => {
                self.description_scroll += 1;
                DetailAction::None
            }
            KeyCode::Up if self.focused_field == DetailField::Description => {
                self.description_scroll = self.description_scroll.saturating_sub(1);
                DetailAction::None
            }
            _ => DetailAction::None,
        }
    }
}

pub fn render_task_detail(frame: &mut Frame, view: &TaskDetailView, area: Rect) {
    let block = Block::default()
        .title(format!(" {} ", view.task.title))
        .borders(Borders::ALL);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // Status
            Constraint::Length(3),  // Priority
            Constraint::Length(2),  // Due date
            Constraint::Length(2),  // Project
            Constraint::Length(2),  // Tags
            Constraint::Length(1),  // Separator
            Constraint::Min(5),     // Description
            Constraint::Length(2),  // Timestamps
        ])
        .split(block.inner(area));

    frame.render_widget(block, area);

    // Render each field...
    render_status_field(frame, &view.task, chunks[0],
        view.focused_field == DetailField::Status);
    render_priority_field(frame, &view.task, chunks[1],
        view.focused_field == DetailField::Priority);
    // ... etc
}
```

### Acceptance Criteria

- [x] Enter on task opens detail view
- [x] All task fields displayed
- [ ] Can navigate fields with Tab (not implemented - uses existing dialog for editing)
- [x] Status toggle works (Space)
- [x] Priority cycle works (p)
- [x] Description scrollable if long (wrapped text display)
- [x] Esc returns to task list
- [x] Changes persist to database

---

## Story 8.3: Weekly Calendar View (Modified)

**Priority:** Low
**Estimate:** Large
**Status:** [x] Complete

### Description

Implement weekly calendar view with task indicators (modified from original monthly plan).

### Tasks

- [x] Create `src/ui/calendar.rs`:
  - Weekly grid layout (7 day cards)
  - Day headers (Mon-Sun)
  - Current day highlighted (yellow)
  - Selected day highlighted (cyan)
  - Days with tasks show indicators (dots)
  - Overdue days in red

- [x] Navigation:
  - Arrow keys to move selected day (left/right for days, up/down for weeks)
  - `h`/`l` for prev/next day
  - `j`/`k` for prev/next week
  - `t` to jump to today
  - Enter to return to main view
  - Esc to return to main view

- [x] Task list for selected day:
  - Show below week calendar
  - List tasks due on that day with status and priority

- [x] Toggle with `c` key

### Code Sketch

```rust
pub struct CalendarView {
    pub year: i32,
    pub month: u32,
    pub selected_day: u32,
    pub tasks_by_day: HashMap<u32, Vec<Task>>,
}

impl CalendarView {
    pub fn new(tasks: &[Task]) -> Self {
        let today = Local::now().date_naive();
        let mut view = Self {
            year: today.year(),
            month: today.month(),
            selected_day: today.day(),
            tasks_by_day: HashMap::new(),
        };
        view.update_tasks(tasks);
        view
    }

    pub fn update_tasks(&mut self, tasks: &[Task]) {
        self.tasks_by_day.clear();
        for task in tasks {
            if let Some(due) = task.due_date {
                let date = due.date_naive();
                if date.year() == self.year && date.month() == self.month {
                    self.tasks_by_day
                        .entry(date.day())
                        .or_default()
                        .push(task.clone());
                }
            }
        }
    }

    pub fn prev_month(&mut self) {
        if self.month == 1 {
            self.month = 12;
            self.year -= 1;
        } else {
            self.month -= 1;
        }
        self.selected_day = 1;
    }

    pub fn next_month(&mut self) {
        if self.month == 12 {
            self.month = 1;
            self.year += 1;
        } else {
            self.month += 1;
        }
        self.selected_day = 1;
    }
}

pub fn render_calendar(frame: &mut Frame, view: &CalendarView, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Header with month/year
            Constraint::Length(2),   // Day names
            Constraint::Length(12),  // Calendar grid (6 rows x 2 lines)
            Constraint::Min(5),      // Tasks for selected day
        ])
        .split(area);

    // Header
    let header = format!("{} {}", month_name(view.month), view.year);
    let header_widget = Paragraph::new(header)
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(header_widget, chunks[0]);

    // Day names
    let days = "  Mon   Tue   Wed   Thu   Fri   Sat   Sun";
    frame.render_widget(Paragraph::new(days), chunks[1]);

    // Calendar grid
    render_calendar_grid(frame, view, chunks[2]);

    // Tasks for selected day
    if let Some(tasks) = view.tasks_by_day.get(&view.selected_day) {
        render_day_tasks(frame, tasks, view.selected_day, chunks[3]);
    }
}

fn render_calendar_grid(frame: &mut Frame, view: &CalendarView, area: Rect) {
    let first_day = NaiveDate::from_ymd_opt(view.year, view.month, 1).unwrap();
    let days_in_month = days_in_month(view.year, view.month);
    let start_weekday = first_day.weekday().num_days_from_monday();

    let mut lines = Vec::new();
    let mut current_line = String::new();

    // Padding for first week
    for _ in 0..start_weekday {
        current_line.push_str("      ");
    }

    for day in 1..=days_in_month {
        let has_tasks = view.tasks_by_day.contains_key(&day);
        let is_selected = day == view.selected_day;
        let is_today = /* check if today */;

        let day_str = if is_selected {
            format!("[{:>2}]", day)
        } else {
            format!(" {:>2} ", day)
        };

        let indicator = if has_tasks { "●" } else { " " };

        current_line.push_str(&format!("{}{} ", day_str, indicator));

        if (start_weekday + day) % 7 == 0 {
            lines.push(current_line);
            current_line = String::new();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    let paragraph = Paragraph::new(lines.join("\n"));
    frame.render_widget(paragraph, area);
}
```

### Acceptance Criteria

- [x] `c` opens calendar view
- [x] Current week displayed as grid
- [x] Can navigate days with arrows
- [x] Can change weeks with j/k
- [x] `t` jumps to today
- [x] Days with tasks show indicator (●)
- [x] Selected day shows task list
- [x] Esc returns to main view

---

## Phase 8 Checklist

Before moving to Phase 9, ensure:

- [x] All 3 stories completed
- [x] Help overlay shows all keybindings
- [x] Task detail view displays all info
- [x] Calendar view navigable
- [x] All views accessible via shortcuts
- [x] Esc returns to main view from all views
