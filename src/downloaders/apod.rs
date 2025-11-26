//! NASA Astronomy Picture of the Day (APOD) downloader
//!
//! This module demonstrates:
//! - HTTP API integration with JSON parsing
//! - Error handling for network requests
//! - File download and saving
//! - Metadata extraction and preservation

use super::traits::{DownloadRequest, DownloadedWallpaper, WallpaperDownloader, WallpaperMetadata};
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use std::path::Path;
use tracing::{debug, warn};

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
pub struct ApodDownloader {
  client: Client,
  api_url: String,
}

impl ApodDownloader {
  /// Create a new APOD downloader
  pub fn new() -> Self {
    Self {
      client: Client::new(),
      // Using demo key for educational purposes
      // In production, users should get their own API key from api.nasa.gov
      api_url: "https://api.nasa.gov/planetary/apod?api_key=DEMO_KEY".to_string(),
    }
  }

  /// Fetch APOD data from NASA API
  async fn fetch_apod_data(&self) -> Result<ApodResponse> {
    debug!("Fetching APOD data from NASA API");

    let response = self
      .client
      .get(&self.api_url)
      .header("User-Agent", "wallflow/1.0 (Wallpaper manager)")
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

    Ok(apod_data)
  }

  /// Download the image file to local storage
  async fn download_image(&self, image_url: &str, file_path: &Path) -> Result<u64> {
    debug!("Downloading image from: {}", image_url);

    let response = self.client.get(image_url).send().await.context("Failed to download image")?;

    if !response.status().is_success() {
      return Err(anyhow!("Image download failed with status: {}", response.status()));
    }

    let _content_length = response.content_length();
    let bytes = response.bytes().await.context("Failed to read image data")?;

    // Ensure the parent directory exists
    if let Some(parent) = file_path.parent() {
      tokio::fs::create_dir_all(parent).await.context("Failed to create download directory")?;
    }

    tokio::fs::write(file_path, &bytes).await.context("Failed to save image to file")?;

    let file_size = bytes.len() as u64;
    debug!("Successfully downloaded {} bytes to {}", file_size, file_path.display());

    Ok(file_size)
  }

  /// Generate a safe filename for the downloaded image
  fn generate_filename(&self, apod_data: &ApodResponse, downloads_dir: &str) -> Result<std::path::PathBuf> {
    // Extract file extension from URL
    let image_url = apod_data.hdurl.as_ref().unwrap_or(&apod_data.url);
    let extension = image_url
      .split('.')
      .next_back()
      .unwrap_or("jpg")
      .split('?') // Remove query parameters
      .next()
      .unwrap_or("jpg");

    // Create safe filename from title and date
    let safe_title = apod_data
      .title
      .chars()
      .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '_' })
      .collect::<String>()
      .replace(' ', "_")
      .trim_matches('_')
      .to_lowercase();

    let filename = format!("apod_{}_{}.{}", apod_data.date, safe_title, extension);
    let file_path = Path::new(downloads_dir).join(filename);

    Ok(file_path)
  }

  /// Extract image dimensions from the downloaded file
  async fn get_image_dimensions(&self, file_path: &Path) -> Option<(u32, u32)> {
    match imagesize::size(file_path) {
      Ok(size) => {
        debug!("Image dimensions: {}x{}", size.width, size.height);
        Some((size.width as u32, size.height as u32))
      }
      Err(e) => {
        warn!("Failed to get image dimensions: {}", e);
        None
      }
    }
  }
}

#[async_trait]
impl WallpaperDownloader for ApodDownloader {
  async fn download(&self, request: &DownloadRequest) -> Result<DownloadedWallpaper> {
    debug!("Starting NASA APOD wallpaper download");

    // Note: APOD doesn't support categories or custom resolutions
    // It always returns today's astronomy picture
    if request.category.is_some() {
      debug!("APOD downloader ignoring category request - returns today's astronomy picture");
    }

    // Step 1: Fetch APOD data from NASA API
    let apod_data = self.fetch_apod_data().await.context("Failed to fetch APOD data")?;

    // Step 2: Determine which image URL to use (prefer HD version)
    let image_url = apod_data.hdurl.as_ref().unwrap_or(&apod_data.url);
    debug!("Using image URL: {}", image_url);

    // Step 3: Generate safe filename for download
    let downloads_dir = crate::config::Config::load_or_default()
      .context("Failed to load configuration")?
      .paths
      .downloads;

    let file_path = self
      .generate_filename(&apod_data, &downloads_dir)
      .context("Failed to generate filename")?;

    // Step 4: Download the image
    let file_size = self.download_image(image_url, &file_path).await.context("Failed to download image")?;

    // Step 5: Extract image metadata
    let dimensions = self.get_image_dimensions(&file_path).await;

    // Step 6: Build result with all metadata
    let downloaded_wallpaper = DownloadedWallpaper {
      file_path: file_path.clone(),
      source_url: Some(image_url.clone()),
      title: Some(apod_data.title.clone()),
      description: Some(apod_data.explanation.clone()),
      metadata: WallpaperMetadata {
        dimensions,
        file_size: Some(file_size),
        format: file_path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_uppercase()),
        source: "NASA APOD".to_string(),
        downloaded_at: Utc::now(),
      },
    };

    debug!("✅ Successfully downloaded APOD wallpaper: {}", apod_data.title);
    Ok(downloaded_wallpaper)
  }

  fn source_name(&self) -> &'static str {
    "apod"
  }

  async fn is_available(&self) -> bool {
    // Test basic connectivity to NASA API
    match self.client.head(&self.api_url).send().await {
      Ok(response) => response.status().is_success(),
      Err(_) => false,
    }
  }
}

impl Default for ApodDownloader {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_filename_generation() {
    let downloader = ApodDownloader::new();

    let test_data = ApodResponse {
      date: "2024-01-15".to_string(),
      title: "Amazing Galaxy View!".to_string(),
      explanation: "A beautiful galaxy".to_string(),
      hdurl: Some("https://example.com/image.jpg".to_string()),
      url: "https://example.com/image.jpg".to_string(),
      media_type: "image".to_string(),
      copyright: None,
    };

    let filename = downloader.generate_filename(&test_data, "/tmp").unwrap();
    let expected = "/tmp/apod_2024-01-15_amazing_galaxy_view.jpg";

    assert_eq!(filename.to_string_lossy(), expected);
  }
}
