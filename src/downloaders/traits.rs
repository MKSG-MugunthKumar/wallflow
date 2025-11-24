use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Request parameters for downloading a wallpaper
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DownloadRequest {
  pub category: Option<String>,
  pub resolution: Option<(u32, u32)>,
  pub quality: Option<String>,
}

/// Result of a successful wallpaper download
#[derive(Debug)]
#[allow(dead_code)]
pub struct DownloadedWallpaper {
  pub file_path: PathBuf,
  pub source_url: Option<String>,
  pub title: Option<String>,
  pub description: Option<String>,
  pub metadata: WallpaperMetadata,
}

/// Metadata about a downloaded wallpaper
#[derive(Debug)]
#[allow(dead_code)]
pub struct WallpaperMetadata {
  pub dimensions: Option<(u32, u32)>,
  pub file_size: Option<u64>,
  pub format: Option<String>,
  pub source: String,
  pub downloaded_at: DateTime<Utc>,
}

/// Trait for wallpaper downloader implementations
#[async_trait]
#[allow(dead_code)]
pub trait WallpaperDownloader {
  /// Download a wallpaper based on the request
  async fn download(&self, request: &DownloadRequest) -> Result<DownloadedWallpaper>;

  /// Get the source name for this downloader
  fn source_name(&self) -> &'static str;

  /// Check if this source requires authentication
  fn requires_auth(&self) -> bool {
    false
  }

  /// Validate configuration for this downloader
  fn validate_config(&self, _config: &crate::config::Config) -> Result<()> {
    Ok(())
  }

  /// Check if this downloader is available (e.g., network connectivity)
  async fn is_available(&self) -> bool {
    true
  }
}
