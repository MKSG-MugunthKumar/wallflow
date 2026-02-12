//! Desktop environment integrations
//!
//! This module provides integrations with various desktop environments
//! and tools for enhanced wallpaper management:
//!
//! - **KDE Plasma**: Ensures KDE apps inherit wallpaper colors
//! - **pywal**: Generates color schemes from wallpapers (Linux)
//! - **macOS**: Controls system appearance and accent colors

mod kde;
mod wal;

#[cfg(target_os = "macos")]
pub mod macos;

pub use kde::set_kde_wallpaper;

#[cfg(target_os = "macos")]
#[allow(unused_imports)]
pub use macos::{
  AccentColor, AppearanceMode, apply_theme_from_wallpaper, get_appearance_mode, set_accent_color, set_appearance_mode, toggle_appearance_mode,
};
