# Wind VCS - Agent Guide

## Build Commands

```bash
cargo build --release
cargo clippy --all
cargo fmt --all
cargo test --all
cargo test --test integration_tests
cargo test --package wind --lib watcher       # Test file watching
cargo bench --package wind --bench perf_benchmarks  # Run performance benchmarks
```

## Project Structure

```
wind/
├── crates/
│   ├── wind/          # Combined core, CLI, and TUI
│   ├── wind-ai/       # AI helpers (stub)
│   ├── wind-bridge/   # Git bridge utilities
│   ├── wind-collab/   # PR/collab features (stub)
│   ├── wind-git/      # Git adapter layer
│   └── wind-storage/  # Object storage primitives
└── docs/
```

## CLI Commands

- `wind init` - Initialize repository
- `wind status` / `wind st` - Show status
- `wind add <files>` / `wind stage` - Stage changes
- `wind commit -m "msg"` - Create commit
- `wind commit --ai` - AI-generated commit message
- `wind log [-n N]` - Show history
- `wind branch [name]` - Branch operations
- `wind checkout <branch>` - Switch branches
- `wind stack list/create/rebase/land` - Stack management
- `wind pr create/update/list` - Pull request operations
- `wind tui` - Launch TUI
- `wind ai enable/configure` - AI configuration
- `wind config get/set` - Configuration

## Integration Tests

Tests are in `crates/wind/tests/integration_tests.rs` and compare wind vs git CLI output:
- Repository initialization
- Git compatibility
- Round-trip operations
- Branch management
- Commit/log operations

Run with: `cargo test --test integration_tests`

## CI/CD

GitHub Actions pipeline (.github/workflows/ci.yml):
- Runs on Linux, macOS, Windows
- Lint with clippy
- Format check with rustfmt
- All tests including integration
- Release builds uploaded as artifacts

## Dependencies

- **clap**: CLI parsing
- **tokio**: Async runtime
- **git2**: Git operations
- **colored**: Terminal colors
- **indicatif**: Progress bars
- **ctrlc**: Signal handling
- **anyhow**: Error handling

## Notes

- TUI, AI, and Collab features are stubs
- Core operations use git2-rs for Git compatibility
- .wind directory stores metadata
- Tests use tempfile for isolation

## Performance Benchmarks

Run benchmarks:
```bash
cargo bench --package wind --bench perf_benchmarks
```

Results saved to `target/criterion/` with HTML reports at `target/criterion/report/index.html`.

See PERFORMANCE.md and BENCHMARKS.md for details.
