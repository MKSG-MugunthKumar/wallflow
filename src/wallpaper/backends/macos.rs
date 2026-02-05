//! macOS wallpaper backends
//!
//! Three backends available in order of preference:
//! 1. `macos-wallpaper` CLI (sindresorhus/macos-wallpaper via Homebrew)
//! 2. Swift helper using NSWorkspace.setDesktopImageURL (bundled)
//! 3. AppleScript fallback (may trigger Gatekeeper warnings)

use super::traits::{MonitorSelection, WallpaperBackend, WallpaperOptions, WallpaperScaling};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::process::Command as AsyncCommand;
use tracing::{debug, warn};

/// Backend using sindresorhus/macos-wallpaper CLI tool
/// Install via: brew install wallpaper
#[cfg(target_os = "macos")]
#[derive(Default)]
pub struct MacOSWallpaperBackend;

#[cfg(target_os = "macos")]
impl MacOSWallpaperBackend {
  pub fn new() -> Self {
    Self
  }
}

#[cfg(target_os = "macos")]
#[async_trait]
impl WallpaperBackend for MacOSWallpaperBackend {
  async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()> {
    let mut cmd = AsyncCommand::new("wallpaper");
    cmd.arg("set");
    cmd.arg(image_path);

    // Add scaling option
    let scale = match options.scaling {
      WallpaperScaling::Fill => "fill",
      WallpaperScaling::Fit => "fit",
      WallpaperScaling::Stretch => "stretch",
      WallpaperScaling::Center => "center",
      WallpaperScaling::Tile => "tile", // Note: may not be supported
    };
    cmd.args(["--scale", scale]);

    // Monitor selection
    match &options.monitor {
      MonitorSelection::All => {
        cmd.arg("--all-screens");
      }
      MonitorSelection::Primary => {
        cmd.args(["--screen", "main"]);
      }
      MonitorSelection::Specific(index) => {
        cmd.args(["--screen", &index.to_string()]);
      }
    }

    let output = cmd.output().await.context("Failed to execute wallpaper command")?;

    if output.status.success() {
      debug!("✅ macos-wallpaper set wallpaper successfully");
      Ok(())
    } else {
      let stderr = String::from_utf8_lossy(&output.stderr);
      Err(anyhow::anyhow!("wallpaper command failed: {}", stderr))
    }
  }

  async fn get_current_wallpaper(&self) -> Result<Option<PathBuf>> {
    let output = AsyncCommand::new("wallpaper")
      .arg("get")
      .output()
      .await
      .context("Failed to get current wallpaper")?;

    if output.status.success() {
      let path_str = String::from_utf8_lossy(&output.stdout);
      let path_str = path_str.trim();
      if !path_str.is_empty() {
        return Ok(Some(PathBuf::from(path_str)));
      }
    }

    Ok(None)
  }

  fn is_available(&self) -> bool {
    which::which("wallpaper").is_ok()
  }

  fn priority(&self) -> u32 {
    100 // Highest priority - best UX
  }

  fn name(&self) -> &'static str {
    "macos-wallpaper"
  }

  fn supported_transitions(&self) -> Vec<String> {
    vec![] // No transition support
  }
}

/// Backend using a bundled Swift helper that calls NSWorkspace.setDesktopImageURL
/// This is the native macOS API and doesn't require external dependencies
#[cfg(target_os = "macos")]
pub struct SwiftNativeBackend {
  helper_path: Option<PathBuf>,
}

#[cfg(target_os = "macos")]
impl Default for SwiftNativeBackend {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(target_os = "macos")]
impl SwiftNativeBackend {
  pub fn new() -> Self {
    // Look for bundled helper in standard locations
    let helper_path = Self::find_helper();
    Self { helper_path }
  }

  fn find_helper() -> Option<PathBuf> {
    // Check various locations for the helper binary
    let candidates = [
      // Next to the wallflow binary
      std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("wallflow-macos-helper"))),
      // In ~/.local/bin
      dirs::home_dir().map(|p| p.join(".local/bin/wallflow-macos-helper")),
      // In /usr/local/bin
      Some(PathBuf::from("/usr/local/bin/wallflow-macos-helper")),
    ];

    candidates.into_iter().flatten().find(|candidate| candidate.exists())
  }

  /// Create and compile the Swift helper on-the-fly if needed
  async fn ensure_helper(&self) -> Result<PathBuf> {
    if let Some(ref path) = self.helper_path {
      return Ok(path.clone());
    }

    // Compile helper to a temp location
    let helper_source = Self::helper_source_code();
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("wallflow_helper.swift");
    let binary_path = temp_dir.join("wallflow_helper");

    // Write source
    tokio::fs::write(&source_path, helper_source)
      .await
      .context("Failed to write helper source")?;

    // Compile
    let output = AsyncCommand::new("swiftc")
      .args(["-o", binary_path.to_str().unwrap(), source_path.to_str().unwrap(), "-framework", "Cocoa"])
      .output()
      .await
      .context("Failed to compile Swift helper")?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(anyhow::anyhow!("Swift compilation failed: {}", stderr));
    }

    Ok(binary_path)
  }

  fn helper_source_code() -> &'static str {
    r#"
import Cocoa

// Usage: wallflow_helper <image_path> [scaling] [screen]
// scaling: fill (default), fit, stretch, center
// screen: all (default), main, <index>

func main() {
    let args = CommandLine.arguments
    guard args.count >= 2 else {
        fputs("Usage: wallflow_helper <image_path> [scaling] [screen]\n", stderr)
        exit(1)
    }

    let imagePath = args[1]
    let scaling = args.count > 2 ? args[2] : "fill"
    let screen = args.count > 3 ? args[3] : "all"

    let url = URL(fileURLWithPath: imagePath)

    // Determine scaling option
    var options: [NSWorkspace.DesktopImageOptionKey: Any] = [:]
    switch scaling {
    case "fill":
        options[.imageScaling] = NSImageScaling.scaleProportionallyUpOrDown.rawValue
        options[.allowClipping] = true
    case "fit":
        options[.imageScaling] = NSImageScaling.scaleProportionallyUpOrDown.rawValue
        options[.allowClipping] = false
    case "stretch":
        options[.imageScaling] = NSImageScaling.scaleAxesIndependently.rawValue
    case "center":
        options[.imageScaling] = NSImageScaling.scaleNone.rawValue
    default:
        options[.imageScaling] = NSImageScaling.scaleProportionallyUpOrDown.rawValue
        options[.allowClipping] = true
    }

    // Determine which screens to set
    let screens: [NSScreen]
    switch screen {
    case "all":
        screens = NSScreen.screens
    case "main":
        screens = NSScreen.main.map { [$0] } ?? []
    default:
        if let index = Int(screen), index < NSScreen.screens.count {
            screens = [NSScreen.screens[index]]
        } else {
            screens = NSScreen.screens
        }
    }

    // Set wallpaper for each screen
    for screen in screens {
        do {
            try NSWorkspace.shared.setDesktopImageURL(url, for: screen, options: options)
        } catch {
            fputs("Error setting wallpaper for screen: \(error)\n", stderr)
            exit(1)
        }
    }

    print("Wallpaper set successfully")
}

main()
"#
  }
}

#[cfg(target_os = "macos")]
#[async_trait]
impl WallpaperBackend for SwiftNativeBackend {
  async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()> {
    let helper_path = self.ensure_helper().await?;

    let scaling = match options.scaling {
      WallpaperScaling::Fill => "fill",
      WallpaperScaling::Fit => "fit",
      WallpaperScaling::Stretch => "stretch",
      WallpaperScaling::Center => "center",
      WallpaperScaling::Tile => "center", // Tile not natively supported, fallback to center
    };

    let screen = match &options.monitor {
      MonitorSelection::All => "all".to_string(),
      MonitorSelection::Primary => "main".to_string(),
      MonitorSelection::Specific(index) => index.to_string(),
    };

    let output = AsyncCommand::new(&helper_path)
      .args([image_path.to_str().unwrap(), scaling, &screen])
      .output()
      .await
      .context("Failed to execute Swift helper")?;

    if output.status.success() {
      debug!("✅ Swift native backend set wallpaper successfully");
      Ok(())
    } else {
      let stderr = String::from_utf8_lossy(&output.stderr);
      Err(anyhow::anyhow!("Swift helper failed: {}", stderr))
    }
  }

  async fn get_current_wallpaper(&self) -> Result<Option<PathBuf>> {
    // NSWorkspace.desktopImageURL can return current wallpaper
    // For simplicity, return None for now
    Ok(None)
  }

  fn is_available(&self) -> bool {
    // Available if Swift compiler exists (ships with Xcode/CLT)
    which::which("swiftc").is_ok()
  }

  fn priority(&self) -> u32 {
    80 // Good priority - native but requires compilation
  }

  fn name(&self) -> &'static str {
    "swift-native"
  }

  fn supported_transitions(&self) -> Vec<String> {
    vec![] // No transition support
  }
}

/// Fallback AppleScript backend
/// Works without dependencies but may trigger Gatekeeper warnings
#[cfg(target_os = "macos")]
#[derive(Default)]
pub struct AppleScriptBackend;

#[cfg(target_os = "macos")]
impl AppleScriptBackend {
  pub fn new() -> Self {
    Self
  }
}

#[cfg(target_os = "macos")]
#[async_trait]
impl WallpaperBackend for AppleScriptBackend {
  async fn set_wallpaper(&self, image_path: &Path, _options: &WallpaperOptions) -> Result<()> {
    warn!("Using AppleScript backend - may trigger Gatekeeper warnings on first use");

    // AppleScript to set wallpaper on all desktops
    let script = format!(
      r#"tell application "System Events"
    tell every desktop
        set picture to "{}"
    end tell
end tell"#,
      image_path.display()
    );

    let output = AsyncCommand::new("osascript")
      .args(["-e", &script])
      .output()
      .await
      .context("Failed to execute AppleScript")?;

    if output.status.success() {
      debug!("✅ AppleScript set wallpaper successfully");
      Ok(())
    } else {
      let stderr = String::from_utf8_lossy(&output.stderr);
      Err(anyhow::anyhow!("AppleScript failed: {}", stderr))
    }
  }

  async fn get_current_wallpaper(&self) -> Result<Option<PathBuf>> {
    let script = r#"tell application "System Events"
    tell current desktop
        get picture
    end tell
end tell"#;

    let output = AsyncCommand::new("osascript")
      .args(["-e", script])
      .output()
      .await
      .context("Failed to get current wallpaper via AppleScript")?;

    if output.status.success() {
      let path_str = String::from_utf8_lossy(&output.stdout);
      let path_str = path_str.trim();
      if !path_str.is_empty() {
        return Ok(Some(PathBuf::from(path_str)));
      }
    }

    Ok(None)
  }

  fn is_available(&self) -> bool {
    which::which("osascript").is_ok()
  }

  fn priority(&self) -> u32 {
    20 // Low priority - use as last resort
  }

  fn name(&self) -> &'static str {
    "applescript"
  }

  fn supported_transitions(&self) -> Vec<String> {
    vec![]
  }
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
  use super::*;

  #[test]
  fn test_macos_wallpaper_backend_available() {
    let backend = MacOSWallpaperBackend::new();
    // Just check it doesn't panic
    let _ = backend.is_available();
  }

  #[test]
  fn test_applescript_backend_available() {
    let backend = AppleScriptBackend::new();
    // osascript should always be available on macOS
    assert!(backend.is_available());
  }

  #[test]
  fn test_swift_native_backend_available() {
    let backend = SwiftNativeBackend::new();
    // swiftc availability depends on Xcode/CLT installation
    let _ = backend.is_available();
  }
}
