use anyhow::Result;

#[cfg(target_os = "linux")]
use std::env;

/// Supported platforms for wallpaper management
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Platform {
  Linux(LinuxDisplayServer),
  MacOS,
  Windows,
}

/// Linux display server types
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum LinuxDisplayServer {
  X11,
  Wayland(WaylandCompositor),
}

/// Wayland compositor detection
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum WaylandCompositor {
  Sway,
  Hyprland,
  Gnome,
  Kde,
  Generic,
}

/// Status of platform dependency checking
#[derive(Debug)]
pub enum PlatformStatus {
  Ready,
  MissingDependency(()),
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
  let wayland_display = env::var("WAYLAND_DISPLAY").ok();
  let xdg_session_type = env::var("XDG_SESSION_TYPE").ok();
  let xdg_current_desktop = env::var("XDG_CURRENT_DESKTOP").ok();
  let x11_display = env::var("DISPLAY").ok();

  tracing::debug!(
    "Platform detection: WAYLAND_DISPLAY={:?}, XDG_SESSION_TYPE={:?}, XDG_CURRENT_DESKTOP={:?}, DISPLAY={:?}",
    wayland_display,
    xdg_session_type,
    xdg_current_desktop,
    x11_display
  );

  // Check if we're running under Wayland
  if wayland_display.is_some() || xdg_session_type.as_deref() == Some("wayland") {
    let compositor = detect_wayland_compositor();
    tracing::debug!("Detected Wayland compositor: {:?}", compositor);
    return Ok(Platform::Linux(LinuxDisplayServer::Wayland(compositor)));
  }

  // Check if we're running under X11
  if x11_display.is_some() || xdg_session_type.as_deref() == Some("x11") {
    tracing::debug!("Detected X11 display server");
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
      return WaylandCompositor::Kde;
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
      // macOS has multiple backend options:
      // 1. macos-wallpaper CLI (brew install wallpaper)
      // 2. Swift compiler for native backend (comes with Xcode/CLT)
      // 3. AppleScript (always available via osascript)
      if which::which("wallpaper").is_ok() || which::which("swiftc").is_ok() || which::which("osascript").is_ok() {
        PlatformStatus::Ready
      } else {
        PlatformStatus::MissingDependency(())
      }
    }

    Ok(Platform::Linux(_)) => {
      let available_backends = detect_available_linux_backends();
      if available_backends.is_empty() {
        PlatformStatus::MissingDependency(())
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
  let backends = vec!["swww", "awww", "swaybg", "hyprpaper", "feh", "nitrogen", "xwallpaper"];

  backends
    .into_iter()
    .filter(|backend| which::which(backend).is_ok())
    .map(String::from)
    .collect()
}

/// Stub for non-Linux platforms (this code path should never be reached)
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
      }
      Platform::MacOS => write!(f, "macOS"),
      Platform::Windows => write!(f, "Windows"),
    }
  }
}

/// Detect whether the system is using dark mode.
///
/// Returns `Some(true)` for dark mode, `Some(false)` for light mode,
/// or `None` if detection fails or the platform is unsupported.
pub fn detect_dark_mode() -> Option<bool> {
  #[cfg(target_os = "linux")]
  return detect_dark_mode_linux();

  #[cfg(target_os = "macos")]
  return detect_dark_mode_macos();

  #[cfg(target_os = "windows")]
  return detect_dark_mode_windows();

  #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
  return None;
}

#[cfg(target_os = "linux")]
fn detect_dark_mode_linux() -> Option<bool> {
  // Try GNOME / GTK-based desktops
  if let Ok(output) = std::process::Command::new("gsettings")
    .args(["get", "org.gnome.desktop.interface", "color-scheme"])
    .output()
    && output.status.success()
  {
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("prefer-dark") {
      tracing::debug!("System dark mode detected via gsettings (GNOME)");
      return Some(true);
    } else if stdout.contains("default") || stdout.contains("prefer-light") {
      tracing::debug!("System light mode detected via gsettings (GNOME)");
      return Some(false);
    }
  }

  // Try KDE Plasma
  if let Ok(output) = std::process::Command::new("kreadconfig5")
    .args(["--group", "General", "--key", "ColorScheme"])
    .output()
    && output.status.success()
  {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let is_dark = stdout.to_lowercase().contains("dark");
    tracing::debug!("System {} mode detected via kreadconfig5 (KDE)", if is_dark { "dark" } else { "light" });
    return Some(is_dark);
  }

  tracing::debug!("Could not detect system dark mode on Linux");
  None
}

#[cfg(target_os = "macos")]
fn detect_dark_mode_macos() -> Option<bool> {
  // AppleInterfaceStyle key only exists when dark mode is active
  if let Ok(output) = std::process::Command::new("defaults")
    .args(["read", "-g", "AppleInterfaceStyle"])
    .output()
  {
    let is_dark = output.status.success();
    tracing::debug!("System {} mode detected via defaults (macOS)", if is_dark { "dark" } else { "light" });
    return Some(is_dark);
  }

  tracing::debug!("Could not detect system dark mode on macOS");
  None
}

#[cfg(target_os = "windows")]
fn detect_dark_mode_windows() -> Option<bool> {
  // AppsUseLightTheme: 0 = dark, 1 = light
  if let Ok(output) = std::process::Command::new("reg")
    .args([
      "query",
      r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize",
      "/v",
      "AppsUseLightTheme",
    ])
    .output()
  {
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      let is_dark = stdout.contains("0x0");
      tracing::debug!("System {} mode detected via registry (Windows)", if is_dark { "dark" } else { "light" });
      return Some(is_dark);
    }
  }

  tracing::debug!("Could not detect system dark mode on Windows");
  None
}

impl std::fmt::Display for WaylandCompositor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      WaylandCompositor::Sway => write!(f, "Sway"),
      WaylandCompositor::Hyprland => write!(f, "Hyprland"),
      WaylandCompositor::Gnome => write!(f, "GNOME"),
      WaylandCompositor::Kde => write!(f, "KDE"),
      WaylandCompositor::Generic => write!(f, "Generic"),
    }
  }
}
