//! awww backend - refactored from existing wallpaper.rs implementation

use super::traits::{WallpaperBackend, WallpaperOptions, WallpaperScaling, MonitorSelection};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::process::Command as AsyncCommand;
use tracing::{debug, warn};

/// Backend for awww wallpaper setter with transition effects
pub struct AwwwBackend;

impl AwwwBackend {
    pub fn new() -> Self {
        Self
    }

    /// Convert wallpaper options to awww command arguments
    fn build_awww_command(&self, image_path: &Path, options: &WallpaperOptions) -> AsyncCommand {
        let mut cmd = AsyncCommand::new("awww");
        cmd.args(["img", &image_path.to_string_lossy()]);

        // Set transition type
        if let Some(transition) = &options.transition {
            cmd.args(["--transition-type", transition]);
        } else {
            cmd.args(["--transition-type", "random"]);
        }

        // Set FPS
        if let Some(fps) = options.fps {
            cmd.args(["--transition-fps", &fps.to_string()]);
        }

        // Set transition step (hardcoded for now, could be configurable)
        cmd.args(["--transition-step", "90"]);

        cmd
    }
}

#[async_trait]
impl WallpaperBackend for AwwwBackend {
    async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()> {
        let mut cmd = self.build_awww_command(image_path, options);

        debug!(
            "Running awww: {} with transition: {:?}, fps: {:?}",
            image_path.display(),
            options.transition,
            options.fps
        );

        let output = cmd.output()
            .await
            .context("Failed to execute awww command")?;

        if output.status.success() {
            debug!("✅ awww wallpaper set successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("awww failed: {}", stderr);
            Err(anyhow::anyhow!("awww command failed: {}", stderr))
        }
    }

    async fn get_current_wallpaper(&self) -> Result<Option<PathBuf>> {
        // awww doesn't support getting current wallpaper
        Ok(None)
    }

    fn is_available(&self) -> bool {
        which::which("awww").is_ok()
    }

    fn priority(&self) -> u32 {
        75 // High priority if available, but not highest
    }

    fn name(&self) -> &'static str {
        "awww"
    }

    fn supported_transitions(&self) -> Vec<String> {
        vec![
            "random".to_string(),
            "fade".to_string(),
            "slideIn".to_string(),
            "slideOut".to_string(),
            "wipe".to_string(),
            "grow".to_string(),
            "outer".to_string(),
            "any".to_string(),
        ]
    }

    fn validate(&self) -> Result<()> {
        if !self.is_available() {
            return Err(anyhow::anyhow!(
                "awww is not available. Install from: https://github.com/ldelossa/awww"
            ));
        }

        // Could add additional validation here (version check, etc.)
        Ok(())
    }
}

impl Default for AwwwBackend {
    fn default() -> Self {
        Self::new()
    }
}
