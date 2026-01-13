//! Tag repository for CRUD operations.
//!
//! This module provides methods for managing tags and their associations
//! with tasks. Tags are stored by name and linked to tasks via a junction table.

use turso::Value;
use uuid::Uuid;

use crate::storage::{Database, Result, StorageError};

/// A tag for categorizing tasks.
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    /// Unique identifier
    pub id: String,
    /// Tag name (unique)
    pub name: String,
}

impl Database {
    /// Creates a new tag with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name for the new tag
    ///
    /// # Returns
    ///
    /// The ID of the created tag.
    ///
    /// # Errors
    ///
    /// Returns an error if a tag with that name already exists.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ratado::storage::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::open_in_memory().await?;
    /// let tag_id = db.insert_tag("urgent").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn insert_tag(&self, name: &str) -> Result<String> {
        let id = Uuid::now_v7().to_string();
        self.execute(
            "INSERT INTO tags (id, name) VALUES (?1, ?2)",
            [Value::Text(id.clone()), Value::Text(name.to_string())],
        )
        .await?;
        Ok(id)
    }

    /// Gets an existing tag by name, or creates it if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `name` - The tag name to look up or create
    ///
    /// # Returns
    ///
    /// The ID of the existing or newly created tag.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get_or_create_tag(&self, name: &str) -> Result<String> {
        // Try to find existing tag
        if let Some(tag) = self.get_tag_by_name(name).await? {
            return Ok(tag.id);
        }

        // Create new tag
        self.insert_tag(name).await
    }

    /// Retrieves a tag by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The tag name to look up
    ///
    /// # Returns
    ///
    /// The tag if found, or `None` if no tag exists with that name.
    pub async fn get_tag_by_name(&self, name: &str) -> Result<Option<Tag>> {
        let row = self
            .query_one("SELECT id, name FROM tags WHERE name = ?1", [name])
            .await?;

        match row {
            Some(row) => Ok(Some(row_to_tag(&row)?)),
            None => Ok(None),
        }
    }

    /// Retrieves a tag by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The tag ID to look up
    ///
    /// # Returns
    ///
    /// The tag if found, or `None` if no tag exists with that ID.
    pub async fn get_tag(&self, id: &str) -> Result<Option<Tag>> {
        let row = self
            .query_one("SELECT id, name FROM tags WHERE id = ?1", [id])
            .await?;

        match row {
            Some(row) => Ok(Some(row_to_tag(&row)?)),
            None => Ok(None),
        }
    }

    /// Retrieves all tags from the database.
    ///
    /// # Returns
    ///
    /// A vector of all tags, ordered alphabetically by name.
    pub async fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let mut rows = self
            .query("SELECT id, name FROM tags ORDER BY name ASC", ())
            .await?;

        let mut tags = Vec::new();
        while let Some(row) = rows.next().await? {
            tags.push(row_to_tag(&row)?);
        }

        Ok(tags)
    }

    /// Deletes a tag by its ID.
    ///
    /// This also removes the tag from all tasks (via CASCADE).
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the tag to delete
    ///
    /// # Returns
    ///
    /// `true` if a tag was deleted, `false` if no tag existed with that ID.
    pub async fn delete_tag(&self, id: &str) -> Result<bool> {
        let rows_affected = self
            .execute("DELETE FROM tags WHERE id = ?1", [id])
            .await?;
        Ok(rows_affected > 0)
    }

    /// Adds a tag to a task.
    ///
    /// If the tag name doesn't exist, it will be created. If the task
    /// already has this tag, this operation is a no-op.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task to tag
    /// * `tag_name` - The tag name to add
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn add_tag_to_task(&self, task_id: &str, tag_name: &str) -> Result<()> {
        let tag_id = self.get_or_create_tag(tag_name).await?;

        // Use INSERT OR IGNORE to handle duplicates gracefully
        self.execute(
            "INSERT OR IGNORE INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
            [Value::Text(task_id.to_string()), Value::Text(tag_id)],
        )
        .await?;

        Ok(())
    }

    /// Removes a tag from a task.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task to remove the tag from
    /// * `tag_id` - The ID of the tag to remove
    ///
    /// # Returns
    ///
    /// `true` if the tag was removed, `false` if the task didn't have that tag.
    pub async fn remove_tag_from_task(&self, task_id: &str, tag_id: &str) -> Result<bool> {
        let rows_affected = self
            .execute(
                "DELETE FROM task_tags WHERE task_id = ?1 AND tag_id = ?2",
                [task_id, tag_id],
            )
            .await?;
        Ok(rows_affected > 0)
    }

    /// Gets all tag names for a task.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task to get tags for
    ///
    /// # Returns
    ///
    /// A vector of tag names associated with the task.
    pub async fn get_tags_for_task(&self, task_id: &str) -> Result<Vec<String>> {
        let mut rows = self
            .query(
                "SELECT t.name FROM tags t
                 JOIN task_tags tt ON t.id = tt.tag_id
                 WHERE tt.task_id = ?1
                 ORDER BY t.name ASC",
                [task_id],
            )
            .await?;

        let mut tags = Vec::new();
        while let Some(row) = rows.next().await? {
            let name = value_to_string(row.get_value(0)?)?;
            tags.push(name);
        }

        Ok(tags)
    }

    /// Gets the count of tasks with a specific tag.
    ///
    /// # Arguments
    ///
    /// * `tag_id` - The tag ID to count tasks for
    ///
    /// # Returns
    ///
    /// The number of tasks with this tag.
    pub async fn get_task_count_by_tag(&self, tag_id: &str) -> Result<usize> {
        let value = self
            .query_scalar(
                "SELECT COUNT(*) FROM task_tags WHERE tag_id = ?1",
                [tag_id],
            )
            .await?;

        match value {
            Some(Value::Integer(count)) => Ok(count as usize),
            _ => Ok(0),
        }
    }

    /// Gets all tags with their task counts.
    ///
    /// # Returns
    ///
    /// A vector of (Tag, task_count) tuples.
    pub async fn get_tags_with_counts(&self) -> Result<Vec<(Tag, usize)>> {
        let mut rows = self
            .query(
                "SELECT t.id, t.name, COUNT(tt.task_id) as count
                 FROM tags t
                 LEFT JOIN task_tags tt ON t.id = tt.tag_id
                 GROUP BY t.id, t.name
                 ORDER BY t.name ASC",
                (),
            )
            .await?;

        let mut result = Vec::new();
        while let Some(row) = rows.next().await? {
            let id = value_to_string(row.get_value(0)?)?;
            let name = value_to_string(row.get_value(1)?)?;
            let count = match row.get_value(2)? {
                Value::Integer(c) => c as usize,
                _ => 0,
            };
            result.push((Tag { id, name }, count));
        }

        Ok(result)
    }
}

/// Converts a database row to a Tag.
fn row_to_tag(row: &turso::Row) -> Result<Tag> {
    let id = value_to_string(row.get_value(0)?)?;
    let name = value_to_string(row.get_value(1)?)?;
    Ok(Tag { id, name })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Task;
    use crate::storage::run_migrations;

    async fn setup_db() -> Database {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_insert_and_get_tag() {
        let db = setup_db().await;

        let id = db.insert_tag("urgent").await.unwrap();
        let tag = db.get_tag(&id).await.unwrap();

        assert!(tag.is_some());
        assert_eq!(tag.unwrap().name, "urgent");
    }

    #[tokio::test]
    async fn test_get_tag_by_name() {
        let db = setup_db().await;

        db.insert_tag("important").await.unwrap();
        let tag = db.get_tag_by_name("important").await.unwrap();

        assert!(tag.is_some());
        assert_eq!(tag.unwrap().name, "important");
    }

    #[tokio::test]
    async fn test_get_or_create_tag() {
        let db = setup_db().await;

        // First call creates the tag
        let id1 = db.get_or_create_tag("newbie").await.unwrap();

        // Second call returns the same tag
        let id2 = db.get_or_create_tag("newbie").await.unwrap();

        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_delete_tag() {
        let db = setup_db().await;

        let id = db.insert_tag("to_delete").await.unwrap();
        let deleted = db.delete_tag(&id).await.unwrap();

        assert!(deleted);
        assert!(db.get_tag(&id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_all_tags() {
        let db = setup_db().await;

        db.insert_tag("zebra").await.unwrap();
        db.insert_tag("alpha").await.unwrap();
        db.insert_tag("beta").await.unwrap();

        let tags = db.get_all_tags().await.unwrap();

        assert_eq!(tags.len(), 3);
        // Should be sorted alphabetically
        assert_eq!(tags[0].name, "alpha");
        assert_eq!(tags[1].name, "beta");
        assert_eq!(tags[2].name, "zebra");
    }

    #[tokio::test]
    async fn test_add_tag_to_task() {
        let db = setup_db().await;

        let task = Task::new("Tagged task");
        db.insert_task(&task).await.unwrap();

        db.add_tag_to_task(&task.id, "important").await.unwrap();
        db.add_tag_to_task(&task.id, "work").await.unwrap();

        let tags = db.get_tags_for_task(&task.id).await.unwrap();

        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"important".to_string()));
        assert!(tags.contains(&"work".to_string()));
    }

    #[tokio::test]
    async fn test_add_duplicate_tag_to_task() {
        let db = setup_db().await;

        let task = Task::new("Task");
        db.insert_task(&task).await.unwrap();

        db.add_tag_to_task(&task.id, "same").await.unwrap();
        db.add_tag_to_task(&task.id, "same").await.unwrap(); // Duplicate

        let tags = db.get_tags_for_task(&task.id).await.unwrap();
        assert_eq!(tags.len(), 1);
    }

    #[tokio::test]
    async fn test_remove_tag_from_task() {
        let db = setup_db().await;

        let task = Task::new("Task");
        db.insert_task(&task).await.unwrap();

        db.add_tag_to_task(&task.id, "removeme").await.unwrap();

        let tag = db.get_tag_by_name("removeme").await.unwrap().unwrap();
        let removed = db.remove_tag_from_task(&task.id, &tag.id).await.unwrap();

        assert!(removed);
        let tags = db.get_tags_for_task(&task.id).await.unwrap();
        assert!(tags.is_empty());
    }

    #[tokio::test]
    async fn test_get_task_count_by_tag() {
        let db = setup_db().await;

        let tag_id = db.insert_tag("shared").await.unwrap();

        let task1 = Task::new("Task 1");
        db.insert_task(&task1).await.unwrap();
        db.add_tag_to_task(&task1.id, "shared").await.unwrap();

        let task2 = Task::new("Task 2");
        db.insert_task(&task2).await.unwrap();
        db.add_tag_to_task(&task2.id, "shared").await.unwrap();

        let count = db.get_task_count_by_tag(&tag_id).await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_get_tags_with_counts() {
        let db = setup_db().await;

        db.insert_tag("empty").await.unwrap();
        db.insert_tag("used").await.unwrap();

        let task = Task::new("Task");
        db.insert_task(&task).await.unwrap();
        db.add_tag_to_task(&task.id, "used").await.unwrap();

        let tags = db.get_tags_with_counts().await.unwrap();

        let empty = tags.iter().find(|(t, _)| t.name == "empty").unwrap();
        assert_eq!(empty.1, 0);

        let used = tags.iter().find(|(t, _)| t.name == "used").unwrap();
        assert_eq!(used.1, 1);
    }

    #[tokio::test]
    async fn test_deleting_tag_removes_from_tasks() {
        let db = setup_db().await;

        let task = Task::new("Task");
        db.insert_task(&task).await.unwrap();

        db.add_tag_to_task(&task.id, "deleteme").await.unwrap();
        let tag = db.get_tag_by_name("deleteme").await.unwrap().unwrap();

        db.delete_tag(&tag.id).await.unwrap();

        let tags = db.get_tags_for_task(&task.id).await.unwrap();
        assert!(tags.is_empty());
    }

    #[tokio::test]
    async fn test_task_with_tags_persists() {
        let db = setup_db().await;

        let mut task = Task::new("Task with tags");
        task.tags = vec!["tag1".to_string(), "tag2".to_string()];
        db.insert_task(&task).await.unwrap();

        let retrieved = db.get_task(&task.id).await.unwrap().unwrap();
        assert_eq!(retrieved.tags.len(), 2);
        assert!(retrieved.tags.contains(&"tag1".to_string()));
        assert!(retrieved.tags.contains(&"tag2".to_string()));
    }
}
