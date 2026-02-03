use crate::app::{App, Message, OwlMood, SortColumn, SortDirection};

pub fn update(app: &mut App, msg: Message) {
    // Tick down status message timer
    if app.status_timer > 0 {
        app.status_timer -= 1;
        if app.status_timer == 0 {
            app.status_message = None;
        }
    }

    match msg {
        Message::Quit => {
            if app.show_kill_confirm {
                app.show_kill_confirm = false;
            } else {
                app.running = false;
            }
        }
        Message::Tick => {
            app.refresh();
        }
        Message::Refresh => {
            app.refresh();
            app.status_message = Some("Refreshed".to_string());
            app.status_timer = 3;
            app.set_owl_mood(OwlMood::Flap, 800);
        }
        Message::NavigateUp => {
            if app.show_kill_confirm {
                return;
            }
            if app.selected > 0 {
                app.selected -= 1;
            } else if !app.processes.is_empty() {
                app.selected = app.processes.len() - 1;
            }
            app.scrollbar_state = app.scrollbar_state.position(app.selected);
            app.set_owl_mood(OwlMood::LookUp, 500);
        }
        Message::NavigateDown => {
            if app.show_kill_confirm {
                return;
            }
            if app.selected + 1 < app.processes.len() {
                app.selected += 1;
            } else {
                app.selected = 0;
            }
            app.scrollbar_state = app.scrollbar_state.position(app.selected);
            app.set_owl_mood(OwlMood::LookDown, 500);
        }
        Message::Kill => {
            if !app.show_kill_confirm {
                if let Some(p) = app.selected_process() {
                    app.kill_target = Some((p.pid, p.name.clone(), p.port));
                    app.show_kill_confirm = true;
                    app.set_owl_mood(OwlMood::Alarmed, 2000);
                }
            }
        }
        Message::ConfirmKill => {
            if app.show_kill_confirm {
                if let Some((pid, name, _port)) = app.kill_target.take() {
                    match kill_process(pid) {
                        Ok(()) => {
                            app.status_message = Some(format!("Killed {} (PID {})", name, pid));
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Failed to kill PID {}: {}", pid, e));
                        }
                    }
                    app.status_timer = 3;
                    app.set_owl_mood(OwlMood::Alarmed, 1500);
                }
                app.show_kill_confirm = false;
                app.refresh();
            }
        }
        Message::CancelKill => {
            app.show_kill_confirm = false;
            app.kill_target = None;
        }
        Message::ToggleAll => {
            if app.show_kill_confirm {
                return;
            }
            app.show_all = !app.show_all;
            app.selected = 0;
            app.refresh();
            app.set_owl_mood(OwlMood::WideEye, 800);
        }
        Message::CycleSort => {
            if app.show_kill_confirm {
                return;
            }
            app.sort_column = match app.sort_column {
                SortColumn::Pid => SortColumn::Name,
                SortColumn::Name => SortColumn::Port,
                SortColumn::Port => SortColumn::Proto,
                SortColumn::Proto => SortColumn::Cpu,
                SortColumn::Cpu => SortColumn::Memory,
                SortColumn::Memory => SortColumn::Uptime,
                SortColumn::Uptime => SortColumn::Pid,
            };
            app.sort_direction = SortDirection::Ascending;
            app.refresh();
        }
        Message::ToggleSortDirection => {
            if app.show_kill_confirm {
                return;
            }
            app.sort_direction = match app.sort_direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
            app.refresh();
        }
    }
}

#[cfg(unix)]
fn kill_process(pid: u32) -> Result<(), String> {
    use std::process::Command;
    let output = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}

#[cfg(windows)]
fn kill_process(pid: u32) -> Result<(), String> {
    use std::process::Command;
    let output = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}
