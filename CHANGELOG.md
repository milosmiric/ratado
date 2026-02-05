# Changelog

All notable changes to Ratado will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-02-03

### Added

- **Quick Capture dialog** - Spotlight-style single-line task entry with inline syntax (`@project #tag !priority due:date`), real-time autocomplete for projects/tags/priorities, and live preview of parsed metadata
- **CLI database path flag** - `--db-path` / `-d` option to specify a custom database file location
- **Version tracking** - Application version is stored in the database (`_app_meta` table) to detect upgrades and downgrades on startup
- **End-to-end test suite** - 21 integration tests using `expectrl` for terminal interaction testing
- **Enhanced project dialog** - Expanded from 8 to 16 preset colors and 16 preset icons, displayed in 2 rows of 8 with colored circle indicators and hint lines
- **Up/down navigation** in project dialog color and icon selectors to move between rows

### Changed

- **Task keybindings** - `a` now opens Quick Capture (was full Add Task form); `A` (Shift+A) opens the full Add Task form
- **Status bar hints** - Updated to show new keybindings including Quick Capture and delete
- **Help screen version** - Now reads version dynamically from `Cargo.toml` instead of being hardcoded
- Optimized task-tag association retrieval for better performance

## [0.1.0] - 2025

### Added

- Initial release
- Task management (create, edit, delete, complete, reopen, move)
- Projects with custom colors and icons
- Tags with autocomplete
- Filtering by status, priority, due date, project, and tag
- Sorting by due date, priority, creation date, and alphabetically
- Vim-style keyboard navigation
- Weekly calendar view
- Full-text search
- Task detail view
- Settings dialog (delete completed tasks, reset database)
- Dark blue-violet theme
- In-app debug log viewer (F12)
- Local SQLite database via Turso
