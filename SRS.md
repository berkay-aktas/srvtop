# srvtop — Software Requirements Specification

## 1. Overview

**srvtop** is a cross-platform terminal user interface (TUI) tool that auto-discovers development servers running on localhost and lets you monitor and kill them. Zero configuration required.

**Tagline:** Like htop, but for your dev servers.

## 2. Problem Statement

Developers run multiple local services simultaneously (frontend dev servers, databases, APIs, caches). Managing them requires:
- Remembering platform-specific commands (`lsof -i :3000`, `netstat -ano`, `ss -tlnp`)
- Opening multiple terminal tabs
- Manually tracking what's running where

No existing tool provides a zero-config, dev-focused, interactive TUI for this.

## 3. Target Users

- Web developers running local dev stacks (Node, Python, Go, Ruby, PHP, Java, Rust)
- Backend developers managing databases and services locally (Postgres, Redis, MongoDB)
- Full-stack developers juggling 3-5+ services during development

## 4. Functional Requirements

### 4.1 Auto-Discovery (FR-1)
- The system SHALL scan all TCP listening ports on the local machine
- The system SHALL identify the process name, PID, port, protocol, and bind address for each listener
- The system SHALL enrich each process with CPU usage (%) and memory usage (bytes, human-readable)
- Discovery SHALL work without any configuration files or prior setup

### 4.2 Dev-Relevance Filtering (FR-2)
- By default, the system SHALL filter to show only dev-relevant processes
- A process is dev-relevant if its name matches a known dev tool (node, python, java, go, ruby, postgres, redis, nginx, vite, webpack, etc.) OR its port matches a well-known dev port (3000, 3001, 4200, 5000, 5173, 5432, 6379, 8000, 8080, 8888, 9000, 27017, etc.)
- The user SHALL be able to toggle to show ALL listening processes via the `--all` flag or the `a` key

### 4.3 TUI Display (FR-3)
- The system SHALL display processes in an interactive table with columns: PID, NAME, PORT, PROTO, CPU%, MEMORY
- The table SHALL support row selection via arrow keys (up/down)
- The selected row SHALL be visually highlighted
- A header SHALL show the tool name, version, and time since last refresh
- A footer SHALL show available keybindings and the count of visible processes
- The table SHALL auto-refresh at a configurable interval (default: 3 seconds)

### 4.4 Process Kill (FR-4)
- The user SHALL be able to kill the selected process by pressing `k`
- A confirmation dialog SHALL appear before killing: "Kill <name> (PID <pid>) on :<port>? [y/n]"
- On confirmation (`y`), the system SHALL terminate the process
- On Unix: send SIGTERM
- On Windows: execute `taskkill /PID <pid> /F`
- A status message SHALL confirm the action ("Killed PID 1234") or report failure ("Process already exited")

### 4.5 Sorting (FR-5)
- The user SHALL be able to sort the table by any column
- Pressing `s` SHALL cycle through sort columns
- Pressing `s` on the same column SHALL toggle ascending/descending
- The currently sorted column SHALL display a directional indicator (▲/▼)

### 4.6 CLI Arguments (FR-6)
- `--all` / `-a`: Show all listening processes, bypass dev filter
- `--interval <SECS>` / `-n <SECS>`: Set refresh interval (default: 3)
- `--port <PORT>` / `-p <PORT>`: Filter to a specific port
- `--help` / `-h`: Print usage information
- `--version` / `-V`: Print version

### 4.7 Keybindings (FR-7)

| Key | Action |
|-----|--------|
| ↑ / ↓ | Navigate rows |
| k | Kill selected process (shows confirmation) |
| y | Confirm kill |
| n / Esc | Cancel kill |
| r | Force refresh |
| a | Toggle all vs dev-only |
| s | Cycle sort column |
| q | Quit |

## 5. Non-Functional Requirements

### 5.1 Performance (NFR-1)
- Full scan + render cycle SHALL complete in under 200ms
- The TUI SHALL remain responsive during scanning (no input lag)
- Memory usage SHALL stay under 20MB for typical workloads (<50 processes)

### 5.2 Cross-Platform (NFR-2)
- The tool SHALL run on Windows 10/11, macOS 12+, and Linux (glibc-based)
- Terminal rendering SHALL work in: Windows Terminal, PowerShell, iTerm2, Terminal.app, and common Linux terminal emulators
- No platform-specific installation steps (single binary)

### 5.3 Zero Configuration (NFR-3)
- The tool SHALL work immediately after installation with no config files
- All behavior SHALL be controllable via CLI arguments only

### 5.4 Error Handling (NFR-4)
- If elevated permissions are required, the tool SHALL display a clear message rather than crash
- If a process exits between scan and kill, the tool SHALL handle gracefully with a status message
- Network scanning errors SHALL be caught and displayed, not propagated as panics

## 6. Technical Architecture

### 6.1 Technology Stack
- Language: Rust (2021 edition)
- TUI framework: ratatui 0.30 with crossterm backend
- Port scanning: `listeners` crate (cross-platform port-to-PID mapping)
- Process metrics: `sysinfo` crate (cross-platform CPU/memory)
- CLI parsing: `clap` 4 with derive macros
- Error handling: `color-eyre`

### 6.2 Architecture Pattern
The Elm Architecture (TEA): Model → View → Update
- **Model** (`app.rs`): Centralized application state
- **View** (`ui.rs`): Pure rendering function, no side effects
- **Update** (`update.rs`): Message-driven state transitions
- **Events** (`event.rs`): Synchronous crossterm polling + tick timer

### 6.3 Module Structure

```
src/
├── main.rs          # Entry point, CLI args, terminal setup/teardown, main loop
├── app.rs           # App state struct, Message enum, SortColumn enum
├── event.rs         # Event polling: keyboard input + tick timer
├── update.rs        # Message handling, state transitions, platform-specific kill
├── ui.rs            # All rendering: table, header, footer, kill confirmation popup
├── scanner.rs       # Port/process discovery orchestrator
└── filter.rs        # Dev-relevance filtering logic
```

### 6.4 Data Flow

```
Every 3 seconds (or on keypress):
  1. event.rs emits Message::Tick or Message::Key(key)
  2. update.rs handles the message:
     - Tick/Refresh → scanner::scan() → filter → sort → update app.processes
     - NavigateUp/Down → adjust selection index
     - Kill → show confirmation popup
     - ConfirmKill → platform-specific kill signal
     - Quit → set app.running = false
  3. ui.rs reads App state → renders frame
```

### 6.5 Key Data Structure

```rust
pub struct DevProcess {
    pub pid: u32,
    pub name: String,
    pub port: u16,
    pub protocol: String,
    pub address: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory_display: String,
}
```

## 7. Distribution

### 7.1 Installation Methods
- `cargo install srvtop` (primary)
- GitHub Releases with prebuilt binaries (Linux x64, macOS x64/ARM, Windows x64)
- Future: Homebrew tap, AUR, scoop

### 7.2 CI/CD
- GitHub Actions: test + clippy on ubuntu-latest, macos-latest, windows-latest
- Release workflow: triggered by git tags (v*), builds and uploads binaries

## 8. Out of Scope (v1)

- Log tailing
- Process restart
- "Open in browser" action
- Mouse support
- Configuration files
- Custom themes
- Search/filter text input
- Process tree view
- Sparklines or historical charts
- Async runtime (tokio)
- UDP listeners

## 9. Success Criteria

- Tool launches and shows dev servers within 1 second on all 3 platforms
- Kill flow works reliably with confirmation
- Auto-refresh updates the display without flicker
- README includes install instructions and a demo GIF
- Published to crates.io
- CI passes on all 3 platforms
