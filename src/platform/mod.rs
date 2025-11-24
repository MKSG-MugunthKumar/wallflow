use anyhow::{anyhow, Result};
use std::env;

/// Supported platforms for wallpaper management
#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Linux(LinuxDisplayServer),
    MacOS,
    Windows,
}

/// Linux display server types
#[derive(Debug, Clone, PartialEq)]
pub enum LinuxDisplayServer {
    X11,
    Wayland(WaylandCompositor),
}

/// Wayland compositor detection
#[derive(Debug, Clone, PartialEq)]
pub enum WaylandCompositor {
    Sway,
    Hyprland,
    Gnome,
    KDE,
    Generic,
}

/// Status of platform dependency checking
#[derive(Debug)]
pub enum PlatformStatus {
    Ready,
    MissingDependency(String),
    UnsupportedPlatform,
}

/// Detect the current platform and display server
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

/// Detect Linux display server and compositor
#[cfg(target_os = "linux")]
fn detect_linux_platform() -> Result<Platform> {
    // Check if we're running under Wayland
    if env::var("WAYLAND_DISPLAY").is_ok() || env::var("XDG_SESSION_TYPE").map(|s| s == "wayland").unwrap_or(false) {
        let compositor = detect_wayland_compositor();
        return Ok(Platform::Linux(LinuxDisplayServer::Wayland(compositor)));
    }

    // Check if we're running under X11
    if env::var("DISPLAY").is_ok() || env::var("XDG_SESSION_TYPE").map(|s| s == "x11").unwrap_or(false) {
        return Ok(Platform::Linux(LinuxDisplayServer::X11));
    }

    // Fallback: assume X11 if we can't detect
    tracing::warn!("Could not detect display server, assuming X11");
    Ok(Platform::Linux(LinuxDisplayServer::X11))
}

/// Detect which Wayland compositor we're running under
#[cfg(target_os = "linux")]
fn detect_wayland_compositor() -> WaylandCompositor {
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        let desktop = desktop.to_lowercase();

        if desktop.contains("sway") {
            return WaylandCompositor::Sway;
        }

        if desktop.contains("hyprland") {
            return WaylandCompositor::Hyprland;
        }

        if desktop.contains("gnome") {
            return WaylandCompositor::Gnome;
        }

        if desktop.contains("kde") {
            return WaylandCompositor::KDE;
        }
    }

    // Check for compositor-specific environment variables
    if env::var("SWAYSOCK").is_ok() {
        return WaylandCompositor::Sway;
    }

    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return WaylandCompositor::Hyprland;
    }

    WaylandCompositor::Generic
}

/// Check if platform dependencies are available
pub fn check_platform_dependencies() -> PlatformStatus {
    match detect_platform() {
        Ok(Platform::MacOS) => {
            if which::which("wallpaper").is_ok() {
                PlatformStatus::Ready
            } else {
                PlatformStatus::MissingDependency(
                    "macOS wallpaper CLI not installed. Run: brew install wallpaper".to_string()
                )
            }
        }

        Ok(Platform::Linux(_)) => {
            let available_backends = detect_available_linux_backends();
            if available_backends.is_empty() {
                PlatformStatus::MissingDependency(
                    "No wallpaper backends found. Install one of: swww, awww, feh, nitrogen".to_string()
                )
            } else {
                PlatformStatus::Ready
            }
        }

        Ok(Platform::Windows) => {
            // PowerShell is built into Windows, no external dependencies needed
            PlatformStatus::Ready
        }

        Err(_) => PlatformStatus::UnsupportedPlatform,
    }
}

/// Detect available wallpaper backends on Linux
#[cfg(target_os = "linux")]
fn detect_available_linux_backends() -> Vec<String> {
    let backends = vec![
        "swww", "awww", "swaybg", "hyprpaper",
        "feh", "nitrogen", "xwallpaper"
    ];

    backends.into_iter()
        .filter(|backend| which::which(backend).is_ok())
        .map(String::from)
        .collect()
}

#[cfg(not(target_os = "linux"))]
fn detect_available_linux_backends() -> Vec<String> {
    vec![]
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Linux(LinuxDisplayServer::X11) => write!(f, "Linux (X11)"),
            Platform::Linux(LinuxDisplayServer::Wayland(compositor)) => {
                write!(f, "Linux (Wayland - {})", compositor)
            },
            Platform::MacOS => write!(f, "macOS"),
            Platform::Windows => write!(f, "Windows"),
        }
    }
}

impl std::fmt::Display for WaylandCompositor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaylandCompositor::Sway => write!(f, "Sway"),
            WaylandCompositor::Hyprland => write!(f, "Hyprland"),
            WaylandCompositor::Gnome => write!(f, "GNOME"),
            WaylandCompositor::KDE => write!(f, "KDE"),
            WaylandCompositor::Generic => write!(f, "Generic"),
        }
    }
}
