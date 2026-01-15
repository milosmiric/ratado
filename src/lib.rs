//! # Ratado
//!
//! Ratado is a terminal-based task manager and reminder application built with
//! Rust and Ratatui. The name combines "Rata" (from Ratatui) with "do" (from todo).
//!
//! ## Features
//!
//! - Task management with priorities and due dates
//! - Project organization
//! - Filtering and sorting
//! - Terminal UI with vim-style navigation
//! - Local SQLite database storage
//!
//! ## Modules
//!
//! - [`app`] - Central application state management
//! - [`models`] - Data structures (Task, Project, Filter, etc.)
//! - [`handlers`] - Keyboard input and command handling
//! - [`storage`] - Database operations
//! - [`ui`] - Terminal UI widgets and views
//! - [`utils`] - Helper functions for dates, IDs, etc.

pub mod app;
pub mod handlers;
pub mod models;
pub mod storage;
pub mod ui;
pub mod utils;

pub use app::{App, AppError, FocusPanel, InputMode, View};
pub use handlers::{handle_event, AppEvent, Command, EventHandler};
