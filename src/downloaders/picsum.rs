//! Lorem Picsum random image downloader
//!
//! Simple wallpaper source that downloads random high-quality photos
//! from https://picsum.photos

use super::client::WallflowClient as Client;
use super::filesystem::FilesystemHelper;
use super::traits::{Wallpaper, WallpaperDownloader};
use crate::config::Config;
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use chrono::Utc;
use std::path::Path;
use tracing::debug;

/// Picsum random photo downloader
pub struct PicsumDownloader;

#[async_trait]
impl WallpaperDownloader for PicsumDownloader {
  async fn download(&self, config: &Config) -> Result<Wallpaper> {
    let resolution = config.get_picsum_resolution()?;
    let url = format!("https://picsum.photos/{}/{}", resolution.width, resolution.height);

    debug!("Fetching random image from Picsum: {}", url);

    let client = Client::from(&config.advanced);
    let response = client.get(&url).send().await.context("Failed to send request to Picsum")?;

    if !response.status().is_success() {
      return Err(anyhow!("Picsum request failed with status: {}", response.status()));
    }

    let bytes = response.bytes().await.context("Failed to read image data from Picsum")?;

    let filename = format!("{}_{}", self.source_name(), FilesystemHelper::make_file_suffix());
    let file_path = Path::new(&config.paths.downloads).join(&filename).with_extension("jpg");

    // Ensure download directory exists
    if let Some(parent) = file_path.parent() {
      tokio::fs::create_dir_all(parent).await.context("Failed to create download directory")?;
    }

    tokio::fs::write(&file_path, &bytes).await.context("Failed to save Picsum image")?;

    let wallpaper = Wallpaper {
      file_path,
      downloaded_at: Utc::now(),
      source: self.source_name().to_string(),
    };

    debug!("Downloaded Picsum wallpaper: {:?}", wallpaper);
    Ok(wallpaper)
  }

  fn source_name(&self) -> &'static str {
    "picsum"
  }
}
