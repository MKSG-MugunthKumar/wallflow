//! macOS wallpaper backends (stub for future implementation)

#[cfg(target_os = "macos")]
pub struct MacOSWallpaperBackend;

#[cfg(target_os = "macos")]
impl MacOSWallpaperBackend {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "macos")]
pub struct AppleScriptBackend;

#[cfg(target_os = "macos")]
impl AppleScriptBackend {
    pub fn new() -> Self {
        Self
    }
}