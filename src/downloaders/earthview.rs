//! Google Earth View wallpaper downloader
//!
//! Downloads satellite imagery wallpapers from Google Earth View.
//! Uses a two-step API: first fetch the list, then fetch individual image details.

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

/// Earth View API endpoint for photo list
const EARTHVIEW_LIST_URL: &str = "https://new-images-preview-dot-earth-viewer.appspot.com/_api/photos.json";

/// Earth View API base URL for individual photos
const EARTHVIEW_API_BASE: &str = "https://new-images-preview-dot-earth-viewer.appspot.com/_api/";

/// Earth View photo list item
#[derive(Debug, Deserialize)]
struct EarthViewListItem {
  /// Unique slug identifier for the image
  slug: String,
}

/// Earth View individual photo details
#[derive(Debug, Deserialize)]
struct EarthViewPhoto {
  /// Photo ID
  #[allow(dead_code)]
  id: String,
  /// URL to the full image
  #[serde(rename = "photoUrl")]
  photo_url: String,
  /// Country name
  country: String,
  /// Region within country (may be "-" if unknown)
  region: String,
  /// Location name
  #[allow(dead_code)]
  name: Option<String>,
  /// Attribution text
  #[allow(dead_code)]
  attribution: Option<String>,
}

/// Google Earth View downloader
pub struct EarthViewDownloader;

#[async_trait]
impl WallpaperDownloader for EarthViewDownloader {
  /// Download a wallpaper from Google Earth View
  /// Note: Earth View ignores query parameters as it returns curated satellite imagery
  async fn download(&self, config: &Config, _query: &[String], opts: &DownloadOptions) -> Result<Wallpaper> {
    debug!("Fetching Earth View photo list");

    let client = Client::from(&config.advanced);

    // Step 1: Fetch the list of available photos
    let list_response = client
      .get(EARTHVIEW_LIST_URL)
      .send()
      .await
      .context("Failed to fetch Earth View photo list")?;

    if !list_response.status().is_success() {
      let status = list_response.status();
      return Err(anyhow!("Earth View list request failed with status: {}", status));
    }

    let photo_list: Vec<EarthViewListItem> = list_response.json().await.context("Failed to parse Earth View photo list")?;

    if photo_list.is_empty() {
      return Err(anyhow!("No photos available from Earth View"));
    }

    // Pick a random photo from the list
    let selected = photo_list
      .choose(&mut rand::thread_rng())
      .ok_or_else(|| anyhow!("Failed to select random Earth View photo"))?;

    debug!("Selected Earth View slug: {}", selected.slug);

    // Step 2: Fetch individual photo details
    let photo_url = format!("{}{}.json", EARTHVIEW_API_BASE, selected.slug);
    let photo_response = client.get(&photo_url).send().await.context("Failed to fetch Earth View photo details")?;

    if !photo_response.status().is_success() {
      let status = photo_response.status();
      return Err(anyhow!("Earth View photo details request failed with status: {}", status));
    }

    let photo: EarthViewPhoto = photo_response.json().await.context("Failed to parse Earth View photo details")?;

    // Normalize the image URL
    let image_url = if photo.photo_url.starts_with("http") {
      photo.photo_url.clone()
    } else {
      format!("https://{}", photo.photo_url)
    };

    debug!("Downloading Earth View image: {} - {}", photo.country, image_url);

    // Download the image
    let image_response = client.get(&image_url).send().await.context("Failed to download Earth View image")?;

    if !image_response.status().is_success() {
      return Err(anyhow!("Earth View image download failed with status: {}", image_response.status()));
    }

    let bytes = image_response.bytes().await.context("Failed to read Earth View image data")?;

    // Build descriptive filename
    let location = if photo.region != "-" && !photo.region.is_empty() {
      format!("{}, {}", photo.region, photo.country)
    } else {
      photo.country.clone()
    };

    let filename = format!(
      "{}_{}_{}.jpg",
      self.source_name(),
      location.replace(", ", "_").replace(' ', "-"),
      FilesystemHelper::make_file_suffix()
    );
    let download_dir = opts.output_dir.as_deref().unwrap_or(Path::new(&config.paths.downloads));
    let file_path = download_dir.join(&filename);

    // Ensure download directory exists
    if let Some(parent) = file_path.parent() {
      tokio::fs::create_dir_all(parent).await.context("Failed to create download directory")?;
    }

    tokio::fs::write(&file_path, &bytes).await.context("Failed to save Earth View image")?;

    let wallpaper = Wallpaper {
      file_path,
      downloaded_at: Utc::now(),
      source: self.source_name().to_string(),
    };

    debug!("Downloaded Earth View wallpaper: {:?}", wallpaper);
    Ok(wallpaper)
  }

  fn source_name(&self) -> &'static str {
    "earthview"
  }
}
