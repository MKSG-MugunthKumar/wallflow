//! NASA Astronomy Picture of the Day (APOD) downloader
//!
//! This module demonstrates:
//! - HTTP API integration with JSON parsing
//! - Error handling for network requests
//! - File download and saving
//! - Metadata extraction and preservation

use super::DownloadOptions;
use super::traits::WallpaperDownloader;
use crate::config::Config;
use crate::downloaders::client::WallflowClient as Client;
use crate::downloaders::filesystem::FilesystemHelper;
use crate::downloaders::traits::Wallpaper;
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use std::path::Path;
use tracing::debug;

/// NASA APOD API response structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ApodResponse {
  /// Date of the image (YYYY-MM-DD format)
  date: String,

  /// Title of the image
  title: String,

  /// Detailed explanation/description
  explanation: String,

  /// High-definition image URL (when available)
  hdurl: Option<String>,

  /// Standard definition image URL
  url: String,

  /// Media type (usually "image" for wallpapers)
  media_type: String,

  /// Copyright information (when applicable)
  copyright: Option<String>,
}

/// NASA APOD downloader implementation
///
/// Educational aspects:
/// - Uses public API without requiring authentication
/// - Demonstrates JSON parsing with optional fields
/// - Shows proper error handling for network operations
/// - Implements file download with progress tracking
pub struct ApodDownloader;

#[async_trait]
impl WallpaperDownloader for ApodDownloader {
  /// Fetch APOD data from NASA API
  /// Note: APOD ignores query parameters as it always returns the picture of the day
  async fn download(&self, config: &Config, _query: &[String], opts: &DownloadOptions) -> Result<Wallpaper> {
    debug!("Fetching APOD data from NASA API");
    let client = Client::from(&config.advanced);
    let url = config.sources.apod.url.clone();
    let api_key: String = config.sources.apod.api_key.clone();

    let response = client
      .get(&url)
      .query(&[("api_key", &api_key)])
      .send()
      .await
      .context("Failed to send request to NASA APOD API")?;

    if !response.status().is_success() {
      let status = response.status();
      let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
      return Err(anyhow!("NASA API request failed with status {}: {}", status, error_text));
    }

    let apod_data: ApodResponse = response.json().await.context("Failed to parse NASA APOD API response as JSON")?;

    debug!("Successfully fetched APOD: {}", apod_data.title);

    // Validate that this is actually an image we can use as wallpaper
    if apod_data.media_type != "image" {
      return Err(anyhow!(
        "Today's APOD is not an image (type: {}), cannot use as wallpaper",
        apod_data.media_type
      ));
    }
    let image_url = apod_data.hdurl.as_ref().unwrap_or(&apod_data.url);
    debug!("Image URL: {}", image_url);
    let response = client.get(image_url).send().await.context("Failed to download image")?;

    if !response.status().is_success() {
      return Err(anyhow!("Image download failed with status: {}", response.status()));
    }

    let _content_length = response.content_length();
    let bytes = response.bytes().await.context("Failed to read image data")?;

    let filename = format!("{}_{}", self.source_name(), FilesystemHelper::make_file_suffix());
    let file_extension = image_url
      .rsplit('.')
      .next()
      .and_then(|ext| {
        let ext = ext.split('?').next().unwrap_or(ext);
        if ext.len() <= 5 { Some(ext) } else { None }
      })
      .unwrap_or("jpg");
    let download_dir = opts.output_dir.as_deref().unwrap_or(Path::new(&config.paths.downloads));
    let file_path = download_dir.join(&filename).with_extension(file_extension);

    // Ensure the parent directory exists
    if let Some(parent) = file_path.parent() {
      tokio::fs::create_dir_all(parent).await.context("Failed to create download directory")?;
    }

    tokio::fs::write(&file_path, &bytes).await.context("Failed to save image to file")?;

    let wallpaper = Wallpaper {
      file_path,
      downloaded_at: Utc::now(),
      source: self.source_name().to_string(),
    };

    debug!("✅ Downloaded wallpaper: {:?}", wallpaper);
    Ok(wallpaper)
  }

  fn source_name(&self) -> &'static str {
    "apod"
  }
}
