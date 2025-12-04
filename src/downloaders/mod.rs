//! Wallpaper downloader system with pluggable sources

pub mod apod;
pub mod bing;
pub mod client;
pub mod earthview;
pub mod filesystem;
pub mod picsum;
pub mod reddit;
pub mod registry;
pub mod traits;
pub mod unsplash;
pub mod wallhaven;

use crate::config::Config;
use anyhow::Result;

/// Download wallpaper from specified source by name
/// The `query` parameter contains additional CLI arguments (e.g., search terms, subreddit names)
pub async fn download_from_source(source: &str, config: &Config, query: &[String]) -> Result<traits::Wallpaper> {
  let registry = registry::DownloaderRegistry::new();
  let downloader = registry.get_downloader(source)?;

  downloader.download(config, query).await
}

/// List all available downloader sources
pub fn list_sources() -> Vec<String> {
  let registry = registry::DownloaderRegistry::new();
  registry.list_sources()
}
