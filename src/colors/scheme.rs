//! Color scheme data structures

use serde::{Deserialize, Serialize};

/// RGB color with floating-point components (0.0-1.0)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rgb {
  pub r: f32,
  pub g: f32,
  pub b: f32,
}

impl Rgb {
  /// Create a new RGB color
  pub fn new(r: f32, g: f32, b: f32) -> Self {
    Self { r, g, b }
  }

  /// Create from 0-255 integer values
  pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
    Self {
      r: r as f32 / 255.0,
      g: g as f32 / 255.0,
      b: b as f32 / 255.0,
    }
  }

  /// Perceived luminance (ITU-R BT.601)
  pub fn luminance(&self) -> f32 {
    0.299 * self.r + 0.587 * self.g + 0.114 * self.b
  }

  /// HSV saturation
  pub fn saturation(&self) -> f32 {
    let max_c = self.r.max(self.g).max(self.b);
    let min_c = self.r.min(self.g).min(self.b);
    if max_c > 0.0 { (max_c - min_c) / max_c } else { 0.0 }
  }

  /// Hue in degrees (0-360)
  pub fn hue(&self) -> f32 {
    let max_c = self.r.max(self.g).max(self.b);
    let min_c = self.r.min(self.g).min(self.b);
    let delta = max_c - min_c;

    if delta <= 0.0 {
      return 0.0;
    }

    let mut h = if max_c == self.r {
      (self.g - self.b) / delta
    } else if max_c == self.g {
      2.0 + (self.b - self.r) / delta
    } else {
      4.0 + (self.r - self.g) / delta
    };

    h *= 60.0;
    if h < 0.0 {
      h += 360.0;
    }
    h
  }

  /// Lighten the color by a factor (0.0-1.0)
  pub fn lightened(&self, amount: f32) -> Self {
    Self {
      r: (self.r + (1.0 - self.r) * amount).min(1.0),
      g: (self.g + (1.0 - self.g) * amount).min(1.0),
      b: (self.b + (1.0 - self.b) * amount).min(1.0),
    }
  }

  /// Darken the color by a factor (0.0-1.0)
  pub fn darkened(&self, amount: f32) -> Self {
    Self {
      r: (self.r * (1.0 - amount)).max(0.0),
      g: (self.g * (1.0 - amount)).max(0.0),
      b: (self.b * (1.0 - amount)).max(0.0),
    }
  }

  /// Increase saturation by a factor
  pub fn saturated(&self, factor: f32) -> Self {
    let gray = self.luminance();
    Self {
      r: (gray + (self.r - gray) * factor).clamp(0.0, 1.0),
      g: (gray + (self.g - gray) * factor).clamp(0.0, 1.0),
      b: (gray + (self.b - gray) * factor).clamp(0.0, 1.0),
    }
  }

  /// Convert to hex string (e.g., "#FF5500")
  pub fn hex(&self) -> String {
    format!(
      "#{:02X}{:02X}{:02X}",
      (self.r * 255.0) as u8,
      (self.g * 255.0) as u8,
      (self.b * 255.0) as u8
    )
  }

  /// Hex without the # prefix
  pub fn hex_strip(&self) -> String {
    format!(
      "{:02X}{:02X}{:02X}",
      (self.r * 255.0) as u8,
      (self.g * 255.0) as u8,
      (self.b * 255.0) as u8
    )
  }

  /// RGB string in "r, g, b" format (0-255)
  pub fn rgb_string(&self) -> String {
    format!("{}, {}, {}", (self.r * 255.0) as u8, (self.g * 255.0) as u8, (self.b * 255.0) as u8)
  }

  /// RGBA string in "r g b a" format (0.0-1.0 floats) for Xcode themes
  pub fn rgba_string(&self, alpha: f32) -> String {
    format!("{:.6} {:.6} {:.6} {:.2}", self.r, self.g, self.b, alpha)
  }

  /// XRGBA format for some apps
  pub fn xrgba_string(&self) -> String {
    format!(
      "{:02x}/{:02x}/{:02x}/ff",
      (self.r * 255.0) as u8,
      (self.g * 255.0) as u8,
      (self.b * 255.0) as u8
    )
  }

  /// Squared Euclidean distance to another color
  pub fn distance_squared(&self, other: &Rgb) -> f32 {
    let dr = self.r - other.r;
    let dg = self.g - other.g;
    let db = self.b - other.b;
    dr * dr + dg * dg + db * db
  }
}

impl Default for Rgb {
  fn default() -> Self {
    Self::new(0.0, 0.0, 0.0)
  }
}

/// A complete color scheme for terminal theming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
  /// Path to the source wallpaper
  pub wallpaper: String,

  /// Whether this is a dark color scheme
  pub is_dark: bool,

  /// Alpha/opacity (0-100)
  pub alpha: u8,

  /// Background color
  pub background: Rgb,

  /// Foreground/text color
  pub foreground: Rgb,

  /// Cursor color
  pub cursor: Rgb,

  /// Terminal colors (color0-color15)
  pub colors: Vec<Rgb>,
}

impl ColorScheme {
  /// Create a new color scheme
  pub fn new(wallpaper: String, is_dark: bool, background: Rgb, foreground: Rgb, cursor: Rgb, colors: Vec<Rgb>) -> Self {
    Self {
      wallpaper,
      is_dark,
      alpha: 100,
      background,
      foreground,
      cursor,
      colors,
    }
  }

  /// Serialize to JSON
  pub fn to_json(&self) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(self)
  }

  /// Deserialize from JSON
  pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
    serde_json::from_str(json)
  }

  /// Export as shell variables (pywal-compatible)
  pub fn to_shell_format(&self) -> String {
    let mut lines = Vec::new();

    lines.push(format!("wallpaper='{}'", self.wallpaper));
    lines.push(format!("background='{}'", self.background.hex()));
    lines.push(format!("foreground='{}'", self.foreground.hex()));
    lines.push(format!("cursor='{}'", self.cursor.hex()));

    for (i, color) in self.colors.iter().enumerate() {
      lines.push(format!("color{}='{}'", i, color.hex()));
    }

    lines.join("\n")
  }

  /// Export as CSS custom properties
  pub fn to_css_format(&self) -> String {
    let mut lines = Vec::new();

    lines.push(":root {".to_string());
    lines.push(format!("  --background: {};", self.background.hex()));
    lines.push(format!("  --foreground: {};", self.foreground.hex()));
    lines.push(format!("  --cursor: {};", self.cursor.hex()));

    for (i, color) in self.colors.iter().enumerate() {
      lines.push(format!("  --color{}: {};", i, color.hex()));
    }

    lines.push("}".to_string());
    lines.join("\n")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_rgb_hex() {
    let color = Rgb::new(1.0, 0.5, 0.0);
    assert_eq!(color.hex(), "#FF7F00");
  }

  #[test]
  fn test_rgb_luminance() {
    let white = Rgb::new(1.0, 1.0, 1.0);
    let black = Rgb::new(0.0, 0.0, 0.0);
    assert!((white.luminance() - 1.0).abs() < 0.001);
    assert!(black.luminance().abs() < 0.001);
  }

  #[test]
  fn test_rgb_lightened() {
    let color = Rgb::new(0.5, 0.5, 0.5);
    let lighter = color.lightened(0.5);
    assert!(lighter.r > color.r);
    assert!(lighter.g > color.g);
    assert!(lighter.b > color.b);
  }

  #[test]
  fn test_color_scheme_json() {
    let scheme = ColorScheme::new(
      "/path/to/wallpaper.jpg".to_string(),
      true,
      Rgb::new(0.1, 0.1, 0.1),
      Rgb::new(0.9, 0.9, 0.9),
      Rgb::new(0.8, 0.8, 0.8),
      vec![Rgb::new(0.0, 0.0, 0.0); 16],
    );

    let json = scheme.to_json().unwrap();
    let parsed = ColorScheme::from_json(&json).unwrap();
    assert_eq!(parsed.wallpaper, scheme.wallpaper);
    assert_eq!(parsed.is_dark, scheme.is_dark);
  }
}
