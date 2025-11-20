use anyhow::{Result, Context};
use tokio::time::{interval, Duration, sleep};
use tracing::{info, warn, error, debug};
use rand::Rng;

use crate::config::Config;
use crate::wallpaper;

/// Run daemon in foreground with automatic wallpaper rotation
pub async fn run_foreground(config: Config) -> Result<()> {
    info!("🌊 wallflow daemon starting");
    info!("   Interval: {}m", config.timer.interval);
    info!("   Randomize: {}", config.timer.randomize);
    info!("   Source: {}", config.sources.default);

    // Parse randomization duration
    let randomize_secs = parse_duration(&config.timer.randomize)
        .unwrap_or_else(|_| {
            warn!("Invalid randomize format '{}', using 0", config.timer.randomize);
            0
        });

    // Initial delay if configured
    if let Some(start_delay) = &config.timer.start_delay {
        if let Ok(delay_secs) = parse_duration(start_delay) {
            if delay_secs > 0 {
                info!("Waiting {}s before starting...", delay_secs);
                sleep(Duration::from_secs(delay_secs)).await;
            }
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
pub async fn run_background(config: Config) -> Result<()> {
    // For now, just run in foreground
    // TODO: Implement proper daemonization using daemonize crate
    warn!("Background mode not yet implemented, running in foreground");
    run_foreground(config).await
}

/// Set wallpaper based on configured default source
async fn set_wallpaper_by_source(config: &Config) -> Result<()> {
    match config.sources.default.as_str() {
        "local" => {
            wallpaper::set_local(config, None).await
        }
        "wallhaven" => {
            wallpaper::set_wallhaven(config, &config.sources.category).await
        }
        "picsum" => {
            wallpaper::set_picsum(config).await
        }
        other => {
            warn!("Unknown source '{}', falling back to local", other);
            wallpaper::set_local(config, None).await
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
        return duration_str.parse::<u64>()
            .context("Invalid duration format");
    };

    let number: u64 = number_part.parse()
        .context("Invalid number in duration")?;

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