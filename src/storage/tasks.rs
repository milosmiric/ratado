//! Task repository for CRUD operations.
//!
//! This module provides methods for creating, reading, updating, and deleting
//! tasks in the database. All datetime values are stored as ISO8601 strings.

use chrono::{DateTime, Utc};
use turso::Value;

use crate::models::{Filter, Priority, SortOrder, Task, TaskStatus};
use crate::storage::{Database, Result, StorageError};

impl Database {
    /// Inserts a new task into the database.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to insert
    ///
    /// # Errors
    ///
    /// Returns an error if the task cannot be inserted (e.g., duplicate ID).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ratado::models::Task;
    /// use ratado::storage::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::open_in_memory().await?;
    /// let task = Task::new("Buy groceries");
    /// db.insert_task(&task).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn insert_task(&self, task: &Task) -> Result<()> {
        self.execute(
            "INSERT INTO tasks (id, title, description, due_date, priority,
             status, project_id, created_at, updated_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            [
                Value::Text(task.id.clone()),
                Value::Text(task.title.clone()),
                option_to_value(&task.description),
                task.due_date.map(|d| Value::Text(d.to_rfc3339())).unwrap_or(Value::Null),
                Value::Text(priority_to_str(task.priority).to_string()),
                Value::Text(status_to_str(task.status).to_string()),
                option_to_value(&task.project_id),
                Value::Text(task.created_at.to_rfc3339()),
                Value::Text(task.updated_at.to_rfc3339()),
                task.completed_at.map(|d| Value::Text(d.to_rfc3339())).unwrap_or(Value::Null),
            ],
        )
        .await?;

        // Insert tags
        for tag in &task.tags {
            self.add_tag_to_task(&task.id, tag).await?;
        }

        Ok(())
    }

    /// Retrieves a task by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The task ID to look up
    ///
    /// # Returns
    ///
    /// The task if found, or `None` if no task exists with that ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails or data cannot be parsed.
    pub async fn get_task(&self, id: &str) -> Result<Option<Task>> {
        let row = self
            .query_one(
                "SELECT id, title, description, due_date, priority, status,
                 project_id, created_at, updated_at, completed_at
                 FROM tasks WHERE id = ?1",
                [id],
            )
            .await?;

        match row {
            Some(row) => {
                let mut task = row_to_task(&row)?;
                task.tags = self.get_tags_for_task(&task.id).await?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    /// Retrieves all tasks from the database.
    ///
    /// # Returns
    ///
    /// A vector of all tasks, including their tags.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn get_all_tasks(&self) -> Result<Vec<Task>> {
        let mut rows = self
            .query(
                "SELECT id, title, description, due_date, priority, status,
                 project_id, created_at, updated_at, completed_at
                 FROM tasks ORDER BY created_at DESC",
                (),
            )
            .await?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next().await? {
            let mut task = row_to_task(&row)?;
            task.tags = self.get_tags_for_task(&task.id).await?;
            tasks.push(task);
        }

        Ok(tasks)
    }

    /// Updates an existing task.
    ///
    /// Also cleans up any orphaned tags (tags no longer associated with any tasks)
    /// after updating the task's tags.
    ///
    /// # Arguments
    ///
    /// * `task` - The task with updated values
    ///
    /// # Errors
    ///
    /// Returns an error if the task doesn't exist or the update fails.
    pub async fn update_task(&self, task: &Task) -> Result<()> {
        let rows_affected = self
            .execute(
                "UPDATE tasks SET
                 title = ?1, description = ?2, due_date = ?3, priority = ?4,
                 status = ?5, project_id = ?6, updated_at = ?7, completed_at = ?8
                 WHERE id = ?9",
                [
                    Value::Text(task.title.clone()),
                    option_to_value(&task.description),
                    task.due_date.map(|d| Value::Text(d.to_rfc3339())).unwrap_or(Value::Null),
                    Value::Text(priority_to_str(task.priority).to_string()),
                    Value::Text(status_to_str(task.status).to_string()),
                    option_to_value(&task.project_id),
                    Value::Text(task.updated_at.to_rfc3339()),
                    task.completed_at.map(|d| Value::Text(d.to_rfc3339())).unwrap_or(Value::Null),
                    Value::Text(task.id.clone()),
                ],
            )
            .await?;

        if rows_affected == 0 {
            return Err(StorageError::NotFound(format!("Task not found: {}", task.id)));
        }

        // Update tags: remove all existing, add current
        self.execute("DELETE FROM task_tags WHERE task_id = ?1", [task.id.as_str()])
            .await?;
        for tag in &task.tags {
            self.add_tag_to_task(&task.id, tag).await?;
        }

        // Clean up any tags that are no longer associated with any tasks
        self.cleanup_orphaned_tags().await?;

        Ok(())
    }

    /// Deletes a task by its ID.
    ///
    /// Also cleans up any orphaned tags (tags no longer associated with any tasks).
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the task to delete
    ///
    /// # Returns
    ///
    /// `true` if a task was deleted, `false` if no task existed with that ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn delete_task(&self, id: &str) -> Result<bool> {
        // Tags associations are deleted automatically via ON DELETE CASCADE
        let rows_affected = self
            .execute("DELETE FROM tasks WHERE id = ?1", [id])
            .await?;

        // Clean up any tags that are no longer associated with any tasks
        self.cleanup_orphaned_tags().await?;

        Ok(rows_affected > 0)
    }

    /// Deletes all tasks belonging to a project.
    ///
    /// Also cleans up any orphaned tags (tags no longer associated with any tasks).
    ///
    /// # Arguments
    ///
    /// * `project_id` - The ID of the project whose tasks should be deleted
    ///
    /// # Returns
    ///
    /// The number of tasks deleted.
    ///
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn delete_tasks_by_project(&self, project_id: &str) -> Result<usize> {
        // Tags associations are deleted automatically via ON DELETE CASCADE
        let rows_affected = self
            .execute("DELETE FROM tasks WHERE project_id = ?1", [project_id])
            .await?;

        // Clean up any tags that are no longer associated with any tasks
        self.cleanup_orphaned_tags().await?;

        Ok(rows_affected as usize)
    }

    /// Moves all tasks from a project to Inbox.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The ID of the project whose tasks should be moved
    ///
    /// # Returns
    ///
    /// The number of tasks moved.
    ///
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub async fn move_tasks_to_inbox(&self, project_id: &str) -> Result<usize> {
        let rows_affected = self
            .execute(
                "UPDATE tasks SET project_id = 'inbox', updated_at = ?1 WHERE project_id = ?2",
                [Value::Text(chrono::Utc::now().to_rfc3339()), Value::Text(project_id.to_string())],
            )
            .await?;
        Ok(rows_affected as usize)
    }

    /// Deletes all completed tasks from the database.
    ///
    /// Also cleans up any orphaned tags (tags no longer associated with any tasks).
    ///
    /// # Returns
    ///
    /// The number of tasks deleted.
    ///
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn delete_completed_tasks(&self) -> Result<usize> {
        // Tags associations are deleted automatically via ON DELETE CASCADE
        let rows_affected = self
            .execute("DELETE FROM tasks WHERE status = 'completed'", ())
            .await?;

        // Clean up any tags that are no longer associated with any tasks
        self.cleanup_orphaned_tags().await?;

        Ok(rows_affected as usize)
    }

    /// Resets the database by deleting all tasks.
    ///
    /// Also cleans up all tags.
    ///
    /// # Returns
    ///
    /// The number of tasks deleted.
    ///
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn delete_all_tasks(&self) -> Result<usize> {
        // Tags associations are deleted automatically via ON DELETE CASCADE
        let rows_affected = self
            .execute("DELETE FROM tasks", ())
            .await?;

        // Clean up all orphaned tags
        self.cleanup_orphaned_tags().await?;

        Ok(rows_affected as usize)
    }

    /// Queries tasks with filtering and sorting.
    ///
    /// # Arguments
    ///
    /// * `filter` - Filter criteria to apply
    /// * `sort` - Sort order for results
    ///
    /// # Returns
    ///
    /// A vector of tasks matching the filter, sorted as specified.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn query_tasks(&self, filter: &Filter, sort: &SortOrder) -> Result<Vec<Task>> {
        // Build the WHERE clause based on filter
        let (where_clause, params) = build_filter_clause(filter);
        let order_clause = build_order_clause(sort);

        let sql = format!(
            "SELECT id, title, description, due_date, priority, status,
             project_id, created_at, updated_at, completed_at
             FROM tasks {} {}",
            where_clause, order_clause
        );

        let mut rows = self.query(&sql, params).await?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next().await? {
            let mut task = row_to_task(&row)?;
            task.tags = self.get_tags_for_task(&task.id).await?;
            tasks.push(task);
        }

        // Apply in-memory filters that can't be done in SQL
        let tasks = match filter {
            Filter::DueToday | Filter::DueThisWeek | Filter::Overdue => {
                tasks.into_iter().filter(|t| filter.matches(t)).collect()
            }
            Filter::ByTag(tag) => tasks
                .into_iter()
                .filter(|t| t.tags.contains(tag))
                .collect(),
            _ => tasks,
        };

        Ok(tasks)
    }

    /// Gets the count of tasks by status.
    ///
    /// # Returns
    ///
    /// A tuple of (pending, in_progress, completed, archived) counts.
    pub async fn get_task_counts(&self) -> Result<(usize, usize, usize, usize)> {
        let pending = self
            .query_scalar(
                "SELECT COUNT(*) FROM tasks WHERE status = 'pending'",
                (),
            )
            .await?
            .map(|v| value_to_i64(&v).unwrap_or(0) as usize)
            .unwrap_or(0);

        let in_progress = self
            .query_scalar(
                "SELECT COUNT(*) FROM tasks WHERE status = 'in_progress'",
                (),
            )
            .await?
            .map(|v| value_to_i64(&v).unwrap_or(0) as usize)
            .unwrap_or(0);

        let completed = self
            .query_scalar(
                "SELECT COUNT(*) FROM tasks WHERE status = 'completed'",
                (),
            )
            .await?
            .map(|v| value_to_i64(&v).unwrap_or(0) as usize)
            .unwrap_or(0);

        let archived = self
            .query_scalar(
                "SELECT COUNT(*) FROM tasks WHERE status = 'archived'",
                (),
            )
            .await?
            .map(|v| value_to_i64(&v).unwrap_or(0) as usize)
            .unwrap_or(0);

        Ok((pending, in_progress, completed, archived))
    }
}

/// Converts a database row to a Task.
fn row_to_task(row: &turso::Row) -> Result<Task> {
    let id = value_to_string(row.get_value(0)?)?;
    let title = value_to_string(row.get_value(1)?)?;
    let description = value_to_option_string(row.get_value(2)?)?;
    let due_date = value_to_option_datetime(row.get_value(3)?)?;
    let priority = str_to_priority(&value_to_string(row.get_value(4)?)?);
    let status = str_to_status(&value_to_string(row.get_value(5)?)?);
    let project_id = value_to_option_string(row.get_value(6)?)?;
    let created_at = value_to_datetime(row.get_value(7)?)?;
    let updated_at = value_to_datetime(row.get_value(8)?)?;
    let completed_at = value_to_option_datetime(row.get_value(9)?)?;

    Ok(Task {
        id,
        title,
        description,
        due_date,
        priority,
        status,
        project_id,
        tags: Vec::new(), // Tags are loaded separately
        created_at,
        updated_at,
        completed_at,
    })
}

/// Builds a WHERE clause for the given filter.
fn build_filter_clause(filter: &Filter) -> (String, Vec<Value>) {
    match filter {
        Filter::All => (String::new(), vec![]),
        Filter::Pending => ("WHERE status = 'pending'".to_string(), vec![]),
        Filter::InProgress => ("WHERE status = 'in_progress'".to_string(), vec![]),
        Filter::Completed => ("WHERE status = 'completed'".to_string(), vec![]),
        Filter::Archived => ("WHERE status = 'archived'".to_string(), vec![]),
        Filter::ByProject(project_id) => (
            "WHERE project_id = ?1".to_string(),
            vec![Value::Text(project_id.clone())],
        ),
        Filter::ByPriority(priority) => (
            "WHERE priority = ?1".to_string(),
            vec![Value::Text(priority_to_str(*priority).to_string())],
        ),
        // These filters need in-memory filtering after loading
        Filter::DueToday | Filter::DueThisWeek | Filter::Overdue => {
            ("WHERE due_date IS NOT NULL AND status NOT IN ('completed', 'archived')".to_string(), vec![])
        }
        Filter::ByTag(_) => (String::new(), vec![]), // Filtered in-memory after loading tags
    }
}

/// Builds an ORDER BY clause for the given sort order.
fn build_order_clause(sort: &SortOrder) -> &'static str {
    match sort {
        SortOrder::DueDateAsc => "ORDER BY due_date ASC NULLS LAST",
        SortOrder::DueDateDesc => "ORDER BY due_date DESC NULLS FIRST",
        SortOrder::PriorityDesc => "ORDER BY CASE priority
            WHEN 'urgent' THEN 0
            WHEN 'high' THEN 1
            WHEN 'medium' THEN 2
            WHEN 'low' THEN 3
            END ASC",
        SortOrder::PriorityAsc => "ORDER BY CASE priority
            WHEN 'low' THEN 0
            WHEN 'medium' THEN 1
            WHEN 'high' THEN 2
            WHEN 'urgent' THEN 3
            END ASC",
        SortOrder::CreatedDesc => "ORDER BY created_at DESC",
        SortOrder::CreatedAsc => "ORDER BY created_at ASC",
        SortOrder::Alphabetical => "ORDER BY title ASC",
    }
}

// Helper functions for type conversions

fn option_to_value(opt: &Option<String>) -> Value {
    match opt {
        Some(s) => Value::Text(s.clone()),
        None => Value::Null,
    }
}

fn priority_to_str(priority: Priority) -> &'static str {
    match priority {
        Priority::Low => "low",
        Priority::Medium => "medium",
        Priority::High => "high",
        Priority::Urgent => "urgent",
    }
}

fn str_to_priority(s: &str) -> Priority {
    match s.to_lowercase().as_str() {
        "low" => Priority::Low,
        "high" => Priority::High,
        "urgent" => Priority::Urgent,
        _ => Priority::Medium,
    }
}

fn status_to_str(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Pending => "pending",
        TaskStatus::InProgress => "in_progress",
        TaskStatus::Completed => "completed",
        TaskStatus::Archived => "archived",
    }
}

fn str_to_status(s: &str) -> TaskStatus {
    match s.to_lowercase().as_str() {
        "in_progress" => TaskStatus::InProgress,
        "completed" => TaskStatus::Completed,
        "archived" => TaskStatus::Archived,
        _ => TaskStatus::Pending,
    }
}

fn value_to_string(value: Value) -> Result<String> {
    match value {
        Value::Text(s) => Ok(s),
        _ => Err(StorageError::Conversion(format!(
            "Expected text, got {:?}",
            value
        ))),
    }
}

fn value_to_option_string(value: Value) -> Result<Option<String>> {
    match value {
        Value::Text(s) => Ok(Some(s)),
        Value::Null => Ok(None),
        _ => Err(StorageError::Conversion(format!(
            "Expected text or null, got {:?}",
            value
        ))),
    }
}

fn value_to_datetime(value: Value) -> Result<DateTime<Utc>> {
    match value {
        Value::Text(s) => DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| StorageError::Conversion(format!("Invalid datetime '{}': {}", s, e))),
        _ => Err(StorageError::Conversion(format!(
            "Expected datetime text, got {:?}",
            value
        ))),
    }
}

fn value_to_option_datetime(value: Value) -> Result<Option<DateTime<Utc>>> {
    match value {
        Value::Text(s) => {
            let dt = DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| StorageError::Conversion(format!("Invalid datetime '{}': {}", s, e)))?;
            Ok(Some(dt))
        }
        Value::Null => Ok(None),
        _ => Err(StorageError::Conversion(format!(
            "Expected datetime text or null, got {:?}",
            value
        ))),
    }
}

fn value_to_i64(value: &Value) -> Result<i64> {
    match value {
        Value::Integer(i) => Ok(*i),
        _ => Err(StorageError::Conversion(format!(
            "Expected integer, got {:?}",
            value
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::run_migrations;
    use chrono::Duration;

    async fn setup_db() -> Database {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_insert_and_get_task() {
        let db = setup_db().await;

        let task = Task::new("Test task");
        db.insert_task(&task).await.unwrap();

        let retrieved = db.get_task(&task.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.title, "Test task");
        assert_eq!(retrieved.priority, Priority::Medium);
        assert_eq!(retrieved.status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_update_task() {
        let db = setup_db().await;

        let mut task = Task::new("Original");
        db.insert_task(&task).await.unwrap();

        task.title = "Updated".to_string();
        task.priority = Priority::High;
        task.updated_at = Utc::now();
        db.update_task(&task).await.unwrap();

        let retrieved = db.get_task(&task.id).await.unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated");
        assert_eq!(retrieved.priority, Priority::High);
    }

    #[tokio::test]
    async fn test_delete_task() {
        let db = setup_db().await;

        let task = Task::new("To be deleted");
        db.insert_task(&task).await.unwrap();

        let deleted = db.delete_task(&task.id).await.unwrap();
        assert!(deleted);

        let retrieved = db.get_task(&task.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_get_all_tasks() {
        let db = setup_db().await;

        db.insert_task(&Task::new("Task 1")).await.unwrap();
        db.insert_task(&Task::new("Task 2")).await.unwrap();
        db.insert_task(&Task::new("Task 3")).await.unwrap();

        let tasks = db.get_all_tasks().await.unwrap();
        assert_eq!(tasks.len(), 3);
    }

    #[tokio::test]
    async fn test_query_tasks_by_status() {
        let db = setup_db().await;

        let task1 = Task::new("Pending task");
        db.insert_task(&task1).await.unwrap();

        let mut task2 = Task::new("Completed task");
        task2.complete();
        db.insert_task(&task2).await.unwrap();

        let pending = db
            .query_tasks(&Filter::Pending, &SortOrder::CreatedDesc)
            .await
            .unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].title, "Pending task");

        let completed = db
            .query_tasks(&Filter::Completed, &SortOrder::CreatedDesc)
            .await
            .unwrap();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].title, "Completed task");
    }

    #[tokio::test]
    async fn test_query_tasks_by_priority() {
        let db = setup_db().await;

        let mut task1 = Task::new("Low priority");
        task1.priority = Priority::Low;
        db.insert_task(&task1).await.unwrap();

        let mut task2 = Task::new("High priority");
        task2.priority = Priority::High;
        db.insert_task(&task2).await.unwrap();

        let high = db
            .query_tasks(&Filter::ByPriority(Priority::High), &SortOrder::CreatedDesc)
            .await
            .unwrap();
        assert_eq!(high.len(), 1);
        assert_eq!(high[0].title, "High priority");
    }

    #[tokio::test]
    async fn test_query_tasks_sorted() {
        let db = setup_db().await;

        let mut task_a = Task::new("Alpha");
        task_a.priority = Priority::Low;
        db.insert_task(&task_a).await.unwrap();

        let mut task_b = Task::new("Beta");
        task_b.priority = Priority::High;
        db.insert_task(&task_b).await.unwrap();

        // Sort by priority descending (high first)
        let tasks = db
            .query_tasks(&Filter::All, &SortOrder::PriorityDesc)
            .await
            .unwrap();
        assert_eq!(tasks[0].title, "Beta"); // High priority first

        // Sort alphabetically
        let tasks = db
            .query_tasks(&Filter::All, &SortOrder::Alphabetical)
            .await
            .unwrap();
        assert_eq!(tasks[0].title, "Alpha");
    }

    #[tokio::test]
    async fn test_task_with_project() {
        let db = setup_db().await;

        let mut task = Task::new("Project task");
        task.project_id = Some("inbox".to_string());
        db.insert_task(&task).await.unwrap();

        let tasks = db
            .query_tasks(&Filter::ByProject("inbox".to_string()), &SortOrder::CreatedDesc)
            .await
            .unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].project_id, Some("inbox".to_string()));
    }

    #[tokio::test]
    async fn test_task_with_due_date() {
        let db = setup_db().await;

        let mut task = Task::new("Due task");
        task.due_date = Some(Utc::now() + Duration::days(1));
        db.insert_task(&task).await.unwrap();

        let retrieved = db.get_task(&task.id).await.unwrap().unwrap();
        assert!(retrieved.due_date.is_some());
    }

    #[tokio::test]
    async fn test_get_task_counts() {
        let db = setup_db().await;

        db.insert_task(&Task::new("Pending 1")).await.unwrap();
        db.insert_task(&Task::new("Pending 2")).await.unwrap();

        let mut completed = Task::new("Completed");
        completed.complete();
        db.insert_task(&completed).await.unwrap();

        let (pending, in_progress, completed_count, archived) = db.get_task_counts().await.unwrap();
        assert_eq!(pending, 2);
        assert_eq!(in_progress, 0);
        assert_eq!(completed_count, 1);
        assert_eq!(archived, 0);
    }
}
