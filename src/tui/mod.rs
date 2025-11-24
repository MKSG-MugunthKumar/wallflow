//! Terminal User Interface for wallflow
//!
//! This module provides an interactive TUI for browsing and managing wallpapers
//! using ratatui with the following features:
//! - Visual wallpaper browser with vim-like navigation
//! - Real-time preview with Kitty graphics protocol support
//! - Collection management and organization
//! - Multiple monitor configuration
//!
//! Educational aspects:
//! - Demonstrates modern TUI development patterns
//! - Shows event-driven architecture with crossterm
//! - Illustrates async programming in TUI context
//! - Examples of terminal graphics integration

pub mod app;
pub mod events;
pub mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
  event::{DisableMouseCapture, EnableMouseCapture},
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io;

/// Initialize and run the TUI application
#[allow(dead_code)]
pub async fn run(config: crate::config::Config) -> Result<()> {
  // Setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

  // Create terminal
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  // Create application state
  let mut app = App::new(config).await?;

  // Run application
  let res = run_app(&mut terminal, &mut app).await;

  // Restore terminal
  disable_raw_mode()?;
  execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
  terminal.show_cursor()?;

  res
}

/// Main application loop
async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
  loop {
    // Draw UI
    terminal.draw(|f| ui::draw(f, app))?;

    // Handle events
    if events::handle_events(app).await? {
      break;
    }
  }

  Ok(())
}

/// Initialize TUI with default terminal
pub async fn run_with_default_terminal(config: crate::config::Config) -> Result<()> {
  let mut terminal = ratatui::init();
  let mut app = App::new(config).await?;

  let result = run_app(&mut terminal, &mut app).await;

  ratatui::restore();
  result
}
