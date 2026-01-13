//! Utility functions for Ratado.
//!
//! This module provides helper functions used throughout the application,
//! including date/time formatting and ID generation.
//!
//! ## Submodules
//!
//! - `datetime` - Date/time formatting and comparison utilities
//! - `ids` - UUID generation
//!
//! ## Examples
//!
//! ```
//! use ratado::utils::{format_relative_date, generate_id};
//! use chrono::Utc;
//!
//! // Format a date
//! let formatted = format_relative_date(Utc::now());
//! assert_eq!(formatted, "Today");
//!
//! // Generate a unique ID
//! let id = generate_id();
//! assert_eq!(id.len(), 36);
//! ```

mod datetime;
mod ids;

pub use datetime::{
    days_until, format_due_date, format_relative_date, is_same_day, is_this_week, is_today, now,
};
pub use ids::generate_id;
