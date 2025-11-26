use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Request parameters for downloading a wallpaper
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DownloadRequest {
  pub category: Option<String>,
  pub resolution: Option<(u32, u32)>,
  pub quality: Option<String>,
}

/// Options for applying wallpaper to desktop
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WallpaperOptions {
  pub transition: Option<String>,
  pub duration: Option<u32>,
  pub fps: Option<u32>,
  pub scaling: WallpaperScaling,
  pub monitor: MonitorSelection,
}

/// How to scale/position the wallpaper
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub enum WallpaperScaling {
  #[default]
  Fill, // Scale to fill screen, may crop
  Fit,     // Scale to fit screen, maintain aspect ratio
  Stretch, // Stretch to fill screen, may distort
  Center,  // Center image, no scaling
  Tile,    // Tile image across screen
}

/// Which monitor(s) to apply wallpaper to
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub enum MonitorSelection {
  Primary, // Primary monitor only
  #[default]
  All, // All monitors
  Specific(u32), // Specific monitor by index
}

/// Trait for wallpaper backend implementations
#[async_trait]
#[allow(dead_code)]
pub trait WallpaperBackend {
  /// Apply wallpaper to the desktop
  async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()>;

  /// Get current wallpaper path (if supported)
  async fn get_current_wallpaper(&self) -> Result<Option<PathBuf>>;

  /// Check if this backend is available on current system
  fn is_available(&self) -> bool;

  /// Get backend priority (higher = preferred)
  fn priority(&self) -> u32;

  /// Backend name for logging and identification
  fn name(&self) -> &'static str;

  /// List of transition effects this backend supports
  fn supported_transitions(&self) -> Vec<String>;

  /// Validate that the backend is properly configured
  fn validate(&self) -> Result<()> {
    if !self.is_available() {
      return Err(anyhow::anyhow!("Backend '{}' is not available", self.name()));
    }
    Ok(())
  }
}

impl Default for WallpaperOptions {
  fn default() -> Self {
    Self {
      transition: None,
      duration: None,
      fps: None,
      scaling: WallpaperScaling::Fill,
      monitor: MonitorSelection::All,
    }
  }
}
