# 🌊 wallflow

**Elegant wallpaper management with smooth transitions and dynamic color schemes**

> **🚧 Rust Rewrite in Progress**: This is being rewritten in Rust for better performance, type safety, and cross-platform support. See [docs/learning-path.md](docs/learning-path.md) for the learning journey.

## ✨ Features

- 🎨 **Multiple Sources**: Local collections, Wallhaven, Picsum
- 🌈 **Dynamic Colors**: Automatic color scheme generation with pywal
- 🔄 **Smooth Transitions**: Configurable animations via awww daemon
- 🤖 **Built-in Daemon**: No more systemd complexity (Rust version)
- ⚙️ **Auto-Resolution**: Detects your display resolution automatically
- 🧹 **Smart Cleanup**: Configurable retention of downloaded wallpapers
- 🔌 **Desktop Integration**: Works with KDE, Hyprland, and more

## 🚀 Quick Start

**Current (Bash) Version:**

```bash
# Set a wallpaper manually
./bin/wallflow local

# See configuration
./bin/wallflow config
```

**New (Rust) Version:**

```bash
# Build and run
cargo build --release
./target/release/wallflow local

# Built-in daemon (no systemd needed!)
./target/release/wallflow daemon --foreground
```

## 📁 Project Structure

```
wallflow/
├── src/                    # Rust source code
│   ├── main.rs            # CLI interface
│   ├── config.rs          # Type-safe YAML configuration
│   ├── daemon.rs          # Built-in scheduling daemon
│   ├── display.rs         # Auto-resolution detection
│   └── wallpaper.rs       # Wallpaper application logic
├── bin/                    # Reference scripts
│   ├── wallflow           # Original bash script (legacy)
│   ├── wallflow-config    # Configuration management (legacy)
│   └── wallflow-reference # URL construction examples
├── docs/                   # Documentation
│   ├── learning-path.md   # Rust learning journey
│   └── explore.md         # Real-world examples & ecosystem
├── config/
│   └── wallflow.yml       # Configuration template
├── Cargo.toml             # Rust project configuration
└── config.example.yml     # Example configuration
```

## 📚 Documentation

- **[Learning Path](docs/learning-path.md)** - Step-by-step Rust learning with wallflow
- **[Ecosystem Explorer](docs/explore.md)** - Real-world Rust CLI/TUI examples
- **[Configuration Guide](config/wallflow.yml)** - Detailed configuration options

## 🎯 Current Status

- ✅ **Rust Foundation**: CLI, config, daemon, display detection
- 🚧 **In Progress**: Wallhaven/Picsum API integration
- ⏳ **Planned**: TUI interface, advanced features

## 🔧 Development

```bash
# Check compilation
cargo check

# Run with verbose logging
cargo run -- --verbose config

# Test daemon mode
cargo run -- daemon --foreground
```

## 🤝 Contributing

This project serves as both a useful tool and a Rust learning experience. Feel free to:

- Try the Rust rewrite and provide feedback
- Suggest improvements or features
- Share your own learning journey

---

_"Evolution from shell scripts to systems programming"_ 🐚➡️🦀
