use anyhow::{Context, Result};
use daemonize::Daemonize;
use rand::Rng;
use std::fs::File;
use tokio::time::{Duration, interval, sleep};
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::wallpaper;

/// Run daemon in foreground with automatic wallpaper rotation
pub async fn run_foreground(config: Config) -> Result<()> {
  info!("🌊 wallflow daemon starting");
  info!("   Interval: {}m", config.timer.interval);
  info!("   Randomize: {}", config.timer.randomize);
  info!("   Source: {}", config.sources.default);

  // Parse randomization duration
  let randomize_secs = parse_duration(&config.timer.randomize).unwrap_or_else(|_| {
    warn!("Invalid randomize format '{}', using 0", config.timer.randomize);
    0
  });

  // Initial delay if configured
  if let Some(start_delay) = &config.timer.start_delay
    && let Ok(delay_secs) = parse_duration(start_delay)
    && delay_secs > 0
  {
    info!("Waiting {}s before starting...", delay_secs);
    sleep(Duration::from_secs(delay_secs)).await;
  }

  // Set initial wallpaper
  info!("Setting initial wallpaper...");
  if let Err(e) = set_wallpaper_by_source(&config).await {
    error!("Failed to set initial wallpaper: {}", e);
    // Continue anyway - might work later
  }

  // Start rotation timer
  let interval_secs = config.timer.interval as u64 * 60;
  let mut timer = interval(Duration::from_secs(interval_secs));

  // Consume the first tick (tokio interval fires immediately on first tick)
  timer.tick().await;

  info!("✅ Daemon started, rotating every {}m", config.timer.interval);

  loop {
    timer.tick().await;

    debug!("Timer tick - rotating wallpaper");

    // Rotate wallpaper
    match set_wallpaper_by_source(&config).await {
      Ok(_) => info!("✨ Wallpaper rotated successfully"),
      Err(e) => {
        error!("❌ Failed to rotate wallpaper: {}", e);
        // Continue running - temporary failures shouldn't stop the daemon
      }
    }

    // Add randomization delay
    if randomize_secs > 0 {
      let mut rng = rand::thread_rng();
      let random_delay = rng.gen_range(0..=randomize_secs);
      if random_delay > 0 {
        debug!("Adding random delay: {}s", random_delay);
        sleep(Duration::from_secs(random_delay)).await;
      }
    }
  }
}

/// Run daemon in background (daemonize)
/// Note: This function daemonizes first, then creates a new tokio runtime
pub fn run_background(config: Config) -> Result<()> {
  // Create PID and log directories
  let home_dir = dirs::home_dir().context("Could not find home directory")?;
  let runtime_dir = home_dir.join(".local/share/wallflow");
  std::fs::create_dir_all(&runtime_dir).context("Failed to create runtime directory")?;

  let pid_file = runtime_dir.join("wallflow.pid");
  let stdout_file = runtime_dir.join("wallflow.log");
  let stderr_file = runtime_dir.join("wallflow_error.log");

  info!("Daemonizing wallflow...");
  info!("PID file: {}", pid_file.display());
  info!("Logs: {}", stdout_file.display());

  // Set up daemonization
  let daemonize = Daemonize::new()
    .pid_file(&pid_file)
    .chown_pid_file(true)
    .working_directory(&runtime_dir)
    .stdout(File::create(&stdout_file)?)
    .stderr(File::create(&stderr_file)?)
    .privileged_action(|| "Daemonizing wallflow");

  // Fork into background - parent returns immediately, child continues
  match daemonize.start() {
    Ok(_) => {
      // We're now in the daemon child process
      // The parent's tokio runtime is gone, create a fresh one
      let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;

      // Initialize logging for the daemon process
      tracing_subscriber::fmt().with_env_filter("wallflow=info").with_target(false).init();

      info!("✅ Daemon process started (PID: {})", std::process::id());

      // Run the daemon in the new runtime
      rt.block_on(run_foreground(config))
    }
    Err(e) => {
      error!("Failed to daemonize: {}", e);
      Err(anyhow::anyhow!("Daemonization failed: {}", e))
    }
  }
}

/// Set wallpaper based on configured default source
async fn set_wallpaper_by_source(config: &Config) -> Result<()> {
  let source = config.sources.default.as_str();
  match source {
    "local" => wallpaper::set_local(config).await,
    // All remote sources use the generic set_from_source with empty query
    // (daemon uses config defaults, not CLI args)
    "wallhaven" | "picsum" | "apod" | "bing" | "reddit" | "earthview" | "unsplash" => wallpaper::set_from_source(config, source, &[]).await,
    other => {
      warn!("Unknown source '{}', falling back to local", other);
      wallpaper::set_local(config).await
    }
  }
}

/// Parse duration string (e.g., "5m", "30s", "2h")
fn parse_duration(duration_str: &str) -> Result<u64> {
  let duration_str = duration_str.trim();

  if duration_str == "0" || duration_str.is_empty() {
    return Ok(0);
  }

  let (number_part, unit_part) = if let Some(pos) = duration_str.find(|c: char| c.is_alphabetic()) {
    let (num, unit) = duration_str.split_at(pos);
    (num, unit)
  } else {
    // No unit, assume seconds
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_duration() {
    assert_eq!(parse_duration("30s").unwrap(), 30);
    assert_eq!(parse_duration("5m").unwrap(), 300);
    assert_eq!(parse_duration("2h").unwrap(), 7200);
    assert_eq!(parse_duration("1d").unwrap(), 86400);
    assert_eq!(parse_duration("0").unwrap(), 0);
    assert_eq!(parse_duration("").unwrap(), 0);

    assert!(parse_duration("invalid").is_err());
    assert!(parse_duration("5x").is_err());
  }
}
