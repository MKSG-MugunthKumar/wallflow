//! Wallpaper management module with cross-platform backend support

pub mod backends;

use crate::config::Config;
use crate::integration;
use anyhow::{Context, Result};
use backends::{
  BackendRegistry,
  traits::{MonitorSelection, WallpaperOptions, WallpaperScaling},
};
use rand::seq::SliceRandom;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Set wallpaper using the best available backend
pub async fn apply_wallpaper(wallpaper_path: &Path, config: &Config) -> Result<()> {
  let registry = BackendRegistry::new();
  let backend = registry.get_best_backend().context("No wallpaper backends available")?;

  let options = build_wallpaper_options(config);

  debug!(
    "Applying wallpaper with {}, options: transition={:?}, scaling={:?}",
    backend.name(),
    options.transition,
    options.scaling
  );

  backend
    .set_wallpaper(wallpaper_path, &options)
    .await
    .context("Failed to apply wallpaper")?;

  // Set KDE Plasma wallpaper if available (ensures KDE apps inherit colors)
  integration::set_kde_wallpaper(wallpaper_path).await;

  if config.integration.pywal.enabled {
    integration::generate_pywal_colors(wallpaper_path, config).await
  }
  info!("✅ Wallpaper {} applied successfully using {}", wallpaper_path.display(), backend.name());

  Ok(())
}

/// Build wallpaper options from configuration
fn build_wallpaper_options(config: &Config) -> WallpaperOptions {
  let transition = match &config.transition.transition_type {
    crate::config::TransitionType::Single(t) => Some(t.clone()),
    crate::config::TransitionType::Multiple(types) => {
      // Pick a random transition from the list
      use rand::seq::SliceRandom;
      types.choose(&mut rand::thread_rng()).cloned()
    }
  };

  WallpaperOptions {
    transition,
    duration: Some(config.transition.duration),
    fps: Some(config.transition.fps),
    scaling: WallpaperScaling::Fill, // Default for now, could be configurable
    monitor: MonitorSelection::All,
  }
}

/// List all available wallpaper backends
pub fn list_backends() -> Vec<String> {
  let registry = BackendRegistry::new();
  registry.list_backends()
}

/// Get information about the current platform and available backends
pub fn platform_info() -> Result<String> {
  use crate::platform::{check_platform_dependencies, detect_platform};

  let platform = detect_platform().context("Failed to detect platform")?;

  let status = check_platform_dependencies();

  let registry = BackendRegistry::new();
  let available_backends = registry.available_backend_names();

  let mut info = String::new();
  info.push_str(&format!("Platform: {}\n", platform));
  info.push_str(&format!("Status: {:?}\n", status));
  info.push_str(&format!("Available backends: {:?}\n", available_backends));

  if let Ok(best_backend) = registry.get_best_backend() {
    info.push_str(&format!("Best backend: {}\n", best_backend.name()));
    info.push_str(&format!("Supported transitions: {:?}\n", best_backend.supported_transitions()));
  } else {
    info.push_str("No working backends found\n");
  }

  Ok(info)
}

//
// Legacy API compatibility functions (preserve existing CLI behavior)
//

/// Set wallpaper from local collection (legacy API)
pub async fn set_local(config: &Config) -> Result<()> {
  let wallpaper_path = select_local_wallpaper(config)?;
  apply_wallpaper(&wallpaper_path, config).await?;
  Ok(())
}

/// Download and set wallpaper from Wallhaven (legacy API)
pub async fn set_wallhaven(_config: &Config) -> Result<()> {
  // This will be implemented with the new downloader system later
  // For now, preserve the existing behavior from wallpaper.rs
  todo!("Wallhaven implementation will be migrated to new downloader system")
}

/// Download random photo from Picsum (legacy API)
pub async fn set_picsum(_config: &Config) -> Result<()> {
  // This will be implemented with the new downloader system later
  // For now, preserve the existing behavior from wallpaper.rs
  todo!("Picsum implementation will be migrated to new downloader system")
}

/// Download and set wallpaper from NASA APOD (new downloader system)
pub async fn set_apod(config: &Config) -> Result<()> {
  info!("Downloading wallpaper from NASA APOD");
  let wallpaper = crate::downloaders::download_from_source("apod", config).await?;
  debug!("Downloaded: {:?}", wallpaper);
  apply_wallpaper(&wallpaper.file_path, config).await?;
  Ok(())
}

/// Select random wallpaper from local collection
fn select_local_wallpaper(config: &Config) -> Result<PathBuf> {
  let wallpaper_dir = Path::new(&config.paths.local);

  if !wallpaper_dir.exists() {
    return Err(anyhow::anyhow!("Local wallpaper directory does not exist: {}", wallpaper_dir.display()));
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
    return Err(anyhow::anyhow!("No wallpapers found in: {}", wallpaper_dir.display()));
  }

  // Select random wallpaper
  let wallpaper = wallpapers
    .choose(&mut rand::thread_rng())
    .ok_or_else(|| anyhow::anyhow!("Failed to select random wallpaper"))?;

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
