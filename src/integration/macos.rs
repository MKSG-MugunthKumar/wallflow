//! macOS appearance/theme integration
//!
//! Provides control over macOS system appearance settings:
//! - Dark/Light mode switching
//! - Accent color changes
//! - Highlight color customization
//!
//! Note: Unlike Linux's pywal which can dynamically theme terminal colors
//! and GTK/Qt applications based on wallpaper colors, macOS theming is
//! limited to system-level settings with a fixed accent color palette.

use std::path::Path;
use tokio::process::Command as AsyncCommand;
use tracing::{debug, info, warn};

/// macOS accent color options
/// These correspond to the colors available in System Settings > Appearance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
#[allow(dead_code)]
pub enum AccentColor {
  Multicolor = -1, // Graphite-ish, follows app
  Red = 0,
  Orange = 1,
  Yellow = 2,
  Green = 3,
  Blue = 4, // Default
  Purple = 5,
  Pink = 6,
}

#[allow(dead_code)]
impl AccentColor {
  /// Parse accent color from string (case-insensitive)
  #[allow(clippy::should_implement_trait)]
  pub fn from_str(s: &str) -> Option<Self> {
    match s.to_lowercase().as_str() {
      "multicolor" | "graphite" | "auto" => Some(Self::Multicolor),
      "red" => Some(Self::Red),
      "orange" => Some(Self::Orange),
      "yellow" => Some(Self::Yellow),
      "green" => Some(Self::Green),
      "blue" => Some(Self::Blue),
      "purple" | "violet" => Some(Self::Purple),
      "pink" | "magenta" => Some(Self::Pink),
      _ => None,
    }
  }

  /// Get the name of the accent color
  pub fn name(&self) -> &'static str {
    match self {
      Self::Multicolor => "multicolor",
      Self::Red => "red",
      Self::Orange => "orange",
      Self::Yellow => "yellow",
      Self::Green => "green",
      Self::Blue => "blue",
      Self::Purple => "purple",
      Self::Pink => "pink",
    }
  }

  /// Suggest an accent color based on dominant color RGB values
  /// This is a simple heuristic - not as sophisticated as pywal
  pub fn from_dominant_color(r: u8, g: u8, b: u8) -> Self {
    // Convert to HSL-ish hue for color matching
    let max = r.max(g).max(b) as f32;
    let min = r.min(g).min(b) as f32;

    // If grayscale or very desaturated, use multicolor
    if max - min < 30.0 {
      return Self::Multicolor;
    }

    let r = r as f32;
    let g = g as f32;
    let b = b as f32;

    // Calculate approximate hue (0-360)
    let hue = if max == r {
      60.0 * (((g - b) / (max - min)) % 6.0)
    } else if max == g {
      60.0 * (((b - r) / (max - min)) + 2.0)
    } else {
      60.0 * (((r - g) / (max - min)) + 4.0)
    };

    let hue = if hue < 0.0 { hue + 360.0 } else { hue };

    // Map hue to accent colors
    match hue as u16 {
      0..=15 | 346..=360 => Self::Red,
      16..=45 => Self::Orange,
      46..=70 => Self::Yellow,
      71..=165 => Self::Green,
      166..=260 => Self::Blue,
      261..=290 => Self::Purple,
      291..=345 => Self::Pink,
      _ => Self::Blue,
    }
  }
}

/// macOS appearance mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AppearanceMode {
  Light,
  Dark,
  Auto, // Follow system/time-based
}

#[allow(dead_code)]
impl AppearanceMode {
  #[allow(clippy::should_implement_trait)]
  pub fn from_str(s: &str) -> Option<Self> {
    match s.to_lowercase().as_str() {
      "light" => Some(Self::Light),
      "dark" => Some(Self::Dark),
      "auto" | "system" => Some(Self::Auto),
      _ => None,
    }
  }
}

/// Check if we're running on macOS
#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn is_macos() -> bool {
  true
}

#[cfg(not(target_os = "macos"))]
fn is_macos() -> bool {
  false
}

/// Set macOS dark/light mode using AppleScript
///
/// This changes the system-wide appearance mode.
#[allow(dead_code)]
pub async fn set_appearance_mode(mode: AppearanceMode) {
  if !is_macos() {
    debug!("Not on macOS, skipping appearance mode change");
    return;
  }

  let script = match mode {
    AppearanceMode::Dark => "tell app \"System Events\" to tell appearance preferences to set dark mode to true",
    AppearanceMode::Light => "tell app \"System Events\" to tell appearance preferences to set dark mode to false",
    AppearanceMode::Auto => {
      // Auto mode requires setting via System Settings, can't be done via AppleScript
      // We'll just log a warning
      warn!("Auto appearance mode cannot be set programmatically, use System Settings");
      return;
    }
  };

  let output = AsyncCommand::new("osascript").args(["-e", script]).output().await;

  match output {
    Ok(output) => {
      if output.status.success() {
        info!("✅ macOS appearance set to {:?}", mode);
      } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Failed to set macOS appearance: {}", stderr);
      }
    }
    Err(e) => {
      warn!("Failed to execute osascript: {}", e);
    }
  }
}

/// Set macOS accent color
///
/// This changes the system-wide accent color. Requires sending specific
/// notifications for running apps to update their appearance.
#[allow(dead_code)]
pub async fn set_accent_color(color: AccentColor) {
  if !is_macos() {
    debug!("Not on macOS, skipping accent color change");
    return;
  }

  // We need to use a Swift script because:
  // 1. defaults write alone doesn't notify running apps
  // 2. We need to send DistributedNotifications
  let swift_script = format!(
    r#"
import Foundation

// Set the accent color preference
let colorValue = {color_value}
UserDefaults.standard.setPersistentDomain(
    ["AppleAccentColor": colorValue == -1 ? nil : colorValue],
    forName: UserDefaults.globalDomain
)

// Sync to disk
UserDefaults.standard.synchronize()

// Send notifications so running apps update their appearance
// Both notifications are needed for full compatibility
let notifications = [
    "AppleColorPreferencesChangedNotification",
    "AppleAquaColorVariantChanged"
]

for name in notifications {{
    DistributedNotificationCenter.default().post(
        name: Notification.Name(name),
        object: nil,
        userInfo: nil,
        deliverImmediately: true
    )
}}

// Small delay to let notifications propagate
Thread.sleep(forTimeInterval: 0.1)
"#,
    color_value = color as i8
  );

  // Write and execute Swift script
  let temp_script = std::env::temp_dir().join("wallflow_accent.swift");

  if let Err(e) = tokio::fs::write(&temp_script, &swift_script).await {
    warn!("Failed to write accent color script: {}", e);
    // Fall back to defaults command (won't update running apps)
    set_accent_color_fallback(color).await;
    return;
  }

  let output = AsyncCommand::new("swift").arg(&temp_script).output().await;

  // Clean up temp file
  let _ = tokio::fs::remove_file(&temp_script).await;

  match output {
    Ok(output) => {
      if output.status.success() {
        info!("✅ macOS accent color set to {}", color.name());
      } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Failed to set accent color via Swift: {}", stderr);
        // Try fallback
        set_accent_color_fallback(color).await;
      }
    }
    Err(e) => {
      debug!("Swift not available ({}), using fallback", e);
      set_accent_color_fallback(color).await;
    }
  }
}

/// Fallback method using defaults command
/// Note: This won't update running apps until they're restarted
#[allow(dead_code)]
async fn set_accent_color_fallback(color: AccentColor) {
  let color_value = color as i8;

  let output = if color_value == -1 {
    // Multicolor = delete the preference
    AsyncCommand::new("defaults").args(["delete", "-g", "AppleAccentColor"]).output().await
  } else {
    AsyncCommand::new("defaults")
      .args(["write", "-g", "AppleAccentColor", "-int", &color_value.to_string()])
      .output()
      .await
  };

  match output {
    Ok(output) => {
      if output.status.success() {
        info!("✅ macOS accent color preference set to {} (restart apps to see changes)", color.name());
      } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // "Domain not found" is expected when deleting non-existent key
        if !stderr.contains("does not exist") {
          warn!("Failed to set accent color preference: {}", stderr);
        }
      }
    }
    Err(e) => {
      warn!("Failed to execute defaults command: {}", e);
    }
  }
}

/// Set macOS highlight color (selection color)
///
/// Unlike accent color, this can be any RGB value.
/// Format: "R G B" where values are 0.0-1.0
#[allow(dead_code)]
pub async fn set_highlight_color(r: f32, g: f32, b: f32) {
  if !is_macos() {
    debug!("Not on macOS, skipping highlight color change");
    return;
  }

  let color_string = format!("{:.6} {:.6} {:.6}", r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0));

  let output = AsyncCommand::new("defaults")
    .args(["write", "-g", "AppleHighlightColor", "-string", &color_string])
    .output()
    .await;

  match output {
    Ok(output) => {
      if output.status.success() {
        debug!("macOS highlight color set to RGB({}, {}, {})", r, g, b);
      } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Failed to set highlight color: {}", stderr);
      }
    }
    Err(e) => {
      warn!("Failed to execute defaults command: {}", e);
    }
  }
}

/// Get current macOS appearance mode
#[allow(dead_code)]
pub async fn get_appearance_mode() -> Option<AppearanceMode> {
  if !is_macos() {
    return None;
  }

  let output = AsyncCommand::new("defaults")
    .args(["read", "-g", "AppleInterfaceStyle"])
    .output()
    .await
    .ok()?;

  if output.status.success() {
    let style = String::from_utf8_lossy(&output.stdout);
    if style.trim().eq_ignore_ascii_case("dark") {
      return Some(AppearanceMode::Dark);
    }
  }

  // If AppleInterfaceStyle is not set or not "Dark", it's Light mode
  Some(AppearanceMode::Light)
}

/// Toggle between dark and light mode
#[allow(dead_code)]
pub async fn toggle_appearance_mode() {
  let current = get_appearance_mode().await;

  let new_mode = match current {
    Some(AppearanceMode::Dark) => AppearanceMode::Light,
    _ => AppearanceMode::Dark,
  };

  set_appearance_mode(new_mode).await;
}

/// Apply macOS theme based on wallpaper (simplified version)
///
/// This is a basic implementation that:
/// 1. Optionally sets dark/light mode based on image brightness
/// 2. Optionally sets accent color based on dominant color
///
/// Note: This is much simpler than pywal since macOS has limited theming options.
#[allow(dead_code)]
#[allow(clippy::collapsible_if)]
pub async fn apply_theme_from_wallpaper(
  _wallpaper_path: &Path,
  set_dark_mode: bool,
  set_accent: bool,
  dominant_color: Option<(u8, u8, u8)>,
  is_dark_image: Option<bool>,
) {
  if !is_macos() {
    debug!("Not on macOS, skipping theme application");
    return;
  }

  // Set appearance mode based on image brightness
  if set_dark_mode {
    if let Some(is_dark) = is_dark_image {
      let mode = if is_dark { AppearanceMode::Dark } else { AppearanceMode::Light };
      set_appearance_mode(mode).await;
    }
  }

  // Set accent color based on dominant color
  if set_accent {
    if let Some((r, g, b)) = dominant_color {
      let accent = AccentColor::from_dominant_color(r, g, b);
      set_accent_color(accent).await;
    }
  }
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
  use super::*;

  #[test]
  fn test_accent_color_from_str() {
    assert_eq!(AccentColor::from_str("red"), Some(AccentColor::Red));
    assert_eq!(AccentColor::from_str("RED"), Some(AccentColor::Red));
    assert_eq!(AccentColor::from_str("Blue"), Some(AccentColor::Blue));
    assert_eq!(AccentColor::from_str("multicolor"), Some(AccentColor::Multicolor));
    assert_eq!(AccentColor::from_str("invalid"), None);
  }

  #[test]
  fn test_accent_color_from_dominant_color() {
    // Red-ish
    assert_eq!(AccentColor::from_dominant_color(255, 50, 50), AccentColor::Red);
    // Green-ish
    assert_eq!(AccentColor::from_dominant_color(50, 200, 50), AccentColor::Green);
    // Blue-ish
    assert_eq!(AccentColor::from_dominant_color(50, 100, 255), AccentColor::Blue);
    // Grayscale -> Multicolor
    assert_eq!(AccentColor::from_dominant_color(128, 128, 128), AccentColor::Multicolor);
  }

  #[test]
  fn test_appearance_mode_from_str() {
    assert_eq!(AppearanceMode::from_str("dark"), Some(AppearanceMode::Dark));
    assert_eq!(AppearanceMode::from_str("LIGHT"), Some(AppearanceMode::Light));
    assert_eq!(AppearanceMode::from_str("auto"), Some(AppearanceMode::Auto));
    assert_eq!(AppearanceMode::from_str("invalid"), None);
  }
}
