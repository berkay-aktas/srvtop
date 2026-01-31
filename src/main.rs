mod app;
mod event;
mod filter;
mod scanner;
mod ui;
mod update;

use std::io;
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::App;
use event::EventHandler;

#[derive(Parser)]
#[command(name = "srvtop", version, about = "Like htop, but for your dev servers")]
struct Cli {
    /// Show all listening processes, not just dev-relevant ones
    #[arg(short, long)]
    all: bool,

    /// Refresh interval in seconds
    #[arg(short = 'n', long = "interval", default_value_t = 3)]
    interval: u64,

    /// Filter to a specific port
    #[arg(short, long)]
    port: Option<u16>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    // Restore terminal on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Splash screen
    let splash_duration = Duration::from_millis(1500);
    let splash_start = Instant::now();
    terminal.draw(ui::draw_splash)?;
    while splash_start.elapsed() < splash_duration {
        if crossterm::event::poll(Duration::from_millis(50))? {
            if let crossterm::event::Event::Key(_) = crossterm::event::read()? {
                break;
            }
        }
    }

    // App + event loop
    let mut app = App::new(cli.all, cli.port, cli.interval);
    let mut events = EventHandler::new(cli.interval);

    while app.running {
        // Render
        terminal.draw(|frame| {
            ui::draw(frame, &mut app);
        })?;

        // Handle events
        if let Some(msg) = events.next() {
            update::update(&mut app, msg);
        }
    }

    // Terminal teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
