use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod config;
mod daemon;
mod display;
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
  Wallhaven {
    /// Search category (nature, abstract, etc.)
    category: Option<String>,
  },
  /// Set random photo from Picsum
  Picsum,
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
}

#[tokio::main]
async fn main() -> Result<()> {
  let cli = Cli::parse();

  // Initialize logging
  let log_level = if cli.verbose { "debug" } else { "info" };
  tracing_subscriber::fmt()
    .with_env_filter(format!("wallflow={}", log_level))
    .with_target(false)
    .init();

  // Load configuration (goodbye AWK nightmare!)
  let mut config = if let Some(config_path) = cli.config {
    Config::load(&config_path)?
  } else {
    Config::load_or_default()?
  };

  // Expand environment variables in paths
  config.expand_paths()?;

  info!("🌊 wallflow starting");

  // Execute command
  match cli.command {
    Commands::Local => {
      info!("Setting local wallpaper");
      wallpaper::set_local(&config).await?;
    }
    Commands::Wallhaven { category } => {
      let search_category = category.unwrap_or(config.sources.category.clone());
      info!("Downloading wallpaper from Wallhaven (category: {})", search_category);
      wallpaper::set_wallhaven(&config, &search_category).await?;
    }
    Commands::Picsum => {
      info!("Downloading random photo from Picsum");
      wallpaper::set_picsum(&config).await?;
    }
    Commands::Daemon { foreground } => {
      if foreground {
        info!("Starting daemon in foreground mode");
        daemon::run_foreground(config).await?;
      } else {
        info!("Starting daemon in background mode");
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
      println!("  # Add to your shell startup script for auto-start:");
      println!("  echo 'wallflow daemon &' >> ~/.zshrc");
    }
  }

  info!("✨ wallflow completed successfully");
  Ok(())
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
  println!("  Category: {}", config.sources.category);
  println!();
  println!("Integration:");
  println!("  Pywal: {}", if config.integration.pywal.enabled { "enabled" } else { "disabled" });
  println!(
    "  Terminal colors: {}",
    if config.integration.desktop.update_terminal_colors {
      "enabled"
    } else {
      "disabled"
    }
  );

  Ok(())
}
