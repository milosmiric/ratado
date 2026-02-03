//! End-to-End Tests for Ratado
//!
//! These tests spawn the actual Ratado binary in a PTY and verify behaviour
//! using two complementary strategies:
//! - **UI verification** (`expect_text`) while the app is running
//! - **Database verification** (`DbVerifier`) after the app exits
//!
//! # Running E2E Tests
//!
//! ```bash
//! cargo test --test e2e_tests
//! ```

mod e2e;

use e2e::RatadoTest;
use expectrl::Expect;
use std::time::Duration;

// ============================================================================
// Application Lifecycle Tests
// ============================================================================

#[test]
fn test_app_starts_and_shows_empty_state() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    // UI: verify empty state message is shown
    app.expect_text("Ready to go");

    // Verify database was created on disk
    assert!(app.database_exists(), "Database file should exist");

    // DB: verify no tasks after quit
    let db = app.quit();
    db.assert_task_count(0);
}

// ============================================================================
// Task Creation Tests
// ============================================================================

#[test]
fn test_add_single_task() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    app.add_task("Buy groceries");

    // UI: the task title should appear in the task list
    app.expect_text("Buy groceries");

    // DB: verify task was persisted
    let db = app.quit();
    db.assert_task_count(1);
    db.assert_task_exists("Buy groceries");
    db.assert_task_pending("Buy groceries");
}

#[test]
fn test_add_multiple_tasks() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    app.add_task("Task One");
    app.expect_text("Task One");

    app.add_task("Task Two");
    app.expect_text("Task Two");

    app.add_task("Task Three");
    app.expect_text("Task Three");

    // DB: verify all tasks were persisted
    let db = app.quit();
    db.assert_task_count(3);
    db.assert_task_exists("Task One");
    db.assert_task_exists("Task Two");
    db.assert_task_exists("Task Three");
    db.assert_pending_count(3);
    db.assert_completed_count(0);
}

#[test]
fn test_empty_task_not_created() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    // Open quick capture dialog and try to submit without typing a title
    app.press("a");
    app.wait(Duration::from_millis(100));
    // Press Enter with empty title
    app.press_enter();
    app.wait(Duration::from_millis(100));

    // DB: no task should have been created
    let db = app.quit();
    db.assert_task_count(0);
}

// ============================================================================
// Task Completion Tests
// ============================================================================

#[test]
fn test_toggle_task_completion() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    // Switch to "All" filter so completed tasks stay visible
    app.set_filter_all();

    app.add_task("Complete me");
    app.expect_text("Complete me");

    // Toggle to completed
    app.toggle_complete();
    app.wait(Duration::from_millis(100));

    // Toggle back to pending (task is still visible with "All" filter)
    app.toggle_complete();
    app.wait(Duration::from_millis(100));

    // DB: task should be pending after two toggles
    let db = app.quit();
    db.assert_task_count(1);
    db.assert_task_pending("Complete me");
}

#[test]
fn test_complete_single_task() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    app.add_task("Finish report");
    app.expect_text("Finish report");

    // Complete the task
    app.toggle_complete();
    app.wait(Duration::from_millis(100));

    // DB: verify it's completed
    let db = app.quit();
    db.assert_task_completed("Finish report");
    db.assert_completed_count(1);
}

#[test]
fn test_complete_multiple_tasks() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    // Switch to "All" filter so completed tasks stay visible and don't shift indices
    app.set_filter_all();

    app.add_task("Task A");
    app.add_task("Task B");
    app.add_task("Task C");

    // Navigate to first task
    app.press("g"); // Jump to top
    app.wait(Duration::from_millis(50));
    app.toggle_complete();
    app.wait(Duration::from_millis(100));

    // Move down and complete second
    app.move_down();
    app.toggle_complete();
    app.wait(Duration::from_millis(100));

    // DB: verify specific tasks
    let db = app.quit();
    db.assert_task_completed("Task A");
    db.assert_task_completed("Task B");
    db.assert_task_pending("Task C");
    db.assert_completed_count(2);
    db.assert_pending_count(1);
}

// ============================================================================
// Task Deletion Tests
// ============================================================================

#[test]
fn test_delete_task() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    app.add_task("Delete me");
    app.expect_text("Delete me");

    // Delete it (press 'd', confirm dialog shows, press 'y')
    app.delete_task();

    // DB: verify deletion
    let db = app.quit();
    db.assert_task_count(0);
    db.assert_task_not_exists("Delete me");
}

#[test]
fn test_delete_one_of_multiple_tasks() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    app.add_task("Keep me");
    app.add_task("Delete me");
    app.add_task("Keep me too");

    // Navigate to second task and delete
    app.press("g"); // Jump to top
    app.wait(Duration::from_millis(50));
    app.move_down();
    app.delete_task();

    // DB: verify correct task was deleted
    let db = app.quit();
    db.assert_task_count(2);
    db.assert_task_exists("Keep me");
    db.assert_task_not_exists("Delete me");
    db.assert_task_exists("Keep me too");
}

#[test]
fn test_cancel_delete() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    app.add_task("Keep me");
    app.expect_text("Keep me");

    // Start delete but cancel with 'n'
    app.press("d");
    app.wait(Duration::from_millis(100));
    app.press("n");
    app.wait(Duration::from_millis(100));

    // DB: task should still exist
    let db = app.quit();
    db.assert_task_count(1);
    db.assert_task_exists("Keep me");
}

// ============================================================================
// Navigation Tests
// ============================================================================

#[test]
fn test_navigation_with_vim_keys() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    // Switch to "All" filter so completed tasks stay visible
    app.set_filter_all();

    app.add_task("First");
    app.add_task("Second");
    app.add_task("Third");

    // Jump to top, complete first task to verify selection
    app.press("g");
    app.wait(Duration::from_millis(50));
    app.toggle_complete();
    app.wait(Duration::from_millis(100));

    // Move down twice to third, complete it
    app.move_down();
    app.move_down();
    app.toggle_complete();
    app.wait(Duration::from_millis(100));

    // Move up to second, complete it
    app.move_up();
    app.toggle_complete();
    app.wait(Duration::from_millis(100));

    // DB: all should be completed
    let db = app.quit();
    db.assert_completed_count(3);
    db.assert_task_completed("First");
    db.assert_task_completed("Second");
    db.assert_task_completed("Third");
}

#[test]
fn test_navigation_with_arrow_keys() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    // Switch to "All" filter so completed tasks stay visible
    app.set_filter_all();

    app.add_task("Task A");
    app.add_task("Task B");

    // Jump to top first
    app.press("g");
    app.wait(Duration::from_millis(50));

    // Navigate down with arrow keys and complete
    app.press_arrow("down");
    app.toggle_complete();
    app.wait(Duration::from_millis(100));

    // DB: second task should be completed
    let db = app.quit();
    db.assert_task_pending("Task A");
    db.assert_task_completed("Task B");
}

// ============================================================================
// Search Tests
// ============================================================================

#[test]
fn test_search_dialog_opens_and_closes() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    app.add_task("Searchable task");
    app.expect_text("Searchable task");

    // Open search
    app.open_search();
    app.wait(Duration::from_millis(100));

    // UI: search dialog should be visible
    app.expect_text("Search");

    // Close with Escape
    app.press_escape();
    app.wait(Duration::from_millis(100));

    // App should still be functional - add another task
    app.add_task("Another task");
    app.expect_text("Another task");

    let db = app.quit();
    db.assert_task_count(2);
}

// ============================================================================
// Priority Tests
// ============================================================================

#[test]
fn test_cycle_priority() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    app.add_task("Priority test");
    app.expect_text("Priority test");

    // Cycle priority multiple times
    app.press("p");
    app.wait(Duration::from_millis(100));
    app.press("p");
    app.wait(Duration::from_millis(100));

    // DB: task should still exist
    let db = app.quit();
    db.assert_task_exists("Priority test");
}

// ============================================================================
// Edit Tests
// ============================================================================

#[test]
fn test_edit_dialog_opens_and_closes() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    app.add_task("Original title");
    app.expect_text("Original title");

    // Open edit dialog
    app.edit_task();
    app.wait(Duration::from_millis(100));

    // Cancel with Escape
    app.press_escape();
    app.wait(Duration::from_millis(100));

    // DB: task should be unchanged
    let db = app.quit();
    db.assert_task_exists("Original title");
}

// ============================================================================
// Panel Switching Tests
// ============================================================================

#[test]
fn test_panel_switching() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    app.add_task("Test task");
    app.expect_text("Test task");

    // Switch panels with h/l
    app.press("h"); // To sidebar
    app.wait(Duration::from_millis(100));
    app.press("l"); // Back to task list
    app.wait(Duration::from_millis(100));

    // Switch with Tab
    app.press_tab();
    app.wait(Duration::from_millis(100));

    // DB: app should still be functional
    let db = app.quit();
    db.assert_task_count(1);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_special_characters_in_title() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    app.add_task("Task with 'quotes' and numbers 123");
    app.expect_text("Task with");

    let db = app.quit();
    db.assert_task_count(1);
}

#[test]
fn test_rapid_operations() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    // Rapid add
    for i in 1..=5 {
        app.add_task(&format!("Rapid task {}", i));
    }

    // Rapid navigation
    for _ in 0..10 {
        app.press("j");
    }
    for _ in 0..10 {
        app.press("k");
    }

    // DB: tasks should all be there
    let db = app.quit();
    db.assert_task_count(5);
}

// ============================================================================
// Full Workflow Tests
// ============================================================================

#[test]
fn test_full_task_lifecycle() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    // Switch to "All" filter so completed tasks stay visible for toggling back
    app.set_filter_all();

    // 1. Create task
    app.add_task("Lifecycle test");
    app.expect_text("Lifecycle test");

    // 2. Complete it
    app.toggle_complete();
    app.wait(Duration::from_millis(100));

    // 3. Uncomplete it
    app.toggle_complete();
    app.wait(Duration::from_millis(100));

    // 4. Delete it
    app.delete_task();
    app.wait(Duration::from_millis(100));

    // DB: should be gone
    let db = app.quit();
    db.assert_task_count(0);
    db.assert_task_not_exists("Lifecycle test");
}

#[test]
fn test_debug_view_toggle() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    app.add_task("Debug test");
    app.expect_text("Debug test");

    // Toggle debug view with F12
    app.session.send("\x1b[24~").expect("Failed to send F12");
    app.wait(Duration::from_millis(200));

    // Toggle back
    app.session.send("\x1b[24~").expect("Failed to send F12");
    app.wait(Duration::from_millis(200));

    // DB: app should still work
    let db = app.quit();
    db.assert_task_exists("Debug test");
}

// ============================================================================
// Filter Tests
// ============================================================================

#[test]
fn test_mixed_task_states() {
    let mut app = RatadoTest::spawn();
    app.wait_for_startup();

    // Switch to "All" filter so completed tasks stay visible
    app.set_filter_all();

    app.add_task("Pending task");
    app.add_task("Completed task");

    // Complete the second task
    app.press("g"); // Jump to top
    app.wait(Duration::from_millis(50));
    app.move_down();
    app.toggle_complete();
    app.wait(Duration::from_millis(100));

    // DB: verify database state
    let db = app.quit();
    db.assert_pending_count(1);
    db.assert_completed_count(1);
    db.assert_task_pending("Pending task");
    db.assert_task_completed("Completed task");
}
