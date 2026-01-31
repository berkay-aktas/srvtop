mod filter;
mod scanner;

use sysinfo::System;

fn main() {
    let mut system = System::new_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_all();

    let all = scanner::scan(&system);
    let dev_only = filter::filter_dev(all);

    println!(
        "{:<8} {:<20} {:<7} {:<6} {:<8} {:<10}",
        "PID", "NAME", "PORT", "PROTO", "CPU%", "MEMORY"
    );
    println!("{}", "-".repeat(60));

    for p in &dev_only {
        println!(
            "{:<8} {:<20} {:<7} {:<6} {:<8.1} {:<10}",
            p.pid, p.name, p.port, p.protocol, p.cpu_percent, p.memory_display
        );
    }

    println!("\n{} dev-relevant processes found", dev_only.len());
}
