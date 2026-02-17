//! awww backend - refactored from existing wallpaper.rs implementation

use super::traits::{WallpaperBackend, WallpaperOptions};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command as AsyncCommand;
use tracing::{debug, warn};

/// Backend for awww wallpaper setter with transition effects
#[derive(Default)]
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
      cmd.args(["--transition-step", &fps.to_string()]);
    }

    // Set duration
    if let Some(duration) = options.duration {
      cmd.args(["--transition-duration", &duration.to_string()]);
    }

    cmd
  }
}

#[async_trait]
impl WallpaperBackend for AwwwBackend {
  async fn set_wallpaper(&self, image_path: &Path, options: &WallpaperOptions) -> Result<()> {
    let mut cmd = self.build_awww_command(image_path, options);

    let awww_path = which::which("awww")
      .map(|p| p.display().to_string())
      .unwrap_or_else(|_| "not found".to_string());
    debug!(
      "Running awww ({}): {} with transition: {:?}, fps: {:?}, fire_and_forget: {}",
      awww_path,
      image_path.display(),
      options.transition,
      options.fps,
      options.fire_and_forget
    );

    // Log the full command for debugging
    debug!("awww command: {:?}", cmd.as_std());

    // Suppress awww's TTY output (progress animations)
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped()); // Keep stderr for error reporting

    if options.fire_and_forget {
      // Spawn without waiting - useful for daemon mode to avoid blocking during transitions
      match cmd.spawn() {
        Ok(child) => {
          debug!("awww spawned (fire-and-forget mode), pid: {:?}", child.id());
          Ok(())
        }
        Err(e) => {
          warn!("Failed to spawn awww command: {}", e);
          Err(anyhow::anyhow!("Failed to spawn awww command: {}", e))
        }
      }
    } else {
      // Wait for completion - useful for CLI to report success/failure
      let output = cmd.output().await.context("Failed to execute awww command")?;

      debug!("awww exit status: {:?}", output.status);

      if output.status.success() {
        debug!("awww wallpaper set successfully");
        Ok(())
      } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("awww failed (exit {}): {}", output.status, stderr);
        Err(anyhow::anyhow!("awww command failed: {}", stderr))
      }
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
      return Err(anyhow::anyhow!("awww is not available. Install from: https://codeberg.org/LGFae/awww"));
    }

    // Could add additional validation here (version check, etc.)
    Ok(())
  }
}
