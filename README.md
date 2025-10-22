# Wind

Wind is a modern version control companion built on top of Git. It keeps your existing Git workflows while adding AI-assisted authoring, stacked change management, and a thoughtful terminal experience.

## Highlights

- **Git-Compatible**: Use Wind interchangeably with standard Git commands.
- **Stack-Aware**: Organize dependent changes and land them safely.
- **AI-Assisted**: Generate commit messages and review context with your preferred provider.
- **Interactive TUI**: Stage, review, and resolve conflicts directly from the terminal.
- **Performance-Oriented**: Optimized operations for large and active repositories.

## Getting Started

```bash
cargo install wind

cd my-project
wind init
wind add .
wind commit -m "Initial commit"
wind push
```

Enable AI helpers when you're ready:

```bash
wind ai configure --provider openai --api-key sk-...
wind ai enable
```

## Why the Name

Wind captures the idea of moving fast and smoothly through your work. Like a steady breeze, Wind is meant to clear the path ahead—helping you glide across branches, stacks, and reviews without the turbulence of manual bookkeeping.

## Project Components

- **wind**: Combined core engine, CLI, and terminal UI built on `git2`.
- **wind-ai**: AI providers and prompt orchestration.
- **wind-bridge**: Git bridge utilities for syncing changesets.
- **wind-collab**: Integrations for pull requests and shared workflows.
- **wind-git**: Low-level Git adapter layer.
- **wind-storage**: Chunked object storage primitives.

## Development

Run the quality gates before submitting changes:

```bash
cargo fmt
cargo clippy
cargo test --all
cargo test --test integration_tests
```

Build optimized binaries with `cargo build --release`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for full guidelines.

## License

MIT License. See [LICENSE](LICENSE) for details.
