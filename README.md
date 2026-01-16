# Ratado

A fast, keyboard-driven terminal task manager built with Rust and [Ratatui](https://ratatui.rs/).

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)

## Features

- **Fast & Lightweight** - Instant startup, minimal resource usage
- **Keyboard-Driven** - Full functionality accessible without a mouse
- **Vim-Style Navigation** - Familiar keybindings for terminal users
- **Offline-First** - All data stored locally in SQLite
- **Projects & Tags** - Organize tasks with projects and flexible tagging
- **Smart Filtering** - Filter by status, priority, due date, project, or tag
- **Weekly Calendar** - Visual overview of tasks by due date
- **Full-Text Search** - Search across task titles and descriptions

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/milosmiric/ratado.git
cd ratado

# Build and install
cargo build --release

# Run
./target/release/ratado
```

### Requirements

- Rust 1.85 or later
- A terminal with Unicode support

## Usage

Launch Ratado:

```bash
ratado
```

Data is stored at `~/Library/Application Support/ratado/ratado.db` (macOS) or `~/.config/ratado/ratado.db` (Linux)

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g` / `Home` | Jump to top |
| `G` / `End` | Jump to bottom |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |
| `Tab` | Switch panel |
| `h` / `←` | Focus sidebar |
| `l` / `→` | Focus task list |

### Tasks

| Key | Action |
|-----|--------|
| `a` | Add new task |
| `e` / `Enter` | Edit selected task |
| `d` | Delete task |
| `Space` | Toggle completion |
| `p` | Cycle priority |
| `t` | Edit tags |
| `m` | Move to project |

### Projects (Sidebar Focused)

| Key | Action |
|-----|--------|
| `a` | Add new project |
| `e` / `Enter` | Edit project |
| `d` | Delete project |
| `Tab` | Switch between Projects/Tags |

### Filters & Views

| Key | Action |
|-----|--------|
| `f` | Open filter/sort dialog |
| `T` | Filter: Due today |
| `W` | Filter: Due this week |
| `1-4` | Filter by priority |
| `/` | Search tasks |
| `c` | Calendar view |
| `S` | Settings |

### General

| Key | Action |
|-----|--------|
| `?` | Show help |
| `r` | Refresh data |
| `F12` | Debug logs |
| `q` | Quit |
| `Ctrl+c` | Force quit |

## Screenshots

### Main View

```
Ratado v0.1.0   [Overdue: 2] [Due Today: 3]  12 tasks total
──────────────────────────────────────────────────────────────────────
PROJECTS          │ TASKS  [Pending]  [Due Date ↑]
                  │
▸ All Tasks (12)  │ ▶ [ ] !! Fix production bug                  Yesterday
  Inbox (2)       │   [ ] !! Review pull request #42             Yesterday
  Work (6)        │   [ ]  ! Deploy v2.0 release                     Today
  Personal (4)    │   [ ]    Update API documentation                Today
                  │   [ ]    Team standup meeting                    Today
                  │   [ ]    Write unit tests              @Work  #backend
                  │   [ ]    Refactor auth module          @Work  #backend
                  │   [ ]  ↓ Clean up old branches                  Friday
                  │   [ ]    Buy groceries                @Personal #home
                  │   [ ]    Schedule dentist appointment       @Personal
                  │   [ ]    Read "Clean Code" chapter 5  @Personal #books
                  │   [ ]  ↓ Organize desk                  @Inbox  #home
                  │
──────────────────────────────────────────────────────────────────────
a Add  e Edit  Space Done  / Search  c Calendar  f Filter  ? Help
```

## Task Attributes

| Attribute | Description |
|-----------|-------------|
| Title | Task name (required) |
| Description | Detailed notes with link support |
| Due Date | When task is due |
| Priority | Low, Medium, High, Urgent |
| Status | Pending, In Progress, Completed, Archived |
| Project | Group tasks under projects |
| Tags | Flexible categorization labels |

## Filtering Options

- **Status**: All, Pending, In Progress, Completed, Archived
- **Date**: Due Today, Due This Week, Overdue
- **Priority**: Urgent, High, Medium, Low
- **Organization**: By Project, By Tag

## Sorting Options

- Due Date (ascending/descending)
- Priority (highest/lowest first)
- Creation Date (newest/oldest)
- Alphabetical

## Architecture

```
src/
├── main.rs              # Entry point
├── app.rs               # Application state
├── ui/                  # Ratatui widgets and views
├── models/              # Task, Project, Filter structs
├── handlers/            # Keyboard input and commands
├── storage/             # SQLite database operations
└── utils/               # Date/time helpers
```

## Development

```bash
# Run in development
cargo run

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt

# Generate documentation
cargo doc --open
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Miloš Mirić**

- GitHub: [@milosmiric](https://github.com/milosmiric)
- LinkedIn: [milosmiric](https://www.linkedin.com/in/milosmiric/)

## Acknowledgments

- [Ratatui](https://ratatui.rs/) - Terminal UI framework
- [Turso](https://turso.tech/) - SQLite-compatible database
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
