# Cross-Platform Wallpaper Setting PRD

## Executive Summary

This PRD outlines the implementation of cross-platform wallpaper setting capabilities for wallflow, moving beyond the current awww-only approach to support Linux (X11/Wayland), macOS, and Windows environments. The goal is to create a robust, platform-aware wallpaper application system that maintains wallflow's educational focus.

## Current State Analysis

### Current Implementation Status

- **Linux Supported**: Multiple backends (awww, swww, feh)
- **Backend Registry**: Implemented with priority-based selection
- **Platform Detection**: Runtime display server detection (X11/Wayland)
- **Transition Support**: Via awww backend

### Platform Research Findings

#### Linux Ecosystem

Modern Linux desktop environments use different wallpaper tools depending on the display server:

**X11 Tools**:

- `feh` - Classic, reliable, basic functionality
- `nitrogen` - GUI and CLI, older but stable
- `xwallpaper` - Modern X11 wallpaper setter

**Wayland Tools**:

- `swaybg` - Sway's wallpaper daemon
- `swww` - Efficient animated wallpaper daemon [(source)](https://wiki.hypr.land/Useful-Utilities/Wallpapers/)
- `hyprpaper` - Hyprland's wallpaper utility
- `mpvpaper` - Video wallpapers

**Universal GUI Managers**:

- [Waypaper](https://github.com/anufrievroman/waypaper) - GUI for Wayland and Xorg
- [Hyprwall](https://github.com/MarkusVolk/hyprwall) - Rust GUI supporting multiple backends
- [Azote](https://github.com/nwg-piotr/azote) - GTK+3 wallpaper browser [(source)](https://www.linuxlinks.com/best-free-open-source-wallpaper-setters/)

#### macOS Solutions

- [macos-wallpaper CLI](https://github.com/sindresorhus/macos-wallpaper) - Swift-based, Homebrew available
- Native AppleScript - Triggers Gatekeeper, not recommended
- Swift Package - Clean API, no Gatekeeper issues

#### Windows Solutions

- [SystemParametersInfo API](https://superuser.com/questions/277805/how-do-i-set-the-desktop-background-on-windows-from-a-script) - Primary Windows wallpaper API
- [PowerShell implementation](https://getps.dev/blog/change-desktop-wallpaper-with-powershell) - User32.dll wrapper
- Registry + SPI calls for Windows 11 compatibility [(source)](https://www.joseespitia.com/2017/09/15/set-wallpaper-powershell-function/)

## Proposed Architecture

### Platform Detection System

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Linux(LinuxDisplayServer),
    MacOS,
    Windows,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinuxDisplayServer {
    X11,
    Wayland(WaylandCompositor),
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaylandCompositor {
    Sway,
    Hyprland,
    Gnome,
    KDE,
    Generic,
}

pub fn detect_platform() -> Result<Platform> {
    // Runtime platform detection logic
}
```

### Wallpaper Backend System

```rust
#[async_trait]
pub trait WallpaperBackend {
    /// Apply wallpaper to the desktop
    async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()>;

    /// Get current wallpaper (if supported)
    async fn get_current_wallpaper(&self) -> Result<Option<PathBuf>>;

    /// Check if this backend is available on current system
    fn is_available(&self) -> bool;

    /// Get backend priority (higher = preferred)
    fn priority(&self) -> u32;

    /// Backend name for logging
    fn name(&self) -> &'static str;

    /// Supported transition effects
    fn supported_transitions(&self) -> Vec<String>;
}

#[derive(Debug, Clone)]
pub struct WallpaperOptions {
    pub transition: Option<String>,
    pub duration: Option<u32>,
    pub fps: Option<u32>,
    pub scaling: WallpaperScaling,
    pub monitor: MonitorSelection,
}

#[derive(Debug, Clone)]
pub enum WallpaperScaling {
    Fill,
    Fit,
    Stretch,
    Center,
    Tile,
}

#[derive(Debug, Clone)]
pub enum MonitorSelection {
    Primary,
    All,
    Specific(u32),
}
```

## Implementation Plan

### Phase 1: Platform Detection & Backend Registry (Week 1)

#### Step 1.1: Platform Detection Module

**File**: `src/platform/mod.rs`

```rust
use anyhow::Result;
use std::env;

pub fn detect_platform() -> Result<Platform> {
    #[cfg(target_os = "linux")]
    return detect_linux_platform();

    #[cfg(target_os = "macos")]
    return Ok(Platform::MacOS);

    #[cfg(target_os = "windows")]
    return Ok(Platform::Windows);

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    return Err(anyhow!("Unsupported platform"));
}

#[cfg(target_os = "linux")]
fn detect_linux_platform() -> Result<Platform> {
    // Check WAYLAND_DISPLAY, XDG_SESSION_TYPE, etc.
    // Detect compositor from XDG_CURRENT_DESKTOP
}
```

#### Step 1.2: Backend Registry System

**File**: `src/wallpaper/backends/mod.rs`

```rust
pub struct BackendRegistry {
    backends: Vec<Box<dyn WallpaperBackend + Send + Sync>>,
}

impl BackendRegistry {
    pub fn new() -> Self {
        let mut registry = Self { backends: Vec::new() };
        registry.register_platform_backends();
        registry
    }

    fn register_platform_backends(&mut self) {
        let platform = crate::platform::detect_platform().unwrap();

        match platform {
            Platform::Linux(LinuxDisplayServer::Wayland(compositor)) => {
                self.register_wayland_backends(compositor);
            }
            Platform::Linux(LinuxDisplayServer::X11) => {
                self.register_x11_backends();
            }
            Platform::MacOS => {
                self.register_macos_backends();
            }
            Platform::Windows => {
                self.register_windows_backends();
            }
        }
    }

    pub async fn get_best_backend(&self) -> Result<&dyn WallpaperBackend> {
        // Return highest priority available backend
    }
}
```

### Phase 2: Linux Backend Implementation (Week 2)

#### Step 2.1: Wayland Backends

**File**: `src/wallpaper/backends/wayland.rs`

```rust
// swww backend - animated wallpapers, modern
pub struct SwwwBackend;

#[async_trait]
impl WallpaperBackend for SwwwBackend {
    async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()> {
        let mut cmd = AsyncCommand::new("swww");
        cmd.args(["img", &image_path.to_string_lossy()]);

        if let Some(transition) = &options.transition {
            cmd.args(["--transition-type", transition]);
        }

        // Execute command with proper error handling
    }

    fn is_available(&self) -> bool {
        which::which("swww").is_ok()
    }

    fn priority(&self) -> u32 { 100 } // Highest for modern Wayland

    fn supported_transitions(&self) -> Vec<String> {
        vec!["fade".to_string(), "wipe".to_string(), "grow".to_string()]
    }
}

// swaybg backend - Sway's official tool
pub struct SwaybgBackend;

// hyprpaper backend - Hyprland's tool
pub struct HyprpaperBackend;
```

#### Step 2.2: X11 Backends

**File**: `src/wallpaper/backends/x11.rs`

```rust
// feh backend - reliable classic
pub struct FehBackend;

#[async_trait]
impl WallpaperBackend for FehBackend {
    async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()> {
        let mut cmd = AsyncCommand::new("feh");
        cmd.args(["--bg-scale", &image_path.to_string_lossy()]);

        // feh doesn't support transitions, but very reliable
    }

    fn priority(&self) -> u32 { 50 } // Lower than modern tools
    fn supported_transitions(&self) -> Vec<String> { vec![] }
}

// nitrogen backend
pub struct NitrogenBackend;

// xwallpaper backend
pub struct XwallpaperBackend;
```

#### Step 2.3: Keep Existing awww Backend

**File**: `src/wallpaper/backends/awww.rs`

```rust
// Refactor existing awww logic into backend system
pub struct AwwwBackend;

impl WallpaperBackend for AwwwBackend {
    fn priority(&self) -> u32 { 75 } // High priority if available

    fn supported_transitions(&self) -> Vec<String> {
        vec!["random".to_string(), "fade".to_string(), /* all awww transitions */]
    }
}
```

### Phase 3: macOS Backend Implementation (Week 3)

#### Step 3.1: macOS Wallpaper CLI Backend

**File**: `src/wallpaper/backends/macos.rs`

```rust
pub struct MacOSWallpaperBackend;

#[async_trait]
impl WallpaperBackend for MacOSWallpaperBackend {
    async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()> {
        let mut cmd = AsyncCommand::new("wallpaper");
        cmd.args(["set", &image_path.to_string_lossy()]);

        // Add scaling options
        match options.scaling {
            WallpaperScaling::Fill => cmd.args(["--scale", "fill"]),
            WallpaperScaling::Fit => cmd.args(["--scale", "fit"]),
            WallpaperScaling::Stretch => cmd.args(["--scale", "stretch"]),
            WallpaperScaling::Center => cmd.args(["--scale", "center"]),
            _ => cmd.args(["--scale", "auto"]),
        };

        // Monitor selection
        match options.monitor {
            MonitorSelection::All => {} // Default behavior
            MonitorSelection::Primary => cmd.args(["--screen", "main"]),
            MonitorSelection::Specific(index) => cmd.args(["--screen", &index.to_string()]),
        }

        let output = cmd.output().await.context("Failed to execute wallpaper command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("wallpaper command failed: {}", stderr));
        }

        Ok(())
    }

    async fn get_current_wallpaper(&self) -> Result<Option<PathBuf>> {
        let output = AsyncCommand::new("wallpaper")
            .arg("get")
            .output()
            .await?;

        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim();
            Ok(Some(PathBuf::from(path_str)))
        } else {
            Ok(None)
        }
    }

    fn is_available(&self) -> bool {
        which::which("wallpaper").is_ok()
    }

    fn priority(&self) -> u32 { 90 } // High priority on macOS
    fn name(&self) -> &'static str { "macos-wallpaper" }
    fn supported_transitions(&self) -> Vec<String> { vec![] } // No transition support
}

// Fallback AppleScript backend (warn about Gatekeeper)
pub struct AppleScriptBackend;

#[async_trait]
impl WallpaperBackend for AppleScriptBackend {
    async fn set_wallpaper(&self, image_path: &Path, _options: &WallpaperOptions) -> Result<()> {
        warn!("Using AppleScript backend - may trigger Gatekeeper warnings");

        let script = format!(
            "tell application \"System Events\" to tell every desktop to set picture to \"{}\"",
            image_path.display()
        );

        let output = AsyncCommand::new("osascript")
            .args(["-e", &script])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("AppleScript failed: {}", stderr));
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        which::which("osascript").is_ok()
    }

    fn priority(&self) -> u32 { 20 } // Very low priority, only as fallback
    fn name(&self) -> &'static str { "applescript" }
}
```

### Phase 4: Windows Backend Implementation (Week 4)

#### Step 4.1: PowerShell SystemParametersInfo Backend

**File**: `src/wallpaper/backends/windows.rs`

```rust
pub struct WindowsSystemParametersBackend;

#[async_trait]
impl WallpaperBackend for WindowsSystemParametersBackend {
    async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()> {
        // Create PowerShell script using SystemParametersInfo
        let powershell_script = self.create_wallpaper_script(image_path, options)?;

        let output = AsyncCommand::new("powershell")
            .args(["-ExecutionPolicy", "Bypass", "-Command", &powershell_script])
            .output()
            .await
            .context("Failed to execute PowerShell command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("PowerShell script failed: {}", stderr));
        }

        Ok(())
    }

    fn create_wallpaper_script(&self, image_path: &Path, options: &WallpaperOptions) -> Result<String> {
        let wallpaper_style = match options.scaling {
            WallpaperScaling::Tile => "0",
            WallpaperScaling::Center => "1",
            WallpaperScaling::Stretch => "2",
            WallpaperScaling::Fit => "6",
            WallpaperScaling::Fill => "10",
        };

        // PowerShell script based on research findings
        let script = format!(r#"
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public class Wallpaper {{
    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    private static extern int SystemParametersInfo(int uAction, int uParam, string lpvParam, int fuWinIni);

    public static void SetWallpaper(string path, string style) {{
        // Set wallpaper style in registry
        Microsoft.Win32.Registry.SetValue(@"HKEY_CURRENT_USER\Control Panel\Desktop", "WallpaperStyle", style);
        Microsoft.Win32.Registry.SetValue(@"HKEY_CURRENT_USER\Control Panel\Desktop", "TileWallpaper", (style == "0") ? "1" : "0");

        // Apply wallpaper
        SystemParametersInfo(0x0014, 0, path, 0x0001 | 0x0002);
    }}
}}
'@

[Wallpaper]::SetWallpaper('{}', '{}')
"#, image_path.display(), wallpaper_style);

        Ok(script)
    }

    fn is_available(&self) -> bool {
        which::which("powershell").is_ok()
    }

    fn priority(&self) -> u32 { 90 } // High priority on Windows
    fn name(&self) -> &'static str { "windows-systemparametersinfo" }
    fn supported_transitions(&self) -> Vec<String> { vec![] }
}
```

### Phase 5: Integration & Configuration (Week 5)

#### Step 5.1: Enhanced Configuration

**File**: `src/config.rs` (additions)

```rust
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PlatformConfig {
    #[serde(default)]
    pub linux: LinuxPlatformConfig,
    #[serde(default)]
    pub macos: MacOSPlatformConfig,
    #[serde(default)]
    pub windows: WindowsPlatformConfig,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct LinuxPlatformConfig {
    pub preferred_backends: Vec<String>, // ["swww", "awww", "feh"]
    pub force_backend: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct MacOSPlatformConfig {
    pub install_wallpaper_cli: bool, // Offer to install via Homebrew
    pub use_applescript_fallback: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct WindowsPlatformConfig {
    pub wallpaper_style: String, // fill, fit, stretch, center, tile
}

// Add to main Config struct
pub struct Config {
    // ... existing fields ...
    #[serde(default)]
    pub platform: PlatformConfig,
}
```

#### Step 5.2: Updated Wallpaper Module

**File**: `src/wallpaper/mod.rs`

```rust
use crate::config::Config;
use crate::wallpaper::backends::BackendRegistry;

pub async fn set_wallpaper_by_source(config: &Config) -> Result<()> {
    // Download wallpaper (existing logic)
    let wallpaper_path = download_wallpaper(config).await?;

    // Apply using best available backend
    let registry = BackendRegistry::new();
    let backend = registry.get_best_backend().await?;

    let options = WallpaperOptions {
        transition: config.transition.transition_type.as_ref().map(|t| t.clone()),
        duration: Some(config.transition.duration),
        fps: Some(config.transition.fps),
        scaling: config.platform.get_scaling_preference(),
        monitor: MonitorSelection::All,
    };

    backend.set_wallpaper(&wallpaper_path, &options).await?;

    info!("✅ Wallpaper applied using {} backend", backend.name());

    Ok(())
}
```

## Educational Benefits

### Cross-Platform Development Patterns

1. **Platform Detection**: Runtime environment detection
2. **Trait System**: Abstract backends with common interface
3. **Command Execution**: Cross-platform process management
4. **Configuration Management**: Platform-specific settings
5. **Error Handling**: Platform-specific error scenarios

### Platform-Specific Learning

1. **Linux**: Display server differences (X11 vs Wayland)
2. **macOS**: CLI tools vs AppleScript, Gatekeeper implications
3. **Windows**: Win32 API usage through PowerShell, registry interaction

## Installation & Setup Automation

### Dependency Detection

```rust
pub async fn check_platform_dependencies() -> PlatformStatus {
    let platform = detect_platform()?;

    match platform {
        Platform::MacOS => {
            if !which::which("wallpaper").is_ok() {
                return PlatformStatus::MissingDependency("macos-wallpaper CLI not installed. Run: brew install wallpaper");
            }
        }
        Platform::Linux(_) => {
            let available_backends = detect_available_linux_backends();
            if available_backends.is_empty() {
                return PlatformStatus::MissingDependency("No wallpaper backends found. Install: swww, feh, or nitrogen");
            }
        }
        Platform::Windows => {
            // PowerShell is built-in, no external dependencies
        }
    }

    PlatformStatus::Ready
}
```

## CLI Enhancements

### New Commands

```bash
# Platform information
wallflow platform-info

# Backend testing
wallflow test-backend swww
wallflow list-backends

# Installation assistance
wallflow install-dependencies
```

## Risk Mitigation

### Platform-Specific Risks

1. **macOS Gatekeeper**: Prefer CLI tools over AppleScript
2. **Linux Fragmentation**: Multiple fallback backends
3. **Windows UAC**: PowerShell execution policy handling

### Fallback Strategy

- Multiple backends per platform in priority order
- Graceful degradation if preferred backend unavailable
- Clear error messages with installation guidance

## Success Criteria

### Technical Metrics

- [x] Support 3+ backends per major platform (awww, swww, feh)
- [x] 95%+ wallpaper application success rate
- [x] Automatic dependency detection (which crate)
- [x] Platform-appropriate default configurations

### Educational Metrics

- [x] Platform detection examples
- [x] Cross-platform trait implementation (WallpaperBackend trait)
- [x] Command execution patterns (async Command)
- [x] Configuration abstraction

This cross-platform approach transforms wallflow from a Linux-only tool into a truly universal wallpaper manager while maintaining its educational mission and demonstrating modern Rust cross-platform development patterns.

## Sources

- [Hyprland Wallpapers Wiki](https://wiki.hypr.land/Useful-Utilities/Wallpapers/)
- [Waypaper - GUI wallpaper manager](https://github.com/anufrievroman/waypaper)
- [Hyprwall - Rust wallpaper GUI](https://github.com/MarkusVolk/hyprwall)
- [Azote - GTK wallpaper browser](https://github.com/nwg-piotr/azote)
- [LinuxLinks - Best Wallpaper Setters](https://www.linuxlinks.com/best-free-open-source-wallpaper-setters/)
- [Windows Wallpaper Script Solutions](https://superuser.com/questions/277805/how-do-i-set-the-desktop-background-on-windows-from-a-script)
- [PowerShell Wallpaper Implementation](https://getps.dev/blog/change-desktop-wallpaper-with-powershell)
- [macOS Wallpaper CLI Tool](https://github.com/sindresorhus/macos-wallpaper)
- [Windows PowerShell Function](https://www.joseespitia.com/2017/09/15/set-wallpaper-powershell-function/)
