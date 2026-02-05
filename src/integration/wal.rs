use crate::config::Config;
use std::path::Path;
use tokio::process::Command as AsyncCommand;
use tracing::{debug, warn};

/// Generate pywal color scheme
pub async fn generate_pywal_colors(wallpaper_path: &Path, config: &Config) {
  let mut cmd = AsyncCommand::new("wal");
  cmd.args(["-sni", &wallpaper_path.to_string_lossy()]);

  if let Some(backend) = &config.integration.pywal.backend {
    cmd.args(["--backend", backend, "--vte"]);
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

        // Notify apps to reload colors
        // Use reload_apps (new) or notify_kitty (deprecated, backward compat)
        if config.integration.reload_apps || config.integration.pywal.notify_kitty {
          notify_apps().await;
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

/// Send signals to terminal apps to trigger config reload
async fn notify_apps() {
  // Kitty: SIGUSR1
  notify_app("kitty", "USR1").await;

  // Ghostty: SIGUSR2
  notify_app("ghostty", "USR2").await;

  // Restore terminal state after signals (prevents prompt corruption)
  let _ = AsyncCommand::new("stty").arg("sane").output().await;
}

/// Send a signal to a process by name
async fn notify_app(process_name: &str, signal: &str) {
  let output = AsyncCommand::new("pkill").args([&format!("-{}", signal), process_name]).output().await;

  match output {
    Ok(output) => {
      if output.status.success() {
        debug!("✅ {} notified to reload colors (SIG{})", process_name, signal);
      } else {
        // pkill returns non-zero if no processes matched - that's fine
        debug!("No {} processes found to notify", process_name);
      }
    }
    Err(e) => {
      debug!("Could not notify {}: {}", process_name, e);
    }
  }
}
