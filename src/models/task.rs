//! Task model and related types.
//!
//! This module defines the core [`Task`] struct along with [`Priority`] and
//! [`TaskStatus`] enums that represent task attributes.

use chrono::{DateTime, Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task priority levels.
///
/// Priority determines the importance of a task and affects how it's displayed
/// and sorted in the UI. Higher priority tasks are typically shown first.
///
/// # Ordering
///
/// Priorities are ordered from lowest to highest:
/// `Low < Medium < High < Urgent`
///
/// # Examples
///
/// ```
/// use ratado::models::Priority;
///
/// assert!(Priority::Urgent > Priority::High);
/// assert!(Priority::High > Priority::Medium);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Priority {
    /// Low priority - tasks that can wait
    Low,
    /// Medium priority - normal tasks (default)
    #[default]
    Medium,
    /// High priority - important tasks that need attention
    High,
    /// Urgent priority - critical tasks requiring immediate action
    Urgent,
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_val = match self {
            Priority::Low => 0,
            Priority::Medium => 1,
            Priority::High => 2,
            Priority::Urgent => 3,
        };
        let other_val = match other {
            Priority::Low => 0,
            Priority::Medium => 1,
            Priority::High => 2,
            Priority::Urgent => 3,
        };
        self_val.cmp(&other_val)
    }
}

/// Task status states.
///
/// Represents the current state of a task in its lifecycle. Tasks typically
/// move from Pending → InProgress → Completed, or can be Archived.
///
/// # Examples
///
/// ```
/// use ratado::models::{Task, TaskStatus};
///
/// let mut task = Task::new("Example");
/// assert_eq!(task.status, TaskStatus::Pending);
///
/// task.complete();
/// assert_eq!(task.status, TaskStatus::Completed);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskStatus {
    /// Task is waiting to be started (default state)
    #[default]
    Pending,
    /// Task is currently being worked on
    InProgress,
    /// Task has been finished
    Completed,
    /// Task has been archived (hidden from normal views)
    Archived,
}

/// A task item.
///
/// Tasks are the core entity in Ratado. Each task has a title, optional
/// description, due date, priority, status, and can be associated with
/// a project and tags.
///
/// # Examples
///
/// ```
/// use ratado::models::{Task, Priority};
/// use chrono::{Utc, Duration};
///
/// // Create a basic task
/// let task = Task::new("Buy groceries");
/// assert_eq!(task.title, "Buy groceries");
///
/// // Create a task with a due date
/// let mut task = Task::new("Submit report");
/// task.due_date = Some(Utc::now() + Duration::days(1));
/// task.priority = Priority::High;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier (UUID v7 string - time-ordered)
    pub id: String,
    /// The task title displayed in the UI
    pub title: String,
    /// Optional longer description with details
    pub description: Option<String>,
    /// Optional due date/time in UTC
    pub due_date: Option<DateTime<Utc>>,
    /// Task priority level
    pub priority: Priority,
    /// Current task status
    pub status: TaskStatus,
    /// Optional project ID this task belongs to
    pub project_id: Option<String>,
    /// Tags associated with this task
    pub tags: Vec<String>,
    /// When the task was created (UTC)
    pub created_at: DateTime<Utc>,
    /// When the task was last modified (UTC)
    pub updated_at: DateTime<Utc>,
    /// When the task was completed (UTC), if applicable
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    /// Creates a new task with the given title.
    ///
    /// The task is created with default values:
    /// - Time-ordered UUID v7 as ID
    /// - Medium priority
    /// - Pending status
    /// - Current timestamp for created_at and updated_at
    /// - No description, due date, project, or tags
    ///
    /// # Arguments
    ///
    /// * `title` - The title for the new task
    ///
    /// # Returns
    ///
    /// A new `Task` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use ratado::models::Task;
    ///
    /// let task = Task::new("Complete project");
    /// assert_eq!(task.title, "Complete project");
    /// ```
    pub fn new(title: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7().to_string(),
            title: title.to_string(),
            description: None,
            due_date: None,
            priority: Priority::default(),
            status: TaskStatus::default(),
            project_id: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    /// Checks if the task is overdue.
    ///
    /// A task is considered overdue if:
    /// - It has a due date
    /// - The due date is in the past
    /// - The task is not completed or archived
    ///
    /// # Returns
    ///
    /// `true` if the task is overdue, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use ratado::models::Task;
    /// use chrono::{Utc, Duration};
    ///
    /// let mut task = Task::new("Overdue task");
    /// task.due_date = Some(Utc::now() - Duration::hours(1));
    /// assert!(task.is_overdue());
    ///
    /// // Completed tasks are never overdue
    /// task.complete();
    /// assert!(!task.is_overdue());
    /// ```
    pub fn is_overdue(&self) -> bool {
        if self.status == TaskStatus::Completed || self.status == TaskStatus::Archived {
            return false;
        }
        match self.due_date {
            Some(due) => due < Utc::now(),
            None => false,
        }
    }

    /// Checks if the task is due today.
    ///
    /// Compares the due date (in local timezone) with today's date.
    ///
    /// # Returns
    ///
    /// `true` if the task is due today, `false` if no due date or due another day
    ///
    /// # Examples
    ///
    /// ```
    /// use ratado::models::Task;
    /// use chrono::Utc;
    ///
    /// let mut task = Task::new("Today's task");
    /// task.due_date = Some(Utc::now());
    /// assert!(task.is_due_today());
    /// ```
    pub fn is_due_today(&self) -> bool {
        match self.due_date {
            Some(due) => {
                let today = Local::now().date_naive();
                let due_local = due.with_timezone(&Local).date_naive();
                today == due_local
            }
            None => false,
        }
    }

    /// Checks if the task is due within the next 7 days.
    ///
    /// # Returns
    ///
    /// `true` if the task is due between now and 7 days from now
    ///
    /// # Examples
    ///
    /// ```
    /// use ratado::models::Task;
    /// use chrono::{Utc, Duration};
    ///
    /// let mut task = Task::new("This week's task");
    /// task.due_date = Some(Utc::now() + Duration::days(3));
    /// assert!(task.is_due_this_week());
    /// ```
    pub fn is_due_this_week(&self) -> bool {
        match self.due_date {
            Some(due) => {
                let now = Utc::now();
                let week_from_now = now + Duration::days(7);
                due >= now && due <= week_from_now
            }
            None => false,
        }
    }

    /// Marks the task as completed.
    ///
    /// Sets the status to `Completed`, records the completion time,
    /// and updates the `updated_at` timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratado::models::{Task, TaskStatus};
    ///
    /// let mut task = Task::new("Finish report");
    /// task.complete();
    /// assert_eq!(task.status, TaskStatus::Completed);
    /// assert!(task.completed_at.is_some());
    /// ```
    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Reopens a completed task.
    ///
    /// Sets the status back to `Pending`, clears the completion time,
    /// and updates the `updated_at` timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratado::models::{Task, TaskStatus};
    ///
    /// let mut task = Task::new("Task");
    /// task.complete();
    /// task.reopen();
    /// assert_eq!(task.status, TaskStatus::Pending);
    /// assert!(task.completed_at.is_none());
    /// ```
    pub fn reopen(&mut self) {
        self.status = TaskStatus::Pending;
        self.completed_at = None;
        self.updated_at = Utc::now();
    }
}

impl Default for Task {
    fn default() -> Self {
        Self::new("Untitled Task")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_new() {
        let task = Task::new("Test Task");
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, Priority::Medium);
        assert!(task.description.is_none());
        assert!(task.due_date.is_none());
    }

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
    fn test_task_without_due_date_not_overdue() {
        let task = Task::new("Test");
        assert!(!task.is_overdue());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Urgent > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }

    #[test]
    fn test_task_complete_and_reopen() {
        let mut task = Task::new("Test");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.completed_at.is_none());

        task.complete();
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completed_at.is_some());

        task.reopen();
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_task_is_due_today() {
        let mut task = Task::new("Test");
        // Set due date to now (today)
        task.due_date = Some(Utc::now());
        assert!(task.is_due_today());

        // Set due date to tomorrow
        task.due_date = Some(Utc::now() + Duration::days(1));
        assert!(!task.is_due_today());
    }
}
