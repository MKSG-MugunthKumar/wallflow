//! Color extraction using k-means clustering
//!
//! This module extracts dominant colors from images and generates
//! terminal-compatible color schemes (pywal format).

mod extractor;
mod scheme;

pub use extractor::{ColorExtractor, ExtractionOptions};
#[allow(unused_imports)]
pub use scheme::{ColorScheme, Rgb};
