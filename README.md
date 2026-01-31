# srvtop

Like `htop`, but for your dev servers.

A cross-platform TUI tool that auto-discovers development servers running on localhost and lets you monitor and kill them. Zero configuration required.

## Install

```
cargo install srvtop
```

## Usage

```
srvtop              # show dev-relevant servers
srvtop --all        # show all listening processes
srvtop -n 5         # refresh every 5 seconds
srvtop -p 3000      # filter to port 3000
```

## Keybindings

| Key | Action |
|-----|--------|
| Up/Down | Navigate rows |
| k | Kill selected process |
| r | Force refresh |
| a | Toggle all/dev-only |
| s | Cycle sort column |
| q | Quit |

## License

MIT
