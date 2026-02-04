//! End-to-End Test Infrastructure for Ratado
//!
//! This module provides utilities for PTY-based interactive testing of the
//! Ratado TUI application using expectrl.
//!
//! # Overview
//!
//! These tests spawn the actual Ratado binary in a pseudo-terminal (PTY),
//! simulating real user interaction. This catches issues that unit tests miss:
//! - Terminal rendering and escape sequence handling
//! - Input buffering and timing
//! - Signal handling
//! - Race conditions in the event loop
//!
//! # Verification Strategy
//!
//! Tests use two complementary verification approaches:
//! - **UI verification** (during test): `expect()` patterns confirm visible output
//! - **Database verification** (after quit): query the DB once the app releases its lock
//!
//! # Test Isolation
//!
//! Each test uses a temporary database via the `--db-path` flag to ensure
//! complete isolation from user data and other tests.

use expectrl::session::OsSession;
use expectrl::{Eof, Expect, Regex};
use ratado::storage::Database;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

/// Default timeout for expect operations
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Extended timeout for slower operations (startup, saving)
pub const EXTENDED_TIMEOUT: Duration = Duration::from_secs(10);

/// Terminal dimensions
const TERM_WIDTH: u16 = 120;
const TERM_HEIGHT: u16 = 80;

/// Test harness for Ratado E2E tests.
///
/// While the app is running, use `expect_text` for UI verification.
/// Call `quit()` to get a `DbVerifier` for post-quit database assertions.
pub struct RatadoTest {
    /// The expectrl session controlling the PTY
    pub session: OsSession,
    /// Temporary directory containing the test database
    _temp_dir: Option<TempDir>,
    /// Path to the test database
    db_path: PathBuf,
    /// Whether the app has already been quit
    exited: bool,
}

/// Verifier for post-quit database assertions.
///
/// Created by `RatadoTest::quit()` after the app exits and releases
/// the database lock.
pub struct DbVerifier {
    _temp_dir: TempDir,
    db_path: PathBuf,
}

impl RatadoTest {
    /// Spawns a new Ratado instance with a fresh temporary database.
    pub fn spawn() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test_ratado.db");

        // Disable animations in spawned process (inherited env var)
        unsafe { std::env::set_var("RATADO_NO_ANIMATIONS", "1") };

        let binary_path = env!("CARGO_BIN_EXE_ratado");
        let cmd = format!("{} -d {}", binary_path, db_path.to_str().unwrap());

        let mut session = expectrl::spawn(&cmd).expect("Failed to spawn ratado");

        session
            .get_process_mut()
            .set_window_size(TERM_WIDTH, TERM_HEIGHT)
            .expect("Failed to set terminal size");

        session.set_expect_timeout(Some(DEFAULT_TIMEOUT));

        RatadoTest {
            session,
            _temp_dir: Some(temp_dir),
            db_path,
            exited: false,
        }
    }

    /// Waits for the application to fully initialize by expecting the main view.
    ///
    /// Animations and splash screen are disabled via RATADO_NO_ANIMATIONS env var.
    pub fn wait_for_startup(&mut self) -> &mut Self {
        self.session.set_expect_timeout(Some(EXTENDED_TIMEOUT));
        // Wait for main view to be ready (Tasks header or empty state)
        let _ = self.session.expect(Regex("Tasks|liftoff"));
        self.session.set_expect_timeout(Some(DEFAULT_TIMEOUT));
        self
    }

    // =========================================================================
    // Input Methods
    // =========================================================================

    pub fn press(&mut self, key: &str) -> &mut Self {
        self.session.send(key).expect("Failed to send key");
        std::thread::sleep(Duration::from_millis(50));
        self
    }

    pub fn type_text(&mut self, text: &str) -> &mut Self {
        self.session.send(text).expect("Failed to send text");
        std::thread::sleep(Duration::from_millis(50));
        self
    }

    pub fn press_enter(&mut self) -> &mut Self {
        self.session.send("\r").expect("Failed to send enter");
        std::thread::sleep(Duration::from_millis(50));
        self
    }

    pub fn press_escape(&mut self) -> &mut Self {
        self.session.send("\x1b").expect("Failed to send escape");
        std::thread::sleep(Duration::from_millis(50));
        self
    }

    pub fn press_tab(&mut self) -> &mut Self {
        self.session.send("\t").expect("Failed to send tab");
        std::thread::sleep(Duration::from_millis(50));
        self
    }

    pub fn press_arrow(&mut self, direction: &str) -> &mut Self {
        let seq = match direction {
            "up" => "\x1b[A",
            "down" => "\x1b[B",
            "right" => "\x1b[C",
            "left" => "\x1b[D",
            _ => panic!("Invalid arrow direction: {}", direction),
        };
        self.session.send(seq).expect("Failed to send arrow key");
        std::thread::sleep(Duration::from_millis(50));
        self
    }

    pub fn wait(&mut self, duration: Duration) -> &mut Self {
        std::thread::sleep(duration);
        self
    }

    // =========================================================================
    // UI Verification (while app is running)
    // =========================================================================

    /// Expects the given regex pattern to appear in the terminal output.
    pub fn expect_text(&mut self, pattern: &str) -> &mut Self {
        self.session
            .expect(Regex(pattern))
            .unwrap_or_else(|_| panic!("Expected pattern '{}' in terminal output", pattern));
        self
    }

    // =========================================================================
    // High-Level Actions
    // =========================================================================

    /// Quits the app and returns a `DbVerifier` for post-quit database checks.
    pub fn quit(&mut self) -> DbVerifier {
        self.session.send("q").expect("Failed to send quit");
        let _ = self.session.expect(Eof);
        self.exited = true;
        std::thread::sleep(Duration::from_millis(200));
        DbVerifier {
            _temp_dir: self._temp_dir.take().expect("TempDir already taken"),
            db_path: self.db_path.clone(),
        }
    }

    /// Opens the Quick Capture dialog, types the title, and submits with Enter.
    pub fn add_task(&mut self, title: &str) -> &mut Self {
        // Open quick capture dialog
        self.press("a");
        std::thread::sleep(Duration::from_millis(100));
        // Type the title
        self.type_text(title);
        // Submit with Enter
        self.press_enter();
        std::thread::sleep(Duration::from_millis(200));
        self
    }

    pub fn move_down(&mut self) -> &mut Self {
        self.press("j")
    }

    pub fn move_up(&mut self) -> &mut Self {
        self.press("k")
    }

    /// Toggles task completion with Space key.
    pub fn toggle_complete(&mut self) -> &mut Self {
        self.press(" ")
    }

    /// Deletes the selected task (presses 'd' then 'y' to confirm).
    pub fn delete_task(&mut self) -> &mut Self {
        self.press("d");
        std::thread::sleep(Duration::from_millis(100));
        self.press("y");
        std::thread::sleep(Duration::from_millis(200));
        self
    }

    pub fn edit_task(&mut self) -> &mut Self {
        self.press("e")
    }

    pub fn open_search(&mut self) -> &mut Self {
        self.press("/")
    }

    /// Switches the task filter to "All" so completed tasks remain visible.
    /// Default filter is "Pending" which hides completed tasks.
    pub fn set_filter_all(&mut self) -> &mut Self {
        // Open filter dialog
        self.press("f");
        std::thread::sleep(Duration::from_millis(100));
        // Move up from "Pending" (index 1) to "All" (index 0)
        self.press("k");
        std::thread::sleep(Duration::from_millis(50));
        // Confirm
        self.press_enter();
        std::thread::sleep(Duration::from_millis(200));
        self
    }

    /// Checks if the database file was created on disk.
    pub fn database_exists(&self) -> bool {
        self.db_path.exists()
    }
}

impl Drop for RatadoTest {
    fn drop(&mut self) {
        if !self.exited {
            let _ = self.session.send("q");
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

// =============================================================================
// DbVerifier - post-quit database assertions
// =============================================================================

impl DbVerifier {
    async fn open_db(&self) -> Database {
        Database::open(&self.db_path)
            .await
            .expect("Failed to open test database for verification")
    }

    pub fn assert_task_count(&self, expected: usize) -> &Self {
        let actual = self.get_task_count();
        assert_eq!(
            actual, expected,
            "Expected {} tasks, but found {}",
            expected, actual
        );
        self
    }

    pub fn assert_task_exists(&self, title: &str) -> &Self {
        assert!(
            self.task_exists(title),
            "Expected task '{}' to exist in database",
            title
        );
        self
    }

    pub fn assert_task_not_exists(&self, title: &str) -> &Self {
        assert!(
            !self.task_exists(title),
            "Expected task '{}' to NOT exist in database",
            title
        );
        self
    }

    pub fn assert_task_completed(&self, title: &str) -> &Self {
        assert!(
            self.task_is_completed(title),
            "Expected task '{}' to be completed",
            title
        );
        self
    }

    pub fn assert_task_pending(&self, title: &str) -> &Self {
        assert!(
            !self.task_is_completed(title),
            "Expected task '{}' to be pending (not completed)",
            title
        );
        self
    }

    pub fn assert_completed_count(&self, expected: usize) -> &Self {
        let actual = self.get_completed_task_count();
        assert_eq!(
            actual, expected,
            "Expected {} completed tasks, but found {}",
            expected, actual
        );
        self
    }

    pub fn assert_pending_count(&self, expected: usize) -> &Self {
        let actual = self.get_pending_task_count();
        assert_eq!(
            actual, expected,
            "Expected {} pending tasks, but found {}",
            expected, actual
        );
        self
    }

    fn get_task_count(&self) -> usize {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                let db = self.open_db().await;
                let mut rows = db
                    .query("SELECT COUNT(*) FROM tasks", ())
                    .await
                    .expect("Failed to count tasks");
                if let Ok(Some(row)) = rows.next().await {
                    row.get::<i64>(0).unwrap_or(0) as usize
                } else {
                    0
                }
            })
    }

    fn get_completed_task_count(&self) -> usize {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                let db = self.open_db().await;
                let mut rows = db
                    .query(
                        "SELECT COUNT(*) FROM tasks WHERE status = 'completed'",
                        (),
                    )
                    .await
                    .expect("Failed to count completed tasks");
                if let Ok(Some(row)) = rows.next().await {
                    row.get::<i64>(0).unwrap_or(0) as usize
                } else {
                    0
                }
            })
    }

    fn get_pending_task_count(&self) -> usize {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                let db = self.open_db().await;
                let mut rows = db
                    .query(
                        "SELECT COUNT(*) FROM tasks WHERE status = 'pending'",
                        (),
                    )
                    .await
                    .expect("Failed to count pending tasks");
                if let Ok(Some(row)) = rows.next().await {
                    row.get::<i64>(0).unwrap_or(0) as usize
                } else {
                    0
                }
            })
    }

    fn task_exists(&self, title: &str) -> bool {
        let title = title.to_string();
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                let db = self.open_db().await;
                let mut rows = db
                    .query(
                        "SELECT COUNT(*) FROM tasks WHERE title = ?1",
                        [title.as_str()],
                    )
                    .await
                    .expect("Failed to check task existence");
                if let Ok(Some(row)) = rows.next().await {
                    row.get::<i64>(0).unwrap_or(0) > 0
                } else {
                    false
                }
            })
    }

    fn task_is_completed(&self, title: &str) -> bool {
        let title = title.to_string();
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                let db = self.open_db().await;
                let mut rows = db
                    .query(
                        "SELECT status FROM tasks WHERE title = ?1",
                        [title.as_str()],
                    )
                    .await
                    .expect("Failed to check task completion");
                if let Ok(Some(row)) = rows.next().await {
                    row.get::<String>(0).unwrap_or_default() == "completed"
                } else {
                    false
                }
            })
    }
}
