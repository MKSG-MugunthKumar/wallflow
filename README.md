# wallflow

**Elegant wallpaper management with smooth transitions**

A modern, Rust-based wallpaper manager that brings fluidity to your Linux desktop with multiple sources, smooth transitions, and dynamic color schemes.

## About This Project

This is my first Rust project. After 20+ years of programming (including C/C++), I decided to learn Rust by building something I'd actually use daily. wallflow started as a learning exercise and evolved into a full-featured wallpaper manager.

The codebase reflects this journey - you'll find clean, idiomatic Rust alongside comments and structure that helped me understand the language. If you're also learning Rust, feel free to explore, learn, and contribute!

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
  - Smooth animated transitions (via [awww](https://github.com/anyrun/awww))
  - Auto-resolution detection (xrandr, sway, wlr-randr)
  - Optional [pywal](https://github.com/dylanaraps/pywal) integration for dynamic color schemes
  - Self-update capability

- **Desktop Support**
  - Hyprland, Sway, and other wlroots compositors
  - KDE Plasma
  - GNOME
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
sudo dnf copr enable mksg/wallflow
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

## Configuration

Configuration file: `~/.config/wallflow/config.yml`

```yaml
paths:
  local: ~/Pictures/Wallpapers
  downloads: ~/Pictures/Downloads

daemon:
  interval: 1800 # 30 minutes
  source: local # or wallhaven, reddit, etc.

transition:
  type: wipe
  duration: 2

integration:
  pywal:
    enabled: true
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

## Acknowledgments

- [awww](https://github.com/anyrun/awww) - Smooth wallpaper transitions
- [pywal](https://github.com/dylanaraps/pywal) - Color scheme generation
- [ratatui](https://github.com/ratatui-org/ratatui) - TUI framework
- The Rust community for excellent documentation and crates
