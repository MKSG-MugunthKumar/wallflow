use anyhow::{Context, Result, anyhow};
use std::process::Command;
use tracing::{debug, warn};

/// Display resolution information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Resolution {
  pub width: u32,
  pub height: u32,
}

#[allow(dead_code)]
impl Resolution {
  pub fn new(width: u32, height: u32) -> Self {
    Self { width, height }
  }

  pub fn as_string(&self) -> String {
    format!("{}x{}", self.width, self.height)
  }

  /// Parse resolution from string (e.g., "1920x1080")
  pub fn from_string(s: &str) -> Result<Self> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() != 2 {
      return Err(anyhow!("Invalid resolution format: {}. Expected: WIDTHxHEIGHT", s));
    }

    let width = parts[0].parse::<u32>().with_context(|| format!("Invalid width: {}", parts[0]))?;
    let height = parts[1].parse::<u32>().with_context(|| format!("Invalid height: {}", parts[1]))?;

    Ok(Resolution::new(width, height))
  }
}

impl Default for Resolution {
  fn default() -> Self {
    // Fallback resolution if detection fails
    Self::new(2560, 1440)
  }
}

/// Get the resolution of the primary display
#[allow(dead_code)]
pub fn get_primary_display_resolution() -> Result<Resolution> {
  // Try different detection methods based on available tools

  // Method 1: Try macOS system_profiler
  #[cfg(target_os = "macos")]
  if let Ok(resolution) = detect_resolution_macos() {
    debug!("Detected resolution via macOS: {}x{}", resolution.width, resolution.height);
    return Ok(resolution);
  }

  // Method 2: Try xrandr (X11)
  #[cfg(target_os = "linux")]
  if let Ok(resolution) = detect_resolution_xrandr() {
    debug!("Detected resolution via xrandr: {}x{}", resolution.width, resolution.height);
    return Ok(resolution);
  }

  // Method 3: Try swaymsg (Sway/wlroots)
  #[cfg(target_os = "linux")]
  if let Ok(resolution) = detect_resolution_sway() {
    debug!("Detected resolution via sway: {}x{}", resolution.width, resolution.height);
    return Ok(resolution);
  }

  // Method 4: Try wlr-randr (wlroots)
  #[cfg(target_os = "linux")]
  if let Ok(resolution) = detect_resolution_wlr_randr() {
    debug!("Detected resolution via wlr-randr: {}x{}", resolution.width, resolution.height);
    return Ok(resolution);
  }

  // Method 5: Try KDE's kscreen-doctor
  #[cfg(target_os = "linux")]
  if let Ok(resolution) = detect_resolution_kscreen() {
    debug!("Detected resolution via kscreen-doctor: {}x{}", resolution.width, resolution.height);
    return Ok(resolution);
  }

  warn!("Could not detect display resolution, using default");
  Ok(Resolution::default())
}

/// Detect resolution on macOS using system_profiler
#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn detect_resolution_macos() -> Result<Resolution> {
  // Use system_profiler to get display information
  let output = Command::new("system_profiler")
    .args(["SPDisplaysDataType", "-json"])
    .output()
    .context("Failed to execute system_profiler")?;

  if !output.status.success() {
    return Err(anyhow!("system_profiler command failed"));
  }

  let stdout = String::from_utf8(output.stdout).context("Invalid UTF-8 from system_profiler")?;

  // Parse JSON to find resolution
  // Look for "_spdisplays_resolution" or "Resolution" field
  // Format is typically "2560 x 1440" or similar
  let re = regex::Regex::new(r"(\d{3,5})\s*x\s*(\d{3,5})").ok();
  for line in stdout.lines() {
    if line.contains("_spdisplays_resolution") || line.contains("Resolution") {
      // Extract resolution pattern like "2560 x 1440"
      if let Some(ref re) = re
        && let Some(caps) = re.captures(line)
      {
        let width: u32 = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
        let height: u32 = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
        if width > 0 && height > 0 {
          return Ok(Resolution::new(width, height));
        }
      }
    }
  }

  // Fallback: try screenresolution tool if available
  if let Ok(output) = Command::new("screenresolution").arg("get").output() {
    if !output.status.success() {
      return Err(anyhow!("No resolution found via macOS methods"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse output like "Display 0: 2560x1440x32@60Hz"
    for line in stdout.lines() {
      if let Some(res_start) = line.find(char::is_numeric) {
        let res_part = &line[res_start..];
        if let Some(x_pos) = res_part.find('x') {
          let width_str = &res_part[..x_pos];
          let rest = &res_part[x_pos + 1..];
          if let Some(end_pos) = rest.find(|c: char| !c.is_ascii_digit()) {
            let height_str = &rest[..end_pos];
            if let (Ok(width), Ok(height)) = (width_str.parse::<u32>(), height_str.parse::<u32>()) {
              return Ok(Resolution::new(width, height));
            }
          }
        }
      }
    }
  }

  Err(anyhow!("No resolution found via macOS methods"))
}

/// Detect resolution using xrandr (X11)
#[allow(dead_code)]
fn detect_resolution_xrandr() -> Result<Resolution> {
  let output = Command::new("xrandr").arg("--current").output().context("Failed to execute xrandr")?;

  if !output.status.success() {
    return Err(anyhow!("xrandr command failed"));
  }

  let stdout = String::from_utf8(output.stdout).context("Invalid UTF-8 from xrandr")?;

  // Look for primary display or first connected display
  for line in stdout.lines() {
    if line.contains(" connected") && (line.contains("primary") || !line.contains("disconnected")) {
      // Parse line like: "DP-1 connected primary 2560x1440+0+0 (normal left inverted right x axis y axis) 597mm x 336mm"
      let parts: Vec<&str> = line.split_whitespace().collect();
      for part in parts {
        if part.contains('x') && part.chars().next().unwrap_or('a').is_ascii_digit() {
          // Extract resolution part (before any + or other chars)
          let resolution_part = part.split('+').next().unwrap_or(part);
          if let Ok(resolution) = Resolution::from_string(resolution_part) {
            return Ok(resolution);
          }
        }
      }
    }
  }

  Err(anyhow!("No resolution found in xrandr output"))
}

/// Detect resolution using swaymsg (Sway)
#[allow(dead_code)]
fn detect_resolution_sway() -> Result<Resolution> {
  let output = Command::new("swaymsg")
    .args(["-t", "get_outputs"])
    .output()
    .context("Failed to execute swaymsg")?;

  if !output.status.success() {
    return Err(anyhow!("swaymsg command failed"));
  }

  let stdout = String::from_utf8(output.stdout).context("Invalid UTF-8 from swaymsg")?;

  // Parse JSON output (simplified - could use serde_json for robustness)
  // Look for "current_mode":{"width":2560,"height":1440,"refresh":59999}
  for line in stdout.lines() {
    if line.contains("current_mode") && line.contains("width") {
      // Extract width and height from JSON
      if let (Some(width_start), Some(height_start)) = (line.find("\"width\":").map(|i| i + 8), line.find("\"height\":").map(|i| i + 9)) {
        let width_end = line[width_start..].find(',').map(|i| i + width_start).unwrap_or(line.len());
        let height_end = line[height_start..].find(',').map(|i| i + height_start).unwrap_or(line.len());

        if let (Ok(width), Ok(height)) = (line[width_start..width_end].parse::<u32>(), line[height_start..height_end].parse::<u32>()) {
          return Ok(Resolution::new(width, height));
        }
      }
    }
  }

  Err(anyhow!("No resolution found in swaymsg output"))
}

/// Detect resolution using wlr-randr (wlroots)
#[allow(dead_code)]
fn detect_resolution_wlr_randr() -> Result<Resolution> {
  let output = Command::new("wlr-randr").output().context("Failed to execute wlr-randr")?;

  if !output.status.success() {
    return Err(anyhow!("wlr-randr command failed"));
  }

  let stdout = String::from_utf8(output.stdout).context("Invalid UTF-8 from wlr-randr")?;

  // Look for lines like "  2560x1440 @ 59.996 Hz (current)"
  for line in stdout.lines() {
    if line.contains("(current)") {
      let trimmed = line.trim();
      if let Some(resolution_end) = trimmed.find(' ') {
        let resolution_str = &trimmed[..resolution_end];
        if let Ok(resolution) = Resolution::from_string(resolution_str) {
          return Ok(resolution);
        }
      }
    }
  }

  Err(anyhow!("No current resolution found in wlr-randr output"))
}

/// Detect resolution using kscreen-doctor (KDE)
#[allow(dead_code)]
fn detect_resolution_kscreen() -> Result<Resolution> {
  let output = Command::new("kscreen-doctor")
    .arg("-j")
    .output()
    .context("Failed to execute kscreen-doctor")?;

  if !output.status.success() {
    return Err(anyhow!("kscreen-doctor command failed"));
  }

  let stdout = String::from_utf8(output.stdout).context("Invalid UTF-8 from kscreen-doctor")?;

  // Parse JSON output (simplified)
  // Look for enabled outputs with current mode
  if stdout.contains("\"enabled\": true") {
    // This is a simplified parser - in practice, you'd use serde_json
    // Look for resolution patterns in the JSON
    for line in stdout.lines() {
      if line.contains("\"size\"") && line.contains("width") && line.contains("height") {
        // Extract from something like: "size": {"width": 2560, "height": 1440}
        if let (Some(width_start), Some(height_start)) = (line.find("\"width\": ").map(|i| i + 9), line.find("\"height\": ").map(|i| i + 10)) {
          let width_end = line[width_start..].find(',').map(|i| i + width_start).unwrap_or(line.len());
          let height_end = line[height_start..].find('}').map(|i| i + height_start).unwrap_or(line.len());

          if let (Ok(width), Ok(height)) = (
            line[width_start..width_end].trim().parse::<u32>(),
            line[height_start..height_end].trim().parse::<u32>(),
          ) {
            return Ok(Resolution::new(width, height));
          }
        }
      }
    }
  }

  Err(anyhow!("No enabled display found in kscreen-doctor output"))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_resolution_from_string() {
    assert!(Resolution::from_string("1920x1080").is_ok());
    assert!(Resolution::from_string("2560x1440").is_ok());
    assert!(Resolution::from_string("3840x2160").is_ok());

    let res = Resolution::from_string("1920x1080").unwrap();
    assert_eq!(res.width, 1920);
    assert_eq!(res.height, 1080);
    assert_eq!(res.as_string(), "1920x1080");

    // Invalid formats
    assert!(Resolution::from_string("invalid").is_err());
    assert!(Resolution::from_string("1920").is_err());
    assert!(Resolution::from_string("1920x").is_err());
  }
}
