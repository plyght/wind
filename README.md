# Wind VCS

A modern version control system built on Git with AI-powered features, stacked diffs, and a beautiful TUI.

## Features

- 🚀 **Git-Compatible**: Full interoperability with Git
- 🤖 **AI-Powered**: Smart commit messages and code suggestions
- 📚 **Stacked Diffs**: Manage chains of dependent branches
- 🎨 **Beautiful TUI**: Interactive terminal interface
- 🔄 **Smart Rebasing**: Automatic conflict resolution
- 🔗 **PR Integration**: Direct GitHub/GitLab integration
- ⚡ **Fast**: Optimized for large repositories

## Quick Start

### Installation

```bash
cargo install wind-cli
```

### Initialize a Repository

```bash
cd my-project
wind init
```

### Basic Workflow

```bash
wind add .
wind commit -m "My changes"

wind commit --ai

wind push
```

## Commands

### Repository Management

- `wind init [path]` - Initialize a new Wind repository
- `wind status` / `wind st` - Show working tree status
- `wind log [-n N] [--graph]` - Show commit history

### File Operations

- `wind add <files>` / `wind stage <files>` - Stage files for commit
- `wind add -a` - Stage all changes
- `wind commit -m "message"` - Create a commit
- `wind commit --ai` - Create commit with AI-generated message

### Branch Management

- `wind branch [name]` - Create a new branch
- `wind branch -l` - List all branches
- `wind branch -d <name>` - Delete a branch
- `wind checkout <branch>` - Switch to a branch

### Stacked Workflows

- `wind stack list` - List all stacks
- `wind stack create <name>` - Create a new stack
- `wind stack rebase` - Rebase entire stack
- `wind stack land` - Merge stack to main

### Pull Requests

- `wind pr create [-t title] [-b body]` - Create a pull request
- `wind pr update <number>` - Update existing PR
- `wind pr list` - List pull requests

### Advanced Operations

- `wind rebase <onto>` - Rebase current branch
- `wind tui` - Launch interactive TUI
- `wind config get/set <key> [value]` - Manage configuration

### AI Features

- `wind ai enable` - Enable AI features
- `wind ai disable` - Disable AI features
- `wind ai configure --provider <provider> --api-key <key>` - Configure AI

## Configuration

### Global Config

```bash
wind config set user.name "Your Name"
wind config set user.email "you@example.com"
```

### AI Configuration

```bash
wind ai configure --provider openai --api-key sk-...

wind ai configure --provider anthropic --api-key sk-ant-...

wind ai configure --provider local
```

### Repository Config

Create `.wind/config.toml`:

```toml
[user]
name = "Your Name"
email = "you@example.com"

[ai]
enabled = true
provider = "openai"

[stack]
auto_rebase = true

[pr]
default_base = "main"
```

## Examples

### Stacked Workflow

```bash
wind init
wind commit -m "Base feature"

wind stack create my-feature
wind branch feature-part-1
wind commit -m "Part 1"

wind checkout main
wind branch feature-part-2
wind commit -m "Part 2"

wind stack rebase

wind stack land
```

### AI-Assisted Commits

```bash
wind add .
wind commit --ai

```

### Interactive TUI

```bash
wind tui

j/k: Navigate
<space>: Stage/unstage
c: Commit
p: Push
q: Quit
```

## Architecture

Wind is built on top of Git and consists of:

- **wind-cli**: Command-line interface
- **wind-core**: Core repository operations
- **wind-tui**: Terminal user interface
- **wind-ai**: AI-powered features
- **wind-collab**: PR and collaboration features

### Storage

```
.wind/
├── config.toml          # Repository configuration
├── stacks/              # Stack metadata
├── cache/               # Performance cache
└── .git -> ../.git      # Pointer to Git repository
```

## Development

### Build from Source

```bash
git clone https://github.com/windvcs/wind
cd wind
cargo build --release
```

### Run Tests

```bash
cargo test --all

cargo test --test integration_tests
```

### Lint and Format

```bash
cargo fmt
cargo clippy
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

Built with:
- [clap](https://github.com/clap-rs/clap) - CLI parsing
- [tokio](https://tokio.rs) - Async runtime
- [ratatui](https://github.com/ratatui-org/ratatui) - TUI framework
- [git2-rs](https://github.com/rust-lang/git2-rs) - Git bindings
