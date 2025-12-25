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
mod updater;
mod wallpaper;

use config::Config;

/// Elegant wallpaper management with smooth transitions 🌊
#[derive(Parser)]
#[command(name = "wallflow")]
#[command(version = env!("CARGO_PKG_VERSION"))]
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

  /// Config file path (default: ~/.config/mksg/wallflow/config.yml)
  #[arg(short, long)]
  config: Option<std::path::PathBuf>,

  /// Output directory for downloaded wallpapers (overrides config)
  #[arg(short, long)]
  output: Option<std::path::PathBuf>,

  /// Download only, don't set as wallpaper
  #[arg(long)]
  no_set: bool,
}

#[derive(Subcommand)]
enum Commands {
  /// Set wallpaper from local collection
  Local,
  /// Download and set wallpaper from Wallhaven (accepts search terms)
  Wallhaven {
    /// Search terms (e.g., "nature mountains")
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,
  },
  /// Set random photo from Picsum
  Picsum,
  /// Download NASA Astronomy Picture of the Day
  Apod,
  /// Download Bing Photo of the Day
  Bing,
  /// Download wallpaper from Reddit (accepts subreddit name)
  Reddit {
    /// Subreddit name (e.g., "earthporn", "wallpapers+cityporn")
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,
  },
  /// Download satellite imagery from Google Earth View
  Earthview,
  /// Download high-resolution photo from Unsplash (accepts search topics)
  Unsplash {
    /// Search topics (e.g., "nature", "architecture")
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,
  },
  /// Run as background daemon with automatic rotation
  Daemon {
    #[command(subcommand)]
    daemon_command: DaemonCommands,
  },
  /// Show current configuration
  Config,
  /// Show usage examples and setup guide
  Examples,
  /// Show platform information and available backends
  PlatformInfo,
  /// List all available wallpaper backends
  ListBackends,
  /// List all available wallpaper sources
  ListSources,
  /// Launch interactive TUI for wallpaper browsing
  Tui,
  /// Check for updates and optionally install them
  Update {
    /// Only check for updates, don't install
    #[arg(short, long)]
    check: bool,
  },
}

#[derive(Subcommand)]
enum DaemonCommands {
  /// Start daemon
  Start {
    /// Run in foreground (don't daemonize)
    #[arg(short, long)]
    foreground: bool,
  },

  /// Stop running daemon
  Stop,

  /// Show daemon status
  Status,

  /// Restart daemon
  Restart,

  /// Reload daemon configuration (sends SIGHUP)
  Reload,

  /// Install daemon to run at system startup
  Install,

  /// Uninstall daemon from system startup
  Uninstall,
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  // Load configuration first (needed for logging setup)
  let mut config = if let Some(config_path) = &cli.config {
    Config::load(config_path)?
  } else {
    Config::load_or_default()?
  };

  // Expand environment variables in paths
  config.expand_paths()?;

  // Initialize enhanced logging system
  logging::init_logging(&config, cli.verbose)?;

  // Log system information and configuration details
  logging::log_system_info(&config);

  // Handle daemon commands that don't need tokio runtime BEFORE creating it
  // (daemonize fork + new runtime doesn't work from within an existing runtime)
  if let Commands::Daemon { daemon_command } = &cli.command {
    match daemon_command {
      DaemonCommands::Stop => return daemon::stop_daemon(),
      DaemonCommands::Reload => return daemon::reload_daemon(),
      DaemonCommands::Status => return daemon::status_daemon(),
      DaemonCommands::Install => return daemon::install_daemon(),
      DaemonCommands::Uninstall => return daemon::uninstall_daemon(),
      DaemonCommands::Start { foreground: false } => return daemon::run_background(config),
      DaemonCommands::Restart => {
        // Stop if running, then start in background
        let _ = daemon::stop_daemon(); // Ignore error if not running
        std::thread::sleep(std::time::Duration::from_secs(1));
        return daemon::run_background(config);
      }
      DaemonCommands::Start { foreground: true } => {
        // Fall through to async runtime for foreground mode
      }
    }
  }

  // Create tokio runtime for all other commands
  let rt = tokio::runtime::Runtime::new()?;
  rt.block_on(async_main(cli, config))
}

async fn async_main(cli: Cli, config: Config) -> Result<()> {
  // Build download options from CLI flags
  let download_opts = downloaders::DownloadOptions {
    output_dir: cli.output.clone(),
    no_set: cli.no_set,
  };

  // Execute command
  match cli.command {
    Commands::Local => {
      wallpaper::set_local(&config).await?;
    }
    Commands::Wallhaven { query } => {
      wallpaper::set_from_source(&config, "wallhaven", &query, &download_opts).await?;
    }
    Commands::Picsum => {
      wallpaper::set_from_source(&config, "picsum", &[], &download_opts).await?;
    }
    Commands::Apod => {
      wallpaper::set_from_source(&config, "apod", &[], &download_opts).await?;
    }
    Commands::Bing => {
      wallpaper::set_from_source(&config, "bing", &[], &download_opts).await?;
    }
    Commands::Reddit { query } => {
      wallpaper::set_from_source(&config, "reddit", &query, &download_opts).await?;
    }
    Commands::Earthview => {
      wallpaper::set_from_source(&config, "earthview", &[], &download_opts).await?;
    }
    Commands::Unsplash { query } => {
      wallpaper::set_from_source(&config, "unsplash", &query, &download_opts).await?;
    }
    Commands::Daemon { daemon_command } => {
      // Most daemon commands are handled in main() before runtime creation
      // Only foreground mode reaches here
      if let DaemonCommands::Start { foreground: true } = daemon_command {
        daemon::run_foreground(config).await?;
      } else {
        unreachable!("Non-foreground daemon commands should be handled before async runtime");
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
      println!("  # Daemon management");
      println!("  wallflow daemon start              # Start in background");
      println!("  wallflow daemon start --foreground # Start in foreground (for testing)");
      println!("  wallflow daemon stop               # Stop running daemon");
      println!("  wallflow daemon status             # Show daemon status");
      println!("  wallflow daemon restart            # Restart daemon");
      println!("  wallflow daemon reload             # Reload configuration");
      println!();
      println!("  # Auto-start at system boot/login");
      println!("  wallflow daemon install            # Install startup service");
      println!("  wallflow daemon uninstall          # Remove startup service");
      println!();
      println!("  # Download from various sources");
      println!("  wallflow wallhaven nature mountains");
      println!("  wallflow reddit earthporn");
      println!("  wallflow unsplash architecture");
      println!("  wallflow bing");
      println!("  wallflow earthview");
      println!("  wallflow apod");
      println!();
      println!("  # Check platform and backends");
      println!("  wallflow platform-info");
      println!("  wallflow list-backends");
      println!("  wallflow list-sources");
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
    Commands::Update { check } => {
      handle_update(check).await?;
    }
  }

  Ok(())
}

async fn handle_update(check_only: bool) -> Result<()> {
  // Check if self-update is possible
  if !updater::can_self_update() {
    println!("Self-update is disabled.");
    println!("wallflow appears to be installed via a package manager.");
    println!("Please update using your package manager instead.");
    return Ok(());
  }

  println!("Checking for updates...");

  match updater::check_for_updates().await {
    Ok(check) => {
      if check.update_available {
        println!("Update available: v{} -> v{}", check.current, check.latest);

        if check_only {
          println!("\nRun 'wallflow update' to install the update.");
        } else {
          println!("\nDownloading and installing update...");
          match updater::perform_update().await {
            Ok(version) => {
              println!("Downloaded v{}", version);
              updater::apply_update()?;
            }
            Err(e) => {
              println!("Failed to download update: {}", e);
              return Err(e);
            }
          }
        }
      } else {
        println!("You're up to date! (v{})", check.current);
      }
    }
    Err(e) => {
      println!("Failed to check for updates: {}", e);
      println!("Check your internet connection or try again later.");
    }
  }

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
  println!();
  println!("Integration:");
  println!("  Pywal: {}", if config.integration.pywal.enabled { "enabled" } else { "disabled" });

  Ok(())
}
