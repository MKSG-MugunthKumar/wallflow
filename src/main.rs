use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod config;
mod daemon;
mod daemon_status;
mod display;
mod downloaders;
mod integration;
mod logging;
mod platform;
mod tui;
mod wallpaper;

use config::Config;

/// Elegant wallpaper management with smooth transitions 🌊
#[derive(Parser)]
#[command(name = "wallflow")]
#[command(version = "0.1.0")]
#[command(about = "Elegant wallpaper management with smooth transitions")]
#[command(
  long_about = "wallflow brings fluidity to your desktop with beautiful wallpaper transitions, dynamic color schemes, and seamless desktop integration."
)]
struct Cli {
  #[command(subcommand)]
  command: Commands,

  /// Enable verbose logging
  #[arg(short, long)]
  verbose: bool,

  /// Config file path (default: ~/.config/wallflow/config.yml)
  #[arg(short, long)]
  config: Option<std::path::PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
  /// Set wallpaper from local collection
  Local,
  /// Download and set wallpaper from Wallhaven
  Wallhaven,
  /// Set random photo from Picsum
  Picsum,
  /// Download NASA Astronomy Picture of the Day
  Apod,
  /// Run as background daemon with automatic rotation
  Daemon {
    /// Run in foreground (don't daemonize)
    #[arg(short, long)]
    foreground: bool,
  },
  /// Show current configuration
  Config,
  /// Show usage examples and setup guide
  Examples,
  /// Show platform information and available backends
  PlatformInfo,
  /// List all available wallpaper backends
  ListBackends,
  /// Test a specific wallpaper backend
  TestBackend {
    /// Backend name to test
    backend: String,
    /// Path to test image (optional, uses default if not provided)
    image: Option<std::path::PathBuf>,
  },
  /// List all available wallpaper sources
  ListSources,
  /// Launch interactive TUI for wallpaper browsing
  Tui,
}

#[tokio::main]
async fn main() -> Result<()> {
  let cli = Cli::parse();

  // Load configuration first (needed for logging setup)
  let mut config = if let Some(config_path) = cli.config {
    Config::load(&config_path)?
  } else {
    Config::load_or_default()?
  };

  // Expand environment variables in paths
  config.expand_paths()?;

  // Initialize enhanced logging system
  logging::init_logging(&config, cli.verbose)?;

  // Log system information and configuration details
  logging::log_system_info(&config);

  // Execute command
  match cli.command {
    Commands::Local => {
      wallpaper::set_local(&config).await?;
    }
    Commands::Wallhaven => {
      wallpaper::set_wallhaven(&config).await?;
    }
    Commands::Picsum => {
      wallpaper::set_picsum(&config).await?;
    }
    Commands::Apod => {
      wallpaper::set_apod(&config).await?;
    }
    Commands::Daemon { foreground } => {
      if foreground {
        daemon::run_foreground(config).await?;
      } else {
        daemon::run_background(config).await?;
      }
    }
    Commands::Config => {
      show_config(&config)?;
    }
    Commands::Examples => {
      println!("🌊 wallflow Usage Examples");
      println!();
      println!("  # Set wallpaper from local collection");
      println!("  wallflow local");
      println!();
      println!("  # Start daemon (background)");
      println!("  wallflow daemon");
      println!();
      println!("  # Start daemon (foreground for testing)");
      println!("  wallflow daemon --foreground");
      println!();
      println!("  # Download from Wallhaven");
      println!("  wallflow wallhaven nature");
      println!();
      println!("  # Check platform and backends");
      println!("  wallflow platform-info");
      println!("  wallflow list-backends");
      println!();
      println!("  # Test a specific backend");
      println!("  wallflow test-backend swww");
      println!();
      println!("  # Add to your shell startup script for auto-start:");
      println!("  echo 'wallflow daemon &' >> ~/.zshrc");
    }
    Commands::PlatformInfo => {
      let info = wallpaper::platform_info()?;
      println!("🌊 wallflow Platform Information");
      println!();
      println!("{}", info);
    }
    Commands::ListBackends => {
      let backends = wallpaper::list_backends();
      println!("🌊 wallflow Available Backends");
      println!();
      for backend in backends {
        println!("  {}", backend);
      }
    }
    Commands::TestBackend { backend, image } => {
      let test_image = if let Some(img) = image {
        img
      } else {
        // Use a default test image or the first local wallpaper
        get_default_test_image(&config)?
      };

      info!("Testing backend '{}' with image: {}", backend, test_image.display());
      wallpaper::test_backend(&backend, &test_image, &config).await?;
    }
    Commands::ListSources => {
      let sources = downloaders::list_sources();
      println!("🌊 wallflow Available Wallpaper Sources");
      println!();
      for source in sources {
        println!("  {}", source);
      }
    }
    Commands::Tui => {
      info!("🎨 Launching TUI wallpaper browser");
      tui::run_with_default_terminal(config).await?;
    }
  }

  Ok(())
}

fn get_default_test_image(config: &Config) -> Result<std::path::PathBuf> {
  use std::path::Path;

  // Try to find the first image in the local wallpaper directory
  let wallpaper_dir = Path::new(&config.paths.local);

  if wallpaper_dir.exists() {
    for entry in std::fs::read_dir(wallpaper_dir)? {
      let entry = entry?;
      let path = entry.path();

      if path.is_file()
        && let Some(extension) = path.extension().and_then(|ext| ext.to_str())
        && config.sources.local.formats.iter().any(|fmt| fmt.eq_ignore_ascii_case(extension))
      {
        return Ok(path);
      }
    }
  }

  // If no local images found, suggest creating a test image
  Err(anyhow::anyhow!(
    "No test images found in {}. Please provide a test image path or add images to your local wallpaper directory.",
    wallpaper_dir.display()
  ))
}

fn show_config(config: &Config) -> Result<()> {
  println!("🌊 wallflow Configuration");
  println!();
  println!("Paths:");
  println!("  Local wallpapers: {}", config.paths.local);
  println!("  Downloads: {}", config.paths.downloads);
  println!();
  println!("Transition:");
  match &config.transition.transition_type {
    config::TransitionType::Single(t) => {
      println!("  Type: {} ({}s)", t, config.transition.duration)
    }
    config::TransitionType::Multiple(types) => {
      println!("  Types: {} ({}s)", types.join(", "), config.transition.duration)
    }
  }
  println!();
  println!("Sources:");
  println!("  Default: {}", config.sources.default);
  println!();
  println!("Integration:");
  println!("  Pywal: {}", if config.integration.pywal.enabled { "enabled" } else { "disabled" });

  Ok(())
}
