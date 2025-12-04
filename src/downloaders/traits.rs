use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

use crate::config::Config;

/// Result of a successful wallpaper download
#[derive(Debug)]
#[allow(dead_code)]
pub struct Wallpaper {
  pub file_path: PathBuf,
  pub source: String,
  pub downloaded_at: DateTime<Utc>,
}

/// Trait for wallpaper downloader implementations
#[async_trait]
pub trait WallpaperDownloader {
  /// Download a wallpaper based on the request
  /// The `query` parameter contains additional CLI arguments (e.g., search terms, subreddit names)
  async fn download(&self, config: &Config, query: &[String]) -> Result<Wallpaper>;

  /// Get the source name for this downloader
  fn source_name(&self) -> &'static str;
}
