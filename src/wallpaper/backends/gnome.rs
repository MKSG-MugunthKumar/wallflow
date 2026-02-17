use super::traits::{WallpaperBackend, WallpaperOptions};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::process::Command as AsyncCommand;
use tracing::debug;

/// GNOME backend using gsettings to set wallpaper
pub struct GnomeBackend;

impl GnomeBackend {
    pub fn new() -> Self {
        Self
    }

    /// Detect if running under a GNOME-based desktop
    fn is_gnome_desktop() -> bool {
        std::env::var("XDG_CURRENT_DESKTOP")
            .map(|d| {
                let d = d.to_lowercase();
                d.contains("gnome") || d.contains("unity") || d.contains("budgie")
            })
            .unwrap_or(false)
    }
}

#[async_trait]
impl WallpaperBackend for GnomeBackend {
    async fn set_wallpaper(&self, image_path: &Path, _options: &WallpaperOptions) -> Result<()> {
        let uri = format!("file://{}", image_path.canonicalize()?.display());

        debug!("Setting GNOME wallpaper via gsettings: {}", uri);

        // Set for light mode
        let output = AsyncCommand::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.background",
                "picture-uri",
                &uri,
            ])
            .output()
            .await
            .context("Failed to execute gsettings")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("gsettings failed (picture-uri): {}", stderr));
        }

        // Set for dark mode
        let output = AsyncCommand::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.background",
                "picture-uri-dark",
                &uri,
            ])
            .output()
            .await
            .context("Failed to execute gsettings for dark mode")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!("gsettings picture-uri-dark failed (may not be supported): {}", stderr);
            // Not fatal — older GNOME versions don't have picture-uri-dark
        }

        debug!("GNOME wallpaper set successfully via gsettings");
        Ok(())
    }

    async fn get_current_wallpaper(&self) -> Result<Option<PathBuf>> {
        let output = AsyncCommand::new("gsettings")
            .args(["get", "org.gnome.desktop.background", "picture-uri"])
            .output()
            .await
            .context("Failed to get current wallpaper")?;

        if output.status.success() {
            let uri = String::from_utf8_lossy(&output.stdout).trim().to_string();
            // Strip quotes and file:// prefix
            let path = uri
                .trim_matches('\'')
                .strip_prefix("file://")
                .unwrap_or(&uri);
            Ok(Some(PathBuf::from(path)))
        } else {
            Ok(None)
        }
    }

    fn is_available(&self) -> bool {
        Self::is_gnome_desktop() && which::which("gsettings").is_ok()
    }

    fn priority(&self) -> u32 {
        80 // Higher than awww (75) for GNOME since it's native
    }

    fn name(&self) -> &'static str {
        "gnome-gsettings"
    }

    fn supported_transitions(&self) -> Vec<String> {
        vec![] // gsettings doesn't support transitions
    }
}

impl Default for GnomeBackend {
    fn default() -> Self {
        Self::new()
    }
}
