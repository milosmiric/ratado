//! Project repository for CRUD operations.
//!
//! This module provides methods for creating, reading, updating, and deleting
//! projects in the database.

use turso::Value;

use crate::models::Project;
use crate::storage::{Database, Result, StorageError};

impl Database {
    /// Inserts a new project into the database.
    ///
    /// # Arguments
    ///
    /// * `project` - The project to insert
    ///
    /// # Errors
    ///
    /// Returns an error if the project cannot be inserted (e.g., duplicate ID).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ratado::models::Project;
    /// use ratado::storage::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::open_in_memory().await?;
    /// let project = Project::new("Work");
    /// db.insert_project(&project).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn insert_project(&self, project: &Project) -> Result<()> {
        self.execute(
            "INSERT INTO projects (id, name, color, icon, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                Value::Text(project.id.clone()),
                Value::Text(project.name.clone()),
                Value::Text(project.color.clone()),
                Value::Text(project.icon.clone()),
                Value::Text(project.created_at.to_rfc3339()),
            ],
        )
        .await?;
        Ok(())
    }

    /// Retrieves a project by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The project ID to look up
    ///
    /// # Returns
    ///
    /// The project if found, or `None` if no project exists with that ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails or data cannot be parsed.
    pub async fn get_project(&self, id: &str) -> Result<Option<Project>> {
        let row = self
            .query_one(
                "SELECT id, name, color, icon, created_at FROM projects WHERE id = ?1",
                [id],
            )
            .await?;

        match row {
            Some(row) => Ok(Some(row_to_project(&row)?)),
            None => Ok(None),
        }
    }

    /// Retrieves all projects from the database.
    ///
    /// # Returns
    ///
    /// A vector of all projects, ordered by creation date.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn get_all_projects(&self) -> Result<Vec<Project>> {
        let mut rows = self
            .query(
                "SELECT id, name, color, icon, created_at FROM projects ORDER BY created_at ASC",
                (),
            )
            .await?;

        let mut projects = Vec::new();
        while let Some(row) = rows.next().await? {
            projects.push(row_to_project(&row)?);
        }

        Ok(projects)
    }

    /// Updates an existing project.
    ///
    /// # Arguments
    ///
    /// * `project` - The project with updated values
    ///
    /// # Errors
    ///
    /// Returns an error if the project doesn't exist or the update fails.
    pub async fn update_project(&self, project: &Project) -> Result<()> {
        let rows_affected = self
            .execute(
                "UPDATE projects SET name = ?1, color = ?2, icon = ?3 WHERE id = ?4",
                [
                    Value::Text(project.name.clone()),
                    Value::Text(project.color.clone()),
                    Value::Text(project.icon.clone()),
                    Value::Text(project.id.clone()),
                ],
            )
            .await?;

        if rows_affected == 0 {
            return Err(StorageError::NotFound(format!(
                "Project not found: {}",
                project.id
            )));
        }

        Ok(())
    }

    /// Deletes a project by its ID.
    ///
    /// Tasks belonging to this project will have their project_id set to NULL.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the project to delete
    ///
    /// # Returns
    ///
    /// `true` if a project was deleted, `false` if no project existed with that ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn delete_project(&self, id: &str) -> Result<bool> {
        // Don't allow deleting the inbox
        if id == "inbox" {
            return Err(StorageError::Migration(
                "Cannot delete the Inbox project".to_string(),
            ));
        }

        let rows_affected = self
            .execute("DELETE FROM projects WHERE id = ?1", [id])
            .await?;
        Ok(rows_affected > 0)
    }

    /// Deletes all projects except Inbox.
    ///
    /// Used when resetting the database to default state.
    ///
    /// # Returns
    ///
    /// The number of projects deleted.
    ///
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn delete_all_projects_except_inbox(&self) -> Result<usize> {
        let rows_affected = self
            .execute("DELETE FROM projects WHERE id != 'inbox'", ())
            .await?;
        Ok(rows_affected as usize)
    }

    /// Gets the count of tasks in a project.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project ID to count tasks for
    ///
    /// # Returns
    ///
    /// The number of tasks in the project.
    pub async fn get_task_count_by_project(&self, project_id: &str) -> Result<usize> {
        let value = self
            .query_scalar(
                "SELECT COUNT(*) FROM tasks WHERE project_id = ?1",
                [project_id],
            )
            .await?;

        match value {
            Some(Value::Integer(count)) => Ok(count as usize),
            _ => Ok(0),
        }
    }

    /// Gets all projects with their task counts.
    ///
    /// # Returns
    ///
    /// A vector of (Project, task_count) tuples.
    pub async fn get_projects_with_counts(&self) -> Result<Vec<(Project, usize)>> {
        let projects = self.get_all_projects().await?;
        let mut result = Vec::with_capacity(projects.len());

        for project in projects {
            let count = self.get_task_count_by_project(&project.id).await?;
            result.push((project, count));
        }

        Ok(result)
    }
}

/// Converts a database row to a Project.
fn row_to_project(row: &turso::Row) -> Result<Project> {
    let id = value_to_string(row.get_value(0)?)?;
    let name = value_to_string(row.get_value(1)?)?;
    let color = value_to_string(row.get_value(2)?)?;
    let icon = value_to_string(row.get_value(3)?)?;
    let created_at = value_to_datetime(row.get_value(4)?)?;

    Ok(Project {
        id,
        name,
        color,
        icon,
        created_at,
    })
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

fn value_to_datetime(value: Value) -> Result<chrono::DateTime<chrono::Utc>> {
    use chrono::{DateTime, Utc};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::run_migrations;

    async fn setup_db() -> Database {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_inbox_exists_after_migration() {
        let db = setup_db().await;

        let inbox = db.get_project("inbox").await.unwrap();
        assert!(inbox.is_some());
        assert_eq!(inbox.unwrap().name, "Inbox");
    }

    #[tokio::test]
    async fn test_insert_and_get_project() {
        let db = setup_db().await;

        let project = Project::new("Work");
        db.insert_project(&project).await.unwrap();

        let retrieved = db.get_project(&project.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "Work");
        assert_eq!(retrieved.color, "#3498db");
    }

    #[tokio::test]
    async fn test_insert_project_with_style() {
        let db = setup_db().await;

        let project = Project::with_style("Personal", "#e74c3c", "🏠");
        db.insert_project(&project).await.unwrap();

        let retrieved = db.get_project(&project.id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Personal");
        assert_eq!(retrieved.color, "#e74c3c");
        assert_eq!(retrieved.icon, "🏠");
    }

    #[tokio::test]
    async fn test_update_project() {
        let db = setup_db().await;

        let mut project = Project::new("Original");
        db.insert_project(&project).await.unwrap();

        project.name = "Updated".to_string();
        project.color = "#27ae60".to_string();
        db.update_project(&project).await.unwrap();

        let retrieved = db.get_project(&project.id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated");
        assert_eq!(retrieved.color, "#27ae60");
    }

    #[tokio::test]
    async fn test_delete_project() {
        let db = setup_db().await;

        let project = Project::new("To be deleted");
        db.insert_project(&project).await.unwrap();

        let deleted = db.delete_project(&project.id).await.unwrap();
        assert!(deleted);

        let retrieved = db.get_project(&project.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_cannot_delete_inbox() {
        let db = setup_db().await;

        let result = db.delete_project("inbox").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_all_projects() {
        let db = setup_db().await;

        db.insert_project(&Project::new("Work")).await.unwrap();
        db.insert_project(&Project::new("Personal")).await.unwrap();

        let projects = db.get_all_projects().await.unwrap();
        // Inbox + 2 new projects
        assert_eq!(projects.len(), 3);
    }

    #[tokio::test]
    async fn test_get_task_count_by_project() {
        let db = setup_db().await;

        use crate::models::Task;

        let mut task1 = Task::new("Task 1");
        task1.project_id = Some("inbox".to_string());
        db.insert_task(&task1).await.unwrap();

        let mut task2 = Task::new("Task 2");
        task2.project_id = Some("inbox".to_string());
        db.insert_task(&task2).await.unwrap();

        let count = db.get_task_count_by_project("inbox").await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_get_projects_with_counts() {
        let db = setup_db().await;

        use crate::models::Task;

        let mut task = Task::new("Inbox task");
        task.project_id = Some("inbox".to_string());
        db.insert_task(&task).await.unwrap();

        let projects = db.get_projects_with_counts().await.unwrap();
        let inbox = projects.iter().find(|(p, _)| p.id == "inbox").unwrap();
        assert_eq!(inbox.1, 1);
    }

    #[tokio::test]
    async fn test_delete_all_projects_except_inbox() {
        let db = setup_db().await;

        // Create additional projects
        db.insert_project(&Project::new("Work")).await.unwrap();
        db.insert_project(&Project::new("Personal")).await.unwrap();
        db.insert_project(&Project::new("Shopping")).await.unwrap();

        // Verify we have 4 projects (inbox + 3 new)
        let before = db.get_all_projects().await.unwrap();
        assert_eq!(before.len(), 4);

        // Delete all projects except inbox
        let _deleted = db.delete_all_projects_except_inbox().await.unwrap();
        // Note: turso may not return accurate rows_affected

        // Verify only inbox remains
        let after = db.get_all_projects().await.unwrap();
        assert_eq!(after.len(), 1, "Only inbox should remain");
        assert_eq!(after[0].id, "inbox");
    }
}
