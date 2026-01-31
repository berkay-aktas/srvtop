# srvtop — Implementation Plan

## Implementation Order

### Step 1: Scaffold
- `cargo init` in the project directory
- Set up Cargo.toml with all dependencies
- Add .gitignore, LICENSE (MIT), README stub
- Initialize git repo

### Step 2: Scanner + Filter (core logic)
- Implement `scanner.rs` — combine `listeners::get_all()` with `sysinfo` enrichment
- Implement `filter.rs` — dev-relevance filtering by process name and port
- Verify with a simple `main.rs` that prints discovered processes to stdout

### Step 3: App State
- Implement `app.rs` — App struct, Message enum, DevProcess struct, SortColumn
- Constructor with defaults, refresh method

### Step 4: Event Loop
- Implement `event.rs` — sync crossterm polling with tick timer
- Wire into `main.rs` with terminal setup/teardown (raw mode, alternate screen)

### Step 5: TUI Rendering
- Implement `ui.rs` — table with header/footer, row selection highlight
- Layout: header (1 line) + table (flex) + footer (2 lines)
- Column widths: PID(7), NAME(flex), PORT(6), PROTO(5), CPU%(7), MEMORY(10)
- Colors: cyan header, green dev ports, dark gray selected row

### Step 6: Update Logic
- Implement `update.rs` — handle all messages
- Navigation (up/down with wrapping)
- Refresh (rescan + filter + sort)
- Sort (cycle columns, toggle direction)
- Toggle all/dev-only
- Quit

### Step 7: Kill Flow
- Add kill confirmation popup overlay in `ui.rs`
- Add ConfirmKill/CancelKill message handling in `update.rs`
- Platform-specific kill: `#[cfg(unix)]` SIGTERM, `#[cfg(windows)]` taskkill
- Status message on success/failure, auto-clears after 3s

### Step 8: CLI Args
- Add clap derive struct in `main.rs`
- Wire --all, --interval, --port flags into App state

### Step 9: Polish
- Edge cases: empty process list, permission errors, terminal resize
- Status messages with auto-clear
- Sort direction indicators (▲/▼) in column headers

### Step 10: Testing + CI
- Unit tests for filter, format_bytes, sort, state transitions
- Integration test: bind TcpListener, verify scan finds it
- GitHub Actions workflow: test on 3 platforms
- Release workflow: build binaries on tag push

### Step 11: README + Launch Prep
- Write README with: description, screenshot/GIF, install instructions, usage, keybindings
- Record terminal demo GIF (< 5MB)
- Publish to crates.io

## Key Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Dependencies: ratatui, crossterm, listeners, sysinfo, clap, color-eyre |
| `src/main.rs` | CLI args, terminal setup/teardown, main loop |
| `src/app.rs` | App state, Message enum, DevProcess struct |
| `src/scanner.rs` | Port discovery + process enrichment |
| `src/filter.rs` | Dev-relevance filtering |
| `src/event.rs` | Keyboard + tick event polling |
| `src/update.rs` | State transitions + kill logic |
| `src/ui.rs` | All TUI rendering |

## Dependencies

```toml
ratatui = "0.30"
crossterm = "0.28"
listeners = "0.3"
sysinfo = "0.33"
color-eyre = "0.6"
clap = { version = "4", features = ["derive"] }
```

## Verification

1. `cargo build` succeeds
2. `cargo run` shows TUI with running dev servers
3. `cargo run -- --all` shows all listeners
4. Arrow keys navigate, `k` → `y` kills, status message shows
5. Auto-refresh works every 3s
6. `cargo test` passes
7. `cargo clippy` clean
