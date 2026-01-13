//! Project model for organizing tasks.
//!
//! Projects provide a way to group related tasks together. Each project
//! has a name, color, and icon for visual identification in the UI.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A project for organizing tasks.
///
/// Projects allow users to group related tasks together. Each project
/// has a customizable color and icon that are displayed in the UI.
///
/// # Examples
///
/// ```
/// use ratado::models::Project;
///
/// // Create a basic project
/// let project = Project::new("Work");
/// assert_eq!(project.name, "Work");
///
/// // Create a styled project
/// let project = Project::with_style("Personal", "#e74c3c", "🏠");
/// assert_eq!(project.color, "#e74c3c");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Unique identifier (UUID v7 string - time-ordered)
    pub id: String,
    /// Project name displayed in the UI
    pub name: String,
    /// Hex color code for the project (e.g., "#3498db")
    pub color: String,
    /// Emoji or icon representing the project
    pub icon: String,
    /// When the project was created (UTC)
    pub created_at: DateTime<Utc>,
}

impl Project {
    /// Creates a new project with the given name.
    ///
    /// Uses default styling (blue color, folder icon).
    ///
    /// # Arguments
    ///
    /// * `name` - The name for the project
    ///
    /// # Returns
    ///
    /// A new `Project` with default color (#3498db) and icon (📁)
    ///
    /// # Examples
    ///
    /// ```
    /// use ratado::models::Project;
    ///
    /// let project = Project::new("Work");
    /// assert_eq!(project.name, "Work");
    /// assert_eq!(project.color, "#3498db");
    /// ```
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::now_v7().to_string(),
            name: name.to_string(),
            color: "#3498db".to_string(),
            icon: "📁".to_string(),
            created_at: Utc::now(),
        }
    }

    /// Creates a new project with custom color and icon.
    ///
    /// # Arguments
    ///
    /// * `name` - The name for the project
    /// * `color` - Hex color code (e.g., "#e74c3c")
    /// * `icon` - Emoji or icon character
    ///
    /// # Returns
    ///
    /// A new `Project` with the specified styling
    ///
    /// # Examples
    ///
    /// ```
    /// use ratado::models::Project;
    ///
    /// let project = Project::with_style("Personal", "#e74c3c", "🏠");
    /// assert_eq!(project.name, "Personal");
    /// assert_eq!(project.color, "#e74c3c");
    /// assert_eq!(project.icon, "🏠");
    /// ```
    pub fn with_style(name: &str, color: &str, icon: &str) -> Self {
        Self {
            id: Uuid::now_v7().to_string(),
            name: name.to_string(),
            color: color.to_string(),
            icon: icon.to_string(),
            created_at: Utc::now(),
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new("Inbox")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_new() {
        let project = Project::new("Work");
        assert_eq!(project.name, "Work");
        assert_eq!(project.color, "#3498db");
        assert!(!project.id.is_empty());
    }

    #[test]
    fn test_project_with_style() {
        let project = Project::with_style("Personal", "#e74c3c", "🏠");
        assert_eq!(project.name, "Personal");
        assert_eq!(project.color, "#e74c3c");
        assert_eq!(project.icon, "🏠");
    }
}
