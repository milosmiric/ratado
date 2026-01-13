# Phase 10: CLI Interface

**Goal:** Implement command-line interface for non-interactive use.

**Prerequisites:** Phase 2 (database operations)

**Outcome:** Users can add/list/complete tasks from command line.

---

## Story 10.1: CLI Argument Parsing

**Priority:** Medium
**Estimate:** Small
**Status:** [ ] Not Started

### Description

Set up command-line argument parsing with clap.

### Tasks

- [ ] Update `src/main.rs` with clap CLI structure:
  - `ratado` (no args) - launch TUI
  - `ratado add <title>` - quick add task
  - `ratado list` - list tasks
  - `ratado complete <id>` - complete a task
  - `ratado version` - show version

- [ ] Global flags:
  - `--config <path>` - custom config file
  - `--no-notifications` - disable notifications
  - `-v, --verbose` - verbose output
  - `-h, --help` - show help
  - `-V, --version` - show version

### Code Sketch

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ratado")]
#[command(author, version, about = "Terminal task manager", long_about = None)]
pub struct Cli {
    /// Path to config file
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Disable desktop notifications
    #[arg(long, global = true)]
    no_notifications: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new task
    Add {
        /// Task title
        title: String,

        /// Due date (e.g., "tomorrow", "2025-01-20")
        #[arg(short, long)]
        due: Option<String>,

        /// Priority (low, medium, high, urgent)
        #[arg(short, long, default_value = "medium")]
        priority: String,

        /// Project name
        #[arg(long)]
        project: Option<String>,
    },

    /// List tasks
    List {
        /// Show only today's tasks
        #[arg(long)]
        today: bool,

        /// Show only this week's tasks
        #[arg(long)]
        week: bool,

        /// Filter by project
        #[arg(long)]
        project: Option<String>,

        /// Show completed tasks
        #[arg(long)]
        completed: bool,

        /// Output format (table, json)
        #[arg(long, default_value = "table")]
        format: String,
    },

    /// Complete a task
    Complete {
        /// Task ID (or partial ID)
        id: String,
    },

    /// Delete a task
    Delete {
        /// Task ID
        id: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
}

// In main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Set up logging based on verbose flag
    let log_level = if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    match cli.command {
        Some(Commands::Add { title, due, priority, project }) => {
            cmd::add_task(title, due, priority, project).await
        }
        Some(Commands::List { today, week, project, completed, format }) => {
            cmd::list_tasks(today, week, project, completed, format).await
        }
        Some(Commands::Complete { id }) => {
            cmd::complete_task(id).await
        }
        Some(Commands::Delete { id, force }) => {
            cmd::delete_task(id, force).await
        }
        None => {
            // Launch TUI
            run_tui(cli.config, cli.no_notifications).await
        }
    }
}
```

### Acceptance Criteria

- [ ] `ratado --help` shows all commands and options
- [ ] `ratado --version` shows version
- [ ] Global flags work with all subcommands
- [ ] No subcommand launches TUI
- [ ] Invalid commands show helpful error

---

## Story 10.2: Quick Add Command

**Priority:** Medium
**Estimate:** Small
**Status:** [ ] Not Started

### Description

Implement `ratado add` for quick task creation from terminal.

### Tasks

- [ ] Create `src/cmd/mod.rs` for CLI commands
- [ ] Create `src/cmd/add.rs`:
  - Parse title (required)
  - Parse due date (natural language: "tomorrow", "friday", "2025-01-20")
  - Parse priority
  - Find or create project
  - Insert task to database
  - Print confirmation

- [ ] Date parsing:
  - Support relative: "today", "tomorrow", "next week"
  - Support day names: "monday", "friday"
  - Support ISO: "2025-01-20"
  - Support natural: "in 3 days"

### Code Sketch

```rust
// src/cmd/add.rs
pub async fn add_task(
    title: String,
    due: Option<String>,
    priority: String,
    project: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = open_database().await?;

    let mut task = Task::new(&title);

    // Parse priority
    task.priority = match priority.to_lowercase().as_str() {
        "low" | "l" => Priority::Low,
        "medium" | "m" => Priority::Medium,
        "high" | "h" => Priority::High,
        "urgent" | "u" => Priority::Urgent,
        _ => {
            eprintln!("Invalid priority '{}', using medium", priority);
            Priority::Medium
        }
    };

    // Parse due date
    if let Some(due_str) = due {
        task.due_date = parse_due_date(&due_str);
        if task.due_date.is_none() {
            eprintln!("Could not parse date '{}', task created without due date", due_str);
        }
    }

    // Find or create project
    if let Some(project_name) = project {
        let projects = db.get_all_projects().await?;
        if let Some(p) = projects.iter().find(|p| p.name.eq_ignore_ascii_case(&project_name)) {
            task.project_id = Some(p.id.clone());
        } else {
            // Create new project
            let new_project = Project::new(&project_name);
            db.insert_project(&new_project).await?;
            task.project_id = Some(new_project.id.clone());
            println!("Created new project: {}", project_name);
        }
    }

    db.insert_task(&task).await?;

    println!("✓ Task added: \"{}\"", task.title);
    if let Some(due) = task.due_date {
        println!("  Due: {}", due.format("%Y-%m-%d %H:%M"));
    }
    println!("  Priority: {:?}", task.priority);
    println!("  ID: {}", &task.id[..8]);

    Ok(())
}

fn parse_due_date(input: &str) -> Option<DateTime<Utc>> {
    let input = input.to_lowercase();
    let now = Local::now();

    match input.as_str() {
        "today" => Some(now.date_naive().and_hms_opt(23, 59, 0)?.and_utc()),
        "tomorrow" => Some((now + Duration::days(1)).date_naive().and_hms_opt(23, 59, 0)?.and_utc()),
        "next week" => Some((now + Duration::weeks(1)).date_naive().and_hms_opt(23, 59, 0)?.and_utc()),
        _ => {
            // Try parsing as date
            if let Ok(date) = NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
                return Some(date.and_hms_opt(23, 59, 0)?.and_utc());
            }
            // Try day name
            if let Some(date) = parse_weekday(&input) {
                return Some(date);
            }
            None
        }
    }
}
```

### Acceptance Criteria

- [ ] `ratado add "Buy milk"` creates task
- [ ] `ratado add "Meeting" --due tomorrow` sets due date
- [ ] `ratado add "Urgent fix" --priority high` sets priority
- [ ] `ratado add "Work task" --project Work` assigns project
- [ ] Prints confirmation with task details
- [ ] Task visible in TUI

---

## Story 10.3: List and Complete Commands

**Priority:** Low
**Estimate:** Small
**Status:** [ ] Not Started

### Description

Implement list and complete CLI commands.

### Tasks

- [ ] Create `src/cmd/list.rs`:
  - Query tasks from database
  - Apply filters (--today, --week, --project)
  - Format output as table or JSON
  - Show ID, title, due date, priority, status

- [ ] Create `src/cmd/complete.rs`:
  - Find task by ID (support partial ID match)
  - Mark as completed
  - Print confirmation

- [ ] Create `src/cmd/delete.rs`:
  - Find task by ID
  - Confirm unless --force
  - Delete from database

### Code Sketch

```rust
// src/cmd/list.rs
pub async fn list_tasks(
    today: bool,
    week: bool,
    project: Option<String>,
    show_completed: bool,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = open_database().await?;
    let mut tasks = db.get_all_tasks().await?;

    // Apply filters
    if today {
        tasks.retain(|t| t.is_due_today());
    }
    if week {
        tasks.retain(|t| t.is_due_this_week());
    }
    if let Some(ref proj) = project {
        let projects = db.get_all_projects().await?;
        if let Some(p) = projects.iter().find(|p| p.name.eq_ignore_ascii_case(proj)) {
            tasks.retain(|t| t.project_id.as_ref() == Some(&p.id));
        }
    }
    if !show_completed {
        tasks.retain(|t| t.status != TaskStatus::Completed);
    }

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&tasks)?);
        }
        _ => {
            print_task_table(&tasks);
        }
    }

    Ok(())
}

fn print_task_table(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }

    println!("{:<10} {:<30} {:<12} {:<10} {:<10}",
        "ID", "TITLE", "DUE", "PRIORITY", "STATUS");
    println!("{}", "-".repeat(74));

    for task in tasks {
        let id = &task.id[..8];
        let title = if task.title.len() > 28 {
            format!("{}...", &task.title[..25])
        } else {
            task.title.clone()
        };
        let due = task.due_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "-".to_string());
        let priority = format!("{:?}", task.priority);
        let status = match task.status {
            TaskStatus::Pending => "[ ]",
            TaskStatus::InProgress => "[>]",
            TaskStatus::Completed => "[✓]",
            TaskStatus::Archived => "[A]",
        };

        println!("{:<10} {:<30} {:<12} {:<10} {:<10}",
            id, title, due, priority, status);
    }

    println!("\n{} task(s)", tasks.len());
}

// src/cmd/complete.rs
pub async fn complete_task(id: String) -> Result<(), Box<dyn std::error::Error>> {
    let db = open_database().await?;
    let tasks = db.get_all_tasks().await?;

    // Find task by partial ID match
    let matching: Vec<_> = tasks.iter()
        .filter(|t| t.id.starts_with(&id))
        .collect();

    match matching.len() {
        0 => {
            eprintln!("No task found with ID starting with '{}'", id);
            std::process::exit(1);
        }
        1 => {
            let mut task = matching[0].clone();
            task.complete();
            db.update_task(&task).await?;
            println!("✓ Completed: \"{}\"", task.title);
        }
        _ => {
            eprintln!("Multiple tasks match '{}', please be more specific:", id);
            for task in matching {
                eprintln!("  {} - {}", &task.id[..12], task.title);
            }
            std::process::exit(1);
        }
    }

    Ok(())
}
```

### Acceptance Criteria

- [ ] `ratado list` shows all pending tasks
- [ ] `ratado list --today` shows today's tasks
- [ ] `ratado list --project Work` filters by project
- [ ] `ratado list --format json` outputs JSON
- [ ] `ratado complete abc123` marks task complete
- [ ] Partial ID matching works
- [ ] `ratado delete abc123` removes task
- [ ] Delete asks for confirmation (unless --force)

---

## Phase 10 Checklist

Before moving to Phase 11, ensure:

- [ ] All 3 stories completed
- [ ] `ratado add` creates tasks from CLI
- [ ] `ratado list` displays tasks
- [ ] `ratado complete` marks tasks done
- [ ] Help text is clear and useful
- [ ] Error messages are helpful
