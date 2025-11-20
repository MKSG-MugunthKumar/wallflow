# wallflow Configuration Guide

This guide covers all aspects of configuring wallflow for your specific needs.

## Configuration File Location

wallflow looks for its configuration in the following order:
1. `~/.config/wallflow/config.yml` (default)
2. `$XDG_CONFIG_HOME/wallflow/config.yml`
3. Built-in defaults (if no config file exists)

## Complete Configuration Reference

### Basic Structure

```yaml
# wallflow Configuration
paths:
  local: "${HOME}/Pictures/Wallpapers"
  downloads: "${HOME}/Pictures/Wallpapers/downloads"

transition:
  type:
    - "fade"
    - "wipe"
    - "outer"
  duration: 5

timer:
  interval: 30
  randomize: "5m"

sources:
  default: "wallhaven"
  category: "nature"

cleanup:
  keep_count: 10

logging:
  enabled: true
  level: "info"
```

## Detailed Configuration Sections

### Paths Configuration

Controls where wallflow stores and looks for wallpapers.

```yaml
paths:
  local: "${HOME}/Pictures/Wallpapers"           # Local wallpaper collection
  downloads: "${HOME}/Pictures/Wallpapers/downloads"  # Downloaded wallpapers
```

**Environment Variables**:
- `${HOME}` - User home directory
- `${WALLPAPER_DIR}` - Custom wallpaper directory (fallback)
- `${XDG_PICTURES_DIR}` - XDG Pictures directory

**Examples**:
```yaml
# Custom directory structure
paths:
  local: "/media/wallpapers/collection"
  downloads: "/tmp/wallflow-cache"

# Using environment variables
paths:
  local: "${WALLPAPER_DIR:-${HOME}/Pictures}"
  downloads: "${XDG_CACHE_HOME:-${HOME}/.cache}/wallflow"
```

### Transition Configuration

Controls how wallpaper changes are animated.

```yaml
transition:
  type:                    # Multiple types (random selection)
    - "fade"              # Smooth fade transition
    - "wipe"              # Directional wipe
    - "outer"             # Expand from outside
    - "random"            # Random awww transition
  # OR single type:
  # type: "fade"          # Fixed transition type

  duration: 5             # Animation duration in seconds
  fps: 30                 # Animation frame rate
```

**Available Transition Types** (awww):
- `none` - Instant change
- `simple` - Simple transition
- `fade` - Smooth fade
- `left`, `right`, `top`, `bottom` - Directional wipes
- `wipe` - Random direction wipe
- `wave` - Wave effect
- `grow` - Grow from center
- `outer` - Expand from outside
- `random` - Random transition each time

**Examples**:
```yaml
# Fast, subtle transitions
transition:
  type: "fade"
  duration: 2
  fps: 60

# Dramatic transitions
transition:
  type:
    - "wave"
    - "grow"
    - "outer"
  duration: 8
  fps: 24

# No animation
transition:
  type: "none"
  duration: 0
```

### Timer Configuration

Controls automatic wallpaper rotation.

```yaml
timer:
  interval: 30            # Minutes between changes
  randomize: "5m"         # Add randomization (systemd format)
  start_delay: "1m"       # Delay after boot/login
```

**Systemd Time Formats**:
- `30s` - 30 seconds
- `5m` - 5 minutes
- `2h` - 2 hours
- `1d` - 1 day

**Examples**:
```yaml
# Frequent changes
timer:
  interval: 15
  randomize: "2m"
  start_delay: "30s"

# Infrequent changes
timer:
  interval: 180           # 3 hours
  randomize: "30m"
  start_delay: "5m"

# Disable randomization
timer:
  interval: 60
  randomize: "0"
```

### Sources Configuration

Controls wallpaper sources and their behavior.

```yaml
sources:
  default: "wallhaven"    # Default source
  category: "nature"      # Default category

  wallhaven:
    api_key: ""           # Optional API key
    resolution: "2560x1440"
    quality: "large"
    purity:
      - "sfw"
    categories:
      - "nature"
      - "landscape"

  picsum:
    width: 2560
    height: 1440
    grayscale: false
    blur: false

  local:
    recursive: true
    formats:
      - "jpg"
      - "png"
      - "webp"
```

### Cleanup Configuration

Controls downloaded file management.

```yaml
cleanup:
  keep_count: 10          # Number of files to keep
  auto_cleanup: true      # Clean up after downloads
  max_age: "30d"          # Maximum file age (optional)
```

### Integration Configuration

Controls desktop environment and application integration.

```yaml
integration:
  pywal:
    enabled: true
    backend: "wal"
    notify_apps:
      - "neovim"
      - "kitty"

  desktop:
    notify_completion: true
    icon_theme_sync: false
```

**Pywal Backends**:
- `wal` - Default pywal backend
- `haishoku` - Haishoku color extraction
- `colorthief` - ColorThief algorithm
- `colorz` - Colorz algorithm

### Logging Configuration

Controls logging behavior.

```yaml
logging:
  enabled: true
  level: "info"           # debug, info, warn, error
  file: ""                # Log file (empty = stderr)
  timestamp: true         # Include timestamps
```

### Advanced Configuration

```yaml
advanced:
  parallel_downloads: 3   # Concurrent downloads
  retry_attempts: 3       # Download retry attempts
  timeout: 30             # Request timeout (seconds)
  user_agent: "wallflow/1.0"  # Custom User-Agent
```

## Configuration Management Commands

### Validate Configuration
```bash
wallflow-config validate
```

### Edit Configuration
```bash
wallflow-config edit
```

### Show Current Configuration
```bash
wallflow-config show
```

### Test Configuration
```bash
wallflow-config test
```

### Initialize Configuration
```bash
wallflow-config init
```

## Common Configuration Examples

### Content Creator Setup
```yaml
timer:
  interval: 60            # Change every hour
sources:
  default: "wallhaven"
  category: "abstract"
transition:
  type: "fade"            # Smooth, non-distracting
  duration: 3
integration:
  pywal:
    enabled: false        # Don't change colors during streams
```

### Developer Productivity Setup
```yaml
timer:
  interval: 120           # Change every 2 hours
sources:
  default: "local"        # Curated local collection
transition:
  type: "none"            # No distractions
cleanup:
  keep_count: 5           # Minimal storage usage
integration:
  pywal:
    enabled: true         # Color coordination with terminal
```

### Digital Art Inspiration Setup
```yaml
timer:
  interval: 15            # Frequent inspiration
sources:
  default: "wallhaven"
  category: "art"
transition:
  type:
    - "fade"
    - "grow"
    - "wave"
  duration: 6
integration:
  pywal:
    enabled: true
    backend: "colorthief"  # Better color extraction for art
```

## Troubleshooting Configuration

### Configuration Not Loading
1. Check file location: `~/.config/wallflow/config.yml`
2. Validate YAML syntax: `wallflow-config validate`
3. Check file permissions: `ls -la ~/.config/wallflow/`

### Transitions Not Working
1. Ensure awww and awww-daemon are installed and running
2. Check transition type spelling
3. Verify Wayland/X11 compatibility

### Timer Not Working
1. Check systemd user services: `systemctl --user status wallflow.timer`
2. Verify configuration: `wallflow-config validate`
3. Regenerate services: `make enable` (in wallflow directory)

### Downloads Failing
1. Check internet connection
2. Verify dependencies (curl, jq)
3. Check API rate limits (Wallhaven)

## Environment Variables

wallflow respects these environment variables:

- `WALLPAPER_DIR` - Override default wallpaper directory
- `XDG_CONFIG_HOME` - Override config directory
- `XDG_CACHE_HOME` - Override cache directory
- `XDG_PICTURES_DIR` - Override pictures directory
- `EDITOR` - Default editor for config editing

## Migration from Other Wallpaper Managers

### From Variety
```bash
# Copy existing wallpapers
cp -r ~/.config/variety/Downloaded/* ~/Pictures/Wallpapers/downloads/

# Update paths in wallflow config
wallflow-config edit
# Set paths.downloads to point to variety downloads
```

### From Nitrogen
```bash
# Nitrogen uses ~/.nitrogen/
# Copy wallpapers to wallflow directory
# Update configuration accordingly
```

### From feh
```bash
# feh typically uses manual paths
# Add your feh wallpaper directories to wallflow local paths
```