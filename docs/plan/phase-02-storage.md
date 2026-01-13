# Phase 2: Storage Layer

**Goal:** Implement the Turso database layer with migrations and CRUD operations.

**Prerequisites:** Phase 1 (models must be implemented)

**Outcome:** Database creates tables, all CRUD operations work, integration tests pass.

---

## Story 2.1: Database Connection Setup

**Priority:** Critical
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Set up Turso database connection and initialization.

### Tasks

- [ ] Create `src/storage/database.rs`:
  - `Database` struct wrapping Turso connection
  - `Database::open(path: &Path) -> Result<Database>` - open or create database
  - `Database::open_in_memory() -> Result<Database>` - for testing
  - `Database::close(self)` - cleanup
  - Error type for database operations

- [ ] Implement XDG path resolution:
  - Use `directories` crate
  - Default path: `~/.config/ratado/ratado.db`
  - Create directory if not exists

- [ ] Add connection error handling with `thiserror`

### Code Sketch

```rust
use turso::Builder;
use directories::ProjectDirs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] turso::Error),
    #[error("Failed to create config directory: {0}")]
    ConfigDir(#[from] std::io::Error),
    #[error("Could not determine config directory")]
    NoConfigDir,
}

pub struct Database {
    conn: turso::Connection,
}

impl Database {
    pub async fn open(path: &Path) -> Result<Self, StorageError> {
        let db = Builder::new_local(path).build().await?;
        let conn = db.connect()?;
        Ok(Self { conn })
    }

    pub async fn open_in_memory() -> Result<Self, StorageError> {
        let db = Builder::new_local(":memory:").build().await?;
        let conn = db.connect()?;
        Ok(Self { conn })
    }

    pub fn default_path() -> Result<PathBuf, StorageError> {
        let proj_dirs = ProjectDirs::from("", "", "ratado")
            .ok_or(StorageError::NoConfigDir)?;
        let data_dir = proj_dirs.config_dir();
        std::fs::create_dir_all(data_dir)?;
        Ok(data_dir.join("ratado.db"))
    }
}
```

### Acceptance Criteria

- [ ] Database creates at `~/.config/ratado/ratado.db`
- [ ] In-memory database works for tests
- [ ] Proper error messages on connection failure
- [ ] Directory created if not exists

---

## Story 2.2: Database Migrations

**Priority:** Critical
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Implement schema migrations for database setup and versioning.

### Tasks

- [ ] Create `src/storage/migrations.rs`:
  - `Migration` struct with version and SQL
  - `MIGRATIONS` static array of all migrations
  - `run_migrations(db: &Database) -> Result<()>`
  - Create `_migrations` version tracking table

- [ ] Implement Migration V1 (initial schema):
  ```sql
  -- Tasks table
  CREATE TABLE IF NOT EXISTS tasks (
      id TEXT PRIMARY KEY,
      title TEXT NOT NULL,
      description TEXT,
      due_date TEXT,  -- ISO8601 timestamp
      priority TEXT NOT NULL DEFAULT 'medium',
      status TEXT NOT NULL DEFAULT 'pending',
      project_id TEXT,
      created_at TEXT NOT NULL,
      updated_at TEXT NOT NULL,
      completed_at TEXT,
      FOREIGN KEY (project_id) REFERENCES projects(id)
  );

  -- Projects table
  CREATE TABLE IF NOT EXISTS projects (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      color TEXT DEFAULT '#4A90D9',
      icon TEXT DEFAULT 'folder',
      created_at TEXT NOT NULL
  );

  -- Tags table
  CREATE TABLE IF NOT EXISTS tags (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL UNIQUE
  );

  -- Task-Tags junction
  CREATE TABLE IF NOT EXISTS task_tags (
      task_id TEXT NOT NULL,
      tag_id TEXT NOT NULL,
      PRIMARY KEY (task_id, tag_id),
      FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
      FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
  );

  -- Indexes
  CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
  CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
  CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority);
  CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_id);
  ```

- [ ] Add migration for default "Inbox" project

### Code Sketch

```rust
struct Migration {
    version: u32,
    description: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial schema",
        sql: include_str!("migrations/001_initial.sql"),
    },
    Migration {
        version: 2,
        description: "Add default Inbox project",
        sql: "INSERT OR IGNORE INTO projects (id, name, created_at)
              VALUES ('inbox', 'Inbox', datetime('now'))",
    },
];

pub async fn run_migrations(db: &Database) -> Result<(), StorageError> {
    // Create migrations table
    db.execute("CREATE TABLE IF NOT EXISTS _migrations (
        version INTEGER PRIMARY KEY,
        applied_at TEXT NOT NULL
    )", ()).await?;

    // Get current version
    let current: u32 = db.query_one(
        "SELECT COALESCE(MAX(version), 0) FROM _migrations", ()
    ).await?;

    // Apply pending migrations
    for migration in MIGRATIONS.iter().filter(|m| m.version > current) {
        db.execute(migration.sql, ()).await?;
        db.execute(
            "INSERT INTO _migrations (version, applied_at) VALUES (?, datetime('now'))",
            (migration.version,)
        ).await?;
        log::info!("Applied migration {}: {}", migration.version, migration.description);
    }

    Ok(())
}
```

### Acceptance Criteria

- [ ] Fresh database gets all tables created
- [ ] Migrations are idempotent (can run multiple times)
- [ ] Schema matches specification
- [ ] Version tracking works correctly
- [ ] Default "Inbox" project exists

---

## Story 2.3: Task Repository

**Priority:** Critical
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Implement CRUD operations for tasks.

### Tasks

- [ ] Create `src/storage/tasks.rs` or add to `database.rs`:
  - `insert_task(&self, task: &Task) -> Result<()>`
  - `get_task(&self, id: &str) -> Result<Option<Task>>`
  - `get_all_tasks(&self) -> Result<Vec<Task>>`
  - `update_task(&self, task: &Task) -> Result<()>`
  - `delete_task(&self, id: &str) -> Result<()>`
  - `query_tasks(&self, filter: &Filter, sort: &SortOrder) -> Result<Vec<Task>>`

- [ ] Implement proper parameter binding (prevent SQL injection)
- [ ] Handle datetime serialization (ISO8601 strings)
- [ ] Add integration tests with in-memory database

### Code Sketch

```rust
impl Database {
    pub async fn insert_task(&self, task: &Task) -> Result<(), StorageError> {
        self.conn.execute(
            "INSERT INTO tasks (id, title, description, due_date, priority,
             status, project_id, created_at, updated_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            (
                &task.id,
                &task.title,
                &task.description,
                task.due_date.map(|d| d.to_rfc3339()),
                task.priority.as_str(),
                task.status.as_str(),
                &task.project_id,
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339(),
                task.completed_at.map(|d| d.to_rfc3339()),
            )
        ).await?;
        Ok(())
    }

    pub async fn get_all_tasks(&self) -> Result<Vec<Task>, StorageError> {
        let rows = self.conn.query("SELECT * FROM tasks", ()).await?;
        let mut tasks = Vec::new();
        while let Some(row) = rows.next().await? {
            tasks.push(Task::from_row(&row)?);
        }
        Ok(tasks)
    }

    // ... other methods
}
```

### Test Cases

```rust
#[tokio::test]
async fn test_insert_and_get_task() {
    let db = Database::open_in_memory().await.unwrap();
    run_migrations(&db).await.unwrap();

    let task = Task::new("Test task");
    db.insert_task(&task).await.unwrap();

    let retrieved = db.get_task(&task.id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().title, "Test task");
}

#[tokio::test]
async fn test_update_task() {
    let db = Database::open_in_memory().await.unwrap();
    run_migrations(&db).await.unwrap();

    let mut task = Task::new("Original");
    db.insert_task(&task).await.unwrap();

    task.title = "Updated".to_string();
    task.priority = Priority::High;
    db.update_task(&task).await.unwrap();

    let retrieved = db.get_task(&task.id).await.unwrap().unwrap();
    assert_eq!(retrieved.title, "Updated");
    assert_eq!(retrieved.priority, Priority::High);
}
```

### Acceptance Criteria

- [ ] All CRUD operations work correctly
- [ ] Tasks persist across database reopens
- [ ] Filtering by status, project, due date works
- [ ] Sorting works correctly
- [ ] All integration tests pass

---

## Story 2.4: Project & Tag Repository

**Priority:** High
**Estimate:** Small
**Status:** [ ] Not Started

### Description

Implement CRUD operations for projects and tags.

### Tasks

- [ ] Create `src/storage/projects.rs`:
  - `insert_project(&self, project: &Project) -> Result<()>`
  - `get_all_projects(&self) -> Result<Vec<Project>>`
  - `get_project(&self, id: &str) -> Result<Option<Project>>`
  - `update_project(&self, project: &Project) -> Result<()>`
  - `delete_project(&self, id: &str) -> Result<()>`
  - `get_task_count_by_project(&self, project_id: &str) -> Result<usize>`

- [ ] Create `src/storage/tags.rs`:
  - `insert_tag(&self, name: &str) -> Result<String>` (returns tag id)
  - `get_all_tags(&self) -> Result<Vec<Tag>>`
  - `get_or_create_tag(&self, name: &str) -> Result<String>`
  - `delete_tag(&self, id: &str) -> Result<()>`
  - `add_tag_to_task(&self, task_id: &str, tag_id: &str) -> Result<()>`
  - `remove_tag_from_task(&self, task_id: &str, tag_id: &str) -> Result<()>`
  - `get_tags_for_task(&self, task_id: &str) -> Result<Vec<Tag>>`
  - `get_task_count_by_tag(&self, tag_id: &str) -> Result<usize>`

- [ ] Add integration tests

### Acceptance Criteria

- [ ] Projects can be created, listed, updated, deleted
- [ ] Tags can be created and assigned to tasks
- [ ] Tags can be removed from tasks
- [ ] Task counts per project/tag are accurate
- [ ] Deleting a tag removes it from all tasks
- [ ] All integration tests pass

---

## Phase 2 Checklist

Before moving to Phase 3, ensure:

- [ ] All 4 stories completed
- [ ] Database creates and persists correctly
- [ ] All CRUD operations work
- [ ] Migrations run successfully
- [ ] All integration tests pass
- [ ] `cargo test` passes
