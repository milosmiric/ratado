//! Database connection and initialization.
//!
//! This module provides the [`Database`] struct for connecting to and interacting
//! with the Turso/SQLite database. It handles connection management, path resolution,
//! and provides a high-level interface for database operations.
//!
//! # Examples
//!
//! ```rust,no_run
//! use ratado::storage::Database;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Open database at default location (~/.config/ratado/ratado.db)
//! let db = Database::open_default().await?;
//!
//! // Or open an in-memory database for testing
//! let db = Database::open_in_memory().await?;
//! # Ok(())
//! # }
//! ```

use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use thiserror::Error;
use turso::{Builder, Connection, Row, Rows, Value};

/// Errors that can occur during storage operations.
///
/// This enum covers all error conditions that can arise when working with
/// the database, including connection failures, query errors, and data
/// conversion issues.
#[derive(Error, Debug)]
pub enum StorageError {
    /// Database operation failed
    #[error("Database error: {0}")]
    Database(#[from] turso::Error),

    /// Failed to create or access the config directory
    #[error("Failed to access config directory: {0}")]
    ConfigDir(#[from] std::io::Error),

    /// Could not determine the user's config directory
    #[error("Could not determine config directory")]
    NoConfigDir,

    /// Data conversion or parsing error
    #[error("Data conversion error: {0}")]
    Conversion(String),

    /// Record not found
    #[error("Record not found: {0}")]
    NotFound(String),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),
}

/// Result type for storage operations.
pub type Result<T> = std::result::Result<T, StorageError>;

/// Database connection wrapper.
///
/// Provides a high-level interface for database operations. The database
/// uses Turso (SQLite-compatible) for local storage.
///
/// # Thread Safety
///
/// The `Database` struct is `Send` and `Sync`, and can be safely shared
/// between threads. Each operation acquires the connection as needed.
///
/// # Examples
///
/// ```rust,no_run
/// use ratado::storage::Database;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = Database::open_in_memory().await?;
///
/// // Execute a query
/// db.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)", ()).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Opens a database at the specified path.
    ///
    /// Creates the database file and parent directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the database file
    ///
    /// # Returns
    ///
    /// A new `Database` instance connected to the specified file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The parent directory cannot be created
    /// - The database file cannot be opened or created
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ratado::storage::Database;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::open(Path::new("/path/to/database.db")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn open(path: &Path) -> Result<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let path_str = path.to_string_lossy();
        let db = Builder::new_local(&path_str).build().await?;
        let conn = db.connect()?;

        Ok(Self { conn })
    }

    /// Opens an in-memory database.
    ///
    /// Useful for testing. Data is lost when the database is dropped.
    ///
    /// # Returns
    ///
    /// A new `Database` instance with an in-memory database.
    ///
    /// # Errors
    ///
    /// Returns an error if the in-memory database cannot be created.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ratado::storage::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::open_in_memory().await?;
    /// // Use for testing...
    /// # Ok(())
    /// # }
    /// ```
    pub async fn open_in_memory() -> Result<Self> {
        let db = Builder::new_local(":memory:").build().await?;
        let conn = db.connect()?;
        Ok(Self { conn })
    }

    /// Opens the database at the default location.
    ///
    /// The default path is `~/.config/ratado/ratado.db` on Linux/macOS.
    /// Creates the directory structure if it doesn't exist.
    ///
    /// # Returns
    ///
    /// A new `Database` instance at the default location.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The config directory cannot be determined
    /// - The directory cannot be created
    /// - The database cannot be opened
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ratado::storage::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::open_default().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn open_default() -> Result<Self> {
        let path = Self::default_path()?;
        Self::open(&path).await
    }

    /// Returns the default database path.
    ///
    /// The default path is `~/.config/ratado/ratado.db` on Linux/macOS.
    /// Creates the config directory if it doesn't exist.
    ///
    /// # Returns
    ///
    /// The path to the default database file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The config directory cannot be determined (e.g., no HOME set)
    /// - The directory cannot be created
    pub fn default_path() -> Result<PathBuf> {
        let proj_dirs =
            ProjectDirs::from("", "", "ratado").ok_or(StorageError::NoConfigDir)?;
        let config_dir = proj_dirs.config_dir();
        std::fs::create_dir_all(config_dir)?;
        Ok(config_dir.join("ratado.db"))
    }

    /// Executes a SQL statement that doesn't return rows.
    ///
    /// Use this for INSERT, UPDATE, DELETE, and DDL statements.
    ///
    /// # Arguments
    ///
    /// * `sql` - The SQL statement to execute
    /// * `params` - Parameters to bind to the statement
    ///
    /// # Returns
    ///
    /// The number of rows affected by the statement.
    ///
    /// # Errors
    ///
    /// Returns an error if the statement fails to execute.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ratado::storage::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::open_in_memory().await?;
    /// let rows_affected = db.execute(
    ///     "INSERT INTO tasks (title) VALUES (?1)",
    ///     ["Buy groceries"]
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute(
        &self,
        sql: impl AsRef<str>,
        params: impl turso::IntoParams,
    ) -> Result<u64> {
        Ok(self.conn.execute(sql, params).await?)
    }

    /// Executes a batch of SQL statements.
    ///
    /// Useful for running multiple statements at once, such as migrations.
    /// Statements are separated by semicolons.
    ///
    /// # Arguments
    ///
    /// * `sql` - Multiple SQL statements separated by semicolons
    ///
    /// # Errors
    ///
    /// Returns an error if any statement fails to execute.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ratado::storage::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::open_in_memory().await?;
    /// db.execute_batch("
    ///     CREATE TABLE test1 (id INTEGER);
    ///     CREATE TABLE test2 (id INTEGER);
    /// ").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_batch(&self, sql: impl AsRef<str>) -> Result<()> {
        Ok(self.conn.execute_batch(sql).await?)
    }

    /// Executes a query and returns the result rows.
    ///
    /// Use this for SELECT statements.
    ///
    /// # Arguments
    ///
    /// * `sql` - The SQL query to execute
    /// * `params` - Parameters to bind to the query
    ///
    /// # Returns
    ///
    /// An iterator over the result rows.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails to execute.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ratado::storage::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::open_in_memory().await?;
    /// let mut rows = db.query("SELECT * FROM tasks WHERE status = ?1", ["pending"]).await?;
    /// while let Some(row) = rows.next().await? {
    ///     // Process row...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query(
        &self,
        sql: impl AsRef<str>,
        params: impl turso::IntoParams,
    ) -> Result<Rows> {
        Ok(self.conn.query(sql, params).await?)
    }

    /// Executes a query and returns the first row.
    ///
    /// # Arguments
    ///
    /// * `sql` - The SQL query to execute
    /// * `params` - Parameters to bind to the query
    ///
    /// # Returns
    ///
    /// The first row of the result, or `None` if no rows match.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails to execute.
    pub async fn query_one(
        &self,
        sql: impl AsRef<str>,
        params: impl turso::IntoParams,
    ) -> Result<Option<Row>> {
        let mut rows = self.query(sql, params).await?;
        Ok(rows.next().await?)
    }

    /// Executes a query and returns a single scalar value.
    ///
    /// Useful for COUNT, MAX, etc. queries that return a single value.
    ///
    /// # Arguments
    ///
    /// * `sql` - The SQL query to execute
    /// * `params` - Parameters to bind to the query
    ///
    /// # Returns
    ///
    /// The first column of the first row, or `None` if no rows match.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails to execute.
    pub async fn query_scalar(
        &self,
        sql: impl AsRef<str>,
        params: impl turso::IntoParams,
    ) -> Result<Option<Value>> {
        if let Some(row) = self.query_one(sql, params).await? {
            Ok(Some(row.get_value(0)?))
        } else {
            Ok(None)
        }
    }

    /// Returns a reference to the underlying connection.
    ///
    /// Use this for advanced operations not covered by the wrapper methods.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_open_in_memory() {
        let db = Database::open_in_memory().await.unwrap();
        db.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)", ())
            .await
            .unwrap();
        db.execute("INSERT INTO test (name) VALUES (?1)", ["Alice"])
            .await
            .unwrap();

        let mut rows = db.query("SELECT name FROM test", ()).await.unwrap();
        let row = rows.next().await.unwrap().unwrap();
        let name = row.get_value(0).unwrap();
        assert_eq!(name, Value::Text("Alice".to_string()));
    }

    #[tokio::test]
    async fn test_execute_batch() {
        let db = Database::open_in_memory().await.unwrap();
        db.execute_batch(
            "
            CREATE TABLE test1 (id INTEGER PRIMARY KEY);
            CREATE TABLE test2 (id INTEGER PRIMARY KEY);
            INSERT INTO test1 (id) VALUES (1);
            INSERT INTO test2 (id) VALUES (2);
        ",
        )
        .await
        .unwrap();

        let value = db
            .query_scalar("SELECT COUNT(*) FROM test1", ())
            .await
            .unwrap();
        assert_eq!(value, Some(Value::Integer(1)));
    }

    #[tokio::test]
    async fn test_query_one() {
        let db = Database::open_in_memory().await.unwrap();
        db.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)", ())
            .await
            .unwrap();
        db.execute("INSERT INTO test (name) VALUES (?1)", ["Bob"])
            .await
            .unwrap();

        let row = db
            .query_one("SELECT name FROM test WHERE id = 1", ())
            .await
            .unwrap();
        assert!(row.is_some());

        let row = db
            .query_one("SELECT name FROM test WHERE id = 999", ())
            .await
            .unwrap();
        assert!(row.is_none());
    }

    #[test]
    fn test_default_path() {
        let path = Database::default_path().unwrap();
        assert!(path.ends_with("ratado.db"));
        assert!(path.to_string_lossy().contains("ratado"));
    }
}
