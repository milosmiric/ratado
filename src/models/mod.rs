//! Data models for Ratado.
//!
//! This module contains all the core data structures used throughout the
//! application. These models represent the domain entities and are designed
//! to be serializable for database storage.
//!
//! ## Key Types
//!
//! - [`Task`] - A task item with title, due date, priority, etc.
//! - [`Project`] - A project for organizing tasks
//! - [`Priority`] - Task priority levels (Low, Medium, High, Urgent)
//! - [`TaskStatus`] - Task states (Pending, InProgress, Completed, Archived)
//! - [`Filter`] - Criteria for filtering task lists
//! - [`SortOrder`] - Options for sorting task lists
//!
//! ## Examples
//!
//! ```
//! use ratado::models::{Task, Priority, Filter};
//!
//! // Create a new task
//! let mut task = Task::new("Buy groceries");
//! task.priority = Priority::High;
//!
//! // Filter tasks
//! let tasks = vec![task];
//! let high_priority = Filter::ByPriority(Priority::High).apply(&tasks);
//! ```

mod filter;
mod project;
mod task;

pub use filter::{Filter, SortOrder};
pub use project::Project;
pub use task::{Priority, Task, TaskStatus};
