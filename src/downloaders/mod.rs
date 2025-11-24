//! Wallpaper downloader system with pluggable sources

pub mod apod;
pub mod registry;
pub mod traits;

use crate::config::Config;
use anyhow::Result;

/// Download wallpaper from specified source by name
pub async fn download_from_source(source: &str, config: &Config) -> Result<traits::DownloadedWallpaper> {
  let registry = registry::DownloaderRegistry::new();
  let downloader = registry.get_downloader(source)?;

  let request = traits::DownloadRequest {
    category: Some(config.sources.category.clone()),
    resolution: None, // Will auto-detect
    quality: None,
  };

  downloader.download(&request).await
}

/// List all available downloader sources
pub fn list_sources() -> Vec<String> {
  let registry = registry::DownloaderRegistry::new();
  registry.list_sources()
}
