use crate::config::{Config, TransitionType};
use anyhow::{Context, Result, anyhow};
use chrono::Local;
use rand::seq::SliceRandom;
use reqwest::Client;
use serde::Deserialize;
use std::fs::File;
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};
use tokio::process::Command as AsyncCommand;
use tracing::{debug, info, warn};

#[derive(Deserialize)]
struct WallhavenResponse {
  data: Vec<WallhavenImage>,
}

#[derive(Deserialize)]
struct WallhavenImage {
  #[allow(dead_code)]
  id: String,
  #[allow(dead_code)]
  url: String,
  #[allow(dead_code)]
  short_url: String,
  path: String,
}

/// Download the actual url
pub async fn get_url_content_as_json(url: &str) -> Result<String> {
  let client = Client::new();
  let response = client.get(url).send().await?;

  if !response.status().is_success() {
    return Err(anyhow!("Request failed: {}", response.status()));
  }

  let wallhaven_response: WallhavenResponse = response.json().await?;

  if wallhaven_response.data.is_empty() {
    return Err(anyhow!("No wallpapers found for this search"));
  }

  Ok(wallhaven_response.data[0].path.clone())
}

pub async fn save_url_content_to_file(url: &str, file_name: &str) -> Result<()> {
  let client = Client::new();
  let response = client.get(url).send().await?;

  if !response.status().is_success() {
    return Err(anyhow!("Request failed: {}", response.status()));
  }

  let bytes = response.bytes().await?;
  let mut file = File::create(file_name)?;
  let mut content = Cursor::new(bytes);
  io::copy(&mut content, &mut file)?;

  Ok(())
}

pub fn make_file_suffix() -> String {
  Local::now().format("%Y%m%d_%H%M%S").to_string()
}

/// Download and set wallpaper from Wallhaven (placeholder)
pub async fn set_wallhaven(config: &Config, category: &str) -> Result<()> {
  let resolution = config.get_wallhaven_resolution()?;
  let api_url = format!(
    "https://wallhaven.cc/api/v1/search?q={category}&categories=100&purity=100&resolutions={}&sorting=random",
    resolution.as_string()
  );

  let image_url = get_url_content_as_json(&api_url).await?;
  let suffix = make_file_suffix();
  let filename = format!("{}/wallhaven_{category}_{suffix}.jpg", config.paths.downloads);

  save_url_content_to_file(&image_url, &filename).await?;
  apply_wallpaper(Path::new(&filename), config).await?;
  Ok(())
}

/// Download random photo from Picsum (placeholder)
pub async fn set_picsum(config: &Config) -> Result<()> {
  let r = config.get_picsum_resolution()?;

  let image_content_url = format!("https://picsum.photos/{}/{}?random", r.width, r.height);
  let suffix = make_file_suffix();
  let filename = format!("{}/picsum_{}.jpg", config.paths.downloads, suffix);

  save_url_content_to_file(&image_content_url, &filename).await?;
  apply_wallpaper(Path::new(&filename), config).await?;
  Ok(())
}
