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

/// Set wallpaper from local collection
pub async fn set_local(config: &Config) -> Result<()> {
  let wallpaper_path = select_local_wallpaper(config)?;
  info!("Selected local wallpaper: {}", wallpaper_path.display());

  apply_wallpaper(&wallpaper_path, config).await?;
  Ok(())
}

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

/// Select random wallpaper from local collection
fn select_local_wallpaper(config: &Config) -> Result<PathBuf> {
  let wallpaper_dir = Path::new(&config.paths.local);

  if !wallpaper_dir.exists() {
    return Err(anyhow!("Local wallpaper directory does not exist: {}", wallpaper_dir.display()));
  }

  // Collect all image files
  let mut wallpapers = Vec::new();
  collect_wallpapers(
    wallpaper_dir,
    &config.sources.local.formats,
    &mut wallpapers,
    config.sources.local.recursive,
  )?;

  if wallpapers.is_empty() {
    return Err(anyhow!("No wallpapers found in: {}", wallpaper_dir.display()));
  }

  // Select random wallpaper
  let wallpaper = wallpapers
    .choose(&mut rand::thread_rng())
    .ok_or_else(|| anyhow!("Failed to select random wallpaper"))?;

  Ok(wallpaper.clone())
}

/// Recursively collect wallpaper files
fn collect_wallpapers(dir: &Path, formats: &[String], wallpapers: &mut Vec<PathBuf>, recursive: bool) -> Result<()> {
  let entries = std::fs::read_dir(dir).with_context(|| format!("Failed to read directory: {}", dir.display()))?;

  for entry in entries {
    let entry = entry?;
    let path = entry.path();

    if path.is_file() {
      if let Some(extension) = path.extension().and_then(|ext| ext.to_str())
        && formats.iter().any(|fmt| fmt.eq_ignore_ascii_case(extension))
      {
        wallpapers.push(path);
      }
    } else if path.is_dir() && recursive {
      collect_wallpapers(&path, formats, wallpapers, recursive)?;
    }
  }

  Ok(())
}

/// Apply wallpaper using awww (and optionally KDE)
async fn apply_wallpaper(wallpaper_path: &Path, config: &Config) -> Result<()> {
  // Get transition type
  let transition = match &config.transition.transition_type {
    TransitionType::Single(t) => t.clone(),
    TransitionType::Multiple(types) => types.choose(&mut rand::thread_rng()).unwrap_or(&"random".to_string()).clone(),
  };

  debug!("Applying wallpaper with transition: {} ({}s)", transition, config.transition.duration);

  // Set wallpaper with awww - THE CLEAN WAY! 🎉
  let awww_success = set_wallpaper_awww(wallpaper_path, &transition, config).await?;

  if awww_success {
    info!("✅ awww wallpaper set successfully");

    // Also set KDE wallpaper for panel inheritance
    set_wallpaper_kde(wallpaper_path).await;

    // Generate color scheme if enabled
    if config.integration.pywal.enabled {
      generate_pywal_colors(wallpaper_path, config).await;
    }
  } else {
    return Err(anyhow!("Failed to set wallpaper with awww"));
  }

  Ok(())
}

/// Set wallpaper using awww
async fn set_wallpaper_awww(wallpaper_path: &Path, transition: &str, config: &Config) -> Result<bool> {
  let mut cmd = AsyncCommand::new("awww");
  cmd.args([
    "img",
    &wallpaper_path.to_string_lossy(),
    "--transition-type",
    transition,
    "--transition-fps",
    &config.transition.fps.to_string(),
    "--transition-step",
    "90", // Could make this configurable
  ]);

  debug!(
    "Running: awww img {} --transition-type {} --transition-fps {}",
    wallpaper_path.display(),
    transition,
    config.transition.fps
  );

  let output = cmd.output().await.context("Failed to execute awww command")?;

  if output.status.success() {
    Ok(true)
  } else {
    let stderr = String::from_utf8_lossy(&output.stderr);
    warn!("awww failed: {}", stderr);
    Ok(false)
  }
}

/// Set KDE wallpaper for panel inheritance
async fn set_wallpaper_kde(wallpaper_path: &Path) {
  if which::which("plasma-apply-wallpaperimage").is_ok() {
    debug!("Setting KDE wallpaper for panel inheritance");

    let output = AsyncCommand::new("plasma-apply-wallpaperimage").arg(wallpaper_path).output().await;

    match output {
      Ok(output) if output.status.success() => {
        info!("✅ KDE wallpaper set successfully");
      }
      Ok(output) => {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Failed to set KDE wallpaper: {}", stderr);
      }
      Err(e) => {
        warn!("Failed to execute plasma-apply-wallpaperimage: {}", e);
      }
    }
  }
}

/// Generate pywal color scheme
async fn generate_pywal_colors(wallpaper_path: &Path, config: &Config) {
  debug!("Generating pywal color scheme");

  let mut cmd = AsyncCommand::new("wal");
  cmd.args(["-i", &wallpaper_path.to_string_lossy(), "-n"]);

  // Add flags to prevent terminal corruption (like we fixed in bash version)
  if !config.integration.desktop.update_terminal_colors {
    cmd.args(["-t", "-e"]);
  }

  let output = cmd.output().await;

  match output {
    Ok(output) if output.status.success() => {
      info!("✅ pywal color scheme generated");

      // Notify applications
      for app in &config.integration.pywal.notify_apps {
        notify_app_color_change(app).await;
      }
    }
    Ok(output) => {
      let stderr = String::from_utf8_lossy(&output.stderr);
      warn!("Failed to generate pywal colors: {}", stderr);
    }
    Err(e) => {
      debug!("pywal not available: {}", e);
    }
  }
}

/// Notify applications about color changes
async fn notify_app_color_change(app: &str) {
  match app {
    "neovim" => {
      // TODO: Implement neovim notification
      debug!("TODO: Notify neovim about color change");
    }
    "kitty" => {
      // TODO: Implement kitty notification
      debug!("TODO: Notify kitty about color change");
    }
    _ => {
      debug!("Unknown app for color notification: {}", app);
    }
  }
}
