//! Application state management for the TUI
//!
//! This module contains the core application state and business logic
//! for the wallpaper browser TUI. It demonstrates:
//! - State management patterns in async Rust
//! - File system interaction and wallpaper discovery
//! - Event-driven state updates
//! - Resource management for large collections

use crate::daemon_status::{DaemonStatus, DaemonStatusManager};
use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Main application state
#[derive(Debug)]
pub struct App {
  /// Should the application exit?
  pub should_quit: bool,

  /// Current configuration
  pub config: crate::config::Config,

  /// Available wallpapers
  pub wallpapers: Vec<WallpaperItem>,

  /// Currently selected wallpaper index
  pub selected: usize,

  /// Current view mode
  pub view_mode: ViewMode,

  /// Status messages for user feedback
  pub status_message: Option<String>,

  /// Loading state
  pub is_loading: bool,

  /// Error state
  pub error_message: Option<String>,

  /// Daemon status tracking
  pub daemon_status: Option<DaemonStatus>,

  /// Daemon status manager
  daemon_status_manager: DaemonStatusManager,
}

/// Wallpaper item with metadata
#[derive(Debug, Clone)]
pub struct WallpaperItem {
  /// File path to the wallpaper
  pub path: PathBuf,

  /// File name for display
  pub name: String,

  /// File size in bytes
  pub size: Option<u64>,

  /// Image dimensions (width, height)
  pub dimensions: Option<(u32, u32)>,

  /// File format extension
  pub format: Option<String>,

  /// Whether this wallpaper is currently set as desktop background
  pub is_current: bool,
}

/// TUI view modes
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
  /// Browse wallpapers in a list
  Browse,

  /// Preview selected wallpaper with details
  Preview,

  /// Settings and configuration
  Settings,

  /// Help screen with keybindings
  Help,
}

#[allow(dead_code)]
impl App {
  /// Create a new application instance
  pub async fn new(config: crate::config::Config) -> Result<Self> {
    debug!("🎨 Initializing TUI application");

    let daemon_status_manager = DaemonStatusManager::new()?;

    let mut app = Self {
      should_quit: false,
      config,
      wallpapers: Vec::new(),
      selected: 0,
      view_mode: ViewMode::Browse,
      status_message: Some("Loading wallpapers...".to_string()),
      is_loading: true,
      error_message: None,
      daemon_status: None,
      daemon_status_manager,
    };

    // Load wallpapers in background
    app.refresh_wallpapers().await?;

    // Load daemon status
    app.update_daemon_status().await?;

    app.is_loading = false;
    app.status_message = Some(format!("Found {} wallpapers", app.wallpapers.len()));

    Ok(app)
  }

  /// Refresh the wallpaper collection
  pub async fn refresh_wallpapers(&mut self) -> Result<()> {
    debug!("Refreshing wallpaper collection");
    self.is_loading = true;
    self.error_message = None;

    let wallpaper_dir = Path::new(&self.config.paths.local);

    if !wallpaper_dir.exists() {
      let error = format!("Wallpaper directory does not exist: {}", wallpaper_dir.display());
      warn!("{}", error);
      self.error_message = Some(error);
      self.is_loading = false;
      return Ok(());
    }

    let mut wallpapers = Vec::new();
    self.collect_wallpapers(
      wallpaper_dir,
      &self.config.sources.local.formats,
      &mut wallpapers,
      self.config.sources.local.recursive,
    )?;

    // Sort wallpapers by name for consistent ordering
    wallpapers.sort_by(|a, b| a.name.cmp(&b.name));

    self.wallpapers = wallpapers;
    self.selected = 0; // Reset selection

    debug!("📁 Loaded {} wallpapers", self.wallpapers.len());
    Ok(())
  }

  /// Recursively collect wallpaper files
  fn collect_wallpapers(&self, dir: &Path, formats: &[String], wallpapers: &mut Vec<WallpaperItem>, recursive: bool) -> Result<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| anyhow::anyhow!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry in entries {
      let entry = entry?;
      let path = entry.path();

      if path.is_file() {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str())
          && formats.iter().any(|fmt| fmt.eq_ignore_ascii_case(extension))
        {
          let wallpaper_item = self.create_wallpaper_item(&path)?;
          wallpapers.push(wallpaper_item);
        }
      } else if path.is_dir() && recursive {
        self.collect_wallpapers(&path, formats, wallpapers, recursive)?;
      }
    }

    Ok(())
  }

  /// Create a wallpaper item with metadata
  fn create_wallpaper_item(&self, path: &Path) -> Result<WallpaperItem> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();

    let size = std::fs::metadata(path).ok().map(|m| m.len());

    let dimensions = imagesize::size(path).ok().map(|s| (s.width as u32, s.height as u32));

    let format = path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_uppercase());

    Ok(WallpaperItem {
      path: path.to_path_buf(),
      name,
      size,
      dimensions,
      format,
      is_current: false, // TODO: Detect current wallpaper
    })
  }

  /// Move selection up
  pub fn select_previous(&mut self) {
    if !self.wallpapers.is_empty() {
      self.selected = if self.selected == 0 {
        self.wallpapers.len() - 1
      } else {
        self.selected - 1
      };
    }
  }

  /// Move selection down
  pub fn select_next(&mut self) {
    if !self.wallpapers.is_empty() {
      self.selected = (self.selected + 1) % self.wallpapers.len();
    }
  }

  /// Get the currently selected wallpaper
  pub fn selected_wallpaper(&self) -> Option<&WallpaperItem> {
    self.wallpapers.get(self.selected)
  }

  /// Set the selected wallpaper as desktop background
  pub async fn apply_selected_wallpaper(&mut self) -> Result<()> {
    if let Some(wallpaper) = self.selected_wallpaper().cloned() {
      debug!("🖼️  Setting wallpaper: {}", wallpaper.name);
      self.status_message = Some("Applying wallpaper...".to_string());

      // Use the wallpaper module to apply the wallpaper
      match crate::wallpaper::apply_wallpaper(&wallpaper.path, &self.config).await {
        Ok(()) => {
          self.status_message = Some(format!("✅ Applied: {}", wallpaper.name));

          // Mark this wallpaper as current and unmark others
          for item in &mut self.wallpapers {
            item.is_current = item.path == wallpaper.path;
          }
        }
        Err(e) => {
          let error = format!("❌ Failed to apply wallpaper: {}", e);
          warn!("{}", error);
          self.error_message = Some(error);
        }
      }
    }
    Ok(())
  }

  /// Switch view mode
  pub fn set_view_mode(&mut self, mode: ViewMode) {
    debug!("Switching to view mode: {:?}", mode);
    self.view_mode = mode;
  }

  /// Handle quit request
  pub fn quit(&mut self) {
    debug!("🚪 Exiting TUI application");
    self.should_quit = true;
  }

  /// Clear status messages
  pub fn clear_messages(&mut self) {
    self.status_message = None;
    self.error_message = None;
  }

  /// Update daemon status information
  pub async fn update_daemon_status(&mut self) -> Result<()> {
    self.daemon_status = self.daemon_status_manager.get_status().await?;
    Ok(())
  }

  /// Check if daemon is currently running
  pub async fn is_daemon_running(&mut self) -> Result<bool> {
    self.daemon_status_manager.is_daemon_running().await
  }

  /// Get daemon time remaining as formatted string
  pub fn daemon_time_remaining(&self) -> Option<String> {
    self.daemon_status.as_ref().map(|s| s.time_remaining_formatted())
  }

  /// Get formatted status information
  pub fn status_info(&self) -> String {
    let mut parts = vec![
      format!("{} wallpapers", self.wallpapers.len()),
      format!("{} selected", self.selected + 1),
      format!("Mode: {:?}", self.view_mode),
    ];

    // Add daemon status if available
    if let Some(ref status) = self.daemon_status {
      let daemon_info = if status.is_stale() {
        "Daemon: Offline".to_string()
      } else {
        format!("Daemon: {} remaining", status.time_remaining_formatted())
      };
      parts.push(daemon_info);
    } else {
      parts.push("Daemon: Unknown".to_string());
    }

    parts.join(" | ")
  }
}

/// Helper function to format file size
pub fn format_file_size(size: u64) -> String {
  const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
  const THRESHOLD: f64 = 1024.0;

  if size == 0 {
    return "0 B".to_string();
  }

  let size_f = size as f64;
  let unit_index = (size_f.log(THRESHOLD).floor() as usize).min(UNITS.len() - 1);
  let value = size_f / THRESHOLD.powi(unit_index as i32);

  if unit_index == 0 {
    format!("{} {}", size, UNITS[unit_index])
  } else {
    format!("{:.1} {}", value, UNITS[unit_index])
  }
}
