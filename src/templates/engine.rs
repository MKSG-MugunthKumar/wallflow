//! Template rendering engine
//!
//! Renders templates by replacing `{variable}` placeholders with color values.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::colors::ColorScheme;

use super::manifest::{ReloadConfig, TemplateManifest};

/// Result of rendering a template bundle
pub struct RenderedTemplate {
  /// Output file path
  pub output_path: String,
  /// Reload config from the manifest, if any
  pub reload: Option<ReloadConfig>,
}

/// Template rendering engine
pub struct TemplateEngine;

impl TemplateEngine {
  /// Build a variable map from a color scheme
  ///
  /// This creates all the pywal-compatible variables like:
  /// - `{background}`, `{foreground}`, `{cursor}`
  /// - `{color0}` through `{color15}`
  /// - `{color0.strip}`, `{color0.rgb}`, `{color0.rgba}`, etc.
  pub fn build_variables(scheme: &ColorScheme) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    // Special colors
    vars.insert("wallpaper".to_string(), scheme.wallpaper.clone());
    vars.insert("alpha".to_string(), scheme.alpha.to_string());
    vars.insert("background".to_string(), scheme.background.hex());
    vars.insert("foreground".to_string(), scheme.foreground.hex());
    vars.insert("cursor".to_string(), scheme.cursor.hex());

    // Alpha variants
    let alpha_decimal = scheme.alpha as f32 / 100.0;
    vars.insert("background.alpha".to_string(), scheme.alpha.to_string());
    vars.insert("background.alpha_dec".to_string(), format!("{:.2}", alpha_decimal));

    // Color0-Color15 with all variants
    for (i, color) in scheme.colors.iter().enumerate() {
      vars.insert(format!("color{}", i), color.hex());
      vars.insert(format!("color{}.strip", i), color.hex_strip());
      vars.insert(format!("color{}.rgb", i), color.rgb_string());
      vars.insert(format!("color{}.xrgba", i), color.xrgba_string());
      vars.insert(format!("color{}.rgba", i), color.rgba_string(1.0));
      vars.insert(format!("color{}.rgba_25", i), color.rgba_string(0.25));
      // Individual RGB float components (for iTerm2 Dynamic Profiles)
      vars.insert(format!("color{}.r", i), format!("{:.10}", color.r));
      vars.insert(format!("color{}.g", i), format!("{:.10}", color.g));
      vars.insert(format!("color{}.b", i), format!("{:.10}", color.b));
    }

    // Strip variants for special colors
    vars.insert("background.strip".to_string(), scheme.background.hex_strip());
    vars.insert("foreground.strip".to_string(), scheme.foreground.hex_strip());
    vars.insert("cursor.strip".to_string(), scheme.cursor.hex_strip());

    // RGB variants
    vars.insert("background.rgb".to_string(), scheme.background.rgb_string());
    vars.insert("foreground.rgb".to_string(), scheme.foreground.rgb_string());
    vars.insert("cursor.rgb".to_string(), scheme.cursor.rgb_string());

    // RGBA variants
    vars.insert("background.rgba".to_string(), scheme.background.rgba_string(1.0));
    vars.insert("foreground.rgba".to_string(), scheme.foreground.rgba_string(1.0));
    vars.insert("cursor.rgba".to_string(), scheme.cursor.rgba_string(1.0));

    // Individual RGB float components for special colors
    vars.insert("background.r".to_string(), format!("{:.10}", scheme.background.r));
    vars.insert("background.g".to_string(), format!("{:.10}", scheme.background.g));
    vars.insert("background.b".to_string(), format!("{:.10}", scheme.background.b));
    vars.insert("foreground.r".to_string(), format!("{:.10}", scheme.foreground.r));
    vars.insert("foreground.g".to_string(), format!("{:.10}", scheme.foreground.g));
    vars.insert("foreground.b".to_string(), format!("{:.10}", scheme.foreground.b));
    vars.insert("cursor.r".to_string(), format!("{:.10}", scheme.cursor.r));
    vars.insert("cursor.g".to_string(), format!("{:.10}", scheme.cursor.g));
    vars.insert("cursor.b".to_string(), format!("{:.10}", scheme.cursor.b));

    vars
  }

  /// Render a template string by replacing `{variable}` placeholders
  pub fn render(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();

    for (key, value) in variables {
      result = result.replace(&format!("{{{}}}", key), value);
    }

    result
  }

  /// Render a template file and return the result
  pub fn render_file<P: AsRef<Path>>(template_path: P, scheme: &ColorScheme) -> Result<String> {
    let template = fs::read_to_string(template_path.as_ref()).context("Failed to read template file")?;

    let variables = Self::build_variables(scheme);
    Ok(Self::render(&template, &variables))
  }

  /// Render a template bundle and write to output directory
  pub fn render_bundle<P: AsRef<Path>, Q: AsRef<Path>>(bundle_path: P, output_dir: Q, scheme: &ColorScheme) -> Result<RenderedTemplate> {
    let bundle = bundle_path.as_ref();
    let output = output_dir.as_ref();

    // Load manifest
    let manifest_path = bundle.join("manifest.json");
    let manifest = TemplateManifest::load(&manifest_path).context("Failed to load manifest.json")?;

    // Load and render template
    let template_path = bundle.join(&manifest.template.file);
    let rendered = Self::render_file(&template_path, scheme)?;

    // Write to output
    fs::create_dir_all(output).context("Failed to create output directory")?;

    let output_path = output.join(&manifest.template.output_name);
    fs::write(&output_path, &rendered).context("Failed to write output file")?;

    Ok(RenderedTemplate {
      output_path: output_path.to_string_lossy().to_string(),
      reload: manifest.reload,
    })
  }

  /// Render all template bundles in a directory
  pub fn render_all<P: AsRef<Path>, Q: AsRef<Path>>(templates_dir: P, output_dir: Q, scheme: &ColorScheme) -> Result<Vec<RenderedTemplate>> {
    let templates = templates_dir.as_ref();
    let output = output_dir.as_ref();
    let mut rendered = Vec::new();

    // Find all .wallflowtemplate bundles
    if !templates.exists() {
      return Ok(rendered);
    }

    for entry in fs::read_dir(templates)? {
      let entry = entry?;
      let path = entry.path();

      if path.is_dir() && path.extension().map(|e| e == "wallflowtemplate").unwrap_or(false) {
        match Self::render_bundle(&path, output, scheme) {
          Ok(result) => {
            rendered.push(result);
          }
          Err(e) => {
            eprintln!("Warning: Failed to render template {:?}: {}", path.file_name(), e);
          }
        }
      }
    }

    Ok(rendered)
  }

  /// Send reload signals to apps based on rendered template manifests
  pub fn notify_apps(rendered: &[RenderedTemplate]) {
    // Small delay to ensure template files are fully flushed before signalling
    std::thread::sleep(std::time::Duration::from_millis(50));

    for rt in rendered {
      if let Some(ref reload) = rt.reload {
        let signal_arg = format!("-{}", reload.signal);
        let _ = std::process::Command::new("pkill")
          .args([&signal_arg, &reload.process_name])
          .stdout(std::process::Stdio::null())
          .stderr(std::process::Stdio::null())
          .status();
      }
    }
  }

  /// Get the default output directory
  ///
  /// - Linux/macOS: `~/.cache/mksg/wallflow`
  /// - Windows: `%LOCALAPPDATA%\mksg\wallflow\cache`
  pub fn default_output_dir() -> std::path::PathBuf {
    if cfg!(windows) {
      dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mksg")
        .join("wallflow")
    } else {
      dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cache")
        .join("mksg")
        .join("wallflow")
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::colors::Rgb;

  fn test_scheme() -> ColorScheme {
    ColorScheme::new(
      "/path/to/wallpaper.jpg".to_string(),
      true,
      Rgb::new(0.1, 0.1, 0.1),
      Rgb::new(0.9, 0.9, 0.9),
      Rgb::new(0.8, 0.2, 0.2),
      vec![Rgb::new(0.0, 0.0, 0.0); 16],
    )
  }

  #[test]
  fn test_build_variables() {
    let scheme = test_scheme();
    let vars = TemplateEngine::build_variables(&scheme);

    assert!(vars.contains_key("background"));
    assert!(vars.contains_key("foreground"));
    assert!(vars.contains_key("color0"));
    assert!(vars.contains_key("color15"));
    assert!(vars.contains_key("color0.strip"));
    assert!(vars.contains_key("background.alpha_dec"));
  }

  #[test]
  fn test_render() {
    let scheme = test_scheme();
    let vars = TemplateEngine::build_variables(&scheme);

    let template = "background: {background}\nforeground: {foreground}";
    let rendered = TemplateEngine::render(template, &vars);

    assert!(rendered.contains("#191919")); // 0.1 * 255 = ~25 = 0x19
    assert!(rendered.contains("#E5E5E5")); // 0.9 * 255 = ~229 = 0xE5
  }

  #[test]
  fn test_render_preserves_unknown() {
    let vars = HashMap::new();
    let template = "known and {unknown}";
    let rendered = TemplateEngine::render(template, &vars);

    assert_eq!(rendered, "known and {unknown}");
  }
}
