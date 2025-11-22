# 🤖 wallflow: Rust Learning Project

**A wallpaper management CLI designed to teach Rust through practical development**

## 🎯 What This Is

wallflow is a Rust-based wallpaper management tool that serves as a structured learning vehicle for systems programming. The project prioritizes education over immediate feature completion, with carefully crafted TODOs and examples to guide development.

## 🚀 Current Features

- **Local wallpaper management** - Random selection from configured directories
- **Built-in daemon** - Self-contained scheduling without systemd
- **Auto-resolution detection** - Supports multiple display systems (xrandr, sway, wlr-randr)
- **Type-safe configuration** - YAML config with compile-time validation
- **Smooth transitions** - Integration with awww for animated wallpaper changes
- **Color scheme integration** - Optional pywal support for dynamic themes

## 📚 Learning Framework

**Educational Structure**:

- [docs/learning-path.md](docs/learning-path.md) - 8-week structured learning journey
- [docs/explore.md](docs/explore.md) - Real-world Rust ecosystem tour
- [bin/wallflow-reference](bin/wallflow-reference) - HTTP API construction examples

**Learning TODOs**: Key features are left as educational exercises with detailed implementation guidance:

```rust
// Example: HTTP integration as learning opportunity
pub async fn set_wallhaven(config: &Config, category: &str) -> Result<()> {
    // TODO: Implement Wallhaven API integration
    // 1. Get resolution from config.get_wallhaven_resolution()?
    // 2. Build API URL (see bin/wallflow-reference for examples)
    // 3. Make HTTP request with reqwest
    // ... detailed steps provided ...
}
```

## 🔧 Architecture

```rust
// Clean CLI structure
#[derive(Parser)]
#[command(name = "wallflow")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// Type-safe configuration
#[derive(Debug, Deserialize)]
pub struct Config {
    pub paths: PathsConfig,
    pub transition: TransitionConfig,
    pub sources: SourcesConfig,
}

// Built-in daemon
pub async fn run_foreground(config: Config) -> Result<()> {
    let mut timer = interval(Duration::from_secs(interval_secs));
    loop {
        timer.tick().await;
        set_wallpaper_by_source(&config).await?;
    }
}
```

## 🎨 TUI Evolution Plan

The project will evolve into a full TUI wallpaper manager with:

- **Interactive wallpaper browser** - Navigate collections with vim bindings
- **Real-time preview** - Live wallpaper preview and color schemes
- **Collection management** - Organize, tag, and curate wallpapers
- **Multiple monitor support** - Per-monitor wallpaper and timing control
- **Plugin system** - Extensible wallpaper sources and transition effects

**TUI Implementation Path**:

1. **Phase 1**: Basic ratatui interface (weeks 5-6 in learning path)
2. **Phase 2**: Vim bindings and navigation (weeks 7-8)
3. **Phase 3**: Advanced features (ongoing learning)

## 🛠️ Development

```bash
# Basic usage
cargo run -- config              # Show configuration
cargo run -- local              # Set local wallpaper

# Built-in daemon (no systemd!)
cargo run -- daemon --foreground

# Learning exercises (placeholders ready for implementation)
cargo run -- wallhaven nature   # TODO: Implement HTTP download
cargo run -- picsum             # TODO: Implement random photos
```

## 🗂️ Project Structure

```
wallflow/
├── src/
│   ├── main.rs      # CLI interface and command routing
│   ├── config.rs    # Type-safe YAML configuration
│   ├── daemon.rs    # Built-in scheduling daemon
│   ├── display.rs   # Auto-resolution detection
│   └── wallpaper.rs # Core wallpaper management + learning TODOs
├── docs/
│   ├── learning-path.md # 8-week structured journey
│   ├── explore.md      # Rust ecosystem examples
│   └── history.md      # Migration story
├── bin/wallflow-reference # HTTP API examples for learning
└── config.example.yml    # Configuration template
```

## 🎓 Learning Goals

**Week 1-4**: Foundation

- Rust fundamentals through config and CLI development
- Async programming with tokio daemon
- HTTP client implementation (educational exercises)

**Week 5-8**: Advanced

- TUI development with ratatui
- Complex state management
- Performance optimization and cross-platform support

**Beyond**: Choose your adventure

- Plugin architecture
- Web API development
- Machine learning integration

## 🚀 Quick Start

```bash
# Test current functionality
cargo run -- config
cargo run -- local

# Start learning journey
cat docs/learning-path.md
```

---

_Learning Rust through building something useful_ 🦀

**Migration history**: [docs/history.md](docs/history.md)
