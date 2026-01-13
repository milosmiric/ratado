# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ratado is a terminal-based task manager and reminder application built with Rust and Ratatui. The name combines "Rata" (from Ratatui) with "do" (from todo).

## Documentation

- `/docs/SPECIFICATION.md` - Full requirements, features, data model, and technical architecture
- `/docs/UI_MOCKUPS.md` - ASCII mockups of all screens and UI states
- `/docs/TESTING_STRATEGY.md` - TDD approach, testing patterns, and examples
- `/docs/plan/` - Phased implementation plan (12 phases, 44 stories)

## Build Commands

```bash
cargo build           # Build the project
cargo build --release # Build with optimizations
cargo run             # Run the TUI application
cargo test            # Run all tests
cargo test <name>     # Run a specific test
cargo clippy          # Run linter
cargo fmt             # Format code
```

## Architecture

```
src/
├── main.rs              # Entry point
├── app.rs               # Application state (central App struct)
├── ui/                  # Ratatui widgets and views
├── models/              # Task, Project, Config structs
├── handlers/            # Keyboard input and command handling
├── storage/             # Turso database operations (~/.config/ratado/ratado.db)
├── notifications/       # Reminder system
└── utils/               # Date/time helpers
```

## Key Design Decisions

- **Database**: Turso - pure Rust SQLite-compatible embedded database (async-first)
- **Logging**: tui-logger with in-app debug view (F12) using `log` facade
- **Vim-style navigation**: j/k for up/down, h/l for panel switching
- **Local-first**: All data stored at `~/.config/ratado/` (ratado.db + config.toml)
- **Central state**: Single `App` struct manages all application state
- **Event loop**: Input events + timer events → update state → render UI

## Code Documentation Guidelines

All code must be well-documented using Rust's documentation conventions. This helps with learning and maintainability.

### Required Documentation

1. **Module-level docs** (`//!`) - Every `mod.rs` or module file should have a top-level description explaining:
   - What the module is responsible for
   - Key types and functions it exports
   - Usage examples where helpful

2. **Public items** (`///`) - All public structs, enums, functions, and traits must have:
   - A brief description of what it does
   - `# Arguments` section for functions with parameters
   - `# Returns` section for non-obvious return values
   - `# Examples` section for complex or non-obvious usage
   - `# Errors` section for functions that return `Result`
   - `# Panics` section if the function can panic

3. **Struct fields** - Document fields that aren't self-explanatory

4. **Enum variants** - Document each variant's purpose and when it's used

### Documentation Format

```rust
//! Module-level documentation goes here.
//!
//! This module handles X and provides Y.

/// Brief description of the struct.
///
/// More detailed explanation if needed, explaining the purpose
/// and how it fits into the larger system.
///
/// # Examples
///
/// ```
/// use ratado::models::Task;
///
/// let task = Task::new("Buy groceries");
/// assert_eq!(task.title, "Buy groceries");
/// ```
pub struct Task {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// The task title displayed in the UI
    pub title: String,
}

/// Creates a new task with the given title.
///
/// # Arguments
///
/// * `title` - The title for the new task
///
/// # Returns
///
/// A new `Task` with default values and a generated UUID
///
/// # Examples
///
/// ```
/// let task = Task::new("Complete project");
/// ```
pub fn new(title: &str) -> Self { ... }
```

### Running Documentation

```bash
cargo doc --open    # Generate and open docs in browser
cargo doc --no-deps # Generate docs without dependencies
```
