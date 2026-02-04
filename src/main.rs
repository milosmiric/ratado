//! Ratado - Terminal Task Manager
//!
//! A terminal-based task manager built with Rust and Ratatui.

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::info;
use ratatui::{backend::CrosstermBackend, Terminal};

use ratado::app::App;
use ratado::handlers::{handle_event, EventHandler};
use ratado::storage::{check_and_update_app_version, run_migrations, Database};
use ratado::ui;

/// A fast, keyboard-driven terminal task manager
#[derive(Parser)]
#[command(name = "ratado")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the database file (defaults to platform-specific location)
    #[arg(short = 'd', long)]
    db_path: Option<PathBuf>,
}

/// Tick rate for the event loop (60fps).
///
/// Polling at 16ms is nearly free (just an OS syscall). The `needs_redraw` flag
/// in the main loop prevents unnecessary rendering when idle, so CPU stays low.
const TICK_RATE: Duration = Duration::from_millis(16);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments (handles --version and --help automatically)
    let cli = Cli::parse();

    // Initialize logging
    tui_logger::init_logger(log::LevelFilter::Debug)?;
    tui_logger::set_default_level(log::LevelFilter::Debug);

    info!("Starting Ratado v{}", env!("CARGO_PKG_VERSION"));

    // Setup panic hook to restore terminal on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal before showing panic
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize database
    let db_path = match cli.db_path {
        Some(path) => path,
        None => Database::default_path()?,
    };
    info!("Opening database at {:?}", db_path);
    let db = Database::open(&db_path).await?;
    run_migrations(&db).await?;
    check_and_update_app_version(&db).await?;

    // Initialize app
    let mut app = App::new(db).await?;
    info!("App initialized with {} tasks", app.tasks.len());

    // Run the main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    restore_terminal()?;

    // Handle any errors from the main loop
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    info!("Ratado exited cleanly");
    Ok(())
}

/// Restores the terminal to its original state.
fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

/// Main application loop.
///
/// Handles rendering, input events, and ticks until the app signals to quit.
/// Runs at a fixed 16ms tick (60fps). The `needs_redraw` flag skips rendering
/// when nothing has changed, keeping idle CPU usage low.
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>>
where
    <B as ratatui::backend::Backend>::Error: 'static,
{
    let mut events = EventHandler::new(TICK_RATE);

    let mut needs_redraw = true;

    loop {
        if needs_redraw {
            // Draw the UI, then process effects on the buffer
            terminal.draw(|frame| {
                ui::draw(frame, app);
                let area = frame.area();

                // Lazy-start splash animation on first frame (needs terminal area)
                if app.current_view == ratado::app::View::Splash && !app.splash_started {
                    app.splash_started = true;
                    app.animation.start_splash();
                }

                // Process pending task animations using render info
                if let Some(render_info) = ui::task_list::take_last_render_info() {
                    // New task coalesce animation
                    if let Some(task_id) = app.pending_new_task_animation.take() {
                        if let Some(rect) = render_info.find_task_rect(&task_id) {
                            app.animation.start_task_new(rect);
                        }
                    }
                    // Task complete flash animation
                    if let Some(task_id) = app.pending_complete_animation.take() {
                        if let Some(rect) = render_info.find_task_rect(&task_id) {
                            app.animation.start_task_complete(rect);
                        }
                    }
                    // Priority change flash animation
                    if let Some((task_id, color)) = app.pending_priority_animation.take() {
                        if let Some(rect) = render_info.find_task_rect(&task_id) {
                            app.animation.start_priority_flash(rect, color);
                        }
                    }
                }

                // Process effects after widgets have rendered
                app.animation.process(frame.buffer_mut(), area);
            })?;
            needs_redraw = false;
        }

        // Wait for and handle the next event
        if let Some(event) = events.next().await {
            match &event {
                ratado::handlers::AppEvent::Key(_)
                | ratado::handlers::AppEvent::Resize(_, _)
                | ratado::handlers::AppEvent::Mouse(_) => {
                    needs_redraw = true;
                }
                ratado::handlers::AppEvent::Tick => {
                    // Redraw on tick if animations are active or state transitions are pending
                    if app.animation.has_active_effects()
                        || app.current_view == ratado::app::View::Splash
                        || !app.dissolving_tasks.is_empty()
                        || app.pending_new_task_animation.is_some()
                        || app.pending_complete_animation.is_some()
                        || app.pending_priority_animation.is_some()
                    {
                        needs_redraw = true;
                    }
                }
            }

            // handle_event returns false when the app should quit
            if !handle_event(app, event).await? {
                break;
            }
        }

        // Double-check quit flag (in case command set it without returning false)
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
