//! Unsplash high-resolution photo downloader
//!
//! Downloads high-quality photos from Unsplash.com.
//! Requires an Access Key (get one at https://unsplash.com/developers).

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

/// Unsplash API endpoint for random photos
const UNSPLASH_API_URL: &str = "https://api.unsplash.com/photos/random";

/// Unsplash photo response
#[derive(Debug, Deserialize)]
struct UnsplashPhoto {
  /// Photo ID
  #[allow(dead_code)]
  id: String,
  /// Image dimensions
  width: u32,
  height: u32,
  /// URLs for different sizes
  urls: UnsplashUrls,
  /// User/photographer info
  #[allow(dead_code)]
  user: UnsplashUser,
  /// Description
  #[allow(dead_code)]
  description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UnsplashUrls {
  /// Full resolution image
  full: String,
  /// Raw image (highest quality)
  #[allow(dead_code)]
  raw: String,
}

#[derive(Debug, Deserialize)]
struct UnsplashUser {
  /// Photographer name
  #[allow(dead_code)]
  name: String,
}

/// Unsplash downloader
pub struct UnsplashDownloader;

#[async_trait]
impl WallpaperDownloader for UnsplashDownloader {
  /// Download a photo from Unsplash
  /// Query parameters are used as search topics (e.g., "wallflow unsplash nature")
  async fn download(&self, config: &Config, query: &[String], opts: &DownloadOptions) -> Result<Wallpaper> {
    let unsplash_config = &config.sources.unsplash;

    // Access Key is required for Unsplash (used as client_id)
    let access_key = unsplash_config.access_key.as_ref().ok_or_else(|| {
      anyhow!(
        "Unsplash requires an Access Key. Get one at https://unsplash.com/developers and add it to config:\n\
         sources:\n  unsplash:\n    access_key: \"your-access-key\""
      )
    })?;

    if access_key.is_empty() {
      return Err(anyhow!("Unsplash access_key is empty"));
    }

    debug!("Fetching random photo from Unsplash");

    let client = Client::from(&config.advanced);

    // Build request with query parameters (access_key is used as client_id)
    let mut request = client
      .get(UNSPLASH_API_URL)
      .query(&[("client_id", access_key.as_str()), ("count", "10"), ("orientation", "landscape")]);

    // Add search query if provided
    let search_query = query.join(" ");
    if !search_query.is_empty() {
      request = request.query(&[("query", search_query.as_str())]);
    }

    let response = request.send().await.context("Failed to send request to Unsplash API")?;

    if !response.status().is_success() {
      let status = response.status();
      let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

      if status.as_u16() == 401 {
        return Err(anyhow!("Unsplash access_key is invalid. Check your configuration."));
      }
      if status.as_u16() == 403 {
        return Err(anyhow!("Unsplash rate limit exceeded. Try again later."));
      }

      return Err(anyhow!("Unsplash API request failed with status {}: {}", status, error_text));
    }

    let photos: Vec<UnsplashPhoto> = response.json().await.context("Failed to parse Unsplash API response")?;

    if photos.is_empty() {
      return Err(anyhow!("No photos returned from Unsplash"));
    }

    // Filter to landscape images with adequate resolution
    let suitable_photos: Vec<&UnsplashPhoto> = photos.iter().filter(|p| p.width > p.height && p.width >= 1920).collect();

    let photos_to_use = if suitable_photos.is_empty() {
      photos.iter().collect()
    } else {
      suitable_photos
    };

    // Pick a random photo
    let photo = photos_to_use
      .choose(&mut rand::thread_rng())
      .ok_or_else(|| anyhow!("Failed to select random Unsplash photo"))?;

    debug!("Selected Unsplash photo: {}x{}", photo.width, photo.height);

    // Use full URL with width parameter for optimal resolution
    let image_url = format!("{}&w=2560&q=85", photo.urls.full);
    debug!("Image URL: {}", image_url);

    // Download the image
    let image_response = client.get(&image_url).send().await.context("Failed to download Unsplash image")?;

    if !image_response.status().is_success() {
      return Err(anyhow!("Unsplash image download failed with status: {}", image_response.status()));
    }

    let bytes = image_response.bytes().await.context("Failed to read Unsplash image data")?;

    let filename = format!("{}_{}", self.source_name(), FilesystemHelper::make_file_suffix());
    let download_dir = opts.output_dir.as_deref().unwrap_or(Path::new(&config.paths.downloads));
    let file_path = download_dir.join(&filename).with_extension("jpg");

    // Ensure download directory exists
    if let Some(parent) = file_path.parent() {
      tokio::fs::create_dir_all(parent).await.context("Failed to create download directory")?;
    }

    tokio::fs::write(&file_path, &bytes).await.context("Failed to save Unsplash image")?;

    let wallpaper = Wallpaper {
      file_path,
      downloaded_at: Utc::now(),
      source: self.source_name().to_string(),
    };

    debug!("Downloaded Unsplash wallpaper: {:?}", wallpaper);
    Ok(wallpaper)
  }

  fn source_name(&self) -> &'static str {
    "unsplash"
  }
}
