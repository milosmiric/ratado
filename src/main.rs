//! Ratado - Terminal Task Manager
//!
//! A terminal-based task manager built with Rust and Ratatui.

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::info;
use ratatui::{backend::CrosstermBackend, Terminal};

use ratado::app::App;
use ratado::storage::{Database, run_migrations};
use ratado::ui;

/// Tick rate for the event loop (250ms for checking reminders)
const TICK_RATE: Duration = Duration::from_millis(250);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let db_path = Database::default_path()?;
    info!("Opening database at {:?}", db_path);
    let db = Database::open(&db_path).await?;
    run_migrations(&db).await?;

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
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>>
where
    <B as ratatui::backend::Backend>::Error: 'static,
{
    let mut last_tick = Instant::now();

    while !app.should_quit {
        // Draw the UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Calculate timeout for event polling
        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());

        // Poll for events
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    handle_key_event(app, key).await?;
                }
                Event::Resize(_, _) => {
                    // Terminal will redraw on next iteration
                }
                _ => {}
            }
        }

        // Handle tick
        if last_tick.elapsed() >= TICK_RATE {
            app.on_tick();
            last_tick = Instant::now();
        }
    }

    Ok(())
}

/// Handles keyboard input.
///
/// For now, implements basic navigation and quit commands.
/// Full handler implementation will come in Phase 4.
async fn handle_key_event(
    app: &mut App,
    key: event::KeyEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    use ratado::{InputMode, View, FocusPanel};

    // Global keybindings (work in any mode)
    match key.code {
        // Quit with Ctrl+C
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            return Ok(());
        }
        // Toggle debug logs with F12
        KeyCode::F(12) => {
            app.current_view = if app.current_view == View::DebugLogs {
                View::Main
            } else {
                View::DebugLogs
            };
            return Ok(());
        }
        _ => {}
    }

    // Mode-specific handling
    match app.input_mode {
        InputMode::Normal => {
            match key.code {
                // Quit
                KeyCode::Char('q') => {
                    app.should_quit = true;
                }
                // Navigation
                KeyCode::Char('j') | KeyCode::Down => {
                    if app.focus == FocusPanel::TaskList {
                        app.select_next_task();
                    } else {
                        app.select_next_project();
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if app.focus == FocusPanel::TaskList {
                        app.select_previous_task();
                    } else {
                        app.select_previous_project();
                    }
                }
                // Panel switching
                KeyCode::Char('h') | KeyCode::Left => {
                    app.focus = FocusPanel::Sidebar;
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    app.focus = FocusPanel::TaskList;
                }
                KeyCode::Tab => {
                    app.toggle_focus();
                }
                // Filter/Sort
                KeyCode::Char('f') => {
                    app.cycle_filter();
                }
                KeyCode::Char('s') => {
                    app.cycle_sort();
                }
                // Help
                KeyCode::Char('?') => {
                    app.current_view = View::Help;
                }
                // Escape from help/other views
                KeyCode::Esc => {
                    app.current_view = View::Main;
                }
                _ => {}
            }
        }
        InputMode::Editing | InputMode::Search => {
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    app.input_buffer.clear();
                    app.input_cursor = 0;
                }
                KeyCode::Char(c) => {
                    app.input_buffer.insert(app.input_cursor, c);
                    app.input_cursor += 1;
                }
                KeyCode::Backspace => {
                    if app.input_cursor > 0 {
                        app.input_cursor -= 1;
                        app.input_buffer.remove(app.input_cursor);
                    }
                }
                KeyCode::Left => {
                    app.input_cursor = app.input_cursor.saturating_sub(1);
                }
                KeyCode::Right => {
                    if app.input_cursor < app.input_buffer.len() {
                        app.input_cursor += 1;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
