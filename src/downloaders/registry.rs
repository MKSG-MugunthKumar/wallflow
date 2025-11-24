use super::apod::ApodDownloader;
use super::traits::WallpaperDownloader;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for managing wallpaper downloaders
pub struct DownloaderRegistry {
  downloaders: HashMap<String, Arc<dyn WallpaperDownloader + Send + Sync>>,
}

#[allow(dead_code)]
impl DownloaderRegistry {
  /// Create a new downloader registry with built-in downloaders
  pub fn new() -> Self {
    let mut registry = Self { downloaders: HashMap::new() };

    registry.register_builtin_downloaders();
    registry
  }

  /// Register all built-in downloaders
  fn register_builtin_downloaders(&mut self) {
    // Register NASA APOD downloader
    self.register_downloader(Arc::new(ApodDownloader::new()));

    // TODO: Register other downloaders as they are implemented
    // self.register_downloader(Arc::new(BingDownloader::new()));
    // self.register_downloader(Arc::new(EarthviewDownloader::new()));
    // self.register_downloader(Arc::new(RedditDownloader::new()));
  }

  /// Register a new downloader
  pub fn register_downloader(&mut self, downloader: Arc<dyn WallpaperDownloader + Send + Sync>) {
    let source_name = downloader.source_name().to_string();
    self.downloaders.insert(source_name, downloader);
  }

  /// Get a downloader by source name
  pub fn get_downloader(&self, source: &str) -> Result<Arc<dyn WallpaperDownloader + Send + Sync>> {
    self
      .downloaders
      .get(source)
      .cloned()
      .ok_or_else(|| anyhow!("Unknown wallpaper source: {}", source))
  }

  /// List all available source names
  pub fn list_sources(&self) -> Vec<String> {
    let mut sources: Vec<String> = self.downloaders.keys().cloned().collect();
    sources.sort();
    sources
  }

  /// Check if a source exists
  pub fn has_source(&self, source: &str) -> bool {
    self.downloaders.contains_key(source)
  }
}

impl Default for DownloaderRegistry {
  fn default() -> Self {
    Self::new()
  }
}
