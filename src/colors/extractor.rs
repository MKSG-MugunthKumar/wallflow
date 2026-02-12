//! Color extraction using k-means clustering
//!
//! This is a Rust port of the Swift ColorExtractor, which uses
//! k-means++ initialization and iterative refinement.

use std::path::Path;

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, imageops::FilterType};
use rand::Rng;

use super::scheme::{ColorScheme, Rgb};

/// Options for color extraction
#[derive(Debug, Clone)]
pub struct ExtractionOptions {
  /// Number of colors to extract (default: 16)
  pub color_count: usize,

  /// Force dark (Some(true)), light (Some(false)), or auto-detect (None)
  pub prefers_dark: Option<bool>,

  /// WCAG-inspired contrast level (1.5 = low, 4.5 = AAA)
  pub contrast_ratio: f32,

  /// How much to adjust background (0.3 = subtle, 0.9 = intense)
  pub background_intensity: f32,
}

impl Default for ExtractionOptions {
  fn default() -> Self {
    Self {
      color_count: 16,
      prefers_dark: None,
      contrast_ratio: 3.0,
      background_intensity: 0.6,
    }
  }
}

/// Extracts dominant colors from images using k-means clustering
pub struct ColorExtractor {
  /// Maximum dimension for resized image (for performance)
  max_dimension: u32,

  /// Sample every Nth pixel (for speed)
  sample_step: u32,

  /// Maximum k-means iterations
  max_iterations: usize,
}

impl Default for ColorExtractor {
  fn default() -> Self {
    Self::new()
  }
}

impl ColorExtractor {
  /// Create a new color extractor with default settings
  pub fn new() -> Self {
    Self {
      max_dimension: 200,
      sample_step: 4,
      max_iterations: 20,
    }
  }

  /// Extract a color scheme from an image file
  pub fn extract<P: AsRef<Path>>(&self, image_path: P, options: &ExtractionOptions) -> Result<ColorScheme> {
    let path = image_path.as_ref();
    let img = image::open(path).context("Failed to open image")?;

    self.extract_from_image(&img, path.to_string_lossy().to_string(), options)
  }

  /// Extract a color scheme from a DynamicImage
  pub fn extract_from_image(&self, image: &DynamicImage, wallpaper_path: String, options: &ExtractionOptions) -> Result<ColorScheme> {
    // 1. Resize image for performance
    let resized = self.resize_image(image);

    // 2. Sample pixels
    let pixels = self.sample_pixels(&resized);

    if pixels.is_empty() {
      anyhow::bail!("No valid pixels found in image");
    }

    // 3. K-means clustering
    let mut centroids = self.kmeans(&pixels, options.color_count);

    // Sort by luminance (darkest first)
    centroids.sort_by(|a, b| a.luminance().partial_cmp(&b.luminance()).unwrap());

    // 4. Generate color scheme
    Ok(self.generate_scheme(wallpaper_path, centroids, options))
  }

  /// Resize image to max_dimension while preserving aspect ratio
  fn resize_image(&self, image: &DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();

    if width <= self.max_dimension && height <= self.max_dimension {
      return image.clone();
    }

    let scale = self.max_dimension as f32 / width.max(height) as f32;
    let new_width = (width as f32 * scale) as u32;
    let new_height = (height as f32 * scale) as u32;

    image.resize(new_width, new_height, FilterType::Triangle)
  }

  /// Sample pixels from the image, filtering out transparent and extreme values
  fn sample_pixels(&self, image: &DynamicImage) -> Vec<Rgb> {
    let (width, height) = image.dimensions();
    let rgba = image.to_rgba8();
    let mut pixels = Vec::with_capacity((width * height / 16) as usize);

    for y in (0..height).step_by(self.sample_step as usize) {
      for x in (0..width).step_by(self.sample_step as usize) {
        let pixel = rgba.get_pixel(x, y);
        let [r, g, b, a] = pixel.0;

        // Skip transparent pixels
        if a < 200 {
          continue;
        }

        let rf = r as f32 / 255.0;
        let gf = g as f32 / 255.0;
        let bf = b as f32 / 255.0;

        // Skip near-black and near-white
        let brightness = (rf + gf + bf) / 3.0;
        if brightness > 0.08 && brightness < 0.92 {
          pixels.push(Rgb::new(rf, gf, bf));
        }
      }
    }

    // If too filtered, sample without brightness filter
    if pixels.len() < 100 {
      pixels.clear();
      for y in (0..height).step_by(self.sample_step as usize) {
        for x in (0..width).step_by(self.sample_step as usize) {
          let pixel = rgba.get_pixel(x, y);
          let [r, g, b, _] = pixel.0;
          pixels.push(Rgb::from_u8(r, g, b));
        }
      }
    }

    pixels
  }

  /// K-means clustering with k-means++ initialization
  fn kmeans(&self, pixels: &[Rgb], k: usize) -> Vec<Rgb> {
    if pixels.len() <= k {
      return pixels.to_vec();
    }

    // Initialize centroids with k-means++
    let mut centroids = self.kmeans_plus_plus_init(pixels, k);
    let mut assignments = vec![0usize; pixels.len()];

    for _ in 0..self.max_iterations {
      let mut changed = false;

      // Assign each pixel to nearest centroid
      for (i, pixel) in pixels.iter().enumerate() {
        let mut min_dist = f32::MAX;
        let mut min_idx = 0;

        for (j, centroid) in centroids.iter().enumerate() {
          let dist = pixel.distance_squared(centroid);
          if dist < min_dist {
            min_dist = dist;
            min_idx = j;
          }
        }

        if assignments[i] != min_idx {
          assignments[i] = min_idx;
          changed = true;
        }
      }

      if !changed {
        break;
      }

      // Update centroids
      let mut sums = vec![(0.0f32, 0.0f32, 0.0f32); k];
      let mut counts = vec![0usize; k];

      for (i, pixel) in pixels.iter().enumerate() {
        let c = assignments[i];
        sums[c].0 += pixel.r;
        sums[c].1 += pixel.g;
        sums[c].2 += pixel.b;
        counts[c] += 1;
      }

      for (c, centroid) in centroids.iter_mut().enumerate() {
        if counts[c] > 0 {
          let count = counts[c] as f32;
          *centroid = Rgb::new(sums[c].0 / count, sums[c].1 / count, sums[c].2 / count);
        }
      }
    }

    centroids
  }

  /// K-means++ initialization for better starting centroids
  fn kmeans_plus_plus_init(&self, pixels: &[Rgb], k: usize) -> Vec<Rgb> {
    let mut rng = rand::thread_rng();
    let mut centroids = Vec::with_capacity(k);

    // First centroid is random
    let first_idx = rng.r#gen_range(0..pixels.len());
    centroids.push(pixels[first_idx]);

    let mut min_distances = vec![f32::MAX; pixels.len()];

    for _ in 1..k {
      let mut total_dist = 0.0f32;

      // Update min distances to nearest existing centroid
      for (i, pixel) in pixels.iter().enumerate() {
        let dist = pixel.distance_squared(centroids.last().unwrap());
        if dist < min_distances[i] {
          min_distances[i] = dist;
        }
        total_dist += min_distances[i];
      }

      // Weighted random selection
      let threshold = rng.r#gen::<f32>() * total_dist;
      let mut cumulative = 0.0f32;
      let mut selected_idx = 0;

      for (i, &dist) in min_distances.iter().enumerate() {
        cumulative += dist;
        if cumulative >= threshold {
          selected_idx = i;
          break;
        }
      }

      centroids.push(pixels[selected_idx]);
    }

    centroids
  }

  /// Generate a color scheme from dominant colors
  fn generate_scheme(&self, wallpaper: String, dominant_colors: Vec<Rgb>, options: &ExtractionOptions) -> ColorScheme {
    // Use user preference if specified, otherwise auto-detect from image luminance
    let is_dark = options.prefers_dark.unwrap_or_else(|| {
      let avg_luminance: f32 = dominant_colors.iter().map(|c| c.luminance()).sum::<f32>() / dominant_colors.len() as f32;
      avg_luminance < 0.5
    });

    // Background and foreground
    let (background, foreground) = if is_dark {
      // Dark mode: darken the darkest extracted color for background
      let bg = dominant_colors
        .first()
        .map(|c| c.darkened(options.background_intensity))
        .unwrap_or(Rgb::new(0.1, 0.1, 0.1));
      (bg, Rgb::new(0.9, 0.9, 0.9))
    } else {
      // Light mode: lighten the lightest extracted color for background
      let bg = dominant_colors
        .last()
        .map(|c| c.lightened(options.background_intensity))
        .unwrap_or(Rgb::new(0.95, 0.95, 0.95));
      (bg, Rgb::new(0.1, 0.1, 0.1))
    };

    // Build 16 terminal colors
    let mut colors = Vec::with_capacity(16);

    // Color 0: background
    colors.push(background);

    // Colors 1-6: dominant colors adjusted for terminal use
    let selected = self.select_terminal_colors(&dominant_colors, 6, is_dark, options.contrast_ratio);
    colors.extend(selected.iter().cloned());

    // Color 7: foreground
    colors.push(foreground);

    // Colors 8-15: brighter versions
    colors.push(background.lightened(0.15));
    for color in &selected {
      if is_dark {
        colors.push(color.saturated(1.2).lightened(0.15));
      } else {
        colors.push(color.saturated(1.1));
      }
    }
    colors.push(foreground);

    // Cursor: first saturated color or foreground
    let cursor = dominant_colors.iter().find(|c| c.saturation() > 0.3).cloned().unwrap_or(foreground);

    ColorScheme::new(wallpaper, is_dark, background, foreground, cursor, colors)
  }

  /// Select colors suitable for terminal use
  fn select_terminal_colors(&self, colors: &[Rgb], count: usize, is_dark: bool, contrast_ratio: f32) -> Vec<Rgb> {
    // Filter for saturated colors
    let mut saturated: Vec<Rgb> = colors.iter().filter(|c| c.saturation() > 0.2).cloned().collect();

    if saturated.len() < count {
      saturated = colors.to_vec();
    }

    // Sort by hue for color variety
    saturated.sort_by(|a, b| a.hue().partial_cmp(&b.hue()).unwrap());

    // Map contrast_ratio (1.5-4.5) to adjustment parameters
    let normalized = (contrast_ratio - 1.5) / 3.0;

    let dark_threshold = 0.15 + normalized * 0.30;
    let dark_adjustment = 0.10 + normalized * 0.25;
    let light_threshold = 0.85 - normalized * 0.30;
    let light_adjustment = 0.20 + normalized * 0.30;

    let mut selected = Vec::with_capacity(count);
    let step = (saturated.len() / count).max(1);

    for i in (0..saturated.len().min(count * step)).step_by(step) {
      let mut color = saturated[i];

      // Increase saturation if too low
      if color.saturation() < 0.4 {
        color = color.saturated(1.5);
      }

      // Apply contrast-aware luminance adjustments
      if is_dark && color.luminance() < dark_threshold {
        color = color.lightened(dark_adjustment);
      } else if !is_dark && color.luminance() > light_threshold {
        color = color.darkened(light_adjustment);
      }

      selected.push(color);
    }

    // Pad with default colors if needed
    let defaults = [
      Rgb::new(0.8, 0.2, 0.2), // red
      Rgb::new(0.2, 0.8, 0.2), // green
      Rgb::new(0.8, 0.8, 0.2), // yellow
      Rgb::new(0.2, 0.4, 0.8), // blue
      Rgb::new(0.8, 0.2, 0.8), // magenta
      Rgb::new(0.2, 0.8, 0.8), // cyan
    ];

    while selected.len() < count {
      selected.push(defaults[selected.len() % defaults.len()]);
    }

    selected.truncate(count);
    selected
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_extraction_options_default() {
    let opts = ExtractionOptions::default();
    assert_eq!(opts.color_count, 16);
    assert_eq!(opts.prefers_dark, None);
    assert!((opts.contrast_ratio - 3.0).abs() < 0.001);
  }

  #[test]
  fn test_kmeans_simple() {
    let extractor = ColorExtractor::new();
    let pixels = vec![
      Rgb::new(1.0, 0.0, 0.0),
      Rgb::new(1.0, 0.1, 0.0),
      Rgb::new(0.0, 1.0, 0.0),
      Rgb::new(0.0, 1.0, 0.1),
      Rgb::new(0.0, 0.0, 1.0),
      Rgb::new(0.1, 0.0, 1.0),
    ];

    let centroids = extractor.kmeans(&pixels, 3);
    assert_eq!(centroids.len(), 3);
  }
}
