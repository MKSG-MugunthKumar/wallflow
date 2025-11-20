use anyhow::{Context, Result, anyhow};
use rand::seq::SliceRandom;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn, debug};
use tokio::process::Command as AsyncCommand;

use crate::config::{Config, TransitionType};

/// Set wallpaper from local collection
pub async fn set_local(config: &Config, _category: Option<&str>) -> Result<()> {
    let wallpaper_path = select_local_wallpaper(config)?;
    info!("Selected local wallpaper: {}", wallpaper_path.display());

    apply_wallpaper(&wallpaper_path, config).await?;
    Ok(())
}

/// Download and set wallpaper from Wallhaven (placeholder)
pub async fn set_wallhaven(config: &Config, category: &str) -> Result<()> {
    info!("🚧 Wallhaven download - Learning opportunity!");
    info!("📚 See bin/wallflow-reference for URL examples");
    info!("🔗 Category: {}, Resolution: auto-detect", category);

    // TODO: Implement Wallhaven API integration
    // 1. Get resolution from config.get_wallhaven_resolution()?
    // 2. Build API URL (see bin/wallflow-reference for examples)
    // 3. Make HTTP request with reqwest
    // 4. Parse JSON response to get wallpaper URL
    // 5. Download image file to downloads directory
    // 6. Call apply_wallpaper() with downloaded file

    info!("💡 Use 'wallflow local' for now, or implement the download yourself!");
    Err(anyhow!("Wallhaven download not yet implemented - learning exercise!"))
}

/// Download random photo from Picsum (placeholder)
pub async fn set_picsum(config: &Config) -> Result<()> {
    info!("🚧 Picsum download - Learning opportunity!");
    info!("📚 See bin/wallflow-reference for URL examples");

    // TODO: Implement Picsum API integration
    // 1. Get resolution from config.get_picsum_resolution()?
    // 2. Build Picsum URL (very simple: https://picsum.photos/WIDTH/HEIGHT?random)
    // 3. Download image with reqwest (follows redirects automatically)
    // 4. Save to downloads directory with timestamp filename
    // 5. Call apply_wallpaper() with downloaded file

    info!("💡 Use 'wallflow local' for now, or implement the download yourself!");
    Err(anyhow!("Picsum download not yet implemented - learning exercise!"))
}

/// Select random wallpaper from local collection
fn select_local_wallpaper(config: &Config) -> Result<PathBuf> {
    let wallpaper_dir = Path::new(&config.paths.local);

    if !wallpaper_dir.exists() {
        return Err(anyhow!("Local wallpaper directory does not exist: {}", wallpaper_dir.display()));
    }

    // Collect all image files
    let mut wallpapers = Vec::new();
    collect_wallpapers(wallpaper_dir, &config.sources.local.formats, &mut wallpapers, config.sources.local.recursive)?;

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
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                if formats.iter().any(|fmt| fmt.eq_ignore_ascii_case(extension)) {
                    wallpapers.push(path);
                }
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
        TransitionType::Multiple(types) => {
            types.choose(&mut rand::thread_rng())
                .unwrap_or(&"random".to_string())
                .clone()
        }
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
        "--transition-type", transition,
        "--transition-fps", &config.transition.fps.to_string(),
        "--transition-step", "90",  // Could make this configurable
    ]);

    debug!("Running: awww img {} --transition-type {} --transition-fps {}",
        wallpaper_path.display(), transition, config.transition.fps);

    let output = cmd.output().await
        .context("Failed to execute awww command")?;

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
    if let Ok(_) = which::which("plasma-apply-wallpaperimage") {
        debug!("Setting KDE wallpaper for panel inheritance");

        let output = AsyncCommand::new("plasma-apply-wallpaperimage")
            .arg(wallpaper_path)
            .output()
            .await;

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

/// Check if a command exists in PATH
fn which(command: &str) -> Result<PathBuf> {
    which::which(command)
        .with_context(|| format!("Command '{}' not found in PATH", command))
}