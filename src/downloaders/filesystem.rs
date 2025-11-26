use chrono::Local;

pub struct FilesystemHelper;

impl FilesystemHelper {
  pub fn make_file_suffix() -> String {
    Local::now().format("%Y%m%d_%H%M%S").to_string()
  }
}
