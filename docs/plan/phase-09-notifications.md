# Phase 9: Notifications

**Goal:** Implement reminder system with in-app and desktop notifications.

**Prerequisites:** Phase 5 (tasks with due dates)

**Outcome:** Users get notified about due and overdue tasks.

---

## Story 9.1: Reminder Check System

**Priority:** High
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Implement background reminder checking on tick events.

### Tasks

- [ ] Create `src/notifications/reminder.rs`:
  - `ReminderChecker` struct
  - Track which tasks have been notified (avoid spam)
  - Check for:
    - Overdue tasks (past due date)
    - Due soon tasks (within reminder window, default 24h)
  - Configurable reminder window from config

- [ ] Integrate with tick events:
  - On each tick, check for due/overdue tasks
  - Generate notifications for new items

- [ ] Track notification state:
  - `notified_tasks: HashSet<String>` (task IDs)
  - Clear when task is completed or edited
  - Persist across session? (optional)

### Code Sketch

```rust
pub struct ReminderChecker {
    notified_overdue: HashSet<String>,
    notified_due_soon: HashSet<String>,
    reminder_window: Duration,
}

pub struct Reminder {
    pub task_id: String,
    pub task_title: String,
    pub reminder_type: ReminderType,
    pub due_date: DateTime<Utc>,
}

pub enum ReminderType {
    Overdue,
    DueSoon,
}

impl ReminderChecker {
    pub fn new(reminder_window_hours: u32) -> Self {
        Self {
            notified_overdue: HashSet::new(),
            notified_due_soon: HashSet::new(),
            reminder_window: Duration::hours(reminder_window_hours as i64),
        }
    }

    pub fn check(&mut self, tasks: &[Task]) -> Vec<Reminder> {
        let now = Utc::now();
        let mut reminders = Vec::new();

        for task in tasks {
            // Skip completed/archived tasks
            if task.status == TaskStatus::Completed || task.status == TaskStatus::Archived {
                continue;
            }

            let Some(due_date) = task.due_date else {
                continue;
            };

            // Check overdue
            if due_date < now && !self.notified_overdue.contains(&task.id) {
                reminders.push(Reminder {
                    task_id: task.id.clone(),
                    task_title: task.title.clone(),
                    reminder_type: ReminderType::Overdue,
                    due_date,
                });
                self.notified_overdue.insert(task.id.clone());
            }
            // Check due soon
            else if due_date > now
                && due_date < now + self.reminder_window
                && !self.notified_due_soon.contains(&task.id)
            {
                reminders.push(Reminder {
                    task_id: task.id.clone(),
                    task_title: task.title.clone(),
                    reminder_type: ReminderType::DueSoon,
                    due_date,
                });
                self.notified_due_soon.insert(task.id.clone());
            }
        }

        reminders
    }

    pub fn clear_task(&mut self, task_id: &str) {
        self.notified_overdue.remove(task_id);
        self.notified_due_soon.remove(task_id);
    }
}

// In App
impl App {
    pub fn on_tick(&mut self) {
        let reminders = self.reminder_checker.check(&self.tasks);
        for reminder in reminders {
            self.pending_notifications.push(reminder);
            log::info!("Reminder: {} - {}", reminder.reminder_type, reminder.task_title);
        }
    }
}
```

### Acceptance Criteria

- [ ] Overdue tasks trigger reminders
- [ ] Tasks due within window trigger reminders
- [ ] Same task doesn't spam notifications
- [ ] Completing task clears notification state
- [ ] Reminder window configurable

---

## Story 9.2: In-App Notification Banner

**Priority:** High
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Show notification banners in the TUI.

### Tasks

- [ ] Create `src/ui/notification.rs`:
  - `NotificationBanner` widget
  - Render at top of screen
  - Styling: warning color for overdue, info for due soon
  - Show task title and due info
  - Dismiss button/action

- [ ] Notification queue:
  - Store pending notifications
  - Show one at a time
  - Auto-dismiss after timeout (optional)
  - Manual dismiss with `x` or Esc

- [ ] Integrate with main layout:
  - Reserve space for banner when notifications pending
  - Animate in/out (optional)

### Code Sketch

```rust
pub struct NotificationBanner {
    pub notifications: VecDeque<Reminder>,
    pub visible: bool,
}

impl NotificationBanner {
    pub fn push(&mut self, reminder: Reminder) {
        self.notifications.push_back(reminder);
        self.visible = true;
    }

    pub fn dismiss(&mut self) {
        self.notifications.pop_front();
        if self.notifications.is_empty() {
            self.visible = false;
        }
    }

    pub fn current(&self) -> Option<&Reminder> {
        self.notifications.front()
    }
}

pub fn render_notification(frame: &mut Frame, banner: &NotificationBanner, area: Rect) {
    let Some(reminder) = banner.current() else {
        return;
    };

    let (icon, style) = match reminder.reminder_type {
        ReminderType::Overdue => (
            "⚠",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        ),
        ReminderType::DueSoon => (
            "🔔",
            Style::default().fg(Color::Yellow)
        ),
    };

    let message = match reminder.reminder_type {
        ReminderType::Overdue => {
            let ago = format_duration_ago(reminder.due_date);
            format!("{} OVERDUE: \"{}\" was due {}", icon, reminder.task_title, ago)
        }
        ReminderType::DueSoon => {
            let until = format_duration_until(reminder.due_date);
            format!("{} DUE SOON: \"{}\" is due in {}", icon, reminder.task_title, until)
        }
    };

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(style);

    let text = Paragraph::new(format!("{}  [x] Dismiss", message))
        .style(style)
        .block(block);

    frame.render_widget(text, area);
}

// Adjust main layout when notification visible
pub fn render_main_view(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = if app.notification_banner.visible {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // Notification banner
                Constraint::Length(3),  // Header
                Constraint::Min(0),     // Main content
                Constraint::Length(1),  // Status bar
            ])
            .split(area)
    } else {
        // Normal layout without banner
    };

    if app.notification_banner.visible {
        notification::render_notification(frame, &app.notification_banner, chunks[0]);
    }
    // ... rest of layout
}
```

### Acceptance Criteria

- [ ] Notifications appear at top of screen
- [ ] Overdue shows with warning styling
- [ ] Due soon shows with info styling
- [ ] Can dismiss with `x` key
- [ ] Multiple notifications queue
- [ ] Layout adjusts for banner

---

## Story 9.3: Desktop Notifications

**Priority:** Medium
**Estimate:** Small
**Status:** [ ] Not Started

### Description

Send desktop notifications using notify-rust.

### Tasks

- [ ] Integrate `notify-rust` crate
- [ ] Send notification for:
  - Overdue tasks (first time only)
  - Tasks due soon
- [ ] Respect config settings:
  - `notifications.enabled`
  - `notifications.desktop`
- [ ] Handle platforms:
  - macOS: native notifications
  - Linux: D-Bus notifications
  - Windows: toast notifications

### Code Sketch

```rust
use notify_rust::{Notification, Timeout};

pub struct DesktopNotifier {
    enabled: bool,
}

impl DesktopNotifier {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn notify(&self, reminder: &Reminder) -> Result<(), notify_rust::error::Error> {
        if !self.enabled {
            return Ok(());
        }

        let (title, urgency) = match reminder.reminder_type {
            ReminderType::Overdue => (
                "Task Overdue!",
                notify_rust::Urgency::Critical,
            ),
            ReminderType::DueSoon => (
                "Task Due Soon",
                notify_rust::Urgency::Normal,
            ),
        };

        let body = match reminder.reminder_type {
            ReminderType::Overdue => {
                format!("\"{}\" is overdue", reminder.task_title)
            }
            ReminderType::DueSoon => {
                let until = format_duration_until(reminder.due_date);
                format!("\"{}\" is due in {}", reminder.task_title, until)
            }
        };

        Notification::new()
            .summary(title)
            .body(&body)
            .appname("Ratado")
            .icon("dialog-warning")  // or custom icon
            .urgency(urgency)
            .timeout(Timeout::Milliseconds(5000))
            .show()?;

        Ok(())
    }
}

// In App on_tick
pub fn on_tick(&mut self) {
    let reminders = self.reminder_checker.check(&self.tasks);
    for reminder in reminders {
        // In-app notification
        self.notification_banner.push(reminder.clone());

        // Desktop notification
        if let Err(e) = self.desktop_notifier.notify(&reminder) {
            log::warn!("Failed to send desktop notification: {}", e);
        }
    }
}
```

### Acceptance Criteria

- [ ] Desktop notifications appear for overdue tasks
- [ ] Desktop notifications appear for due soon tasks
- [ ] Can be disabled in config
- [ ] Works on macOS
- [ ] Works on Linux (if D-Bus available)
- [ ] Graceful fallback if notifications unavailable

---

## Phase 9 Checklist

Before moving to Phase 10, ensure:

- [ ] All 3 stories completed
- [ ] Reminder checker identifies due/overdue tasks
- [ ] In-app banners display
- [ ] Desktop notifications work (when enabled)
- [ ] No notification spam (same task)
- [ ] Config settings respected
