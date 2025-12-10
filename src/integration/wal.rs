use crate::config::Config;
use std::path::Path;
use tokio::process::Command as AsyncCommand;
use tracing::{debug, warn};

/// Generate pywal color scheme
pub async fn generate_pywal_colors(wallpaper_path: &Path, config: &Config) {
  let mut cmd = AsyncCommand::new("wal");
  cmd.args(["-sni", &wallpaper_path.to_string_lossy()]);

  if config.integration.pywal.backend.is_some() {
    cmd.args(["--backend", config.integration.pywal.backend.as_ref().unwrap(), "--vte"]);
  }

  let output = cmd.output().await;

  match output {
    Ok(output) => {
      let stdout = String::from_utf8_lossy(&output.stdout);
      let stderr = String::from_utf8_lossy(&output.stderr);

      if output.status.success() {
        debug!("✅ pywal color scheme generated");
        if !stdout.is_empty() {
          debug!("stdout: {}", stdout);
        }

        // Notify Kitty to reload colors if enabled
        if config.integration.pywal.notify_kitty {
          notify_kitty().await;
        }
      } else {
        warn!("pywal failed: {}", stderr);
      }
    }
    Err(e) => {
      debug!("pywal not available: {}", e);
    }
  }
}

/// Send SIGUSR1 to all Kitty processes to trigger config reload
async fn notify_kitty() {
  let output = AsyncCommand::new("pkill").args(["-USR1", "kitty"]).output().await;

  match output {
    Ok(output) => {
      if output.status.success() {
        debug!("✅ Kitty notified to reload colors");
      } else {
        // pkill returns non-zero if no processes matched - that's fine
        debug!("No Kitty processes found to notify");
      }
    }
    Err(e) => {
      debug!("Could not notify Kitty: {}", e);
    }
  }
}
