//! Application state management.
//!
//! This module contains the central [`App`] struct that manages all application
//! state. Following the Elm architecture pattern, the App struct holds the
//! complete state of the application and is updated in response to events.

/// Central application state.
///
/// The `App` struct holds all state for the Ratado application. It follows
/// the single source of truth pattern where all UI state, data, and
/// configuration is managed in one place.
///
/// # Examples
///
/// ```
/// use ratado::App;
///
/// let app = App::new();
/// assert!(!app.should_quit);
/// ```
pub struct App {
    /// Whether the application should exit on the next event loop iteration.
    /// Set to `true` when the user requests to quit (e.g., pressing 'q').
    pub should_quit: bool,
}

impl App {
    /// Creates a new `App` instance with default state.
    ///
    /// # Returns
    ///
    /// A new `App` with `should_quit` set to `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratado::App;
    ///
    /// let app = App::new();
    /// assert!(!app.should_quit);
    /// ```
    pub fn new() -> Self {
        Self { should_quit: false }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
