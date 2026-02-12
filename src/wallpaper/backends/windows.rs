//! Windows wallpaper backends (stub for future implementation)

#[cfg(target_os = "windows")]
#[derive(Default)]
pub struct WindowsSystemParametersBackend;

#[cfg(target_os = "windows")]
impl WindowsSystemParametersBackend {
  pub fn new() -> Self {
    Self
  }
}
