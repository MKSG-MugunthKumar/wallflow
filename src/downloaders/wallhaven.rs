//! Wallhaven API downloader
//!
//! Downloads wallpapers from https://wallhaven.cc using their public API.
//! Supports filtering by resolution, purity, categories, and search terms.

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

/// Wallhaven API search response
#[derive(Debug, Deserialize)]
struct WallhavenResponse {
  data: Vec<WallhavenImage>,
}

/// Individual wallpaper from Wallhaven
#[derive(Debug, Deserialize)]
struct WallhavenImage {
  /// Direct URL to the full image
  path: String,
  /// Image resolution (e.g., "1920x1080")
  #[allow(dead_code)]
  resolution: String,
}

/// Wallhaven downloader implementation
pub struct WallhavenDownloader;

impl WallhavenDownloader {
  /// Convert purity array to API bitmask
  /// sfw=1, sketchy=1, nsfw=1 -> "111"
  fn purity_to_bitmask(purity: &[String]) -> String {
    let sfw = purity.iter().any(|p| p.eq_ignore_ascii_case("sfw"));
    let sketchy = purity.iter().any(|p| p.eq_ignore_ascii_case("sketchy"));
    let nsfw = purity.iter().any(|p| p.eq_ignore_ascii_case("nsfw"));

    // Default to SFW if nothing specified
    if !sfw && !sketchy && !nsfw {
      return "100".to_string();
    }

    format!("{}{}{}", sfw as u8, sketchy as u8, nsfw as u8)
  }

  /// Convert categories to API bitmask
  /// general=1, anime=1, people=1 -> "111"
  fn categories_to_bitmask(categories: &[String]) -> String {
    let general = categories.iter().any(|c| c.eq_ignore_ascii_case("general"));
    let anime = categories.iter().any(|c| c.eq_ignore_ascii_case("anime"));
    let people = categories.iter().any(|c| c.eq_ignore_ascii_case("people"));

    // Default to general if nothing specified
    if !general && !anime && !people {
      return "100".to_string();
    }

    format!("{}{}{}", general as u8, anime as u8, people as u8)
  }

  /// Build search query from categories (non-bitmask terms become search tags)
  fn build_search_query(config_q: &str, categories: &[String]) -> String {
    let mut terms: Vec<String> = Vec::new();

    // Add explicit query if provided
    if !config_q.is_empty() {
      terms.push(config_q.to_string());
    }

    // Add category terms that aren't the API category types
    let api_categories = ["general", "anime", "people"];
    for cat in categories {
      if !api_categories.iter().any(|c| cat.eq_ignore_ascii_case(c)) {
        terms.push(cat.clone());
      }
    }

    terms.join(" ")
  }
}

#[async_trait]
impl WallpaperDownloader for WallhavenDownloader {
  /// Download a wallpaper from Wallhaven
  /// Query parameters are used as search terms (e.g., "wallflow wallhaven nature mountains")
  async fn download(&self, config: &Config, query: &[String], opts: &DownloadOptions) -> Result<Wallpaper> {
    let wallhaven_config = &config.sources.wallhaven;
    let resolution = config.get_wallhaven_resolution()?;

    // Build query parameters
    let purity = Self::purity_to_bitmask(&wallhaven_config.purity);
    let categories = Self::categories_to_bitmask(&wallhaven_config.categories);

    // CLI query takes precedence over config query
    let cli_query = query.join(" ");
    let base_query = if cli_query.is_empty() { &wallhaven_config.q } else { &cli_query };
    let search_query = Self::build_search_query(base_query, &wallhaven_config.categories);
    let resolution_str = format!("{}x{}", resolution.width, resolution.height);

    debug!(
      "Searching Wallhaven: q='{}', resolution={}, purity={}, categories={}",
      search_query, resolution_str, purity, categories
    );

    let client = Client::from(&config.advanced);

    // Build request with query parameters
    let mut request = client.get(&wallhaven_config.url).query(&[
      ("purity", purity.as_str()),
      ("categories", categories.as_str()),
      ("atleast", resolution_str.as_str()),
      ("sorting", "random"),
    ]);

    // Add search query if not empty
    if !search_query.is_empty() {
      request = request.query(&[("q", search_query.as_str())]);
    }

    // Add API key if provided (for higher rate limits)
    if let Some(api_key) = &wallhaven_config.api_key
      && !api_key.is_empty()
    {
      request = request.query(&[("apikey", api_key.as_str())]);
    }

    let response = request.send().await.context("Failed to send request to Wallhaven API")?;

    if !response.status().is_success() {
      let status = response.status();
      let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
      return Err(anyhow!("Wallhaven API request failed with status {}: {}", status, error_text));
    }

    let wallhaven_data: WallhavenResponse = response.json().await.context("Failed to parse Wallhaven API response")?;

    if wallhaven_data.data.is_empty() {
      return Err(anyhow!(
        "No wallpapers found matching criteria (resolution: {}, purity: {}, categories: {})",
        resolution_str,
        purity,
        categories
      ));
    }

    // Pick a random wallpaper from results
    let image = wallhaven_data
      .data
      .choose(&mut rand::thread_rng())
      .ok_or_else(|| anyhow!("Failed to select random wallpaper"))?;

    debug!("Selected wallpaper: {}", image.path);

    // Download the actual image
    let image_response = client.get(&image.path).send().await.context("Failed to download wallpaper image")?;

    if !image_response.status().is_success() {
      return Err(anyhow!("Image download failed with status: {}", image_response.status()));
    }

    let bytes = image_response.bytes().await.context("Failed to read image data")?;

    // Extract file extension from URL
    let file_extension = image
      .path
      .rsplit('.')
      .next()
      .and_then(|ext| {
        let ext = ext.split('?').next().unwrap_or(ext);
        if ext.len() <= 5 { Some(ext) } else { None }
      })
      .unwrap_or("jpg");

    let filename = format!("{}_{}", self.source_name(), FilesystemHelper::make_file_suffix());
    let download_dir = opts.output_dir.as_deref().unwrap_or(Path::new(&config.paths.downloads));
    let file_path = download_dir.join(&filename).with_extension(file_extension);

    // Ensure download directory exists
    if let Some(parent) = file_path.parent() {
      tokio::fs::create_dir_all(parent).await.context("Failed to create download directory")?;
    }

    tokio::fs::write(&file_path, &bytes).await.context("Failed to save wallpaper image")?;

    let wallpaper = Wallpaper {
      file_path,
      downloaded_at: Utc::now(),
      source: self.source_name().to_string(),
    };

    debug!("Downloaded Wallhaven wallpaper: {:?}", wallpaper);
    Ok(wallpaper)
  }

  fn source_name(&self) -> &'static str {
    "wallhaven"
  }
}
