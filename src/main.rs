mod app;
mod event;
mod filter;
mod scanner;
mod ui;
mod update;

use std::io;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::App;
use event::EventHandler;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App + event loop
    let mut app = App::new(false, None);
    let mut events = EventHandler::new(3);

    while app.running {
        // Render
        terminal.draw(|frame| {
            ui::draw(frame, &app);
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
