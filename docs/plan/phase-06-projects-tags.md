# Phase 6: Projects & Tags

**Goal:** Implement project filtering and tag management.

**Prerequisites:** Phase 5 (task operations)

**Outcome:** Users can organize tasks by project and tags.

---

## Story 6.1: Project Selection & Filtering

**Priority:** High
**Estimate:** Medium
**Status:** [x] Complete

### Description

Enable selecting projects in sidebar to filter task list.

### Tasks

- [ ] Implement sidebar project navigation:
  - Up/Down when sidebar focused
  - Track `selected_project_index` in App
  - Index 0 = "All Tasks"
  - Index 1+ = specific projects

- [ ] Apply project filter to task list:
  - When project selected, filter visible_tasks to that project
  - Update `Filter::ByProject(project_id)`
  - "All Tasks" clears project filter

- [ ] Update task counts:
  - Show accurate count per project
  - Update counts when tasks change

- [ ] Visual feedback:
  - Highlight selected project
  - Show filter indicator in status bar

### Code Sketch

```rust
impl App {
    pub fn selected_project(&self) -> Option<&Project> {
        if self.selected_project_index == 0 {
            None  // "All Tasks"
        } else {
            self.projects.get(self.selected_project_index - 1)
        }
    }

    pub fn visible_tasks(&self) -> Vec<&Task> {
        let mut tasks: Vec<&Task> = self.tasks.iter()
            .filter(|t| {
                // Apply project filter
                if let Some(project) = self.selected_project() {
                    if t.project_id.as_ref() != Some(&project.id) {
                        return false;
                    }
                }
                // Apply other filters
                self.filter.matches(t)
            })
            .collect();

        self.sort.apply(&mut tasks);
        tasks
    }
}

// Navigation commands when sidebar focused
Command::NavigateUp if app.focus == FocusPanel::Sidebar => {
    app.selected_project_index = app.selected_project_index.saturating_sub(1);
    // Reset task selection when project changes
    app.selected_task_index = if app.visible_tasks().is_empty() {
        None
    } else {
        Some(0)
    };
    Ok(true)
}
```

### Acceptance Criteria

- [ ] Can navigate projects with j/k when sidebar focused
- [ ] Selecting project filters task list
- [ ] "All Tasks" shows all tasks
- [ ] Task counts accurate per project
- [ ] Status bar shows current project filter
- [ ] Task selection resets when changing projects

---

## Story 6.2: Project Management

**Priority:** Medium
**Estimate:** Medium
**Status:** [x] Complete

### Description

Add ability to create, rename, and delete projects.

### Tasks

- [ ] Create project dialog:
  - Similar to add task dialog but simpler
  - Fields: name, color (hex input or preset picker)
  - Triggered by a key (e.g., `P` for new project)

- [ ] Implement project rename:
  - Edit existing project
  - Update in database

- [ ] Implement project delete:
  - Confirmation dialog
  - Options for tasks in project:
    - Move to Inbox (default)
    - Delete all tasks
  - Cannot delete "Inbox" project

- [ ] Color display:
  - Show project color indicator in sidebar
  - Use color in task badges

### Code Sketch

```rust
pub struct ProjectDialog {
    pub name: TextInput,
    pub color: String,
    pub editing_project_id: Option<String>,
}

impl ProjectDialog {
    pub fn new() -> Self {
        Self {
            name: TextInput::new(),
            color: "#4A90D9".to_string(),
            editing_project_id: None,
        }
    }

    pub fn from_project(project: &Project) -> Self {
        Self {
            name: TextInput::with_value(project.name.clone()),
            color: project.color.clone(),
            editing_project_id: Some(project.id.clone()),
        }
    }
}

// Delete project with task handling
pub async fn delete_project(
    db: &Database,
    project_id: &str,
    task_action: ProjectDeleteAction,
) -> Result<()> {
    match task_action {
        ProjectDeleteAction::MoveToInbox => {
            db.execute(
                "UPDATE tasks SET project_id = 'inbox' WHERE project_id = ?",
                (project_id,)
            ).await?;
        }
        ProjectDeleteAction::DeleteTasks => {
            db.execute(
                "DELETE FROM tasks WHERE project_id = ?",
                (project_id,)
            ).await?;
        }
    }
    db.delete_project(project_id).await?;
    Ok(())
}
```

### Acceptance Criteria

- [ ] Can create new projects
- [ ] Can rename existing projects
- [ ] Can delete projects (with task handling)
- [ ] Cannot delete Inbox project
- [ ] Project colors display in sidebar
- [ ] Changes persist to database

---

## Story 6.3: Tag Management

**Priority:** Medium
**Estimate:** Medium
**Status:** [x] Complete

### Description

Implement tag assignment and filtering.

### Tasks

- [ ] Add tag input to task dialog:
  - Comma-separated tag input
  - Show existing tags as chips/badges
  - Create new tags on-the-fly

- [ ] Implement tag autocomplete:
  - As user types, show matching existing tags
  - Tab or Enter to select suggestion

- [ ] Tag display on task rows:
  - Show tags as colored badges
  - Truncate if too many

- [ ] Filter by tag:
  - Select tag in sidebar to filter
  - Multiple tag selection (optional, can be OR)

- [ ] Tag list in sidebar:
  - Show all tags with task counts
  - Navigate and select

### Code Sketch

```rust
// Tag input in add/edit task dialog
pub struct TagInput {
    pub input: TextInput,
    pub tags: Vec<String>,
    pub suggestions: Vec<String>,
    pub selected_suggestion: Option<usize>,
}

impl TagInput {
    pub fn handle_key(&mut self, key: KeyEvent, all_tags: &[Tag]) {
        match key.code {
            KeyCode::Char(',') | KeyCode::Enter => {
                // Add current input as tag
                let tag = self.input.value().trim().to_string();
                if !tag.is_empty() && !self.tags.contains(&tag) {
                    self.tags.push(tag);
                }
                self.input = TextInput::new();
                self.update_suggestions(all_tags);
            }
            KeyCode::Backspace if self.input.value().is_empty() => {
                // Remove last tag
                self.tags.pop();
            }
            KeyCode::Tab => {
                // Accept suggestion
                if let Some(idx) = self.selected_suggestion {
                    if let Some(tag) = self.suggestions.get(idx) {
                        self.tags.push(tag.clone());
                        self.input = TextInput::new();
                    }
                }
            }
            _ => {
                self.input.handle_key(key);
                self.update_suggestions(all_tags);
            }
        }
    }

    fn update_suggestions(&mut self, all_tags: &[Tag]) {
        let query = self.input.value().to_lowercase();
        self.suggestions = all_tags.iter()
            .filter(|t| t.name.to_lowercase().contains(&query))
            .filter(|t| !self.tags.contains(&t.name))
            .map(|t| t.name.clone())
            .take(5)
            .collect();
        self.selected_suggestion = if self.suggestions.is_empty() {
            None
        } else {
            Some(0)
        };
    }
}

// Render tags on task row
fn render_tags(tags: &[String], max_width: u16) -> Span {
    let tag_str: String = tags.iter()
        .map(|t| format!("#{}", t))
        .collect::<Vec<_>>()
        .join(" ");

    if tag_str.len() > max_width as usize {
        Span::styled(
            format!("{}...", &tag_str[..max_width as usize - 3]),
            Style::default().fg(Color::Magenta)
        )
    } else {
        Span::styled(tag_str, Style::default().fg(Color::Magenta))
    }
}
```

### Acceptance Criteria

- [ ] Can add tags to tasks in dialog
- [ ] Autocomplete suggests existing tags
- [ ] Tags display on task rows
- [ ] Can filter by tag from sidebar
- [ ] Tag counts accurate in sidebar
- [ ] New tags created automatically
- [ ] Tags persist to database

---

## Phase 6 Checklist

Before moving to Phase 7, ensure:

- [x] All 3 stories completed
- [x] Project selection filters tasks
- [x] Can create/edit/delete projects
- [x] Tags can be added to tasks
- [x] Tag autocomplete works
- [x] Can filter by tag
- [x] All changes persist to database
- [x] Tests pass (206 tests passing)
