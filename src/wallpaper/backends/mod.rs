// Platform-specific backend modules
#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

pub mod traits;

pub mod registry;

#[cfg(target_os = "linux")]
pub mod awww;

pub use registry::BackendRegistry;
pub use traits::WallpaperBackend;

pub use awww::AwwwBackend;
