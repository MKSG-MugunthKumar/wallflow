//! Event handling for the TUI application
//!
//! This module demonstrates async event handling patterns with crossterm,
//! including:
//! - Keyboard input processing with vim-like bindings
//! - Mouse interaction support
//! - Async state updates
//! - Error handling in interactive context

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tracing::debug;

use super::app::{App, ViewMode};

/// Handle terminal events and update application state
pub async fn handle_events(app: &mut App) -> Result<bool> {
  // Update daemon status periodically (every few seconds)
  static mut LAST_UPDATE: Option<std::time::Instant> = None;
  let now = std::time::Instant::now();

  unsafe {
    if LAST_UPDATE.is_none_or(|last| now.duration_since(last).as_secs() >= 2) {
      if let Err(e) = app.update_daemon_status().await {
        debug!("Failed to update daemon status: {}", e);
      }
      LAST_UPDATE = Some(now);
    }
  }

  // Poll for events with a timeout to avoid blocking
  if event::poll(Duration::from_millis(100))? {
    match event::read()? {
      Event::Key(key_event) => {
        return handle_key_event(app, key_event).await;
      }
      Event::Mouse(_) => {
        // Mouse events can be handled here for future enhancement
      }
      Event::Resize(_, _) => {
        // Terminal resize events - automatically handled by ratatui
        debug!("Terminal resized");
      }
      _ => {}
    }
  }

  Ok(false) // Don't quit
}

/// Handle keyboard input events
async fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<bool> {
  // Global keybindings (work in all modes)
  if handle_global_keys(app, &key).await? {
    return Ok(true); // Should quit
  }

  // Mode-specific keybindings
  match app.view_mode {
    ViewMode::Browse => handle_browse_keys(app, &key).await?,
    ViewMode::Preview => handle_preview_keys(app, &key).await?,
    ViewMode::Help => handle_help_keys(app, &key).await?,
  }

  Ok(false)
}

/// Handle global keybindings that work in all modes
async fn handle_global_keys(app: &mut App, key: &KeyEvent) -> Result<bool> {
  match (key.modifiers, key.code) {
    // Quit application
    (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
      app.quit();
      return Ok(true);
    }
    (KeyModifiers::NONE, KeyCode::Char('q')) => {
      if app.view_mode == ViewMode::Browse {
        app.quit();
        return Ok(true);
      }
    }
    (KeyModifiers::NONE, KeyCode::Esc) => {
      // Return to browse mode from any other mode
      if app.view_mode != ViewMode::Browse {
        app.set_view_mode(ViewMode::Browse);
      }
    }

    // Mode switching
    (KeyModifiers::NONE, KeyCode::Char('?')) => {
      app.set_view_mode(ViewMode::Help);
    }

    // Clear messages
    (KeyModifiers::NONE, KeyCode::Char('c')) => {
      if app.view_mode == ViewMode::Browse {
        app.clear_messages();
      }
    }

    _ => {}
  }

  Ok(false)
}

/// Handle keybindings in browse mode
async fn handle_browse_keys(app: &mut App, key: &KeyEvent) -> Result<()> {
  match key.code {
    // Vim-style navigation
    KeyCode::Char('j') | KeyCode::Down => {
      app.select_next();
      debug!("Selected wallpaper: {}/{}", app.selected + 1, app.wallpapers.len());
    }
    KeyCode::Char('k') | KeyCode::Up => {
      app.select_previous();
      debug!("Selected wallpaper: {}/{}", app.selected + 1, app.wallpapers.len());
    }

    // Page navigation
    KeyCode::Char('J') => {
      // Jump down 10 items
      for _ in 0..10 {
        app.select_next();
      }
    }
    KeyCode::Char('K') => {
      // Jump up 10 items
      for _ in 0..10 {
        app.select_previous();
      }
    }

    // First/last navigation
    KeyCode::Char('g') => {
      app.selected = 0;
      app.request_thumbnail();
      debug!("Jumped to first wallpaper");
    }
    KeyCode::Char('G') => {
      if !app.wallpapers.is_empty() {
        app.selected = app.wallpapers.len() - 1;
        app.request_thumbnail();
        debug!("Jumped to last wallpaper");
      }
    }

    // Actions
    KeyCode::Enter | KeyCode::Char(' ') => {
      app.apply_selected_wallpaper().await?;
    }
    KeyCode::Char('p') => {
      app.set_view_mode(ViewMode::Preview);
    }
    KeyCode::Char('e') => {
      // Signal to open editor (handled by main loop)
      app.open_editor = true;
    }
    KeyCode::Char('r') => {
      app.status_message = Some("Refreshing wallpapers...".to_string());
      app.refresh_wallpapers().await?;
    }

    // Quick quit
    KeyCode::Char('q') => {
      app.quit();
    }

    _ => {}
  }

  Ok(())
}

/// Handle keybindings in preview mode
async fn handle_preview_keys(app: &mut App, key: &KeyEvent) -> Result<()> {
  match key.code {
    // Navigation still works in preview mode
    KeyCode::Char('j') | KeyCode::Down => {
      app.select_next();
    }
    KeyCode::Char('k') | KeyCode::Up => {
      app.select_previous();
    }

    // Apply wallpaper
    KeyCode::Enter | KeyCode::Char(' ') => {
      app.apply_selected_wallpaper().await?;
    }

    // Return to browse mode
    KeyCode::Char('b') | KeyCode::Esc => {
      app.set_view_mode(ViewMode::Browse);
    }

    _ => {}
  }

  Ok(())
}

/// Handle keybindings in help mode
async fn handle_help_keys(_app: &mut App, key: &KeyEvent) -> Result<()> {
  if key.code == KeyCode::Esc {
    // Help-specific bindings would go here
    // For now, just handle escape to return to browse
    // Handled by global keys
  }

  Ok(())
}
