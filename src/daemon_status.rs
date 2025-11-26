//! Daemon status tracking and IPC for wallflow
//!
//! This module provides shared state tracking for the wallpaper daemon,
//! allowing the TUI to display real-time information about:
//! - Time remaining until next wallpaper change
//! - Current daemon status (running, stopped, error)
//! - Last rotation timestamp
//! - Next rotation schedule
//!
//! Educational aspects:
//! - File-based IPC patterns for simple daemon communication
//! - JSON serialization for structured data exchange
//! - Timestamp arithmetic and duration calculations
//! - Cross-process state sharing

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tracing::debug;

/// Daemon status information shared via file system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
  /// Process ID of the daemon
  pub pid: u32,

  /// When the daemon was started
  pub started_at: DateTime<Utc>,

  /// Last wallpaper rotation time
  pub last_rotation: Option<DateTime<Utc>>,

  /// Next scheduled rotation time
  pub next_rotation: DateTime<Utc>,

  /// Current wallpaper file path
  pub current_wallpaper: Option<String>,

  /// Daemon configuration snapshot
  pub config: DaemonConfig,

  /// Last update timestamp (for staleness detection)
  pub updated_at: DateTime<Utc>,
}

/// Essential daemon configuration for status display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
  /// Rotation interval in minutes
  pub interval_minutes: u32,

  /// Randomization range
  pub randomize: String,

  /// Default wallpaper source
  pub source: String,
}

#[allow(dead_code)]
impl DaemonStatus {
  /// Create new daemon status
  pub fn new(config: &crate::config::Config) -> Self {
    let now = Utc::now();
    let interval = Duration::from_secs(config.timer.interval as u64 * 60);
    let next_rotation = now + chrono::Duration::from_std(interval).unwrap();

    Self {
      pid: std::process::id(),
      started_at: now,
      last_rotation: None,
      next_rotation,
      current_wallpaper: None,
      config: DaemonConfig {
        interval_minutes: config.timer.interval,
        randomize: config.timer.randomize.clone(),
        source: config.sources.default.clone(),
      },
      updated_at: now,
    }
  }

  /// Update with new rotation information
  pub fn update_rotation(&mut self, wallpaper_path: Option<String>) {
    let now = Utc::now();
    self.last_rotation = Some(now);
    self.current_wallpaper = wallpaper_path;

    // Calculate next rotation time
    let interval = Duration::from_secs(self.config.interval_minutes as u64 * 60);

    // Add randomization if configured
    let randomize_secs = parse_duration(&self.config.randomize).unwrap_or(0);
    let total_interval = interval + Duration::from_secs(randomize_secs / 2); // Average randomization

    self.next_rotation = now + chrono::Duration::from_std(total_interval).unwrap();
    self.updated_at = now;
  }

  /// Get time remaining until next rotation
  pub fn time_remaining(&self) -> Result<Duration> {
    let now = Utc::now();

    if self.next_rotation > now {
      let remaining = self.next_rotation - now;
      Ok(remaining.to_std().context("Invalid duration")?)
    } else {
      // Rotation is overdue
      Ok(Duration::ZERO)
    }
  }

  /// Get formatted time remaining as string
  pub fn time_remaining_formatted(&self) -> String {
    match self.time_remaining() {
      Ok(duration) => {
        if duration.is_zero() {
          "Rotation overdue".to_string()
        } else {
          format_duration(duration)
        }
      }
      Err(_) => "Unknown".to_string(),
    }
  }

  /// Check if status is stale (not updated recently)
  pub fn is_stale(&self) -> bool {
    let now = Utc::now();
    let age = now - self.updated_at;
    age.num_seconds() > 300 // 5 minutes
  }
}

/// Daemon status manager for file-based IPC
#[derive(Debug)]
pub struct DaemonStatusManager {
  status_file: PathBuf,
  status: Option<DaemonStatus>,
}

#[allow(dead_code)]
impl DaemonStatusManager {
  /// Create new status manager
  pub fn new() -> Result<Self> {
    let status_file = get_status_file_path()?;
    Ok(Self { status_file, status: None })
  }

  /// Initialize daemon status (called by daemon on startup)
  pub async fn initialize_daemon(&mut self, config: &crate::config::Config) -> Result<()> {
    self.status = Some(DaemonStatus::new(config));
    self.save().await?;
    debug!("Initialized daemon status at {}", self.status_file.display());
    Ok(())
  }

  /// Update daemon status with new rotation
  pub async fn update_rotation(&mut self, wallpaper_path: Option<String>) -> Result<()> {
    if let Some(ref mut status) = self.status {
      status.update_rotation(wallpaper_path);
      self.save().await?;
      debug!("Updated daemon status with new rotation");
    }
    Ok(())
  }

  /// Load daemon status from file
  pub async fn load(&mut self) -> Result<()> {
    if self.status_file.exists() {
      let content = fs::read_to_string(&self.status_file).await.context("Failed to read daemon status file")?;

      let status: DaemonStatus = serde_json::from_str(&content).context("Failed to parse daemon status JSON")?;

      self.status = Some(status);
      debug!("Loaded daemon status from {}", self.status_file.display());
    } else {
      debug!("No daemon status file found");
    }
    Ok(())
  }

  /// Save daemon status to file
  async fn save(&self) -> Result<()> {
    if let Some(ref status) = self.status {
      // Ensure directory exists
      if let Some(parent) = self.status_file.parent() {
        fs::create_dir_all(parent).await.context("Failed to create status directory")?;
      }

      let json = serde_json::to_string_pretty(status).context("Failed to serialize daemon status")?;

      fs::write(&self.status_file, json).await.context("Failed to write daemon status file")?;
    }
    Ok(())
  }

  /// Get current daemon status
  pub async fn get_status(&mut self) -> Result<Option<DaemonStatus>> {
    self.load().await?;
    Ok(self.status.clone())
  }

  /// Check if daemon is running
  pub async fn is_daemon_running(&mut self) -> Result<bool> {
    if let Some(status) = self.get_status().await? {
      if status.is_stale() {
        return Ok(false);
      }

      // Check if process is actually running
      #[cfg(unix)]
      {
        use std::process::Command;
        let output = Command::new("kill")
          .arg("-0") // Signal 0 just checks if process exists
          .arg(status.pid.to_string())
          .output();

        match output {
          Ok(result) => Ok(result.status.success()),
          Err(_) => Ok(false),
        }
      }

      #[cfg(not(unix))]
      {
        // For non-Unix systems, just check staleness
        Ok(!status.is_stale())
      }
    } else {
      Ok(false)
    }
  }

  /// Remove status file (when daemon stops)
  pub async fn cleanup(&self) -> Result<()> {
    if self.status_file.exists() {
      fs::remove_file(&self.status_file).await.context("Failed to remove daemon status file")?;
      debug!("Cleaned up daemon status file");
    }
    Ok(())
  }
}

/// Get the path for daemon status file
fn get_status_file_path() -> Result<PathBuf> {
  let home_dir = dirs::home_dir().context("Could not find home directory")?;
  let runtime_dir = home_dir.join(".local/share/wallflow");
  Ok(runtime_dir.join("daemon_status.json"))
}

/// Parse duration string (same as daemon.rs)
#[allow(dead_code)]
fn parse_duration(duration_str: &str) -> Result<u64> {
  let duration_str = duration_str.trim();

  if duration_str == "0" || duration_str.is_empty() {
    return Ok(0);
  }

  let (number_part, unit_part) = if let Some(pos) = duration_str.find(|c: char| c.is_alphabetic()) {
    let (num, unit) = duration_str.split_at(pos);
    (num, unit)
  } else {
    return duration_str.parse::<u64>().context("Invalid duration format");
  };

  let number: u64 = number_part.parse().context("Invalid number in duration")?;

  let multiplier = match unit_part {
    "s" | "sec" | "second" | "seconds" => 1,
    "m" | "min" | "minute" | "minutes" => 60,
    "h" | "hr" | "hour" | "hours" => 3600,
    "d" | "day" | "days" => 86400,
    _ => return Err(anyhow::anyhow!("Unknown duration unit: {}", unit_part)),
  };

  Ok(number * multiplier)
}

/// Format duration as human-readable string
fn format_duration(duration: Duration) -> String {
  let total_secs = duration.as_secs();

  let hours = total_secs / 3600;
  let minutes = (total_secs % 3600) / 60;
  let seconds = total_secs % 60;

  if hours > 0 {
    format!("{}h {}m {}s", hours, minutes, seconds)
  } else if minutes > 0 {
    format!("{}m {}s", minutes, seconds)
  } else {
    format!("{}s", seconds)
  }
}
