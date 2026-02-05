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

use std::cmp::Ordering;

use crate::storage::{Database, Result, StorageError};
use log::{info, warn};
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
    Migration {
        version: 3,
        description: "Add app metadata table for version tracking",
        sql: "CREATE TABLE IF NOT EXISTS _app_meta (
                  key TEXT PRIMARY KEY,
                  value TEXT NOT NULL,
                  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
              )",
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

/// Compares two semantic version strings (e.g., "0.2.0" vs "0.10.0").
///
/// Parses each version as `major.minor.patch` and compares numerically.
/// Falls back to lexicographic string comparison if parsing fails.
fn compare_versions(a: &str, b: &str) -> Ordering {
    fn parse(v: &str) -> Option<(u32, u32, u32)> {
        let mut parts = v.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        Some((major, minor, patch))
    }

    match (parse(a), parse(b)) {
        (Some(va), Some(vb)) => va.cmp(&vb),
        _ => a.cmp(b),
    }
}

/// Gets the stored application version from the `_app_meta` table.
///
/// Returns `None` if no version has been stored yet (fresh install or
/// upgrade from a version before version tracking was added).
async fn get_stored_app_version(db: &Database) -> Result<Option<String>> {
    let row = db
        .query_one(
            "SELECT value FROM _app_meta WHERE key = 'app_version'",
            (),
        )
        .await?;

    match row {
        Some(row) => match row.get_value(0)? {
            Value::Text(v) => Ok(Some(v)),
            _ => Ok(None),
        },
        None => Ok(None),
    }
}

/// Stores the application version in the `_app_meta` table.
async fn set_app_version(db: &Database, version: &str) -> Result<()> {
    db.execute(
        "INSERT OR REPLACE INTO _app_meta (key, value, updated_at)
         VALUES ('app_version', ?1, datetime('now'))",
        [Value::Text(version.to_string())],
    )
    .await?;
    Ok(())
}

/// Checks the stored application version and updates it if needed.
///
/// This function should be called after [`run_migrations`] on every startup.
/// It detects four scenarios:
///
/// - **No stored version** (fresh install or pre-version-tracking DB): stores current version
/// - **Same version**: logs info, no database write
/// - **Upgrade** (stored < current): logs the upgrade, updates stored version
/// - **Downgrade** (stored > current): logs a warning, updates stored version
///
/// # Arguments
///
/// * `db` - The database connection (must have `_app_meta` table already created via migrations)
///
/// # Errors
///
/// Returns an error if the version cannot be read from or written to the database.
pub async fn check_and_update_app_version(db: &Database) -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    let stored = get_stored_app_version(db).await?;

    match stored {
        None => {
            info!(
                "No stored app version found — recording v{} (fresh install or upgrade from pre-tracking version)",
                current
            );
            set_app_version(db, current).await?;
        }
        Some(ref stored_version) if stored_version == current => {
            info!("App version unchanged (v{})", current);
        }
        Some(ref stored_version) => match compare_versions(stored_version, current) {
            Ordering::Less => {
                info!(
                    "App upgraded from v{} to v{}",
                    stored_version, current
                );
                set_app_version(db, current).await?;
            }
            Ordering::Greater => {
                warn!(
                    "App downgraded from v{} to v{} — database was last used by a newer version",
                    stored_version, current
                );
                set_app_version(db, current).await?;
            }
            Ordering::Equal => {
                // Should not reach here due to the string equality check above,
                // but handle it gracefully
                info!("App version unchanged (v{})", current);
            }
        },
    }

    Ok(())
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

        // Should still have exactly 3 migrations recorded
        let result = db
            .query_scalar("SELECT COUNT(*) FROM _migrations", ())
            .await
            .unwrap();
        assert_eq!(result, Some(Value::Integer(3)));
    }

    #[tokio::test]
    async fn test_version_tracking() {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();

        let version = get_current_version(&db).await.unwrap();
        assert_eq!(version, 3); // We have 3 migrations
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

    #[tokio::test]
    async fn test_app_meta_table_created() {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();

        let result = db
            .query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_app_meta'",
                (),
            )
            .await
            .unwrap();
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[tokio::test]
    async fn test_app_version_stored_on_first_run() {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();

        // No version stored yet
        let stored = get_stored_app_version(&db).await.unwrap();
        assert!(stored.is_none());

        // After check_and_update, version should be stored
        check_and_update_app_version(&db).await.unwrap();
        let stored = get_stored_app_version(&db).await.unwrap();
        assert_eq!(stored, Some(env!("CARGO_PKG_VERSION").to_string()));
    }

    #[tokio::test]
    async fn test_app_version_idempotent() {
        let db = Database::open_in_memory().await.unwrap();
        run_migrations(&db).await.unwrap();

        // Call twice
        check_and_update_app_version(&db).await.unwrap();
        check_and_update_app_version(&db).await.unwrap();

        // Should still have exactly one entry
        let result = db
            .query_scalar("SELECT COUNT(*) FROM _app_meta WHERE key = 'app_version'", ())
            .await
            .unwrap();
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[test]
    fn test_compare_versions() {
        assert_eq!(compare_versions("0.1.0", "0.2.0"), Ordering::Less);
        assert_eq!(compare_versions("0.2.0", "0.2.0"), Ordering::Equal);
        assert_eq!(compare_versions("0.2.0", "0.1.0"), Ordering::Greater);
        assert_eq!(compare_versions("1.0.0", "0.9.9"), Ordering::Greater);
        assert_eq!(compare_versions("0.10.0", "0.2.0"), Ordering::Greater);
        assert_eq!(compare_versions("0.2.0", "0.10.0"), Ordering::Less);
    }
}
