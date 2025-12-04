# Contributing to wallflow

First off, thanks for considering contributing to wallflow! This is a learning project, and contributions of all skill levels are welcome.

## About This Project

wallflow is my first Rust project. I'm learning as I go, which means:

- The code might not always follow best practices (yet)
- I'm open to learning better patterns from experienced Rustaceans
- Beginner contributions are especially welcome - I understand the learning curve

## Ways to Contribute

### For Beginners

- **Documentation**: Fix typos, improve explanations, add examples
- **Bug reports**: Found something broken? Open an issue
- **Feature requests**: Ideas are welcome
- **Code cleanup**: Small refactors, better variable names, etc.

### For Experienced Rustaceans

- **Code review**: Help improve code quality with constructive feedback
- **Architecture**: Suggest better patterns or structures
- **Performance**: Identify bottlenecks and optimizations
- **New features**: See [TODO.md](TODO.md) for the roadmap

## Development Setup

```bash
# Clone the repo
git clone https://github.com/MKSG-MugunthKumar/wallflow
cd wallflow

# Build
cargo build

# Run tests
cargo test

# Run with verbose logging
cargo run -- --verbose local

# Check formatting and lints
cargo fmt --check
cargo clippy
```

## Pull Request Process

1. **Fork** the repository
2. **Create a branch** for your feature (`git checkout -b feature/amazing-feature`)
3. **Make your changes**
4. **Test** your changes (`cargo test`, `cargo clippy`)
5. **Commit** with a clear message
6. **Push** to your fork
7. **Open a PR** against `main`

### Commit Messages

Keep them simple and descriptive:

```
Add wallhaven search support
Fix daemon interval parsing
Update README installation instructions
```

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Add comments for non-obvious logic
- Keep functions focused and reasonably sized

## Project Structure

```
src/
├── main.rs           # CLI entry point
├── config/           # Configuration parsing
├── daemon.rs         # Background daemon
├── display.rs        # Resolution detection
├── downloaders/      # Wallpaper source implementations
│   ├── mod.rs
│   ├── wallhaven.rs
│   ├── reddit.rs
│   └── ...
├── integration/      # Desktop integrations (pywal, KDE)
├── tui/              # Terminal UI (ratatui)
├── updater.rs        # Self-update functionality
└── wallpaper/        # Wallpaper setting backends
```

## Adding a New Wallpaper Source

1. Create a new file in `src/downloaders/`
2. Implement the `WallpaperDownloader` trait
3. Register in `src/downloaders/registry.rs`
4. Add CLI command in `src/main.rs`
5. Update documentation

See existing downloaders (e.g., `bing.rs`, `picsum.rs`) for examples.

## Questions?

- Open an issue for bugs or feature discussions
- Tag your PR with `[WIP]` if it's a work in progress

## Code of Conduct

Be kind. We're all here to learn and build something useful.

---

Thanks for contributing!
