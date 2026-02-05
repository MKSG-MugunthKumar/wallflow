//! Enhanced logging system with file output and configuration support
//!
//! This module provides comprehensive logging configuration that integrates
//! with the YAML config file, supporting:
//! - Multiple log levels (debug, info, warn, error)
//! - Optional file output with automatic directory creation
//! - Timestamp formatting control
//! - CLI verbose override capability
//!
//! Educational aspects:
//! - Demonstrates tracing_subscriber configuration patterns
//! - Shows file system integration with logging
//! - Illustrates conditional compilation for different output modes

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::MakeWriterExt;

use crate::config::{Config, LoggingConfig};

/// Initialize the logging system based on configuration
pub fn init_logging(config: &Config, verbose_override: bool) -> Result<()> {
  // Determine the effective logging configuration
  let effective_config = resolve_logging_config(&config.logging, verbose_override);

  // Create environment filter
  let env_filter = create_env_filter(&effective_config.level)?;

  // Build the subscriber based on whether file logging is enabled
  match effective_config.file {
    Some(ref file_path) => {
      init_with_file_logging(&effective_config, file_path, env_filter)?;
    }
    None => {
      init_console_only_logging(&effective_config, env_filter)?;
    }
  }

  // Log initialization details
  tracing::debug!("🔍 Logging initialized successfully");
  if verbose_override {
    tracing::debug!("Verbose mode enabled via CLI flag");
  }
  if let Some(file) = &effective_config.file {
    tracing::debug!("File logging enabled: {}", file);
  }

  Ok(())
}

/// Resolve the effective logging configuration considering CLI overrides
fn resolve_logging_config(config: &LoggingConfig, verbose_override: bool) -> EffectiveLoggingConfig {
  // Treat empty string as None (no file logging)
  let file = config.file.as_ref().filter(|s| !s.trim().is_empty()).cloned();

  EffectiveLoggingConfig {
    enabled: config.enabled,
    level: if verbose_override { "debug".to_string() } else { config.level.clone() },
    file,
    timestamp: config.timestamp,
    verbose_override,
  }
}

/// Effective logging configuration after resolving CLI overrides
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct EffectiveLoggingConfig {
  enabled: bool,
  level: String,
  file: Option<String>,
  timestamp: bool,
  verbose_override: bool,
}

/// Initialize console-only logging
fn init_console_only_logging(config: &EffectiveLoggingConfig, env_filter: EnvFilter) -> Result<()> {
  // Use a simpler approach without conditional timestamp types
  if config.timestamp {
    tracing_subscriber::fmt()
      .with_env_filter(env_filter)
      .with_target(false)
      .with_thread_ids(false)
      .with_file(false)
      .with_line_number(false)
      .with_level(true)
      .with_ansi(true)
      .try_init()
      .map_err(|e| anyhow::anyhow!("Failed to initialize console logging with timestamps: {}", e))?;
  } else {
    tracing_subscriber::fmt()
      .with_env_filter(env_filter)
      .with_target(false)
      .with_thread_ids(false)
      .with_file(false)
      .with_line_number(false)
      .with_level(true)
      .with_ansi(true)
      .without_time()
      .try_init()
      .map_err(|e| anyhow::anyhow!("Failed to initialize console logging without timestamps: {}", e))?;
  }

  Ok(())
}

/// Initialize logging with both console and file output
fn init_with_file_logging(config: &EffectiveLoggingConfig, file_path: &str, env_filter: EnvFilter) -> Result<()> {
  // Expand environment variables in file path
  let expanded_path = shellexpand::full(file_path).context("Failed to expand environment variables in log file path")?;
  let log_path = Path::new(expanded_path.as_ref());

  // Ensure the parent directory exists
  if let Some(parent_dir) = log_path.parent() {
    fs::create_dir_all(parent_dir).with_context(|| format!("Failed to create log directory: {}", parent_dir.display()))?;
  }

  // Create file appender
  let file_appender = tracing_appender::rolling::never(
    log_path.parent().unwrap_or_else(|| Path::new(".")),
    log_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("wallflow.log")),
  );

  // For dual output, we'll use the simpler approach with a single subscriber
  // that logs to both console and file
  if config.timestamp {
    tracing_subscriber::fmt()
      .with_env_filter(env_filter)
      .with_writer(std::io::stderr.and(file_appender))
      .with_target(false) // Console settings
      .with_thread_ids(false)
      .with_file(false)
      .with_line_number(false)
      .with_level(true)
      .with_ansi(true) // ANSI colors work on console, ignored in files
      .try_init()
      .map_err(|e| anyhow::anyhow!("Failed to initialize dual logging with timestamps: {}", e))?;
  } else {
    tracing_subscriber::fmt()
      .with_env_filter(env_filter)
      .with_writer(std::io::stderr.and(file_appender))
      .with_target(false)
      .with_thread_ids(false)
      .with_file(false)
      .with_line_number(false)
      .with_level(true)
      .with_ansi(true)
      .without_time()
      .try_init()
      .map_err(|e| anyhow::anyhow!("Failed to initialize dual logging without timestamps: {}", e))?;
  }

  Ok(())
}

/// Create environment filter for the specified log level
fn create_env_filter(level: &str) -> Result<EnvFilter> {
  // Parse the level string to ensure it's valid
  let normalized_level = normalize_log_level(level)?;

  // Create filter that applies to wallflow and its modules
  let filter_directive = format!("wallflow={}", normalized_level);

  EnvFilter::try_new(&filter_directive).with_context(|| format!("Invalid log level configuration: {}", level))
}

/// Normalize log level string to valid tracing levels
fn normalize_log_level(level: &str) -> Result<&str> {
  match level.to_lowercase().as_str() {
    "trace" => Ok("trace"),
    "debug" => Ok("debug"),
    "info" => Ok("info"),
    "warn" | "warning" => Ok("warn"),
    "error" => Ok("error"),
    _ => Err(anyhow::anyhow!(
      "Invalid log level: '{}'. Valid levels: trace, debug, info, warn, error",
      level
    )),
  }
}

/// Log system information and configuration details
pub fn log_system_info(config: &Config) {
  use tracing::debug;

  // Log key configuration details
  debug!("Configuration loaded:");
  debug!("  Local wallpapers: {}", config.paths.local);
  debug!("  Downloads: {}", config.paths.downloads);
  debug!("  Default source: {}", config.sources.default);

  // Log platform and backend information
  if let Ok(platform_info) = crate::wallpaper::platform_info() {
    for line in platform_info.lines() {
      if !line.is_empty() {
        debug!("Platform: {}", line.trim());
      }
    }
  }

  // Log integration status
  if config.integration.pywal.enabled {
    debug!(
      "Pywal integration: enabled (backend: {})",
      config.integration.pywal.backend.as_deref().unwrap_or("default")
    );
  } else {
    debug!("Pywal integration: disabled");
  }

  if let Some(file) = &config.logging.file {
    debug!("📝 File logging enabled: {}", file);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::LoggingConfig;

  #[test]
  fn test_normalize_log_level() {
    assert_eq!(normalize_log_level("debug").unwrap(), "debug");
    assert_eq!(normalize_log_level("DEBUG").unwrap(), "debug");
    assert_eq!(normalize_log_level("Info").unwrap(), "info");
    assert_eq!(normalize_log_level("warn").unwrap(), "warn");
    assert_eq!(normalize_log_level("warning").unwrap(), "warn");
    assert_eq!(normalize_log_level("ERROR").unwrap(), "error");

    assert!(normalize_log_level("invalid").is_err());
  }

  #[test]
  fn test_resolve_logging_config() {
    let config = LoggingConfig {
      enabled: true,
      level: "info".to_string(),
      file: Some("/tmp/test.log".to_string()),
      timestamp: true,
    };

    // Test without verbose override
    let effective = resolve_logging_config(&config, false);
    assert_eq!(effective.level, "info");
    assert!(!effective.verbose_override);

    // Test with verbose override
    let effective = resolve_logging_config(&config, true);
    assert_eq!(effective.level, "debug");
    assert!(effective.verbose_override);
  }

  #[test]
  fn test_resolve_logging_config_empty_file_path() {
    // Empty string should be treated as None (no file logging)
    let config = LoggingConfig {
      enabled: true,
      level: "info".to_string(),
      file: Some("".to_string()),
      timestamp: true,
    };
    let effective = resolve_logging_config(&config, false);
    assert!(effective.file.is_none());

    // Whitespace-only should also be treated as None
    let config = LoggingConfig {
      enabled: true,
      level: "info".to_string(),
      file: Some("   ".to_string()),
      timestamp: true,
    };
    let effective = resolve_logging_config(&config, false);
    assert!(effective.file.is_none());

    // Valid path should be preserved
    let config = LoggingConfig {
      enabled: true,
      level: "info".to_string(),
      file: Some("/tmp/test.log".to_string()),
      timestamp: true,
    };
    let effective = resolve_logging_config(&config, false);
    assert_eq!(effective.file, Some("/tmp/test.log".to_string()));
  }
}
