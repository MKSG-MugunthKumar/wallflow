//! Bing Photo of the Day downloader
//!
//! Downloads the daily wallpaper from Bing's image archive.
//! Simple JSON API that returns up to 8 recent images.

use super::DownloadOptions;
use super::client::WallflowClient as Client;
use super::filesystem::FilesystemHelper;
use super::traits::{Wallpaper, WallpaperDownloader};
use crate::config::Config;
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use chrono::Utc;
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::path::Path;
use tracing::debug;

/// Bing API endpoint for daily images
const BING_API_URL: &str = "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=8&mkt=en-US";

/// Bing API response structure
#[derive(Debug, Deserialize)]
struct BingResponse {
  images: Vec<BingImage>,
}

/// Individual image from Bing
#[derive(Debug, Deserialize)]
struct BingImage {
  /// URL base path (without resolution suffix)
  urlbase: String,
  /// Copyright/description text
  copyright: String,
  /// Whether this is marked as a wallpaper
  #[serde(default)]
  wp: bool,
}

/// Bing Photo of the Day downloader
pub struct BingDownloader;

#[async_trait]
impl WallpaperDownloader for BingDownloader {
  /// Download Bing Photo of the Day
  /// Note: Bing ignores query parameters as it returns daily curated images
  async fn download(&self, config: &Config, _query: &[String], opts: &DownloadOptions) -> Result<Wallpaper> {
    debug!("Fetching Bing Photo of the Day");

    let client = Client::from(&config.advanced);
    let response = client.get(BING_API_URL).send().await.context("Failed to send request to Bing API")?;

    if !response.status().is_success() {
      let status = response.status();
      let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
      return Err(anyhow!("Bing API request failed with status {}: {}", status, error_text));
    }

    let bing_data: BingResponse = response.json().await.context("Failed to parse Bing API response")?;

    if bing_data.images.is_empty() {
      return Err(anyhow!("No images returned from Bing API"));
    }

    // Filter to only wallpaper-marked images, or use all if none are marked
    let wallpaper_images: Vec<&BingImage> = bing_data.images.iter().filter(|img| img.wp).collect();
    let images = if wallpaper_images.is_empty() {
      bing_data.images.iter().collect()
    } else {
      wallpaper_images
    };

    // Pick a random image
    let image = images
      .choose(&mut rand::thread_rng())
      .ok_or_else(|| anyhow!("Failed to select random Bing image"))?;

    debug!("Selected Bing image: {}", image.copyright);

    // Build UHD image URL
    let image_url = format!("https://www.bing.com{}_UHD.jpg", image.urlbase);
    debug!("Image URL: {}", image_url);

    // Download the image
    let image_response = client.get(&image_url).send().await.context("Failed to download Bing image")?;

    if !image_response.status().is_success() {
      return Err(anyhow!("Bing image download failed with status: {}", image_response.status()));
    }

    let bytes = image_response.bytes().await.context("Failed to read Bing image data")?;

    let filename = format!("{}_{}", self.source_name(), FilesystemHelper::make_file_suffix());
    let download_dir = opts.output_dir.as_deref().unwrap_or(Path::new(&config.paths.downloads));
    let file_path = download_dir.join(&filename).with_extension("jpg");

    // Ensure download directory exists
    if let Some(parent) = file_path.parent() {
      tokio::fs::create_dir_all(parent).await.context("Failed to create download directory")?;
    }

    tokio::fs::write(&file_path, &bytes).await.context("Failed to save Bing image")?;

    let wallpaper = Wallpaper {
      file_path,
      downloaded_at: Utc::now(),
      source: self.source_name().to_string(),
    };

    debug!("Downloaded Bing wallpaper: {:?}", wallpaper);
    Ok(wallpaper)
  }

  fn source_name(&self) -> &'static str {
    "bing"
  }
}
