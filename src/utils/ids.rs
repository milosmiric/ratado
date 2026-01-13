//! ID generation utilities.
//!
//! This module provides functions for generating unique identifiers
//! used throughout the application.

use uuid::Uuid;

/// Generates a new unique identifier.
///
/// Creates a UUID v7 (time-ordered) identifier and returns it as a string.
/// UUID v7 IDs are sortable by creation time, which is useful for database
/// indexing and ordering. These IDs are used for tasks, projects, and other entities.
///
/// # Returns
///
/// A 36-character UUID string in the format `xxxxxxxx-xxxx-7xxx-yxxx-xxxxxxxxxxxx`
///
/// # Examples
///
/// ```
/// use ratado::utils::generate_id;
///
/// let id = generate_id();
/// assert_eq!(id.len(), 36);
///
/// // Each call generates a unique ID
/// let id2 = generate_id();
/// assert_ne!(id, id2);
/// ```
pub fn generate_id() -> String {
    Uuid::now_v7().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id_is_valid_uuid() {
        let id = generate_id();
        // Should be able to parse it back as a UUID
        assert!(Uuid::parse_str(&id).is_ok());
    }

    #[test]
    fn test_generate_id_is_unique() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_id_format() {
        let id = generate_id();
        // UUID v7 format: xxxxxxxx-xxxx-7xxx-yxxx-xxxxxxxxxxxx
        assert_eq!(id.len(), 36);
        assert_eq!(id.chars().nth(8), Some('-'));
        assert_eq!(id.chars().nth(13), Some('-'));
        assert_eq!(id.chars().nth(14), Some('7')); // Version 7
        assert_eq!(id.chars().nth(18), Some('-'));
        assert_eq!(id.chars().nth(23), Some('-'));
    }
}
