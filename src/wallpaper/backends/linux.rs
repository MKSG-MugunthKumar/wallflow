//! Linux-specific wallpaper backends for X11 and Wayland

use super::traits::{WallpaperBackend, WallpaperOptions, WallpaperScaling};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::process::Command as AsyncCommand;
use tracing::debug;

/// swaybg backend for Sway
pub struct SwaybgBackend;

impl SwaybgBackend {
  pub fn new() -> Self {
    Self
  }
}

#[async_trait]
impl WallpaperBackend for SwaybgBackend {
  async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()> {
    let mut cmd = AsyncCommand::new("swaybg");

    // swaybg doesn't support transitions, but supports scaling modes
    let mode = match options.scaling {
      WallpaperScaling::Fill => "fill",
      WallpaperScaling::Fit => "fit",
      WallpaperScaling::Stretch => "stretch",
      WallpaperScaling::Center => "center",
      WallpaperScaling::Tile => "tile",
    };

    cmd.args(["-i", &image_path.to_string_lossy(), "-m", mode]);

    let output = cmd.output().await.context("Failed to execute swaybg")?;

    if output.status.success() {
      debug!("✅ swaybg wallpaper set successfully");
      Ok(())
    } else {
      let stderr = String::from_utf8_lossy(&output.stderr);
      Err(anyhow::anyhow!("swaybg failed: {}", stderr))
    }
  }

  async fn get_current_wallpaper(&self) -> Result<Option<PathBuf>> {
    Ok(None)
  }

  fn is_available(&self) -> bool {
    which::which("swaybg").is_ok()
  }

  fn priority(&self) -> u32 {
    85
  }
  fn name(&self) -> &'static str {
    "swaybg"
  }
  fn supported_transitions(&self) -> Vec<String> {
    vec![]
  }
}

/// hyprpaper backend for Hyprland
pub struct HyprpaperBackend;

impl HyprpaperBackend {
  pub fn new() -> Self {
    Self
  }
}

#[async_trait]
impl WallpaperBackend for HyprpaperBackend {
  async fn set_wallpaper(&self, image_path: &Path, _options: &WallpaperOptions) -> Result<()> {
    // hyprpaper requires preloading and then setting
    let mut preload_cmd = AsyncCommand::new("hyprctl");
    preload_cmd.args(["hyprpaper", "preload", &image_path.to_string_lossy()]);

    let output = preload_cmd.output().await.context("Failed to preload with hyprpaper")?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(anyhow::anyhow!("hyprpaper preload failed: {}", stderr));
    }

    // Set wallpaper on all monitors
    let mut set_cmd = AsyncCommand::new("hyprctl");
    set_cmd.args(["hyprpaper", "wallpaper", &format!(",{}", image_path.display())]);

    let output = set_cmd.output().await.context("Failed to set wallpaper with hyprpaper")?;

    if output.status.success() {
      debug!("✅ hyprpaper wallpaper set successfully");
      Ok(())
    } else {
      let stderr = String::from_utf8_lossy(&output.stderr);
      Err(anyhow::anyhow!("hyprpaper failed: {}", stderr))
    }
  }

  async fn get_current_wallpaper(&self) -> Result<Option<PathBuf>> {
    Ok(None)
  }

  fn is_available(&self) -> bool {
    which::which("hyprctl").is_ok()
  }

  fn priority(&self) -> u32 {
    90
  }
  fn name(&self) -> &'static str {
    "hyprpaper"
  }
  fn supported_transitions(&self) -> Vec<String> {
    vec![]
  }
}

/// feh backend for X11
pub struct FehBackend;

impl FehBackend {
  pub fn new() -> Self {
    Self
  }
}

#[async_trait]
impl WallpaperBackend for FehBackend {
  async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()> {
    let mut cmd = AsyncCommand::new("feh");

    let bg_option = match options.scaling {
      WallpaperScaling::Fill => "--bg-fill",
      WallpaperScaling::Fit => "--bg-max",
      WallpaperScaling::Stretch => "--bg-scale",
      WallpaperScaling::Center => "--bg-center",
      WallpaperScaling::Tile => "--bg-tile",
    };

    cmd.args([bg_option, &image_path.to_string_lossy()]);

    let output = cmd.output().await.context("Failed to execute feh")?;

    if output.status.success() {
      debug!("✅ feh wallpaper set successfully");
      Ok(())
    } else {
      let stderr = String::from_utf8_lossy(&output.stderr);
      Err(anyhow::anyhow!("feh failed: {}", stderr))
    }
  }

  async fn get_current_wallpaper(&self) -> Result<Option<PathBuf>> {
    Ok(None) // feh doesn't provide current wallpaper info
  }

  fn is_available(&self) -> bool {
    which::which("feh").is_ok()
  }

  fn priority(&self) -> u32 {
    50
  }
  fn name(&self) -> &'static str {
    "feh"
  }
  fn supported_transitions(&self) -> Vec<String> {
    vec![]
  }
}

/// nitrogen backend for X11
pub struct NitrogenBackend;

impl NitrogenBackend {
  pub fn new() -> Self {
    Self
  }
}

#[async_trait]
impl WallpaperBackend for NitrogenBackend {
  async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()> {
    let mut cmd = AsyncCommand::new("nitrogen");

    let mode = match options.scaling {
      WallpaperScaling::Fill => "--set-zoom-fill",
      WallpaperScaling::Fit => "--set-scaled",
      WallpaperScaling::Stretch => "--set-auto",
      WallpaperScaling::Center => "--set-centered",
      WallpaperScaling::Tile => "--set-tiled",
    };

    cmd.args([mode, &image_path.to_string_lossy()]);

    let output = cmd.output().await.context("Failed to execute nitrogen")?;

    if output.status.success() {
      debug!("✅ nitrogen wallpaper set successfully");
      Ok(())
    } else {
      let stderr = String::from_utf8_lossy(&output.stderr);
      Err(anyhow::anyhow!("nitrogen failed: {}", stderr))
    }
  }

  async fn get_current_wallpaper(&self) -> Result<Option<PathBuf>> {
    Ok(None)
  }

  fn is_available(&self) -> bool {
    which::which("nitrogen").is_ok()
  }

  fn priority(&self) -> u32 {
    45
  }
  fn name(&self) -> &'static str {
    "nitrogen"
  }
  fn supported_transitions(&self) -> Vec<String> {
    vec![]
  }
}

/// xwallpaper backend for X11
pub struct XwallpaperBackend;

impl XwallpaperBackend {
  pub fn new() -> Self {
    Self
  }
}

#[async_trait]
impl WallpaperBackend for XwallpaperBackend {
  async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()> {
    let mut cmd = AsyncCommand::new("xwallpaper");

    let flag = match options.scaling {
      WallpaperScaling::Fill => "--zoom",
      WallpaperScaling::Fit => "--maximize",
      WallpaperScaling::Stretch => "--stretch",
      WallpaperScaling::Center => "--center",
      WallpaperScaling::Tile => "--tile",
    };

    cmd.args([flag, &image_path.to_string_lossy()]);

    let output = cmd.output().await.context("Failed to execute xwallpaper")?;

    if output.status.success() {
      debug!("✅ xwallpaper wallpaper set successfully");
      Ok(())
    } else {
      let stderr = String::from_utf8_lossy(&output.stderr);
      Err(anyhow::anyhow!("xwallpaper failed: {}", stderr))
    }
  }

  async fn get_current_wallpaper(&self) -> Result<Option<PathBuf>> {
    Ok(None)
  }

  fn is_available(&self) -> bool {
    which::which("xwallpaper").is_ok()
  }

  fn priority(&self) -> u32 {
    55
  }
  fn name(&self) -> &'static str {
    "xwallpaper"
  }
  fn supported_transitions(&self) -> Vec<String> {
    vec![]
  }
}
