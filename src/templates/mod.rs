//! Template rendering for terminal and app theming
//!
//! This module handles rendering color scheme templates in pywal-compatible format.
//! Templates use simple `{variable}` substitution.
//!
//! Templates are downloaded from the wallflow-templates GitHub repo on first use
//! and stored locally in `~/.config/mksg/wallflow/templates/`.

mod download;
mod engine;
mod manifest;

pub use download::{ensure_templates, templates_dir};
pub use engine::TemplateEngine;
#[allow(unused_imports)]
pub use manifest::{Detection, InstallConfig, ReloadConfig, TemplateFile, TemplateManifest, UiConfig};
