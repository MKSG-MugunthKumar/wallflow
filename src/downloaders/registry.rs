use super::apod::ApodDownloader;
use super::picsum::PicsumDownloader;
use super::traits::WallpaperDownloader;
use super::wallhaven::WallhavenDownloader;
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
    self.register_downloader(Arc::new(ApodDownloader));
    self.register_downloader(Arc::new(PicsumDownloader));
    self.register_downloader(Arc::new(WallhavenDownloader));
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
