//! Seeds a demo database with sample projects, tasks, and tags.
//!
//! Usage: cargo run --example seed_demo -- /path/to/demo.db

use chrono::{Duration, Utc};
use ratado::models::{Priority, Project, Task, TaskStatus};
use ratado::storage::{run_migrations, Database};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/demo.db".to_string());

    // Ensure parent dir exists
    if let Some(parent) = Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Remove old database if exists
    let _ = std::fs::remove_file(&db_path);

    let db = Database::open(Path::new(&db_path)).await?;
    run_migrations(&db).await?;

    // === Create Projects ===
    let platform = Project::with_style("Platform", "#e67e22", "🔧");
    let sprint = Project::with_style("Sprint 24", "#3498db", "🏃");
    let architecture = Project::with_style("Architecture", "#9b59b6", "🏗️");
    let hiring = Project::with_style("Hiring", "#2ecc71", "👥");
    let infra = Project::with_style("Infrastructure", "#e74c3c", "☁️");

    db.insert_project(&platform).await?;
    db.insert_project(&sprint).await?;
    db.insert_project(&architecture).await?;
    db.insert_project(&hiring).await?;
    db.insert_project(&infra).await?;

    let now = Utc::now();

    let tasks = vec![
        // === Platform - core engineering work ===
        make_task(
            "Investigate P0: memory leak in auth service",
            Some("Production pods restarting every ~4h. Heap dumps show growing connection pool. Check gRPC client lifecycle."),
            Some(now - Duration::hours(6)),
            Priority::Urgent,
            TaskStatus::InProgress,
            Some(&platform.id),
            vec!["incident", "production", "p0"],
        ),
        make_task(
            "Review PR #1847 - rate limiter middleware",
            Some("New token bucket implementation. Verify Redis fallback behavior and check for race conditions in the sliding window counter."),
            Some(now + Duration::hours(4)),
            Priority::High,
            TaskStatus::Pending,
            Some(&platform.id),
            vec!["code-review", "backend"],
        ),
        make_task(
            "Fix flaky integration test: test_concurrent_writes",
            Some("Fails ~15% of CI runs. Likely a timing issue with the test database setup. Check transaction isolation levels."),
            Some(now + Duration::days(1)),
            Priority::Medium,
            TaskStatus::Pending,
            Some(&platform.id),
            vec!["testing", "ci", "flaky"],
        ),
        make_task(
            "Bump OpenSSL to 3.2.1 (CVE-2024-0727)",
            Some("Security patch for PKCS12 decoding vulnerability. Update base Docker images and rebuild."),
            Some(now + Duration::hours(8)),
            Priority::High,
            TaskStatus::Pending,
            Some(&platform.id),
            vec!["security", "dependencies"],
        ),
        make_task(
            "Deploy hotfix v4.7.2 to production",
            None,
            Some(now - Duration::days(1)),
            Priority::Urgent,
            TaskStatus::Completed,
            Some(&platform.id),
            vec!["deploy", "production"],
        ),

        // === Sprint 24 - current sprint tasks ===
        make_task(
            "Implement WebSocket connection pooling",
            Some("Current implementation creates new connections per request. Design a pool with health checks, max idle time, and graceful drain."),
            Some(now + Duration::days(3)),
            Priority::High,
            TaskStatus::InProgress,
            Some(&sprint.id),
            vec!["feature", "performance", "backend"],
        ),
        make_task(
            "Add OpenTelemetry tracing to order service",
            Some("Instrument key spans: order creation, payment processing, inventory check. Add custom attributes for SLO tracking."),
            Some(now + Duration::days(2)),
            Priority::Medium,
            TaskStatus::Pending,
            Some(&sprint.id),
            vec!["observability", "tracing"],
        ),
        make_task(
            "Write migration: add composite index on (user_id, created_at)",
            Some("Query planner shows sequential scan on orders table for user timeline queries. Expected 40x improvement."),
            Some(now + Duration::days(1)),
            Priority::High,
            TaskStatus::Pending,
            Some(&sprint.id),
            vec!["database", "performance"],
        ),
        make_task(
            "Refactor error handling to use thiserror",
            Some("Replace anyhow with thiserror in the API layer. Define proper error enums for each module. Keep anyhow in CLI/scripts."),
            Some(now + Duration::days(5)),
            Priority::Low,
            TaskStatus::Pending,
            Some(&sprint.id),
            vec!["refactor", "rust"],
        ),
        make_task(
            "Ship user notification preferences API",
            None,
            Some(now - Duration::days(2)),
            Priority::High,
            TaskStatus::Completed,
            Some(&sprint.id),
            vec!["feature", "api"],
        ),

        // === Architecture - design and technical decisions ===
        make_task(
            "Write RFC: event-driven order processing",
            Some("Propose migration from synchronous REST calls to async event bus (Kafka). Cover: schema evolution, exactly-once delivery, dead letter queues, monitoring."),
            Some(now + Duration::days(4)),
            Priority::High,
            TaskStatus::InProgress,
            Some(&architecture.id),
            vec!["rfc", "architecture", "kafka"],
        ),
        make_task(
            "Evaluate CQRS for read-heavy analytics endpoints",
            Some("Current pattern hits the write DB for dashboard queries. Benchmark read replicas vs materialized views vs dedicated read models."),
            Some(now + Duration::days(6)),
            Priority::Medium,
            TaskStatus::Pending,
            Some(&architecture.id),
            vec!["architecture", "database", "cqrs"],
        ),
        make_task(
            "Design API versioning strategy for v2",
            Some("Options: URL path (/v2/), header-based, content negotiation. Need backward compat for 6 months. Draft deprecation policy."),
            Some(now + Duration::days(7)),
            Priority::Medium,
            TaskStatus::Pending,
            Some(&architecture.id),
            vec!["api", "architecture", "rfc"],
        ),
        make_task(
            "Review: database sharding proposal",
            Some("Team proposed hash-based sharding on tenant_id. Review the shard key selection, cross-shard query strategy, and rebalancing plan."),
            Some(now + Duration::days(3)),
            Priority::High,
            TaskStatus::Pending,
            Some(&architecture.id),
            vec!["architecture", "database", "review"],
        ),

        // === Hiring - team building ===
        make_task(
            "Review take-home: senior backend candidate (Alex M.)",
            Some("Rust CLI project submission. Check error handling patterns, test coverage, API design choices. Prep feedback for debrief."),
            Some(now + Duration::hours(6)),
            Priority::High,
            TaskStatus::Pending,
            Some(&hiring.id),
            vec!["hiring", "review"],
        ),
        make_task(
            "Write job description: Staff Platform Engineer",
            Some("Focus on distributed systems experience, Rust/Go, Kubernetes. Emphasize architecture ownership and mentorship expectations."),
            Some(now + Duration::days(2)),
            Priority::Medium,
            TaskStatus::Pending,
            Some(&hiring.id),
            vec!["hiring", "writing"],
        ),
        make_task(
            "Conduct system design interview - 2:00 PM",
            None,
            Some(now + Duration::days(1)),
            Priority::High,
            TaskStatus::Pending,
            Some(&hiring.id),
            vec!["hiring", "interview"],
        ),

        // === Infrastructure ===
        make_task(
            "Set up Terraform modules for new AWS region",
            Some("Expand to eu-west-1 for GDPR compliance. Replicate: VPC, EKS cluster, RDS, ElastiCache. Use existing modules as templates."),
            Some(now + Duration::days(5)),
            Priority::Medium,
            TaskStatus::Pending,
            Some(&infra.id),
            vec!["terraform", "aws", "infrastructure"],
        ),
        make_task(
            "Configure Prometheus alerting for SLOs",
            Some("Define burn rate alerts for p99 latency (< 200ms) and error rate (< 0.1%). Set up PagerDuty integration and runbooks."),
            Some(now + Duration::days(3)),
            Priority::Medium,
            TaskStatus::InProgress,
            Some(&infra.id),
            vec!["monitoring", "slo", "prometheus"],
        ),
        make_task(
            "Migrate CI from Jenkins to GitHub Actions",
            Some("Phase 1: build + test pipelines. Phase 2: deployment workflows. Phase 3: decommission Jenkins. Target: 2 week rollout."),
            Some(now + Duration::days(7)),
            Priority::Low,
            TaskStatus::Pending,
            Some(&infra.id),
            vec!["ci-cd", "github-actions"],
        ),
        make_task(
            "Rotate production database credentials",
            None,
            Some(now + Duration::hours(3)),
            Priority::High,
            TaskStatus::Pending,
            Some(&infra.id),
            vec!["security", "database", "ops"],
        ),

        // === Inbox - unsorted items ===
        make_task(
            "Prep talking points for eng all-hands Friday",
            Some("Topics: Q1 reliability improvements, new hire introductions, upcoming architecture changes, team OKR progress."),
            Some(now + Duration::days(2)),
            Priority::Medium,
            TaskStatus::Pending,
            Some("inbox"),
            vec!["meeting", "leadership"],
        ),
        make_task(
            "Read: 'Building Microservices' ch. 8-10",
            None,
            None,
            Priority::Low,
            TaskStatus::Pending,
            Some("inbox"),
            vec!["reading", "learning"],
        ),
        make_task(
            "Respond to architecture review comments",
            None,
            Some(now + Duration::hours(2)),
            Priority::Medium,
            TaskStatus::Pending,
            Some("inbox"),
            vec!["review", "follow-up"],
        ),
    ];

    for task in &tasks {
        db.insert_task(task).await?;
    }

    println!("Demo database seeded at: {}", db_path);
    println!("  Projects: 6 (including Inbox)");
    println!("  Tasks: {}", tasks.len());
    println!("\nRun with: cargo run -- -d {}", db_path);

    Ok(())
}

fn make_task(
    title: &str,
    description: Option<&str>,
    due_date: Option<chrono::DateTime<Utc>>,
    priority: Priority,
    status: TaskStatus,
    project_id: Option<&str>,
    tags: Vec<&str>,
) -> Task {
    let mut task = Task::new(title);
    task.description = description.map(|s| s.to_string());
    task.due_date = due_date;
    task.priority = priority;
    task.status = status;
    task.project_id = project_id.map(|s| s.to_string());
    task.tags = tags.into_iter().map(|s| s.to_string()).collect();
    if status == TaskStatus::Completed {
        task.completed_at = Some(Utc::now());
    }
    task
}
