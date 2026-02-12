//! Template manifest data structures
//!
//! Matches the `.wallflowtemplate` bundle format used by the Swift app.

use serde::{Deserialize, Serialize};

/// Template manifest (manifest.json inside a .wallflowtemplate bundle)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateManifest {
  /// Unique identifier (e.g., "mksg.kitty")
  pub id: String,

  /// Display name (e.g., "Kitty")
  pub name: String,

  /// Author name
  #[serde(default)]
  pub author: String,

  /// Description of the template
  #[serde(default)]
  pub description: String,

  /// Category (e.g., "terminals", "editors")
  #[serde(default)]
  pub category: String,

  /// Icon name
  #[serde(default)]
  pub icon: String,

  /// App detection configuration
  pub detection: Detection,

  /// Template file configuration
  pub template: TemplateFile,

  /// Installation configuration
  pub install: InstallConfig,

  /// Reload configuration (signal to send after rendering)
  #[serde(default)]
  pub reload: Option<ReloadConfig>,

  /// UI/help configuration
  #[serde(default)]
  pub ui: UiConfig,
}

/// Reload configuration for notifying apps after template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReloadConfig {
  /// Signal name (e.g., "USR1", "USR2")
  pub signal: String,

  /// Process name to signal (e.g., "kitty", "ghostty")
  pub process_name: String,
}

/// App detection configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Detection {
  /// macOS bundle identifiers to detect
  #[serde(default)]
  pub bundle_ids: Vec<String>,

  /// File paths to check for app existence
  #[serde(default)]
  pub paths: Vec<String>,
}

/// Template file configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateFile {
  /// Template file name within the bundle
  pub file: String,

  /// Output file name (e.g., "colors-kitty.conf")
  pub output_name: String,
}

/// Installation configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallConfig {
  /// Installation method ("symlink" or "copy")
  #[serde(default)]
  pub method: String,

  /// Whether to create parent directories
  #[serde(default)]
  pub create_directories: bool,

  /// Destination paths for the output file
  #[serde(default)]
  pub destinations: Vec<String>,
}

/// UI configuration for user-facing help
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
  /// Description shown in settings
  #[serde(default)]
  pub description: String,

  /// Code snippet to show user for manual config
  #[serde(default)]
  pub config_snippet: Option<String>,

  /// Path to the app's config file
  #[serde(default)]
  pub config_path: Option<String>,

  /// Help URL
  #[serde(default)]
  pub help_url: Option<String>,
}

impl TemplateManifest {
  /// Load from JSON string
  pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
    serde_json::from_str(json)
  }

  /// Load from manifest.json file
  pub fn load<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
    let content = std::fs::read_to_string(path)?;
    Ok(Self::from_json(&content)?)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_manifest() {
    let json = r#"{
            "id": "mksg.kitty",
            "name": "Kitty",
            "author": "mksg",
            "description": "Dynamic color scheme for Kitty terminal",
            "category": "terminals",
            "icon": "terminal",
            "detection": {
                "bundleIds": ["net.kovidgoyal.kitty"],
                "paths": ["/Applications/kitty.app"]
            },
            "template": {
                "file": "template.conf",
                "outputName": "colors-kitty.conf"
            },
            "install": {
                "method": "symlink",
                "createDirectories": true,
                "destinations": []
            },
            "ui": {
                "description": "Add include line to kitty.conf",
                "configSnippet": "include ~/.cache/mksg/wallflow/colors-kitty.conf"
            }
        }"#;

    let manifest = TemplateManifest::from_json(json).unwrap();
    assert_eq!(manifest.id, "mksg.kitty");
    assert_eq!(manifest.name, "Kitty");
    assert_eq!(manifest.detection.bundle_ids, vec!["net.kovidgoyal.kitty"]);
    assert_eq!(manifest.template.output_name, "colors-kitty.conf");
  }
}
