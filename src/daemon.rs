use anyhow::{Context, Result};
use daemonize::Daemonize;
use rand::Rng;
use std::fs::File;
use tokio::time::{Duration, interval, sleep};
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::downloaders::DownloadOptions;
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

  // Download templates if native color engine is enabled
  if config.colors.enabled && config.colors.engine == "native" {
    match crate::templates::ensure_templates() {
      Ok(dir) => info!("Templates ready at {}", dir.display()),
      Err(e) => warn!("Failed to download templates (will retry later): {}", e),
    }
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
  let runtime_dir = home_dir.join(".local/share/mksg/wallflow");
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

      // Note: Logging is already initialized in main() before daemonization.
      // After fork, stderr is redirected to wallflow_error.log, so tracing output goes there.
      // Do NOT re-initialize tracing here - it would panic with "global subscriber already set".

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

/// Get the PID file path
fn get_pid_file() -> Result<std::path::PathBuf> {
  let home_dir = dirs::home_dir().context("Could not find home directory")?;
  Ok(home_dir.join(".local/share/mksg/wallflow/wallflow.pid"))
}

/// Read the daemon PID from the PID file
fn read_daemon_pid() -> Result<i32> {
  let pid_file = get_pid_file()?;
  let pid_str = std::fs::read_to_string(&pid_file).with_context(|| format!("Could not read PID file: {}", pid_file.display()))?;
  let pid: i32 = pid_str.trim().parse().context("Invalid PID in file")?;
  Ok(pid)
}

/// Stop the running daemon
pub fn stop_daemon() -> Result<()> {
  let pid = read_daemon_pid()?;

  // Send SIGTERM to gracefully stop the daemon
  let output = std::process::Command::new("kill").arg(pid.to_string()).output()?;

  if output.status.success() {
    println!("✅ Daemon stopped (PID: {})", pid);
    // Clean up PID file
    let pid_file = get_pid_file()?;
    let _ = std::fs::remove_file(pid_file);
    Ok(())
  } else {
    Err(anyhow::anyhow!("Failed to stop daemon (PID: {})", pid))
  }
}

/// Reload the daemon configuration
pub fn reload_daemon() -> Result<()> {
  let pid = read_daemon_pid()?;

  // Send SIGHUP to trigger config reload
  let output = std::process::Command::new("kill").args(["-HUP", &pid.to_string()]).output()?;

  if output.status.success() {
    println!("✅ Daemon configuration reloaded (PID: {})", pid);
    Ok(())
  } else {
    Err(anyhow::anyhow!("Failed to reload daemon (PID: {})", pid))
  }
}

/// Check if daemon is running
fn is_daemon_running() -> bool {
  if let Ok(pid) = read_daemon_pid() {
    // Check if process exists by sending signal 0
    std::process::Command::new("kill")
      .args(["-0", &pid.to_string()])
      .output()
      .map(|o| o.status.success())
      .unwrap_or(false)
  } else {
    false
  }
}

/// Show daemon status
pub fn status_daemon() -> Result<()> {
  println!("📊 wallflow Daemon Status");
  println!();

  if is_daemon_running() {
    let pid = read_daemon_pid()?;
    println!("   🟢 Status: Running");
    println!("   📍 PID: {}", pid);

    // Try to read status from daemon_status.json
    let home_dir = dirs::home_dir().context("Could not find home directory")?;
    let status_file = home_dir.join(".local/share/mksg/wallflow/daemon_status.json");
    if status_file.exists()
      && let Ok(content) = std::fs::read_to_string(&status_file)
      && let Ok(status) = serde_json::from_str::<serde_json::Value>(&content)
    {
      if let Some(current) = status.get("current_wallpaper").and_then(|v| v.as_str()) {
        println!("   🖼️  Current: {}", current);
      }
      if let Some(next) = status.get("next_rotation").and_then(|v| v.as_str()) {
        println!("   ⏰ Next rotation: {}", next);
      }
    }

    let log_file = home_dir.join(".local/share/mksg/wallflow/wallflow.log");
    println!("   📄 Log file: {}", log_file.display());
  } else {
    println!("   🔴 Status: Not running");
    println!("   💡 Use 'wallflow daemon start' to start the daemon");
  }

  Ok(())
}

/// Get the path to the current executable
fn get_executable_path() -> Result<String> {
  std::env::current_exe()
    .context("Could not determine executable path")?
    .to_str()
    .map(|s| s.to_string())
    .context("Executable path is not valid UTF-8")
}

/// Install daemon to run at system startup
pub fn install_daemon() -> Result<()> {
  let exe_path = get_executable_path()?;

  #[cfg(target_os = "linux")]
  {
    install_systemd_service(&exe_path)
  }

  #[cfg(target_os = "macos")]
  {
    install_launchd_service(&exe_path)
  }

  #[cfg(target_os = "windows")]
  {
    Err(anyhow::anyhow!(
      "Windows service installation not yet implemented. Use Task Scheduler manually."
    ))
  }

  #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
  {
    Err(anyhow::anyhow!("Unsupported platform for daemon installation"))
  }
}

/// Uninstall daemon from system startup
pub fn uninstall_daemon() -> Result<()> {
  #[cfg(target_os = "linux")]
  {
    uninstall_systemd_service()
  }

  #[cfg(target_os = "macos")]
  {
    uninstall_launchd_service()
  }

  #[cfg(target_os = "windows")]
  {
    Err(anyhow::anyhow!("Windows service uninstallation not yet implemented."))
  }

  #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
  {
    Err(anyhow::anyhow!("Unsupported platform for daemon uninstallation"))
  }
}

#[cfg(target_os = "linux")]
fn install_systemd_service(exe_path: &str) -> Result<()> {
  let home_dir = dirs::home_dir().context("Could not find home directory")?;
  let service_dir = home_dir.join(".config/systemd/user");
  std::fs::create_dir_all(&service_dir).context("Failed to create systemd user directory")?;

  let service_file = service_dir.join("wallflow.service");

  let service_content = format!(
    r#"[Unit]
Description=Wallflow Daemon
After=default.target

[Service]
Type=simple
ExecStart={} daemon start --foreground
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
"#,
    exe_path
  );

  std::fs::write(&service_file, service_content).context("Failed to write service file")?;

  println!("📝 Created service file: {}", service_file.display());

  // Reload systemd and enable the service
  let reload = std::process::Command::new("systemctl").args(["--user", "daemon-reload"]).output()?;

  if !reload.status.success() {
    return Err(anyhow::anyhow!("Failed to reload systemd"));
  }

  let enable = std::process::Command::new("systemctl")
    .args(["--user", "enable", "--now", "wallflow"])
    .output()?;

  if enable.status.success() {
    println!("✅ wallflow daemon installed and started");
    println!();
    println!("   Manage with:");
    println!("   systemctl --user status wallflow");
    println!("   systemctl --user stop wallflow");
    println!("   systemctl --user start wallflow");
    println!("   journalctl --user -u wallflow -f");
    Ok(())
  } else {
    Err(anyhow::anyhow!("Failed to enable service: {}", String::from_utf8_lossy(&enable.stderr)))
  }
}

#[cfg(target_os = "linux")]
fn uninstall_systemd_service() -> Result<()> {
  // Stop and disable the service
  let _ = std::process::Command::new("systemctl").args(["--user", "stop", "wallflow"]).output();

  let disable = std::process::Command::new("systemctl").args(["--user", "disable", "wallflow"]).output()?;

  if !disable.status.success() {
    warn!("Service may not have been enabled: {}", String::from_utf8_lossy(&disable.stderr));
  }

  // Remove the service file
  let home_dir = dirs::home_dir().context("Could not find home directory")?;
  let service_file = home_dir.join(".config/systemd/user/wallflow.service");

  if service_file.exists() {
    std::fs::remove_file(&service_file).context("Failed to remove service file")?;
    println!("🗑️  Removed service file: {}", service_file.display());
  }

  // Reload systemd
  let _ = std::process::Command::new("systemctl").args(["--user", "daemon-reload"]).output();

  println!("✅ wallflow daemon uninstalled");
  Ok(())
}

#[cfg(target_os = "macos")]
fn install_launchd_service(exe_path: &str) -> Result<()> {
  let home_dir = dirs::home_dir().context("Could not find home directory")?;
  let launch_agents_dir = home_dir.join("Library/LaunchAgents");
  std::fs::create_dir_all(&launch_agents_dir).context("Failed to create LaunchAgents directory")?;

  let plist_file = launch_agents_dir.join("com.mksg.wallflow.plist");
  let log_dir = home_dir.join(".local/share/mksg/wallflow");
  std::fs::create_dir_all(&log_dir).context("Failed to create log directory")?;

  let plist_content = format!(
    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.mksg.wallflow</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>daemon</string>
        <string>start</string>
        <string>--foreground</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}/wallflow.log</string>
    <key>StandardErrorPath</key>
    <string>{}/wallflow_error.log</string>
</dict>
</plist>
"#,
    exe_path,
    log_dir.display(),
    log_dir.display()
  );

  std::fs::write(&plist_file, plist_content).context("Failed to write plist file")?;

  println!("📝 Created plist file: {}", plist_file.display());

  // Load the service
  let load = std::process::Command::new("launchctl")
    .args(["load", plist_file.to_str().unwrap()])
    .output()?;

  if load.status.success() {
    println!("✅ wallflow daemon installed and started");
    println!();
    println!("   Manage with:");
    println!("   launchctl list | grep wallflow");
    println!("   launchctl stop com.mksg.wallflow");
    println!("   launchctl start com.mksg.wallflow");
    Ok(())
  } else {
    Err(anyhow::anyhow!("Failed to load service: {}", String::from_utf8_lossy(&load.stderr)))
  }
}

#[cfg(target_os = "macos")]
fn uninstall_launchd_service() -> Result<()> {
  let home_dir = dirs::home_dir().context("Could not find home directory")?;
  let plist_file = home_dir.join("Library/LaunchAgents/com.mksg.wallflow.plist");

  if plist_file.exists() {
    // Unload the service
    let _ = std::process::Command::new("launchctl")
      .args(["unload", plist_file.to_str().unwrap()])
      .output();

    // Remove the plist file
    std::fs::remove_file(&plist_file).context("Failed to remove plist file")?;
    println!("🗑️  Removed plist file: {}", plist_file.display());
  }

  println!("✅ wallflow daemon uninstalled");
  Ok(())
}

/// Set wallpaper based on configured default source
/// Uses fire-and-forget mode to avoid blocking during transitions
async fn set_wallpaper_by_source(config: &Config) -> Result<()> {
  let source = config.sources.default.as_str();
  // Daemon always sets wallpaper (no --no-set)
  let opts = DownloadOptions::default();
  match source {
    "local" => wallpaper::set_local_daemon(config).await,
    // All remote sources use the generic set_from_source with empty query
    // (daemon uses config defaults, not CLI args)
    "wallhaven" | "picsum" | "apod" | "bing" | "reddit" | "earthview" | "unsplash" => {
      wallpaper::set_from_source_daemon(config, source, &[], &opts).await
    }
    other => {
      warn!("Unknown source '{}', falling back to local", other);
      wallpaper::set_local_daemon(config).await
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
