# wallflow

[![Build Status](https://github.com/MKSG-MugunthKumar/wallflow/actions/workflows/release.yml/badge.svg)](https://github.com/MKSG-MugunthKumar/wallflow/actions/workflows/release.yml)
[![Crates.io](https://img.shields.io/crates/v/wallflow)](https://crates.io/crates/wallflow)
[![License](https://img.shields.io/github/license/MKSG-MugunthKumar/wallflow)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS-blue)](https://github.com/MKSG-MugunthKumar/wallflow)

**Elegant wallpaper management with smooth transitions**

A modern, Rust-based wallpaper manager that brings fluidity to your Linux desktop with multiple sources, smooth transitions, and dynamic color schemes.

## About This Project

This is my first Rust project. I decided to learn Rust by building something I'd actually use daily. wallflow started as a learning exercise and evolved into a reasonably decent wallpaper manager. While toying around with hyprland, I used [awww](https://codeberg.org/LGFae/awww) for smooth transitions and loved it. So I decided to do something like variety, but powered by aww so you have smooth transitions. KWin in KDE support wlr-layer-shell, and awww works perfectly. Sorry, GNOME.

Thus, wallflow was born.

The codebase reflects this journey and that means, it is not perfect. Use it at your own risk. But hey, it's open source! Feel free to contribute and help me improve it.

## Features

- **Multiple Wallpaper Sources**

  - Local collections
  - [Wallhaven](https://wallhaven.cc) - Search-based wallpapers
  - [Unsplash](https://unsplash.com) - High-resolution photos
  - [Reddit](https://reddit.com) - Subreddit-based (earthporn, wallpapers, etc.)
  - [NASA APOD](https://apod.nasa.gov) - Astronomy Picture of the Day
  - [Bing](https://www.bing.com) - Daily photo
  - [Google Earth View](https://earthview.withgoogle.com) - Satellite imagery
  - [Picsum](https://picsum.photos) - Random photos

- **Smart Features**

  - Built-in daemon for automatic wallpaper rotation
  - Smooth animated transitions (via [awww](https://codeberg.org/LGFae/awww))
  - Auto-resolution detection (xrandr, sway, wlr-randr)
  - Optional [pywal](https://github.com/dylanaraps/pywal) integration for dynamic color schemes
  - Self-update capability

- **Desktop Support**
  - [Hyprland](https://hyprland.org), Sway, and other wlroots compositors
  - KDE Plasma (KWin supports wlr-layer-shell)
  - X11 (feh, nitrogen)

## Installation

### From crates.io

```bash
cargo install wallflow
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/MKSG-MugunthKumar/wallflow/releases):

```bash
# Download and install
chmod +x wallflow-x86_64-unknown-linux-gnu
mv wallflow-x86_64-unknown-linux-gnu ~/.local/bin/wallflow
```

### Fedora (COPR)

```bash
sudo dnf copr enable uproar9991/wallflow
sudo dnf install wallflow
```

### From Source

```bash
git clone https://github.com/MKSG-MugunthKumar/wallflow
cd wallflow
cargo build --release
cp target/release/wallflow ~/.local/bin/
```

## Quick Start

```bash
# Set wallpaper from local collection
wallflow local

# Download from online sources
wallflow wallhaven nature mountains
wallflow reddit earthporn
wallflow unsplash architecture
wallflow bing
wallflow apod
wallflow earthview

# Run as daemon (auto-rotate)
wallflow daemon

# Run daemon in foreground (for testing)
wallflow daemon --foreground

# Check for updates
wallflow update --check
```

## Running as a Service (Linux)

wallflow can run as a background service that automatically rotates your wallpaper.

### Option 1: systemd (recommended)

```bash
# Install binary and service
make install
make install-service
make enable-service

# Management commands
make status              # Check service status
make logs                # Follow service logs
make disable-service     # Stop and disable
make uninstall-service   # Remove service file
```

### Option 2: XDG Autostart

Simpler alternative that starts wallflow on login:

```bash
make install
make install-autostart   # Starts on next login

make uninstall-autostart # Remove
```

## Configuration

Configuration file location:

- **Linux**: `~/.config/mksg/wallflow/config.yml`
- **macOS**: `~/Library/Application Support/mksg/wallflow/config.yml`

Create it by copying the example:

```bash
# Linux
mkdir -p ~/.config/mksg/wallflow
cp config.example.yml ~/.config/mksg/wallflow/config.yml

# macOS
mkdir -p ~/Library/Application\ Support/mksg/wallflow
cp config.example.yml ~/Library/Application\ Support/mksg/wallflow/config.yml
```

See [config.example.yml](config.example.yml) for all options.

## Updating

wallflow can update itself:

```bash
# Check for updates
wallflow update --check

# Download and install update
wallflow update
```

Note: Self-update is disabled for package manager installations (dnf, apt). Use your package manager instead.

## Contributing

Contributions are welcome! This is a learning project, so:

- **Beginners**: Your PRs are welcome. We all start somewhere.
- **Experts**: Help improve the code quality and teach better patterns.

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Roadmap

See [TODO.md](TODO.md) for planned features including:

- Interactive TUI wallpaper browser
- Vim-style keybindings
- Collection management and tagging
- Multi-monitor support
- Plugin system

## License

MIT - See [LICENSE](LICENSE)

## Trademark

"wallflow" is a trademark of Mind Kraft Studios Group (MKSG). See [TRADEMARK.md](TRADEMARK.md) for usage guidelines.

## Acknowledgments

- [awww](https://codeberg.org/LGFae/awww) - Smooth wallpaper transitions
- [pywal](https://github.com/dylanaraps/pywal) - Color scheme generation
- [ratatui](https://github.com/ratatui-org/ratatui) - TUI framework (not yet)
- The Rust community for excellent documentation and crates
