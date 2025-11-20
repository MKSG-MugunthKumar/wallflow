# wallflow for Content Creators

Configuration optimized for streaming, recording, and content creation.

## Goals
- Non-distracting transitions
- Stable color schemes during streams
- Curated wallpaper collection
- Minimal system impact

## Configuration

```yaml
# Content Creator Setup
paths:
  local: "${HOME}/Pictures/Stream-Backgrounds"
  downloads: "${HOME}/Pictures/Stream-Backgrounds/downloads"

transition:
  type: "fade"  # Smooth, non-distracting
  duration: 2   # Quick transitions

timer:
  interval: 120  # Change every 2 hours
  randomize: "10m"

sources:
  default: "local"  # Curated collection only

cleanup:
  keep_count: 10

integration:
  pywal:
    enabled: false  # Don't change colors during streams

logging:
  enabled: false  # Minimal logs
```

## Workflow

1. **Curate Wallpapers**: Create a collection of stream-safe backgrounds
2. **Pre-Stream Setup**: Set specific wallpaper before going live
3. **During Stream**: Minimal changes with smooth transitions
4. **Post-Stream**: Re-enable dynamic wallpapers

## Commands

```bash
# Set specific wallpaper before stream
wallflow local

# Disable timer during stream
systemctl --user stop wallflow.timer

# Re-enable after stream
systemctl --user start wallflow.timer
```