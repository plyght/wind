# wind-tui

Terminal UI for Wind VCS with lazygit-quality UX.

## Architecture

### Event Loop
- **Input Task**: Captures keyboard events via crossterm, dispatches to command registry
- **Model Task**: Manages repo state, executes background async tasks (status refresh, diff loading)
- **Render Loop**: Draws UI with ratatui at 4 FPS, triggered by state changes

### Components

| Component | Description | Keys |
|-----------|-------------|------|
| **Status Panel** | Current branch, staged/unstaged counts | - |
| **Files Panel** | Working tree files with stage/unstage | `Space`, `a`, `u` |
| **Diff Viewer** | Syntax-highlighted diff for selected file | - |
| **Commit Editor** | Modal text editor for commit messages | `c`, `Ctrl+Enter`, `Esc` |
| **Branch Graph** | ASCII-art branch visualization | `b` |
| **Command Palette** | Fuzzy command search | `Ctrl+p` |
| **Jobs Panel** | Background task progress indicators | - |
| **Notifications** | Transient success/error toasts | - |

### Command Registry

Extensible pattern mapping keybindings → commands. Default vim-style keys:

- `h/j/k/l`: Navigation
- `Space`: Stage/unstage file
- `a`: Stage all
- `u`: Unstage all
- `c`: Open commit editor
- `Ctrl+Enter`: Confirm commit
- `Esc`: Cancel/close
- `Tab`/`Shift+Tab`: Cycle panes
- `Ctrl+u/d`: Page up/down
- `r`: Refresh
- `q`: Quit

### Configuration

Load from `~/.config/wind/tui.toml`:

```toml
[theme]
fg = "white"
bg = "black"
accent = "cyan"
selection = "darkgray"
border = "gray"
added = "green"
removed = "red"
modified = "yellow"
```

RGB colors: `accent = { rgb = [100, 200, 255] }`

## Usage

```bash
cargo run --bin wind-tui
```

## Dependencies

- `ratatui`: TUI framework
- `crossterm`: Terminal manipulation
- `tokio`: Async runtime
- `tui-textarea`: Text input widget
- `anyhow`: Error handling
