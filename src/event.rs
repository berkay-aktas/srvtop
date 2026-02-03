use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::Message;

pub struct EventHandler {
    tick_rate: Duration,
    last_tick: Instant,
}

impl EventHandler {
    pub fn new(tick_rate_secs: u64) -> Self {
        Self {
            tick_rate: Duration::from_secs(tick_rate_secs),
            last_tick: Instant::now(),
        }
    }

    pub fn next(&mut self) -> Option<Message> {
        let until_tick = self
            .tick_rate
            .saturating_sub(self.last_tick.elapsed());
        // Cap poll at 200ms so the mascot animation stays smooth
        let timeout = until_tick.min(Duration::from_millis(200));

        if event::poll(timeout).ok()? {
            if let Event::Key(key) = event::read().ok()? {
                return self.handle_key(key);
            }
        }

        if self.last_tick.elapsed() >= self.tick_rate {
            self.last_tick = Instant::now();
            return Some(Message::Tick);
        }

        None
    }

    fn handle_key(&self, key: KeyEvent) -> Option<Message> {
        if key.kind != crossterm::event::KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Message::Quit)
            }
            KeyCode::Up | KeyCode::Char('k') => Some(Message::NavigateUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Message::NavigateDown),
            KeyCode::Char('x') => Some(Message::Kill),
            KeyCode::Char('y') => Some(Message::ConfirmKill),
            KeyCode::Char('n') | KeyCode::Esc => Some(Message::CancelKill),
            KeyCode::Char('r') => Some(Message::Refresh),
            KeyCode::Char('a') => Some(Message::ToggleAll),
            KeyCode::Char('s') => Some(Message::CycleSort),
            KeyCode::Char('S') => Some(Message::ToggleSortDirection),
            _ => None,
        }
    }
}
