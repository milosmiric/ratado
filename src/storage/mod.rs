//! Database storage module.
//!
//! This module handles all database operations using Turso (SQLite-compatible).
//! Data is stored locally at `~/Library/Application Support/ratado/ratado.db` (macOS)
//! or `~/.config/ratado/ratado.db` (Linux).
//!
//! ## Architecture
//!
//! - [`Database`] - Connection management and low-level query execution
//! - [`migrations`] - Schema versioning and upgrades
//! - Task/Project/Tag repositories - CRUD operations for domain models
//!
//! ## Usage
//!
//! ```rust,no_run
//! use ratado::storage::{Database, run_migrations};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Open database and run migrations
//! let db = Database::open_default().await?;
//! run_migrations(&db).await?;
//!
//! // Now ready to use...
//! # Ok(())
//! # }
//! ```

mod database;
mod migrations;
mod projects;
mod tags;
mod tasks;

pub use database::{Database, Result, StorageError};
pub use migrations::run_migrations;
pub use tags::Tag;
