# wallflow Copilot Instructions

## Project Overview

wallflow is a **Rust-based wallpaper management CLI** designed as both a practical tool and a structured learning project. It replaces bash scripts with type-safe Rust, featuring a built-in daemon, smooth transitions, and multi-source wallpaper management.

**Key Architecture:**

- **CLI Interface**: `clap`-based command parsing with subcommands (`local`, `wallhaven`, `picsum`, `daemon`)
- **Type-Safe Config**: YAML configuration with `serde` validation (replaces "horrific AWK parser")
- **Built-in Daemon**: Self-contained scheduling without systemd dependency
- **Multi-Platform Display Detection**: Auto-resolution via `xrandr`, `sway`, `wlr-randr`
- **Async Design**: `tokio` runtime for HTTP downloads and concurrent operations

## Critical Development Patterns

### Configuration Management

- **Always use `Config::expand_paths()`** after loading to resolve `${HOME}` variables
- Config is loaded via `Config::load_or_default()` with XDG Base Directory compliance
- All paths are `PathBuf` with `shellexpand` for environment variable expansion

```rust
// Correct pattern for config handling
let mut config = Config::load_or_default()?;
config.expand_paths()?;  // Essential for path resolution
```

### Error Handling Convention

- Use `anyhow::Result<()>` for most functions
- Add context with `.with_context()` for meaningful error messages
- Educational TODOs are marked with detailed implementation steps

### Logging Strategy

- Use `tracing` with structured logging: `info!`, `warn!`, `error!`, `debug!()`
- Emoji prefixes for user-facing messages: `🌊`, `🚧`, `📚`, `💡`
- Verbose mode controlled by CLI flag, filters to `wallflow=debug`

## Key Implementation Points

### Display Resolution Detection

Multi-platform resolution detection in `src/display.rs`:

- Falls back through: `xrandr` → `swaymsg` → `wlr-randr` → default (2560x1440)
- Returns `Resolution` struct with `.as_string()` method for API calls
- Always handle detection failures gracefully

### Wallpaper Application Pipeline

1. **Selection**: `select_local_wallpaper()` uses `rand::SliceRandom` for random selection
2. **Transitions**: Support single string or array of transition types via `TransitionType` enum
3. **Integration**: Call external tools (`awww`, `pywal`) via `tokio::process::Command`

### Daemon Architecture

- **Foreground mode**: Uses `tokio::time::interval` for scheduling
- **Randomization**: Parse duration strings (`"5m"`, `"30s"`) into seconds
- **Error resilience**: Continue operation even if individual wallpaper sets fail

### Learning-Oriented TODOs

HTTP integrations are intentionally left as educational exercises:

- `wallpaper::set_wallhaven()` - Detailed implementation steps provided
- `wallpaper::set_picsum()` - Simple API integration example
- Reference implementations in `bin/wallflow-reference`

## Development Workflow

### Build & Test Commands

```bash
# Check compilation without running
cargo check

# Run with verbose logging
cargo run -- --verbose config

# Test daemon in foreground
cargo run -- daemon --foreground

# Build optimized release
cargo build --release
```

### Configuration Testing

- Use `config.example.yml` as template
- Test path expansion with environment variables
- Verify XDG compliance (`~/.config/wallflow/config.yml`)

### Cross-Platform Considerations

- Display detection varies by window manager
- Path separators handled by `std::path::PathBuf`
- External tool availability checked with `which` crate

## Integration Points

### External Dependencies

- **awww**: Wallpaper transitions (called via subprocess)
- **pywal**: Color scheme generation (optional)
- **Display managers**: xrandr, sway, wlr-randr for resolution detection

### API Integration Patterns

When implementing HTTP downloads:

1. Use `crate::display::get_primary_display_resolution()` for dynamic sizing
2. Follow `bin/wallflow-reference` for URL construction examples
3. Save downloads to `config.paths.downloads` with timestamp naming
4. Implement cleanup based on `config.cleanup.keep_count`

## Common Gotchas

- **Path expansion**: Always call `config.expand_paths()` after loading config
- **Async context**: HTTP operations require `async/await` with `tokio::main`
- **Error context**: Add meaningful context to errors for debugging
- **Resolution detection**: Handle failures gracefully with sensible defaults
- **Daemon signals**: Foreground mode for development, background for production
