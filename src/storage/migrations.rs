//! Database schema migrations.
//!
//! This module handles database schema versioning and upgrades. Migrations
//! are applied automatically when the database is opened, ensuring the
//! schema is always up to date.
//!
//! ## How Migrations Work
//!
//! 1. Each migration has a version number and SQL to execute
//! 2. A `_migrations` table tracks which versions have been applied
//! 3. On startup, any pending migrations are applied in order
//! 4. Migrations are idempotent - running them multiple times is safe
//!
//! ## Adding New Migrations
//!
//! 1. Create a new SQL file in `src/storage/migrations/` (e.g., `002_add_reminders.sql`)
//! 2. Add a new entry to the `MIGRATIONS` array with the next version number
//! 3. The migration will be applied automatically on next startup

use crate::storage::{Database, Result, StorageError};
use log::info;
use turso::Value;

/// A database migration.
///
/// Each migration has a version number, description, and SQL to execute.
/// Migrations are applied in order by version number.
struct Migration {
    /// Version number (must be unique and sequential)
    version: u32,
    /// Human-readable description of what this migration does
    description: &'static str,
    /// SQL statements to execute (can be multiple statements separated by semicolons)
    sql: &'static str,
}

/// All migrations for the Ratado database.
///
/// Migrations are applied in order by version number. Each migration
/// should be idempotent (safe to run multiple times).
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial schema",
        sql: include_str!("migrations/001_initial.sql"),
    },
    Migration {
        version: 2,
        description: "Add default Inbox project",
        sql: "INSERT OR IGNORE INTO projects (id, name, color, icon, created_at)
              VALUES ('inbox', 'Inbox', '#3498db', '📥', strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
    },
];

/// Runs all pending database migrations.
///
/// This function should be called after opening the database to ensure
/// the schema is up to date. It's safe to call multiple times - already
/// applied migrations will be skipped.
///
/// # Arguments
///
/// * `db` - The database connection to migrate
///
/// # Returns
///
/// Ok(()) if all migrations were applied successfully, or an error if any failed.
///
/// # Errors
///
/// Returns an error if:
/// - The migrations table cannot be created
/// - Any migration fails to execute
/// - The version cannot be recorded
///
/// # Examples
///
/// ```rust,no_run
/// use ratado::storage::{Database, run_migrations};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = Database::open_in_memory().await?;
/// run_migrations(&db).await?;
/// // Database is now ready to use
/// # Ok(())
/// # }
/// ```
pub async fn run_migrations(db: &Database) -> Result<()> {
    // Create migrations tracking table
    db.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )",
        (),
    )
    .await?;

    // Get current version
    let current_version = get_current_version(db).await?;
    info!("Current database version: {}", current_version);

    // Apply pending migrations
    for migration in MIGRATIONS.iter().filter(|m| m.version > current_version) {
        info!(
            "Applying migration {}: {}",
            migration.version, migration.description
        );

        // Execute the migration SQL
        db.execute_batch(migration.sql).await.map_err(|e| {
            StorageError::Migration(format!(
                "Failed to apply migration {}: {}",
                migration.version, e
            ))
        })?;

        // Record the migration
        db.execute(
            "INSERT INTO _migrations (version, description, applied_at)
             VALUES (?1, ?2, datetime('now'))",
            [
                Value::Integer(migration.version as i64),
                Value::Text(migration.description.to_string()),
            ],
        )
        .await?;

        info!("Migration {} applied successfully", migration.version);
    }

    let final_version = get_current_version(db).await?;
    if final_version > current_version {
        info!(
            "Database migrated from version {} to {}",
            current_version, final_version
        );
    } else {
        info!("Database is up to date (version {})", final_version);
    }

    Ok(())
}

/// Gets the current database schema version.
///
/// Returns 0 if no migrations have been applied yet.
async fn get_current_version(db: &Database) -> Result<u32> {
    let value = db
        .query_scalar("SELECT COALESCE(MAX(version), 0) FROM _migrations", ())
        .await?;

    match value {
        Some(Value::Integer(v)) => Ok(v as u32),
        Some(Value::Null) | None => Ok(0),
        other => Err(StorageError::Conversion(format!(
            "Unexpected version value: {:?}",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_migrations_fresh_db() {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();

        // Check that tables were created
        let result = db
            .query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tasks'",
                (),
            )
            .await
            .unwrap();
        assert_eq!(result, Some(Value::Integer(1)));

        // Check that projects table exists
        let result = db
            .query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='projects'",
                (),
            )
            .await
            .unwrap();
        assert_eq!(result, Some(Value::Integer(1)));

        // Check that Inbox project was created
        let result = db
            .query_scalar("SELECT COUNT(*) FROM projects WHERE id = 'inbox'", ())
            .await
            .unwrap();
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[tokio::test]
    async fn test_migrations_are_idempotent() {
        let db = Database::open_in_memory().await.unwrap();

        // Run migrations twice
        run_migrations(&db).await.unwrap();
        run_migrations(&db).await.unwrap();

        // Should still have exactly 2 migrations recorded
        let result = db
            .query_scalar("SELECT COUNT(*) FROM _migrations", ())
            .await
            .unwrap();
        assert_eq!(result, Some(Value::Integer(2)));
    }

    #[tokio::test]
    async fn test_version_tracking() {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();

        let version = get_current_version(&db).await.unwrap();
        assert_eq!(version, 2); // We have 2 migrations
    }

    #[tokio::test]
    async fn test_indexes_created() {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();

        // Check that indexes were created
        let result = db
            .query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_tasks_%'",
                (),
            )
            .await
            .unwrap();

        // We create 5 indexes on tasks table
        assert_eq!(result, Some(Value::Integer(5)));
    }
}
