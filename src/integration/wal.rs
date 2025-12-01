use crate::config::Config;
use std::path::Path;
use tokio::process::Command as AsyncCommand;
use tracing::{debug, warn};

/// Generate pywal color scheme
pub async fn generate_pywal_colors(wallpaper_path: &Path, config: &Config) {
  let mut cmd = AsyncCommand::new("wal");
  cmd.args(["-ni", &wallpaper_path.to_string_lossy()]);

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
      } else {
        warn!("pywal failed: {}", stderr);
      }
    }
    Err(e) => {
      debug!("pywal not available: {}", e);
    }
  }
}
