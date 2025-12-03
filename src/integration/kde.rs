use std::path::Path;
use tokio::process::Command as AsyncCommand;
use tracing::{debug, warn};

/// Check if KDE Plasma is available by looking for plasma-apply-wallpaperimage
fn is_kde_available() -> bool {
  which::which("plasma-apply-wallpaperimage").is_ok()
}

/// Set KDE Plasma wallpaper using plasma-apply-wallpaperimage
///
/// This ensures that KDE applications inherit the wallpaper colors correctly.
/// The function is called after the main wallpaper backend sets the wallpaper,
/// so that both the compositor and KDE are in sync.
pub async fn set_kde_wallpaper(wallpaper_path: &Path) {
  if !is_kde_available() {
    debug!("plasma-apply-wallpaperimage not available, skipping KDE integration");
    return;
  }

  let output = AsyncCommand::new("plasma-apply-wallpaperimage").arg(wallpaper_path).output().await;

  match output {
    Ok(output) => {
      if output.status.success() {
        debug!("KDE Plasma wallpaper set successfully");
      } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Failed to set KDE Plasma wallpaper: {}", stderr);
      }
    }
    Err(e) => {
      debug!("Failed to execute plasma-apply-wallpaperimage: {}", e);
    }
  }
}
