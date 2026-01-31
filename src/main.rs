mod app;
mod event;
mod filter;
mod scanner;

use std::io;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::{App, Message};
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
            frame.render_widget(
                ratatui::widgets::Paragraph::new(format!(
                    "srvtop — {} processes (press q to quit)",
                    app.processes.len()
                )),
                frame.area(),
            );
        })?;

        // Handle events
        if let Some(msg) = events.next() {
            match msg {
                Message::Quit => app.running = false,
                Message::Tick | Message::Refresh => app.refresh(),
                Message::NavigateUp => {
                    if app.selected > 0 {
                        app.selected -= 1;
                    }
                }
                Message::NavigateDown => {
                    if app.selected + 1 < app.processes.len() {
                        app.selected += 1;
                    }
                }
                _ => {}
            }
        }
    }

    // Terminal teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
