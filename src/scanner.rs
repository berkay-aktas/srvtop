use sysinfo::{Pid, System};

pub struct DevProcess {
    pub pid: u32,
    pub name: String,
    pub port: u16,
    pub protocol: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory_display: String,
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn scan(system: &System) -> Result<Vec<DevProcess>, String> {
    let listeners = match listeners::get_all() {
        Ok(l) => l,
        Err(e) => return Err(format!("Failed to scan ports: {}", e)),
    };

    let mut processes = Vec::new();

    for listener in listeners {
        let pid = listener.process.pid;
        let port = listener.socket.port();
        let protocol = format!("{:?}", listener.protocol);

        let (name, cpu_percent, memory_bytes) =
            if let Some(proc) = system.process(Pid::from(pid as usize)) {
                (
                    proc.name().to_string_lossy().to_string(),
                    proc.cpu_usage(),
                    proc.memory(),
                )
            } else {
                (listener.process.name.clone(), 0.0, 0)
            };

        let memory_display = format_bytes(memory_bytes);

        processes.push(DevProcess {
            pid,
            name,
            port,
            protocol,
            cpu_percent,
            memory_bytes,
            memory_display,
        });
    }

    Ok(processes)
}

#[cfg(test)]
impl DevProcess {
    pub fn test(name: &str, port: u16) -> Self {
        Self {
            pid: 1000,
            name: name.to_string(),
            port,
            protocol: "TCP".to_string(),
            cpu_percent: 0.0,
            memory_bytes: 0,
            memory_display: "0 B".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bytes_zero() {
        assert_eq!(format_bytes(0), "0 B");
    }

    #[test]
    fn format_bytes_bytes() {
        assert_eq!(format_bytes(512), "512 B");
    }

    #[test]
    fn format_bytes_kb() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
    }

    #[test]
    fn format_bytes_mb() {
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(25 * 1024 * 1024), "25.0 MB");
    }

    #[test]
    fn format_bytes_gb() {
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_bytes(2 * 1024 * 1024 * 1024), "2.0 GB");
    }
}
