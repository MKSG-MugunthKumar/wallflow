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
  apply_wallpaper_with_options(wallpaper_path, config, false).await
}

/// Set wallpaper with fire-and-forget option (for daemon mode)
pub async fn apply_wallpaper_daemon(wallpaper_path: &Path, config: &Config) -> Result<()> {
  apply_wallpaper_with_options(wallpaper_path, config, true).await
}

/// Internal function that handles both CLI and daemon modes
async fn apply_wallpaper_with_options(wallpaper_path: &Path, config: &Config, fire_and_forget: bool) -> Result<()> {
  debug!(
    "apply_wallpaper_with_options: path={}, fire_and_forget={}",
    wallpaper_path.display(),
    fire_and_forget
  );

  let registry = BackendRegistry::new();

  debug!("Available backends: {:?}", registry.available_backend_names());

  let backend = registry.get_best_backend().context("No wallpaper backends available")?;

  let options = build_wallpaper_options(config, fire_and_forget);

  debug!(
    "Applying wallpaper with {}, options: transition={:?}, scaling={:?}, fire_and_forget={}",
    backend.name(),
    options.transition,
    options.scaling,
    options.fire_and_forget
  );

  debug!("Calling {}.set_wallpaper({})", backend.name(), wallpaper_path.display());

  backend
    .set_wallpaper(wallpaper_path, &options)
    .await
    .context("Failed to apply wallpaper")?;

  // Set KDE Plasma wallpaper if available (ensures KDE apps inherit colors)
  integration::set_kde_wallpaper(wallpaper_path).await;

  // Color theming pipeline
  if config.colors.enabled {
    apply_color_theme(wallpaper_path, config);
  }

  info!("✅ Wallpaper {} applied successfully using {}", wallpaper_path.display(), backend.name());

  Ok(())
}

/// Apply color theme after wallpaper is set.
/// Runs native k-means++ extraction and renders templates.
fn apply_color_theme(wallpaper_path: &Path, config: &Config) {
  match config.colors.engine.as_str() {
    "native" => {
      let options = crate::colors::ExtractionOptions {
        contrast_ratio: config.colors.contrast_ratio,
        background_intensity: config.colors.background_intensity,
        prefers_dark: config.colors.prefer_dark.or_else(crate::platform::detect_dark_mode),
        ..Default::default()
      };

      let extractor = crate::colors::ColorExtractor::new();
      match extractor.extract(wallpaper_path, &options) {
        Ok(scheme) => {
          let output_dir = crate::templates::TemplateEngine::default_output_dir();
          if let Err(e) = std::fs::create_dir_all(&output_dir) {
            tracing::warn!("Failed to create output dir: {}", e);
            return;
          }

          // Save color scheme JSON
          match scheme.to_json() {
            Ok(json) => {
              let scheme_file = output_dir.join("colors.json");
              if let Err(e) = std::fs::write(&scheme_file, &json) {
                tracing::warn!("Failed to write colors.json: {}", e);
              } else {
                debug!("Color scheme saved to {}", scheme_file.display());
              }
            }
            Err(e) => tracing::warn!("Failed to serialize color scheme: {}", e),
          }

          // Render templates if available
          let tpl_dir = crate::templates::templates_dir();
          if tpl_dir.exists() {
            match crate::templates::TemplateEngine::render_all(&tpl_dir, &output_dir, &scheme) {
              Ok(rendered) => {
                if !rendered.is_empty() {
                  debug!("Rendered {} templates", rendered.len());
                  if config.integration.reload_apps {
                    crate::templates::TemplateEngine::notify_apps(&rendered);
                  }
                }
              }
              Err(e) => tracing::warn!("Failed to render templates: {}", e),
            }
          }
        }
        Err(e) => {
          tracing::warn!("Color extraction failed: {}", e);
        }
      }
    }
    other => {
      tracing::warn!("Unknown colors engine '{}', skipping", other);
    }
  }
}

/// Build wallpaper options from configuration
fn build_wallpaper_options(config: &Config, fire_and_forget: bool) -> WallpaperOptions {
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
    fire_and_forget,
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

/// Download and set wallpaper from any registered source
/// The `query` parameter contains additional arguments (e.g., search terms, subreddit names)
pub async fn set_from_source(config: &Config, source: &str, query: &[String], opts: &crate::downloaders::DownloadOptions) -> Result<()> {
  info!("Downloading wallpaper from {}", source);
  let wallpaper = crate::downloaders::download_from_source(source, config, query, opts).await?;
  debug!("Downloaded: {:?}", wallpaper);

  if opts.no_set {
    // Just print the path for the caller to use
    println!("{}", wallpaper.file_path.display());
  } else {
    apply_wallpaper(&wallpaper.file_path, config).await?;
  }

  Ok(())
}

/// Download and set wallpaper from source (daemon mode - fire and forget)
pub async fn set_from_source_daemon(config: &Config, source: &str, query: &[String], opts: &crate::downloaders::DownloadOptions) -> Result<()> {
  info!("Downloading wallpaper from {}", source);
  let wallpaper = crate::downloaders::download_from_source(source, config, query, opts).await?;
  debug!("Downloaded: {:?}", wallpaper);

  if opts.no_set {
    println!("{}", wallpaper.file_path.display());
  } else {
    apply_wallpaper_daemon(&wallpaper.file_path, config).await?;
  }

  Ok(())
}

/// Set wallpaper from local collection
pub async fn set_local(config: &Config) -> Result<()> {
  let wallpaper_path = select_local_wallpaper(config)?;
  apply_wallpaper(&wallpaper_path, config).await?;
  Ok(())
}

/// Set wallpaper from local collection (daemon mode - fire and forget)
pub async fn set_local_daemon(config: &Config) -> Result<()> {
  let wallpaper_path = select_local_wallpaper(config)?;
  apply_wallpaper_daemon(&wallpaper_path, config).await?;
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
