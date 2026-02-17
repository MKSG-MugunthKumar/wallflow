//! Template downloading from GitHub releases
//!
//! Downloads wallflow-templates from the GitHub releases tarball on first use.
//! Stores templates in `~/.config/mksg/wallflow/templates/`.
//! Tracks version in a `.version` file to know when to update.
//! Never overwrites user-modified templates.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

const TEMPLATES_REPO: &str = "MKSG-MugunthKumar/wallflow-templates";
const TEMPLATES_VERSION: &str = "v1.0.0";

/// Get the templates directory path
///
/// `~/.config/mksg/wallflow/templates/`
pub fn templates_dir() -> PathBuf {
  dirs::config_dir()
    .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".config"))
    .join("mksg")
    .join("wallflow")
    .join("templates")
}

/// Ensure templates are available locally.
/// Downloads from GitHub releases on first use or when a new version is available.
/// Returns the path to the templates directory.
pub async fn ensure_templates() -> Result<PathBuf> {
  let dir = templates_dir();
  let version_file = dir.join(".version");

  // Check if we already have the current version
  if version_file.exists()
    && let Ok(current_version) = fs::read_to_string(&version_file)
    && current_version.trim() == TEMPLATES_VERSION
  {
    debug!("Templates already at {}", TEMPLATES_VERSION);
    return Ok(dir);
  }

  // Need to download or update
  info!("Downloading wallflow templates {}...", TEMPLATES_VERSION);
  match download_templates(&dir).await {
    Ok(()) => {
      // Write version marker
      fs::write(&version_file, TEMPLATES_VERSION).context("Failed to write templates version file")?;
      info!("Templates downloaded to {}", dir.display());
    }
    Err(e) => {
      // If templates dir already has some templates, gracefully continue
      if dir.exists() && has_templates(&dir) {
        warn!("Failed to download templates ({}), using existing", e);
        return Ok(dir);
      }
      return Err(e).context("Failed to download templates and no local templates exist");
    }
  }

  Ok(dir)
}

/// Check if the templates directory has at least one .wallflowtemplate bundle
fn has_templates(dir: &Path) -> bool {
  if let Ok(entries) = fs::read_dir(dir) {
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_dir() && path.extension().map(|e| e == "wallflowtemplate").unwrap_or(false) {
        return true;
      }
    }
  }
  false
}

/// Download templates tarball from GitHub and extract to templates dir
async fn download_templates(templates_dir: &Path) -> Result<()> {
  let url = format!("https://github.com/{}/archive/refs/tags/{}.tar.gz", TEMPLATES_REPO, TEMPLATES_VERSION);

  debug!("Fetching templates from {}", url);

  let response = reqwest::get(&url).await.context("Failed to fetch templates tarball")?;

  if !response.status().is_success() {
    anyhow::bail!("Failed to download templates: HTTP {}", response.status());
  }

  let bytes = response.bytes().await.context("Failed to read response body")?;

  // Extract tarball on a blocking thread (CPU-bound work)
  let dest = templates_dir.to_path_buf();
  tokio::task::spawn_blocking(move || extract_tarball(&bytes, &dest))
    .await
    .context("Template extraction task failed")??;

  Ok(())
}

/// Extract .wallflowtemplate bundles from a gzipped tarball
fn extract_tarball(bytes: &[u8], templates_dir: &Path) -> Result<()> {
  let gz = flate2::read::GzDecoder::new(bytes);
  let mut archive = tar::Archive::new(gz);

  fs::create_dir_all(templates_dir).context("Failed to create templates directory")?;

  // Extract .wallflowtemplate directories from the tarball
  // GitHub tarballs have a top-level directory like "Owner-repo-<sha>/"
  for entry in archive.entries().context("Failed to read tar entries")? {
    let mut entry = entry.context("Failed to read tar entry")?;
    let path = entry.path().context("Failed to get entry path")?.into_owned();

    // Skip the top-level directory prefix
    let components: Vec<_> = path.components().collect();
    if components.len() < 2 {
      continue;
    }

    // Reconstruct path without the top-level directory
    let relative: PathBuf = components[1..].iter().collect();

    // Only extract .wallflowtemplate directories and their contents
    let relative_str = relative.to_string_lossy();
    if !relative_str.contains(".wallflowtemplate") {
      continue;
    }

    let dest = templates_dir.join(&relative);

    if entry.header().entry_type().is_dir() {
      fs::create_dir_all(&dest)?;
    } else if entry.header().entry_type().is_file() {
      // Don't overwrite user-modified files
      if dest.exists() {
        let mut new_content = Vec::new();
        entry.read_to_end(&mut new_content)?;

        let existing_content = fs::read(&dest).unwrap_or_default();
        if existing_content != new_content {
          debug!("Skipping modified template file: {}", dest.display());
          continue;
        }
      }

      if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
      }
      let mut file = fs::File::create(&dest)?;
      std::io::copy(&mut entry, &mut file)?;
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_templates_dir() {
    let dir = templates_dir();
    assert!(dir.to_string_lossy().contains("wallflow"));
    assert!(dir.to_string_lossy().contains("templates"));
  }
}
