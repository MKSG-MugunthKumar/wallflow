//! wallflow - Elegant wallpaper management
//!
//! This library provides wallpaper downloading functionality from various online sources.
//! It can be used as a dependency by other crates that need wallpaper fetching capabilities.
//!
//! # Example
//!
//! ```no_run
//! use wallflow::{Config, DownloadOptions, download_from_source};
//!
//! async fn example() -> anyhow::Result<()> {
//!     let config = Config::load_or_default()?;
//!     let opts = DownloadOptions::default();
//!
//!     let wallpaper = download_from_source("wallhaven", &config, &["nature".into()], &opts).await?;
//!     println!("Downloaded: {:?}", wallpaper.file_path);
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod display;
pub mod downloaders;

// Re-export main types for convenience
pub use config::Config;
pub use downloaders::traits::Wallpaper;
pub use downloaders::{DownloadOptions, download_from_source, list_sources};
