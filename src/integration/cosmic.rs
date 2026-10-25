//! COSMIC desktop theme integration.

use std::path::{Path, PathBuf};

use crate::colors::Rgb;

/// Set the COSMIC accent color for light and dark themes.
///
/// COSMIC watches its per-key config files, so the theme updates without a
/// session restart. COSMIC has two on-disk formats; use the newest schema
/// installed on the system.
pub fn set_cosmic_accent(color: &Rgb) {
  let Some(config_dir) = cosmic_config_dir() else {
    return;
  };

  let Some(version) = cosmic_schema_version() else {
    tracing::debug!("COSMIC theme schema not found; skipping accent update");
    return;
  };

  let value = match version.as_str() {
    "v2" => accent_v2(color),
    _ => accent_v1(color),
  };

  for mode in ["Dark", "Light"] {
    let path = config_dir.join(format!("com.system76.CosmicTheme.{mode}")).join(&version).join("accent");
    if let Err(error) = write_config_value(&path, &value) {
      tracing::warn!("Failed to update COSMIC {mode} accent color: {error}");
    } else {
      tracing::info!("Updated COSMIC {mode} accent color");
    }
  }
}

fn cosmic_config_dir() -> Option<PathBuf> {
  if let Some(xdg_config_home) = std::env::var_os("XDG_CONFIG_HOME").filter(|p| !p.is_empty()) {
    return Some(PathBuf::from(xdg_config_home).join("cosmic"));
  }
  std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/cosmic"))
}

fn cosmic_schema_version() -> Option<String> {
  ["v2", "v1"]
    .into_iter()
    .find(|version| {
      Path::new("/usr/share/cosmic/com.system76.CosmicTheme.Dark")
        .join(version)
        .join("accent")
        .is_file()
    })
    .map(str::to_owned)
}

fn write_config_value(path: &Path, value: &str) -> std::io::Result<()> {
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)?;
  }
  std::fs::write(path, value)
}

fn accent_v1(color: &Rgb) -> String {
  let hover = color.darkened(0.1);
  let pressed = color.darkened(0.35);
  let on_disabled = color.darkened(0.5);
  format!(
    "(base: {}, hover: {}, pressed: {}, selected: {}, selected_text: {}, focus: {}, divider: {}, on: {}, disabled: {}, on_disabled: {}, border: {}, disabled_border: {})",
    rgba(color, 1.0),
    rgba(&hover, 1.0),
    rgba(&pressed, 1.0),
    rgba(&hover, 1.0),
    rgba(color, 1.0),
    rgba(color, 1.0),
    rgba(&Rgb::new(0.0, 0.0, 0.0), 1.0),
    rgba(&on_color(color), 1.0),
    rgba(color, 1.0),
    rgba(&on_disabled, 1.0),
    rgba(color, 1.0),
    rgba(color, 0.5),
  )
}

fn accent_v2(color: &Rgb) -> String {
  let hover = color.darkened(0.1);
  let pressed = color.darkened(0.35);
  let on_disabled = color.darkened(0.5);
  format!(
    "(base: \"{}\", hover: \"{}\", pressed: \"{}\", selected: \"{}\", selected_text: \"{}\", focus: \"{}\", divider: \"{}\", on: \"{}\", disabled: \"{}\", on_disabled: \"{}\", border: \"{}\", disabled_border: \"{}\")",
    hex(color, 1.0),
    hex(&hover, 1.0),
    hex(&pressed, 1.0),
    hex(&hover, 1.0),
    hex(color, 1.0),
    hex(color, 1.0),
    hex(&Rgb::new(0.0, 0.0, 0.0), 1.0),
    hex(&on_color(color), 1.0),
    hex(color, 1.0),
    hex(&on_disabled, 1.0),
    hex(color, 1.0),
    hex(color, 0.5),
  )
}

fn rgba(color: &Rgb, alpha: f32) -> String {
  format!(
    "(red: {:.7}, green: {:.7}, blue: {:.7}, alpha: {:.1})",
    color.r.clamp(0.0, 1.0),
    color.g.clamp(0.0, 1.0),
    color.b.clamp(0.0, 1.0),
    alpha.clamp(0.0, 1.0),
  )
}

fn hex(color: &Rgb, alpha: f32) -> String {
  let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
  format!(
    "#{:02X}{:02X}{:02X}{:02X}",
    channel(color.r),
    channel(color.g),
    channel(color.b),
    channel(alpha),
  )
}

fn on_color(color: &Rgb) -> Rgb {
  if color.luminance() > 0.45 {
    Rgb::new(0.0, 0.0, 0.0)
  } else {
    Rgb::new(1.0, 1.0, 1.0)
  }
}
