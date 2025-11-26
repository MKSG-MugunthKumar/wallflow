//! Wallpaper downloader system with pluggable sources

pub mod apod;
pub mod client;
pub mod filesystem;
pub mod registry;
pub mod traits;

use crate::config::Config;
use anyhow::Result;

/// Download wallpaper from specified source by name
pub async fn download_from_source(source: &str, config: &Config) -> Result<traits::Wallpaper> {
  let registry = registry::DownloaderRegistry::new();
  let downloader = registry.get_downloader(source)?;

  downloader.download(config).await
}

/// List all available downloader sources
pub fn list_sources() -> Vec<String> {
  let registry = registry::DownloaderRegistry::new();
  registry.list_sources()
}
