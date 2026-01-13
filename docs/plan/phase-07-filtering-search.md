# Phase 7: Filtering & Search

**Goal:** Implement filter menus, sorting, and search functionality.

**Prerequisites:** Phase 5 (basic task operations)

**Outcome:** Users can filter, sort, and search tasks effectively.

---

## Story 7.1: Filter Menu

**Priority:** High
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Implement the filter dropdown menu.

### Tasks

- [ ] Create `src/ui/dialogs/filter_menu.rs`:
  - Dropdown/popup menu widget
  - Filter options:
    - All Tasks
    - Pending
    - In Progress
    - Completed
    - Archived (toggle show/hide)
    - ───────────
    - Due Today
    - Due This Week
    - Overdue
    - No Due Date
    - ───────────
    - Urgent Priority
    - High Priority
  - Show task count per filter option
  - Navigate with j/k or arrows
  - Enter to select
  - Esc to close

- [ ] Trigger with `f` key
- [ ] Apply selected filter to `app.filter`
- [ ] Show active filter in status bar

### Code Sketch

```rust
pub struct FilterMenu {
    pub options: Vec<FilterOption>,
    pub selected: usize,
}

pub struct FilterOption {
    pub label: String,
    pub filter: Option<Filter>,  // None = separator
    pub count: usize,
}

impl FilterMenu {
    pub fn new(app: &App) -> Self {
        let options = vec![
            FilterOption::filter("All Tasks", Filter::All, app.tasks.len()),
            FilterOption::filter("Pending", Filter::Pending,
                app.tasks.iter().filter(|t| t.status == TaskStatus::Pending).count()),
            FilterOption::filter("In Progress", Filter::InProgress,
                app.tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count()),
            FilterOption::filter("Completed", Filter::Completed,
                app.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count()),
            FilterOption::separator(),
            FilterOption::filter("Due Today", Filter::DueToday,
                app.tasks.iter().filter(|t| t.is_due_today()).count()),
            FilterOption::filter("Due This Week", Filter::DueThisWeek,
                app.tasks.iter().filter(|t| t.is_due_this_week()).count()),
            FilterOption::filter("Overdue", Filter::Overdue,
                app.tasks.iter().filter(|t| t.is_overdue()).count()),
            FilterOption::separator(),
            FilterOption::filter("Urgent", Filter::ByPriority(Priority::Urgent),
                app.tasks.iter().filter(|t| t.priority == Priority::Urgent).count()),
            FilterOption::filter("High Priority", Filter::ByPriority(Priority::High),
                app.tasks.iter().filter(|t| t.priority == Priority::High).count()),
        ];

        Self { options, selected: 0 }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Filter> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_down();
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_up();
                None
            }
            KeyCode::Enter => {
                self.options.get(self.selected).and_then(|o| o.filter.clone())
            }
            _ => None,
        }
    }

    fn move_down(&mut self) {
        loop {
            self.selected = (self.selected + 1) % self.options.len();
            if self.options[self.selected].filter.is_some() {
                break;
            }
        }
    }
}

pub fn render_filter_menu(frame: &mut Frame, menu: &FilterMenu, area: Rect) {
    let block = Block::default()
        .title("Filter by")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let items: Vec<ListItem> = menu.options.iter()
        .enumerate()
        .map(|(i, opt)| {
            if opt.filter.is_none() {
                // Separator
                ListItem::new("─────────────────────")
                    .style(Style::default().fg(Color::DarkGray))
            } else {
                let selected = i == menu.selected;
                let marker = if selected { "▸ " } else { "  " };
                let style = if selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!("{}{:<20} ({:>3})", marker, opt.label, opt.count))
                    .style(style)
            }
        })
        .collect();

    let list = List::new(items).block(block);

    // Render centered popup
    let popup_area = centered_rect(30, 60, area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(list, popup_area);
}
```

### Acceptance Criteria

- [ ] `f` opens filter menu
- [ ] All filter options shown with counts
- [ ] Can navigate with j/k or arrows
- [ ] Enter applies filter
- [ ] Esc closes without changing
- [ ] Filter shown in status bar
- [ ] Task list updates when filter applied

---

## Story 7.2: Sort Menu

**Priority:** High
**Estimate:** Small
**Status:** [ ] Not Started

### Description

Implement sorting options menu.

### Tasks

- [ ] Create `src/ui/dialogs/sort_menu.rs`:
  - Similar to filter menu
  - Sort options:
    - Due Date (ascending) ✓
    - Due Date (descending)
    - Priority (high first)
    - Priority (low first)
    - Created (newest)
    - Created (oldest)
    - Alphabetical (A-Z)
    - Alphabetical (Z-A)
  - Show checkmark on current sort
  - Navigate and select

- [ ] Trigger with `s` key
- [ ] Apply to `app.sort`
- [ ] Remember sort preference during session

### Code Sketch

```rust
pub struct SortMenu {
    pub options: Vec<(String, SortOrder)>,
    pub selected: usize,
    pub current: SortOrder,
}

impl SortMenu {
    pub fn new(current: SortOrder) -> Self {
        let options = vec![
            ("Due Date (soonest first)".to_string(), SortOrder::DueDateAsc),
            ("Due Date (latest first)".to_string(), SortOrder::DueDateDesc),
            ("Priority (urgent first)".to_string(), SortOrder::PriorityDesc),
            ("Priority (low first)".to_string(), SortOrder::PriorityAsc),
            ("Created (newest)".to_string(), SortOrder::CreatedDesc),
            ("Created (oldest)".to_string(), SortOrder::CreatedAsc),
            ("Alphabetical (A-Z)".to_string(), SortOrder::Alphabetical),
        ];

        let selected = options.iter()
            .position(|(_, s)| *s == current)
            .unwrap_or(0);

        Self { options, selected, current }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.options.iter()
            .enumerate()
            .map(|(i, (label, sort))| {
                let is_current = *sort == self.current;
                let is_selected = i == self.selected;
                let marker = if is_current { "✓ " } else { "  " };
                let style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!("{}{}", marker, label)).style(style)
            })
            .collect();

        // Render as popup
    }
}
```

### Acceptance Criteria

- [ ] `s` opens sort menu
- [ ] All sort options available
- [ ] Current sort marked with ✓
- [ ] Enter applies sort
- [ ] Task list reorders immediately
- [ ] Sort persists during session

---

## Story 7.3: Search View

**Priority:** High
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Implement full-text search functionality.

### Tasks

- [ ] Create `src/ui/search.rs`:
  - Full-screen or large popup view
  - Search input at top
  - Live filtering as user types
  - Results list below
  - Highlight matching text in results

- [ ] Implement search logic:
  - Search in task title
  - Search in task description
  - Case-insensitive
  - Show context around matches

- [ ] Navigation:
  - Type to search
  - j/k or arrows to navigate results
  - Enter to open selected task (detail view or edit)
  - Esc to close search

- [ ] Trigger with `/` key

### Code Sketch

```rust
pub struct SearchView {
    pub query: TextInput,
    pub results: Vec<SearchResult>,
    pub selected: usize,
}

pub struct SearchResult {
    pub task: Task,
    pub title_match: Option<(usize, usize)>,  // start, end of match
    pub desc_match: Option<String>,  // snippet with match
}

impl SearchView {
    pub fn search(&mut self, tasks: &[Task]) {
        let query = self.query.value().to_lowercase();
        if query.is_empty() {
            self.results.clear();
            return;
        }

        self.results = tasks.iter()
            .filter_map(|task| {
                let title_lower = task.title.to_lowercase();
                let title_match = title_lower.find(&query).map(|start| {
                    (start, start + query.len())
                });

                let desc_match = task.description.as_ref().and_then(|desc| {
                    let desc_lower = desc.to_lowercase();
                    desc_lower.find(&query).map(|pos| {
                        // Extract snippet around match
                        let start = pos.saturating_sub(20);
                        let end = (pos + query.len() + 20).min(desc.len());
                        format!("...{}...", &desc[start..end])
                    })
                });

                if title_match.is_some() || desc_match.is_some() {
                    Some(SearchResult {
                        task: task.clone(),
                        title_match,
                        desc_match,
                    })
                } else {
                    None
                }
            })
            .collect();

        self.selected = 0;
    }

    pub fn handle_key(&mut self, key: KeyEvent, tasks: &[Task]) -> SearchAction {
        match key.code {
            KeyCode::Esc => SearchAction::Close,
            KeyCode::Enter => {
                if let Some(result) = self.results.get(self.selected) {
                    SearchAction::OpenTask(result.task.id.clone())
                } else {
                    SearchAction::None
                }
            }
            KeyCode::Down | KeyCode::Char('j') if !self.results.is_empty() => {
                self.selected = (self.selected + 1) % self.results.len();
                SearchAction::None
            }
            KeyCode::Up | KeyCode::Char('k') if !self.results.is_empty() => {
                self.selected = self.selected.checked_sub(1)
                    .unwrap_or(self.results.len() - 1);
                SearchAction::None
            }
            _ => {
                self.query.handle_key(key);
                self.search(tasks);
                SearchAction::None
            }
        }
    }
}

pub fn render_search(frame: &mut Frame, search: &SearchView, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Search input
            Constraint::Min(0),     // Results
        ])
        .split(area);

    // Search input
    let input_block = Block::default()
        .title("Search")
        .borders(Borders::ALL);
    let input = Paragraph::new(format!("/{}", search.query.value()))
        .block(input_block);
    frame.render_widget(input, chunks[0]);

    // Results
    if search.results.is_empty() && !search.query.value().is_empty() {
        let no_results = Paragraph::new("No matching tasks found")
            .alignment(Alignment::Center);
        frame.render_widget(no_results, chunks[1]);
    } else {
        let items: Vec<ListItem> = search.results.iter()
            .enumerate()
            .map(|(i, result)| {
                let selected = i == search.selected;
                // Render with highlighted matches
                render_search_result(result, selected)
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, chunks[1]);
    }
}
```

### Acceptance Criteria

- [ ] `/` opens search view
- [ ] Can type search query
- [ ] Results filter in real-time
- [ ] Matches highlighted in results
- [ ] Can navigate results with j/k
- [ ] Enter opens selected task
- [ ] Esc closes search
- [ ] Searches title and description

---

## Story 7.4: Quick Filters

**Priority:** Low
**Estimate:** Small
**Status:** [ ] Not Started

### Description

Implement keyboard shortcuts for common filters.

### Tasks

- [ ] Implement quick filter keys:
  - `T` - Filter to today's tasks only
  - `W` - Filter to this week's tasks
  - `A` - Toggle show archived tasks
  - `1` - Low priority only
  - `2` - Medium priority only
  - `3` - High priority only
  - `4` - Urgent priority only
  - `0` or `Esc` - Clear all filters

- [ ] Update status bar to show active quick filter

### Code Sketch

```rust
// In key mapping
KeyCode::Char('T') => Some(Command::SetFilter(Filter::DueToday)),
KeyCode::Char('W') => Some(Command::SetFilter(Filter::DueThisWeek)),
KeyCode::Char('A') => Some(Command::ToggleShowArchived),
KeyCode::Char('1') => Some(Command::SetFilter(Filter::ByPriority(Priority::Low))),
KeyCode::Char('2') => Some(Command::SetFilter(Filter::ByPriority(Priority::Medium))),
KeyCode::Char('3') => Some(Command::SetFilter(Filter::ByPriority(Priority::High))),
KeyCode::Char('4') => Some(Command::SetFilter(Filter::ByPriority(Priority::Urgent))),
KeyCode::Char('0') => Some(Command::ClearFilter),

// Command execution
Command::SetFilter(filter) => {
    app.filter = filter;
    app.selected_task_index = if app.visible_tasks().is_empty() {
        None
    } else {
        Some(0)
    };
    Ok(true)
}

Command::ClearFilter => {
    app.filter = Filter::All;
    app.selected_task_index = Some(0);
    Ok(true)
}

Command::ToggleShowArchived => {
    app.show_archived = !app.show_archived;
    Ok(true)
}
```

### Acceptance Criteria

- [ ] `T` shows only today's tasks
- [ ] `W` shows only this week's tasks
- [ ] `1-4` filter by priority level
- [ ] `0` or `Esc` clears filter
- [ ] `A` toggles archived visibility
- [ ] Filter indicator shown in status bar

---

## Phase 7 Checklist

Before moving to Phase 8, ensure:

- [ ] All 4 stories completed
- [ ] Filter menu works with all options
- [ ] Sort menu works
- [ ] Search finds tasks by title/description
- [ ] Quick filter keys work
- [ ] Filters can be cleared
- [ ] Status bar shows active filter/sort
- [ ] All tests pass
