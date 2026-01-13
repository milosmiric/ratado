//! Filtering and sorting for task lists.
//!
//! This module provides [`Filter`] and [`SortOrder`] types for querying
//! and organizing task collections.

use super::task::{Priority, Task, TaskStatus};

/// Filter criteria for tasks.
///
/// Filters determine which tasks are included in a view. Multiple filters
/// can be combined by applying them sequentially.
///
/// # Examples
///
/// ```
/// use ratado::models::{Task, Filter, Priority};
///
/// let tasks = vec![
///     Task::new("High priority task"),
///     Task::new("Low priority task"),
/// ];
///
/// // Filter by priority
/// let high_priority = Filter::ByPriority(Priority::High);
/// let filtered = high_priority.apply(&tasks);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Filter {
    /// Show all tasks (no filtering)
    All,
    /// Show only pending tasks (default filter)
    #[default]
    Pending,
    /// Show only tasks in progress
    InProgress,
    /// Show only completed tasks
    Completed,
    /// Show only archived tasks
    Archived,
    /// Show tasks due today
    DueToday,
    /// Show tasks due within the next 7 days
    DueThisWeek,
    /// Show overdue tasks
    Overdue,
    /// Show tasks belonging to a specific project
    ByProject(String),
    /// Show tasks with a specific tag
    ByTag(String),
    /// Show tasks with a specific priority
    ByPriority(Priority),
}

impl Filter {
    /// Applies this filter to a list of tasks.
    ///
    /// Returns references to tasks that match the filter criteria.
    ///
    /// # Arguments
    ///
    /// * `tasks` - Slice of tasks to filter
    ///
    /// # Returns
    ///
    /// Vector of references to matching tasks
    ///
    /// # Examples
    ///
    /// ```
    /// use ratado::models::{Task, Filter};
    ///
    /// let tasks = vec![Task::new("Task 1"), Task::new("Task 2")];
    /// let all = Filter::All.apply(&tasks);
    /// assert_eq!(all.len(), 2);
    /// ```
    pub fn apply<'a>(&self, tasks: &'a [Task]) -> Vec<&'a Task> {
        tasks.iter().filter(|task| self.matches(task)).collect()
    }

    /// Checks if a single task matches this filter.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to check
    ///
    /// # Returns
    ///
    /// `true` if the task matches the filter criteria
    ///
    /// # Examples
    ///
    /// ```
    /// use ratado::models::{Task, Filter, TaskStatus};
    ///
    /// let task = Task::new("Pending task");
    /// assert!(Filter::Pending.matches(&task));
    /// assert!(Filter::All.matches(&task));
    /// ```
    pub fn matches(&self, task: &Task) -> bool {
        match self {
            Filter::All => true,
            Filter::Pending => task.status == TaskStatus::Pending,
            Filter::InProgress => task.status == TaskStatus::InProgress,
            Filter::Completed => task.status == TaskStatus::Completed,
            Filter::Archived => task.status == TaskStatus::Archived,
            Filter::DueToday => task.is_due_today(),
            Filter::DueThisWeek => task.is_due_this_week(),
            Filter::Overdue => task.is_overdue(),
            Filter::ByProject(project_id) => task.project_id.as_ref() == Some(project_id),
            Filter::ByTag(tag) => task.tags.contains(tag),
            Filter::ByPriority(priority) => task.priority == *priority,
        }
    }
}

/// Sort order options for task lists.
///
/// Determines how tasks are ordered when displayed. Can be applied
/// to a mutable slice of task references.
///
/// # Examples
///
/// ```
/// use ratado::models::{Task, SortOrder, Priority};
///
/// let mut task1 = Task::new("A task");
/// task1.priority = Priority::Low;
/// let mut task2 = Task::new("B task");
/// task2.priority = Priority::High;
///
/// let tasks = vec![task1, task2];
/// let mut refs: Vec<&Task> = tasks.iter().collect();
///
/// SortOrder::PriorityDesc.apply(&mut refs);
/// assert_eq!(refs[0].title, "B task"); // High priority first
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    /// Sort by due date, earliest first (default)
    #[default]
    DueDateAsc,
    /// Sort by due date, latest first
    DueDateDesc,
    /// Sort by priority, highest first
    PriorityDesc,
    /// Sort by priority, lowest first
    PriorityAsc,
    /// Sort by creation date, newest first
    CreatedDesc,
    /// Sort by creation date, oldest first
    CreatedAsc,
    /// Sort alphabetically by title
    Alphabetical,
}

impl SortOrder {
    /// Sorts a slice of task references in place.
    ///
    /// # Arguments
    ///
    /// * `tasks` - Mutable slice of task references to sort
    ///
    /// # Examples
    ///
    /// ```
    /// use ratado::models::{Task, SortOrder};
    ///
    /// let task1 = Task::new("Zebra");
    /// let task2 = Task::new("Apple");
    /// let tasks = vec![task1, task2];
    /// let mut refs: Vec<&Task> = tasks.iter().collect();
    ///
    /// SortOrder::Alphabetical.apply(&mut refs);
    /// assert_eq!(refs[0].title, "Apple");
    /// ```
    pub fn apply(&self, tasks: &mut [&Task]) {
        match self {
            SortOrder::DueDateAsc => {
                // Tasks with due dates first (earliest first), then tasks without (by created date)
                tasks.sort_by(|a, b| {
                    match (&a.due_date, &b.due_date) {
                        (Some(a_due), Some(b_due)) => a_due.cmp(b_due),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.created_at.cmp(&b.created_at),
                    }
                });
            }
            SortOrder::DueDateDesc => {
                // Tasks with due dates first (latest first), then tasks without (by created date desc)
                tasks.sort_by(|a, b| {
                    match (&a.due_date, &b.due_date) {
                        (Some(a_due), Some(b_due)) => b_due.cmp(a_due),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => b.created_at.cmp(&a.created_at),
                    }
                });
            }
            SortOrder::PriorityDesc => {
                tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
            }
            SortOrder::PriorityAsc => {
                tasks.sort_by(|a, b| a.priority.cmp(&b.priority));
            }
            SortOrder::CreatedDesc => {
                tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            SortOrder::CreatedAsc => {
                tasks.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            }
            SortOrder::Alphabetical => {
                tasks.sort_by(|a, b| a.title.cmp(&b.title));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_filter_all() {
        let tasks = vec![Task::new("Task 1"), Task::new("Task 2")];
        let filtered = Filter::All.apply(&tasks);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_status() {
        let task1 = Task::new("Pending");
        let mut task2 = Task::new("Completed");
        task2.complete();

        let tasks = vec![task1.clone(), task2.clone()];

        let pending = Filter::Pending.apply(&tasks);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].title, "Pending");

        let completed = Filter::Completed.apply(&tasks);
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].title, "Completed");
    }

    #[test]
    fn test_filter_by_priority() {
        let mut task1 = Task::new("Low");
        task1.priority = Priority::Low;
        let mut task2 = Task::new("High");
        task2.priority = Priority::High;

        let tasks = vec![task1, task2];
        let high_priority = Filter::ByPriority(Priority::High).apply(&tasks);
        assert_eq!(high_priority.len(), 1);
        assert_eq!(high_priority[0].title, "High");
    }

    #[test]
    fn test_filter_overdue() {
        let mut task1 = Task::new("Overdue");
        task1.due_date = Some(Utc::now() - Duration::hours(1));
        let mut task2 = Task::new("Future");
        task2.due_date = Some(Utc::now() + Duration::days(1));

        let tasks = vec![task1, task2];
        let overdue = Filter::Overdue.apply(&tasks);
        assert_eq!(overdue.len(), 1);
        assert_eq!(overdue[0].title, "Overdue");
    }

    #[test]
    fn test_sort_by_priority() {
        let mut task1 = Task::new("Low");
        task1.priority = Priority::Low;
        let mut task2 = Task::new("High");
        task2.priority = Priority::High;
        let mut task3 = Task::new("Medium");
        task3.priority = Priority::Medium;

        let tasks = vec![task1, task2, task3];
        let mut refs: Vec<&Task> = tasks.iter().collect();

        SortOrder::PriorityDesc.apply(&mut refs);
        assert_eq!(refs[0].title, "High");
        assert_eq!(refs[1].title, "Medium");
        assert_eq!(refs[2].title, "Low");
    }

    #[test]
    fn test_sort_alphabetical() {
        let task1 = Task::new("Zebra");
        let task2 = Task::new("Apple");
        let task3 = Task::new("Mango");

        let tasks = vec![task1, task2, task3];
        let mut refs: Vec<&Task> = tasks.iter().collect();

        SortOrder::Alphabetical.apply(&mut refs);
        assert_eq!(refs[0].title, "Apple");
        assert_eq!(refs[1].title, "Mango");
        assert_eq!(refs[2].title, "Zebra");
    }

    #[test]
    fn test_sort_due_date_with_none() {
        // Tasks with due dates should come first, then tasks without
        let mut task_with_due = Task::new("Has due date");
        task_with_due.due_date = Some(Utc::now() + Duration::days(1));

        let task_no_due_1 = Task::new("No due date 1");
        // Small delay to ensure different created_at
        let mut task_no_due_2 = Task::new("No due date 2");
        task_no_due_2.created_at = Utc::now() + Duration::seconds(1);

        let tasks = vec![task_no_due_1, task_with_due, task_no_due_2];
        let mut refs: Vec<&Task> = tasks.iter().collect();

        SortOrder::DueDateAsc.apply(&mut refs);
        // Task with due date should be first
        assert_eq!(refs[0].title, "Has due date");
        // Tasks without due date should follow, sorted by created_at
        assert_eq!(refs[1].title, "No due date 1");
        assert_eq!(refs[2].title, "No due date 2");
    }
}
