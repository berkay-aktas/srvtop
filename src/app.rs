use sysinfo::System;

use crate::filter;
use crate::scanner::{self, DevProcess};

#[derive(Clone, Copy, PartialEq)]
pub enum SortColumn {
    Pid,
    Name,
    Port,
    Proto,
    Cpu,
    Memory,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

pub enum Message {
    Tick,
    Quit,
    NavigateUp,
    NavigateDown,
    Kill,
    ConfirmKill,
    CancelKill,
    Refresh,
    ToggleAll,
    CycleSort,
    ToggleSortDirection,
}

pub struct App {
    pub running: bool,
    pub processes: Vec<DevProcess>,
    pub selected: usize,
    pub show_all: bool,
    pub sort_column: SortColumn,
    pub sort_direction: SortDirection,
    pub show_kill_confirm: bool,
    pub status_message: Option<String>,
    pub status_timer: u8,
    pub system: System,
    pub filter_port: Option<u16>,
}

impl App {
    pub fn new(show_all: bool, filter_port: Option<u16>) -> Self {
        let mut system = System::new_all();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_all();

        let mut app = Self {
            running: true,
            processes: Vec::new(),
            selected: 0,
            show_all,
            sort_column: SortColumn::Port,
            sort_direction: SortDirection::Ascending,
            show_kill_confirm: false,
            status_message: None,
            status_timer: 0,
            system,
            filter_port,
        };
        app.refresh();
        app
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();

        let mut processes = match scanner::scan(&self.system) {
            Ok(p) => p,
            Err(e) => {
                self.status_message = Some(e);
                self.status_timer = 3;
                return;
            }
        };

        if !self.show_all {
            processes = filter::filter_dev(processes);
        }

        if let Some(port) = self.filter_port {
            processes.retain(|p| p.port == port);
        }

        self.sort(&mut processes);
        self.processes = processes;

        if self.selected >= self.processes.len() && !self.processes.is_empty() {
            self.selected = self.processes.len() - 1;
        }
    }

    pub fn sort(&self, processes: &mut [DevProcess]) {
        let dir = self.sort_direction;
        processes.sort_by(|a, b| {
            let ord = match self.sort_column {
                SortColumn::Pid => a.pid.cmp(&b.pid),
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::Port => a.port.cmp(&b.port),
                SortColumn::Proto => a.protocol.cmp(&b.protocol),
                SortColumn::Cpu => a.cpu_percent.partial_cmp(&b.cpu_percent).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Memory => a.memory_bytes.cmp(&b.memory_bytes),
            };
            match dir {
                SortDirection::Ascending => ord,
                SortDirection::Descending => ord.reverse(),
            }
        });
    }

    pub fn selected_process(&self) -> Option<&DevProcess> {
        self.processes.get(self.selected)
    }
}
