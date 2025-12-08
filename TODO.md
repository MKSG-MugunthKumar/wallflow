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

## Phase 1: TUI Foundation ✅

### Basic Interface

- [x] Full-screen TUI with ratatui
- [x] Directory browser for local wallpapers
- [x] Thumbnail preview (Kitty/Sixel/iTerm2 via ratatui-image)
- [x] Status bar with current wallpaper info
- [x] Basic keyboard navigation (j/k, J/K, g/G)
- [x] Config display panel with 'e' to edit in $EDITOR
- [x] Help screen (? key)

### Layout

## Phase 2: Vim Keybindings

- [x] j/k navigation (up/down)
- [x] g/G for top/bottom
- [x] J/K for page up/down (10 items)
- [ ] / for search with fuzzy matching
- [ ] : for command mode

### Command Mode

```
:set wallpaper
:source wallhaven
:search nature mountains
:favorites add
:quit
```

---

## Phase 3: Multi-Monitor Support

- [ ] Per-monitor wallpaper selection
- [ ] Span mode (single wallpaper across monitors)
- [ ] Hot-reload on monitor connect/disconnect
- [ ] Per virtual desktop wallpaper selection
- [ ] Mac Support (Partial - In `feature/macOS` branch)

---

## Future Ideas: Collections, Plugins and Other Platforms

- [ ] Favorites list (Do not delete these when auto cleanup happens)
- [ ] Windows Support
- [ ] Custom wallpaper sources as plugins
- [ ] Lua/WASM plugin API
- [ ] Time-based wallpaper rules (day/night)
- [ ] Weather-based wallpapers
- [ ] Festive aware themes
- [ ] Screen lock integration
- [ ] Live wallpapers (video/animated)

---

## Technical Debt

- [ ] Comprehensive test coverage
- [ ] Better error messages
- [ ] Logging improvements
- [ ] Performance profiling
- [ ] Documentation generation

---

## Contributing

Pick any unchecked item and submit a PR! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Items marked with higher phases aren't necessarily harder - they're just further in the roadmap. Feel free to jump ahead if something interests you.
