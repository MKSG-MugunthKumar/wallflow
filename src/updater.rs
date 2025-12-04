//! Self-update functionality for wallflow
//!
//! Downloads and installs updates from GitHub releases.

use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::fs;
use std::io::Write;
use tracing::info;

const REPO_OWNER: &str = "MKSG-MugunthKumar";
const REPO_NAME: &str = "wallflow";

#[derive(Deserialize)]
struct GitHubRelease {
  tag_name: String,
  assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
  name: String,
  browser_download_url: String,
}

/// Version check result
pub struct UpdateCheck {
  pub current: String,
  pub latest: String,
  pub update_available: bool,
}

/// Check if a new version is available on GitHub
pub async fn check_for_updates() -> Result<UpdateCheck> {
  let current_version = env!("CARGO_PKG_VERSION");

  info!("Current version: {}", current_version);
  info!("Checking for updates from GitHub...");

  let url = format!("https://api.github.com/repos/{}/{}/releases/latest", REPO_OWNER, REPO_NAME);

  let client = reqwest::Client::builder()
    .user_agent("wallflow-update-checker")
    .timeout(std::time::Duration::from_secs(10))
    .build()?;

  let response = client.get(&url).send().await?;

  if !response.status().is_success() {
    return Err(anyhow!("GitHub API returned status: {}", response.status()));
  }

  let release: GitHubRelease = response.json().await?;
  let latest_version = release.tag_name.trim_start_matches('v').to_string();

  info!("Latest version available: {}", latest_version);

  let update_available = latest_version != current_version;

  if update_available {
    info!("New version available: {} -> {}", current_version, latest_version);
  } else {
    info!("Already on latest version");
  }

  Ok(UpdateCheck {
    current: current_version.to_string(),
    latest: latest_version,
    update_available,
  })
}

/// Perform the self-update process
pub async fn perform_update() -> Result<String> {
  info!("Starting self-update process...");

  let url = format!("https://api.github.com/repos/{}/{}/releases/latest", REPO_OWNER, REPO_NAME);

  let client = reqwest::Client::builder()
    .user_agent("wallflow-update-checker")
    .timeout(std::time::Duration::from_secs(120))
    .build()?;

  let response = client.get(&url).send().await?;
  let release: GitHubRelease = response.json().await?;

  // Determine the asset name based on platform
  let asset_name = get_asset_name();

  let asset = release
    .assets
    .iter()
    .find(|a| a.name == asset_name || a.name == "wallflow")
    .ok_or_else(|| anyhow!("No suitable binary found in release (looking for '{}')", asset_name))?;

  info!("Downloading update from: {}", asset.browser_download_url);
  println!("Downloading {}...", asset.name);

  // Download the new binary
  let binary_response = client.get(&asset.browser_download_url).send().await?;

  let total_size = binary_response.content_length();
  let binary_data = binary_response.bytes().await?;

  if let Some(size) = total_size {
    println!("Downloaded {} bytes", size);
  }

  // Get current executable path
  let current_exe = std::env::current_exe()?;
  let temp_new = current_exe.with_extension("new");

  // Write new binary to temp location
  let mut file = fs::File::create(&temp_new)?;
  file.write_all(&binary_data)?;
  drop(file);

  // Make executable on Unix
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(&temp_new)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&temp_new, perms)?;
  }

  // Create update script that will run after we exit
  let script_path = current_exe.with_extension("update.sh");
  let script_content = format!(
    r#"#!/bin/bash
sleep 1
mv "{current}" "{current}.bak"
mv "{new}" "{current}"
chmod +x "{current}"
rm "{current}.bak" 2>/dev/null
rm -- "$0"
echo "Update complete! Run 'wallflow --version' to verify."
"#,
    current = current_exe.display(),
    new = temp_new.display()
  );

  let mut script_file = fs::File::create(&script_path)?;
  script_file.write_all(script_content.as_bytes())?;
  drop(script_file);

  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(&script_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms)?;
  }

  info!("Update prepared successfully");
  Ok(release.tag_name.trim_start_matches('v').to_string())
}

/// Get the expected asset name for the current platform
fn get_asset_name() -> String {
  #[cfg(target_os = "linux")]
  {
    #[cfg(target_arch = "x86_64")]
    return "wallflow-x86_64-unknown-linux-gnu".to_string();
    #[cfg(target_arch = "aarch64")]
    return "wallflow-aarch64-unknown-linux-gnu".to_string();
  }

  #[cfg(target_os = "macos")]
  {
    #[cfg(target_arch = "x86_64")]
    return "wallflow-x86_64-apple-darwin".to_string();
    #[cfg(target_arch = "aarch64")]
    return "wallflow-aarch64-apple-darwin".to_string();
  }

  #[cfg(target_os = "windows")]
  return "wallflow-x86_64-pc-windows-msvc.exe".to_string();

  #[allow(unreachable_code)]
  "wallflow".to_string()
}

/// Check if the app can update itself (not installed via package manager)
pub fn can_self_update() -> bool {
  if let Ok(exe_path) = std::env::current_exe() {
    let path_str = exe_path.to_string_lossy();

    // If installed via package manager locations, disable self-update
    if path_str.starts_with("/usr/bin")
      || path_str.starts_with("/usr/local/bin")
      || path_str.starts_with("/snap")
      || path_str.starts_with("/flatpak")
      || path_str.starts_with("/nix")
    {
      info!("Self-update disabled: installed via package manager");
      return false;
    }

    // Check if we have write permission to the executable
    if let Ok(metadata) = std::fs::metadata(&exe_path)
      && metadata.permissions().readonly()
    {
      info!("Self-update disabled: no write permission");
      return false;
    }
  }

  true
}

/// Apply the update by running the update script and exiting
pub fn apply_update() -> Result<()> {
  let exe_path = std::env::current_exe()?;
  let script_path = exe_path.with_extension("update.sh");

  if !script_path.exists() {
    return Err(anyhow!("Update script not found. Run 'wallflow update' first."));
  }

  info!("Executing update script and exiting: {:?}", script_path);
  println!("Applying update...");

  // Launch the update script in the background
  std::process::Command::new("sh").arg(&script_path).spawn()?;

  // Exit current process so the script can replace the binary
  std::process::exit(0);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_can_self_update() {
    let can_update = can_self_update();
    println!("Can self-update: {}", can_update);
  }

  #[test]
  fn test_get_asset_name() {
    let name = get_asset_name();
    println!("Asset name for this platform: {}", name);
    assert!(!name.is_empty());
  }
}
