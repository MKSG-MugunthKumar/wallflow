use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

/// Request parameters for downloading a wallpaper
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub category: Option<String>,
    pub resolution: Option<(u32, u32)>,
    pub quality: Option<String>,
}

/// Result of a successful wallpaper download
#[derive(Debug)]
pub struct DownloadedWallpaper {
    pub file_path: PathBuf,
    pub source_url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub metadata: WallpaperMetadata,
}

/// Metadata about a downloaded wallpaper
#[derive(Debug)]
pub struct WallpaperMetadata {
    pub dimensions: Option<(u32, u32)>,
    pub file_size: Option<u64>,
    pub format: Option<String>,
    pub source: String,
    pub downloaded_at: DateTime<Utc>,
}

/// Options for applying wallpaper to desktop
#[derive(Debug, Clone)]
pub struct WallpaperOptions {
    pub transition: Option<String>,
    pub duration: Option<u32>,
    pub fps: Option<u32>,
    pub scaling: WallpaperScaling,
    pub monitor: MonitorSelection,
}

/// How to scale/position the wallpaper
#[derive(Debug, Clone)]
pub enum WallpaperScaling {
    Fill,     // Scale to fill screen, may crop
    Fit,      // Scale to fit screen, maintain aspect ratio
    Stretch,  // Stretch to fill screen, may distort
    Center,   // Center image, no scaling
    Tile,     // Tile image across screen
}

/// Which monitor(s) to apply wallpaper to
#[derive(Debug, Clone)]
pub enum MonitorSelection {
    Primary,           // Primary monitor only
    All,               // All monitors
    Specific(u32),     // Specific monitor by index
}

/// Trait for wallpaper backend implementations
#[async_trait]
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

impl Default for WallpaperScaling {
    fn default() -> Self {
        WallpaperScaling::Fill
    }
}

impl Default for MonitorSelection {
    fn default() -> Self {
        MonitorSelection::All
    }
}
