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
