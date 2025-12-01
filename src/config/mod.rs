use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration structure

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
  pub paths: PathsConfig,
  pub transition: TransitionConfig,
  pub timer: TimerConfig,
  pub sources: SourcesConfig,
  pub cleanup: CleanupConfig,
  pub integration: IntegrationConfig,
  pub logging: LoggingConfig,
  #[serde(default)]
  pub advanced: AdvancedConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PathsConfig {
  pub local: String,
  pub downloads: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransitionConfig {
  #[serde(rename = "type")]
  pub transition_type: TransitionType,
  pub duration: u32,
  #[serde(default = "default_fps")]
  pub fps: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TransitionType {
  Single(String),
  Multiple(Vec<String>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimerConfig {
  pub interval: u32,
  pub randomize: String,
  #[serde(default)]
  pub start_delay: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourcesConfig {
  pub default: String,
  #[serde(default)]
  pub wallhaven: WallhavenConfig,
  #[serde(default)]
  pub picsum: PicsumConfig,
  #[serde(default)]
  pub local: LocalConfig,
  #[serde(default)]
  pub apod: ApodConfig,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct WallhavenConfig {
  pub url: String,
  #[serde(default)]
  pub q: String,
  #[serde(default)]
  pub resolution: Option<String>, // Auto-detect if None
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PicsumConfig {
  #[serde(default)]
  pub width: Option<u32>, // Auto-detect if None
  #[serde(default)]
  pub height: Option<u32>, // Auto-detect if None
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ApodConfig {
  #[serde(default = "default_apod_api_url")]
  pub url: String,
  #[serde(default = "default_apod_api_key")]
  pub api_key: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct LocalConfig {
  #[serde(default = "default_true")]
  pub recursive: bool,
  #[serde(default = "default_formats")]
  pub formats: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CleanupConfig {
  pub keep_count: u32,
  #[serde(default = "default_true")]
  pub auto_cleanup: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct IntegrationConfig {
  #[serde(default)]
  pub pywal: PywalConfig,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PywalConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default)]
  pub backend: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct LoggingConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default = "default_log_level")]
  pub level: String,
  #[serde(default)]
  pub file: Option<String>,
  #[serde(default = "default_true")]
  pub timestamp: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AdvancedConfig {
  #[serde(default = "default_parallel_downloads")]
  pub parallel_downloads: u32,
  #[serde(default = "default_retry_attempts")]
  pub retry_attempts: u32,
  #[serde(default = "default_timeout")]
  pub timeout: u32,
  #[serde(default = "default_user_agent")]
  pub user_agent: String,
}

// Default value functions (serde uses these for missing fields)
fn default_fps() -> u32 {
  30
}

fn default_apod_api_url() -> String {
  "https://api.nasa.gov/planetary/apod".to_string()
}

fn default_apod_api_key() -> String {
  "DEMO_KEY".to_string()
}
fn default_true() -> bool {
  true
}
fn default_formats() -> Vec<String> {
  vec!["jpg".to_string(), "jpeg".to_string(), "png".to_string(), "webp".to_string()]
}
fn default_log_level() -> String {
  "info".to_string()
}
fn default_user_agent() -> String {
  "Wallflow/1.0 (+https://github.com/MKSG-MugunthKumar/wallflow)".to_string()
}
fn default_parallel_downloads() -> u32 {
  3
}
fn default_retry_attempts() -> u32 {
  3
}
fn default_timeout() -> u32 {
  30
}

impl Config {
  /// Load configuration from file - SO MUCH CLEANER than AWK! ✨
  pub fn load(path: &Path) -> Result<Self> {
    let contents = std::fs::read_to_string(path).with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: Config = serde_yaml::from_str(&contents).with_context(|| format!("Failed to parse YAML config: {}", path.display()))?;

    Ok(config)
  }

  /// Get default config file path (XDG compliant)
  pub fn default_path() -> PathBuf {
    dirs::config_dir()
      .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config"))
      .join("wallflow")
      .join("config.yml")
  }

  /// Load with fallback to defaults
  pub fn load_or_default() -> Result<Self> {
    let path = Self::default_path();

    if path.exists() {
      Self::load(&path)
    } else {
      tracing::warn!("Config file not found at {}, using defaults", path.display());
      Ok(Self::default())
    }
  }

  /// Expand environment variables in paths
  pub fn expand_paths(&mut self) -> Result<()> {
    self.paths.local = resolve_wallpaper_path(&self.paths.local);
    self.paths.downloads = resolve_wallpaper_path(&self.paths.downloads);
    Ok(())
  }

  /// Get wallhaven resolution (from config or auto-detect)
  #[allow(dead_code)]
  pub fn get_wallhaven_resolution(&self) -> Result<crate::display::Resolution> {
    match &self.sources.wallhaven.resolution {
      Some(res_str) => crate::display::Resolution::from_string(res_str),
      None => crate::display::get_primary_display_resolution(),
    }
  }

  /// Get picsum resolution (from config or auto-detect)
  #[allow(dead_code)]
  pub fn get_picsum_resolution(&self) -> Result<crate::display::Resolution> {
    match (self.sources.picsum.width, self.sources.picsum.height) {
      (Some(w), Some(h)) => Ok(crate::display::Resolution::new(w, h)),
      _ => crate::display::get_primary_display_resolution(),
    }
  }
}

/// Create default configuration
impl Default for Config {
  fn default() -> Self {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let pictures = dirs::picture_dir().unwrap_or_else(|| home.join("Pictures"));
    let wallpapers = pictures.join("Wallpapers");

    Self {
      paths: PathsConfig {
        local: wallpapers.to_string_lossy().to_string(),
        downloads: wallpapers.join("downloads").to_string_lossy().to_string(),
      },
      transition: TransitionConfig {
        transition_type: TransitionType::Single("random".to_string()),
        duration: 5,
        fps: 30,
      },
      timer: TimerConfig {
        interval: 30,
        randomize: "5m".to_string(),
        start_delay: Some("1m".to_string()),
      },
      sources: SourcesConfig {
        default: "local".to_string(),
        wallhaven: WallhavenConfig::default(),
        picsum: PicsumConfig::default(),
        local: LocalConfig::default(),
        apod: ApodConfig::default(),
      },
      cleanup: CleanupConfig {
        keep_count: 10,
        auto_cleanup: true,
      },
      integration: IntegrationConfig::default(),
      logging: LoggingConfig::default(),
      advanced: AdvancedConfig::default(),
    }
  }
}

fn resolve_wallpaper_path(path: &str) -> String {
  let path_obj = Path::new(path);

  if path_obj.is_absolute() {
    // Absolute path - expand any env vars like $HOME or ~
    shellexpand::full(path).map(|s| s.into_owned()).unwrap_or_else(|_| path.to_string())
  } else {
    // Relative path - resolve from XDG pictures dir
    let base = dirs::picture_dir().unwrap_or_else(|| PathBuf::from("~/Pictures"));
    let full_path = base.join(path_obj);
    let full_path_str = full_path.to_string_lossy();
    shellexpand::full(&full_path_str)
      .map(|s| s.into_owned())
      .unwrap_or_else(|_| full_path_str.into_owned())
  }
}
