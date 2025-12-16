#[cfg(target_os = "linux")]
use super::awww;

/// Registry for managing wallpaper backends
use crate::platform::{Platform, detect_platform};
use crate::wallpaper::backends::WallpaperBackend;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use tracing::{debug, warn};

pub struct BackendRegistry {
  backends: Vec<Arc<dyn WallpaperBackend + Send + Sync>>,
}

impl BackendRegistry {
  /// Create a new backend registry with platform-appropriate backends
  pub fn new() -> Self {
    let mut registry = Self { backends: Vec::new() };

    registry.register_platform_backends();
    registry
  }

  /// Register backends appropriate for the current platform
  fn register_platform_backends(&mut self) {
    match detect_platform() {
      Ok(platform) => {
        debug!("Detected platform: {}", platform);
        self.register_backends_for_platform(&platform);
      }
      Err(e) => {
        warn!("Failed to detect platform: {}, registering all backends", e);
        self.register_all_backends();
      }
    }

    debug!("Registered {} wallpaper backends", self.backends.len());
  }

  /// Register backends for a specific platform
  fn register_backends_for_platform(&mut self, platform: &Platform) {
    match platform {
      Platform::Linux(display_server) => {
        #[cfg(target_os = "linux")]
        self.register_linux_backends(display_server);
        #[cfg(not(target_os = "linux"))]
        {
          let _ = display_server; // Suppress unused warning
          warn!("Linux backends not available on this platform");
        }
      }

      Platform::MacOS => {
        #[cfg(target_os = "macos")]
        self.register_macos_backends();
        #[cfg(not(target_os = "macos"))]
        warn!("macOS backends not available on this platform");
      }

      Platform::Windows => {
        #[cfg(target_os = "windows")]
        self.register_windows_backends();
        #[cfg(not(target_os = "windows"))]
        warn!("Windows backends not available on this platform");
      }
    }
  }

  /// Register all available backends (fallback)
  fn register_all_backends(&mut self) {
    #[cfg(target_os = "linux")]
    {
      use crate::platform::{LinuxDisplayServer, WaylandCompositor};
      // Try both X11 and Wayland backends
      self.register_linux_backends(&LinuxDisplayServer::X11);
      self.register_linux_backends(&LinuxDisplayServer::Wayland(WaylandCompositor::Generic));
    }

    #[cfg(target_os = "macos")]
    self.register_macos_backends();

    #[cfg(target_os = "windows")]
    self.register_windows_backends();

    #[cfg(target_os = "linux")]
    self.register_awww_backend();
  }

  /// Register Linux-specific backends
  #[cfg(target_os = "linux")]
  fn register_linux_backends(&mut self, display_server: &crate::platform::LinuxDisplayServer) {
    use super::linux::*;

    match display_server {
      crate::platform::LinuxDisplayServer::Wayland(compositor) => {
        // Wayland backends in priority order

        use crate::wallpaper::backends::AwwwBackend;
        self.register_backend(Arc::new(AwwwBackend::new()));

        match compositor {
          crate::platform::WaylandCompositor::Sway => {
            self.register_backend(Arc::new(SwaybgBackend::new()));
          }
          crate::platform::WaylandCompositor::Hyprland => {
            self.register_backend(Arc::new(HyprpaperBackend::new()));
          }
          _ => {}
        }
      }

      crate::platform::LinuxDisplayServer::X11 => {
        // X11 backends in priority order
        self.register_backend(Arc::new(FehBackend::new()));
        self.register_backend(Arc::new(NitrogenBackend::new()));
        self.register_backend(Arc::new(XwallpaperBackend::new()));
      }
    }
  }

  /// Register macOS-specific backends
  /// Priority order:
  /// 1. macos-wallpaper CLI (brew install wallpaper) - best UX
  /// 2. Swift native backend using NSWorkspace API - requires swiftc
  /// 3. AppleScript fallback - always available but may trigger Gatekeeper
  #[cfg(target_os = "macos")]
  fn register_macos_backends(&mut self) {
    use super::macos::*;

    // Highest priority: macos-wallpaper CLI tool
    self.register_backend(Arc::new(MacOSWallpaperBackend::new()));

    // Medium priority: Swift native backend (compiles helper on-the-fly)
    self.register_backend(Arc::new(SwiftNativeBackend::new()));

    // Lowest priority: AppleScript fallback
    self.register_backend(Arc::new(AppleScriptBackend::new()));
  }

  /// Register Windows-specific backends
  #[cfg(target_os = "windows")]
  fn register_windows_backends(&mut self) {
    use super::windows::*;

    self.register_backend(Arc::new(WindowsSystemParametersBackend::new()));
  }

  /// Register awww backend if available
  #[cfg(target_os = "linux")]
  fn register_awww_backend(&mut self) {
    self.register_backend(Arc::new(awww::AwwwBackend::new()));
  }

  /// Register a backend if it's available
  fn register_backend(&mut self, backend: Arc<dyn WallpaperBackend + Send + Sync>) {
    if backend.is_available() {
      debug!("Registered backend: {} (priority: {})", backend.name(), backend.priority());
      self.backends.push(backend);
    } else {
      debug!("Backend not available: {}", backend.name());
    }
  }

  /// Get the best available backend (highest priority that's available)
  pub fn get_best_backend(&self) -> Result<Arc<dyn WallpaperBackend + Send + Sync>> {
    if self.backends.is_empty() {
      return Err(anyhow!(
        "No wallpaper backends available. Please install a wallpaper setter like feh, swww, or awww"
      ));
    }

    // Backends are registered in priority order, but let's sort to be sure
    let mut sorted_backends: Vec<_> = self.backends.iter().collect();
    sorted_backends.sort_by_key(|backend| std::cmp::Reverse(backend.priority()));

    for backend in &sorted_backends {
      if let Err(e) = backend.validate() {
        debug!("Backend {} failed validation: {}", backend.name(), e);
        continue;
      }

      debug!("Selected backend: {}", backend.name());
      return Ok((*backend).clone());
    }

    Err(anyhow!("No working wallpaper backends found"))
  }

  /// List all registered backends
  pub fn list_backends(&self) -> Vec<String> {
    let mut backends: Vec<_> = self
      .backends
      .iter()
      .map(|b| format!("{} (priority: {}, available: {})", b.name(), b.priority(), b.is_available()))
      .collect();

    backends.sort();
    backends
  }

  /// Get all available backend names
  pub fn available_backend_names(&self) -> Vec<String> {
    self.backends.iter().filter(|b| b.is_available()).map(|b| b.name().to_string()).collect()
  }
}
