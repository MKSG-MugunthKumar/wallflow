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
pub use wal::generate_pywal_colors;

#[cfg(target_os = "macos")]
pub use macos::{
    set_accent_color, set_appearance_mode, toggle_appearance_mode,
    apply_theme_from_wallpaper, get_appearance_mode,
    AccentColor, AppearanceMode,
};
