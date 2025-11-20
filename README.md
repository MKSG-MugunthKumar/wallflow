# 🌊 wallflow

**Elegant, configurable wallpaper management with smooth transitions and dynamic color schemes**

wallflow is a modern wallpaper manager that brings fluidity to your desktop experience. It automatically rotates wallpapers from multiple sources with beautiful transitions, generates dynamic color schemes, and integrates seamlessly with your desktop environment.

## ✨ Features

- 🎨 **Multiple Sources**: Local collections, Wallhaven, Picsum
- 🌈 **Dynamic Colors**: Automatic color scheme generation with pywal
- 🔄 **Smooth Transitions**: Configurable animations via awww daemon
- ⚙️ **Fully Configurable**: YAML-based configuration
- ⏰ **Automated Rotation**: systemd timer integration
- 🧹 **Smart Cleanup**: Configurable retention of downloaded wallpapers
- 🔌 **Desktop Integration**: Works with Hyprland, KDE Plasma, and more
- 🎯 **Cross-Platform**: Linux (Wayland/X11) support

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/MKSG-MugunthKumar/wallflow.git
cd wallflow
./install.sh
```

### Basic Usage

```bash
# Set a random local wallpaper
wallflow local

# Download and set from Wallhaven
wallflow wallhaven nature

# Random photo from Picsum
wallflow picsum

# Enable automatic rotation (every 30 minutes)
systemctl --user enable --now wallflow.timer
```

## 📁 Project Structure

```
wallflow/
├── bin/
│   ├── wallflow              # Main wallpaper manager
│   ├── wallflow-config       # Configuration management tool
│   └── wallflow-install      # Installation and setup script
├── config/
│   └── wallflow.yml          # Default configuration template
├── systemd/
│   ├── wallflow.service.template
│   ├── wallflow.timer.template
│   └── awww-daemon.service.template  # AWWW daemon service
├── docs/
│   ├── CONFIGURATION.md      # Detailed configuration guide
│   └── INTEGRATION.md        # Desktop environment integration
├── examples/
│   ├── configs/              # Example configurations
│   └── workflows/            # Usage examples
└── install.sh                # Quick installation script
```

## ⚙️ Configuration

wallflow uses a comprehensive YAML configuration file at `~/.config/wallflow/config.yml` that controls all behavior without requiring code modifications.

### Quick Setup

```bash
# Initialize default configuration
wallflow-config init

# Edit configuration
wallflow-config edit

# Validate configuration
wallflow-config validate
```

### Configuration Structure

```yaml
# Directory paths (XDG Base Directory compliant)
paths:
  local: "${XDG_PICTURES_DIR:-${HOME}/Pictures}/Wallpapers"
  downloads: "${XDG_PICTURES_DIR:-${HOME}/Pictures}/Wallpapers/downloads"

# Transition animations
transition:
  type: ["fade", "wipe", "outer", "random"]  # Single type or array
  duration: 5     # Seconds
  fps: 30         # Frame rate

# Automated rotation settings
timer:
  interval: 30        # Minutes between changes
  randomize: "5m"     # Add timing randomization
  start_delay: "1m"   # Delay after boot

# Wallpaper sources
sources:
  default: "wallhaven"    # local, wallhaven, picsum
  category: "nature"
  wallhaven:
    api_key: ""           # Optional for rate limits
    resolution: "2560x1440"
    quality: "large"      # large, medium, small
    purity: ["sfw"]       # sfw, sketchy, nsfw
  picsum:
    width: 2560
    height: 1440
  local:
    recursive: true       # Search subdirectories
    formats: ["jpg", "jpeg", "png", "webp"]

# Cleanup management
cleanup:
  keep_count: 10          # Number of downloads to retain
  auto_cleanup: true

# Desktop integration
integration:
  pywal:
    enabled: true
    notify_apps: ["neovim", "kitty"]  # neovim requires RedsXDD/neopywal.nvim plugin
  desktop:
    notify_completion: true

# Logging
logging:
  enabled: true
  level: "info"           # debug, info, warn, error
  timestamp: true

# Advanced settings
advanced:
  parallel_downloads: 3
  retry_attempts: 3
  timeout: 30
```

### Configuration Management

```bash
# Configuration commands
wallflow-config show       # Display current configuration
wallflow-config edit       # Open in editor ($EDITOR)
wallflow-config validate   # Check syntax and values
wallflow-config test       # Test by running wallflow
wallflow-config regenerate # Regenerate systemd services

# Service management (automatic daemon-reload included)
systemctl --user restart wallflow.timer
systemctl --user status wallflow.timer
journalctl --user -u wallflow.service -f
```

### Environment Variables

wallflow follows XDG Base Directory specifications with intelligent fallback:

**Path Resolution Order:**
1. `WALLPAPER_DIR` - Custom override (if set)
2. `XDG_PICTURES_DIR/Wallpapers` - XDG standard location
3. `$HOME/Pictures/Wallpapers` - Traditional fallback

**Supported Variables:**
- `${XDG_PICTURES_DIR}` - XDG pictures directory (e.g., `/home/user/Pictures`)
- `${WALLPAPER_DIR}` - Direct wallpaper directory override
- `${XDG_CONFIG_HOME}` - XDG config directory (for wallflow config)
- `${HOME}` - User home directory

**Examples:**
```bash
# Use custom wallpaper directory
export WALLPAPER_DIR="/media/storage/wallpapers"

# Use custom XDG pictures location
export XDG_PICTURES_DIR="/home/user/Media/Pictures"
```

### Example Configurations

**Minimal Setup** (examples/configs/minimal.yml):
```yaml
transition:
  type: "fade"
  duration: 3
timer:
  interval: 60
sources:
  default: "local"
cleanup:
  keep_count: 5
```

**Power User Setup** (examples/configs/power-user.yml):
```yaml
transition:
  type: ["fade", "wipe", "outer", "grow", "wave"]
  duration: 8
timer:
  interval: 20
  randomize: "10m"
sources:
  default: "wallhaven"
  wallhaven:
    resolution: "3840x2160"
    quality: "large"
integration:
  pywal:
    enabled: true
    notify_apps: ["neovim", "kitty"]  # neovim requires RedsXDD/neopywal.nvim plugin
logging:
  level: "debug"
cleanup:
  keep_count: 20
```

### Migration & Setup

**From Other Wallpaper Managers:**
```bash
# Quick setup for new users
./install.sh
wallflow-config init
wallflow-config edit  # Customize paths and preferences

# Enable automation
make enable
```

**Customization Examples:**
```bash
# Change rotation interval to 15 minutes
sed -i 's/interval: 30/interval: 15/' ~/.config/wallflow/config.yml
wallflow-config regenerate  # Automatically reloads systemd

# Use only local wallpapers
sed -i 's/default: "wallhaven"/default: "local"/' ~/.config/wallflow/config.yml

# Enable debug logging
sed -i 's/level: "info"/level: "debug"/' ~/.config/wallflow/config.yml
```

For complete configuration details, see [CONFIGURATION.md](docs/CONFIGURATION.md).

## 🎯 Use Cases

- **Desktop Environments**: Hyprland, KDE, GNOME integration
- **Content Creators**: Dynamic backgrounds for streaming/recording
- **Developers**: Distraction-free automated wallpaper rotation
- **Digital Artists**: Color palette inspiration from wallpapers
- **System Admins**: Centralized wallpaper management across machines

## 🛠️ Requirements

**Core Dependencies:**
- **awww**: Wayland wallpaper daemon with smooth transitions ([Installation Guide](https://codeberg.org/LGFae/awww))
- **awww-daemon**: Background service for wallpaper management
- **curl**: For downloading wallpapers from online sources
- **jq**: JSON parsing for API responses

**Platform Support:**
- **KDE Plasma** - ✅ Tested and working
- **Hyprland, Sway, wlroots-based compositors** - Should work (awww compatible)
- **GNOME** - ❓ Not yet tested, may work but uncertain due to wlr-layer-shell requirements

**Optional:**
- **pywal** (color schemes), **yq** (YAML processing)
- **systemd** (for automation timers)

**pywal Integration Requirements:**
- **Neovim**: Requires [RedsXDD/neopywal.nvim](https://github.com/RedsXDD/neopywal.nvim) plugin
  ```lua
  -- In your Neovim config (init.lua or plugins.lua):
  { "RedsXDD/neopywal.nvim", name = "neopywal", lazy = false, priority = 1000 }
  ```
- **Kitty**: Built-in pywal support (no additional setup needed)

**Installing awww:**
```bash
# Clone and build from source
git clone https://codeberg.org/LGFae/awww.git
cd awww
cargo build --release
cp target/release/{awww,awww-daemon} ~/.local/bin/

# Start the daemon
systemctl --user enable --now awww-daemon
```

## 🤝 Contributing

wallflow welcomes contributions! Whether it's:
- 🐛 Bug reports and fixes
- 💡 Feature requests and implementations
- 📚 Documentation improvements
- 🎨 New wallpaper sources
- 🔌 Desktop environment integrations

## 📄 License

MIT License - See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [awww](https://codeberg.org/LGFae/awww) - An Answer to your Wayland Wallpaper Woes (smooth wallpaper transitions)
- [pywal](https://github.com/dylanaraps/pywal) - Dynamic color scheme generation
- [Wallhaven](https://wallhaven.cc/) - Community wallpaper platform
- [Picsum](https://picsum.photos/) - Random photo service

---

*"Let your wallpapers flow as smoothly as your workflow"* 🌊