# 🌊 wallflow

**Elegant wallpaper management with smooth transitions and dynamic color schemes**

## ✨ Features

- 🎨 **Multiple Sources**: Local collections, Wallhaven, Picsum
- 🌈 **Dynamic Colors**: Automatic color scheme generation with pywal
- 🔄 **Smooth Transitions**: Configurable animations via awww daemon
- 🤖 **Built-in Daemon**: No more systemd complexity (Rust version)
- ⚙️ **Auto-Resolution**: Detects your display resolution automatically
- 🧹 **Smart Cleanup**: Configurable retention of downloaded wallpapers
- 🔌 **Desktop Integration**: Works with KDE, Hyprland, and more

## 🚀 Quick Start

```bash
# Build and run
cargo build --release
./target/release/wallflow local

# Built-in daemon (no systemd needed!)
./target/release/wallflow daemon --foreground
```

## 📚 Documentation

- **[Learning Path](docs/learning-path.md)** - Step-by-step Rust learning with wallflow
- **[Ecosystem Explorer](docs/explore.md)** - Real-world Rust CLI/TUI examples
- **[Configuration Guide](config/wallflow.yml)** - Detailed configuration options

## 🎯 Current Status

- ✅ **Rust Foundation**: CLI, config, daemon, display detection
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
