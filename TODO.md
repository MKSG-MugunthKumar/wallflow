# wallflow Roadmap

## Current Status

### Completed

- [x] CLI with multiple commands
- [x] Local wallpaper selection
- [x] Online sources: Wallhaven, Unsplash, Reddit, Bing, APOD, Earth View, Picsum
- [x] Built-in daemon with auto-rotation
- [x] Smooth transitions via awww
- [x] Pywal integration for dynamic color schemes
- [x] Auto-resolution detection (xrandr, sway, wlr-randr)
- [x] Self-update from GitHub releases
- [x] COPR packaging for Fedora
- [x] crates.io publishing

---

## Phase 1: TUI Foundation

### Basic Interface

- [ ] Full-screen TUI with ratatui
- [ ] Directory browser for local wallpapers
- [ ] Thumbnail preview (sixel/kitty graphics protocol)
- [ ] Status bar with current wallpaper info
- [ ] Basic keyboard navigation

### Layout

```
┌─────────────────────────────────────────────────────┐
│ wallflow                              source: local │
├─────────────────────────────────────────────────────┤
│ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐    │
│ │         │ │         │ │         │ │         │    │
│ │  thumb  │ │  thumb  │ │  thumb  │ │  thumb  │    │
│ │         │ │         │ │         │ │         │    │
│ └─────────┘ └─────────┘ └─────────┘ └─────────┘    │
│ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐    │
│ │         │ │         │ │         │ │         │    │
│ │  thumb  │ │  thumb  │ │  thumb  │ │  thumb  │    │
│ │         │ │         │ │         │ │         │    │
│ └─────────┘ └─────────┘ └─────────┘ └─────────┘    │
├─────────────────────────────────────────────────────┤
│ j/k: navigate  Enter: set  q: quit  /: search      │
└─────────────────────────────────────────────────────┘
```

---

## Phase 2: Vim Keybindings

- [ ] hjkl navigation
- [ ] gg/G for top/bottom
- [ ] / for search with fuzzy matching
- [ ] : for command mode
- [ ] Visual selection with v
- [ ] Marks with m and '

### Command Mode

```
:set wallpaper
:source wallhaven
:search nature mountains
:favorites add
:quit
```

---

## Phase 3: Collection Management

- [ ] Favorites list
- [ ] Custom collections/albums
- [ ] Tagging system
- [ ] Smart collections (by color, resolution, source)
- [ ] Import/export collections
- [ ] Duplicate detection

---

## Phase 4: Multi-Monitor Support

- [ ] Per-monitor wallpaper selection
- [ ] Per-monitor rotation schedules
- [ ] Span mode (single wallpaper across monitors)
- [ ] Monitor arrangement visualization
- [ ] Hot-reload on monitor connect/disconnect

---

## Phase 5: Advanced Features

### Preview System

- [ ] Real-time wallpaper preview before applying
- [ ] Color scheme preview (pywal)
- [ ] Side-by-side comparison
- [ ] History with undo

### Plugin System

- [ ] Custom wallpaper sources as plugins
- [ ] Custom transition effects
- [ ] Post-processing filters
- [ ] Lua/WASM plugin API

### Scheduling

- [ ] Time-based wallpaper rules (day/night)
- [ ] Calendar integration
- [ ] Weather-based wallpapers
- [ ] Location-aware themes

---

## Future Ideas

- [ ] Web UI for remote control
- [ ] Mobile companion app
- [ ] AI-powered wallpaper recommendations
- [ ] Wallpaper generation with Stable Diffusion
- [ ] Community wallpaper sharing
- [ ] Screen lock integration
- [ ] Live wallpapers (video/animated)

---

## Technical Debt

- [ ] Comprehensive test coverage
- [ ] Better error messages
- [ ] Logging improvements
- [ ] Performance profiling
- [ ] Documentation generation
- [ ] CI/CD improvements

---

## Contributing

Pick any unchecked item and submit a PR! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Items marked with higher phases aren't necessarily harder - they're just further in the roadmap. Feel free to jump ahead if something interests you.
