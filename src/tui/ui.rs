//! UI rendering for the TUI application
//!
//! This module demonstrates modern terminal UI development with ratatui,
//! including:
//! - Responsive layout design with constraints
//! - List widgets with scrolling and selection
//! - Status bars and information displays
//! - Color schemes and styling
//! - Error and loading state visualization
//! - Terminal image rendering with ratatui-image

use ratatui::{
  prelude::*,
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};
use ratatui_image::StatefulImage;

use super::app::{App, ViewMode, WallpaperItem, format_file_size};

/// Main UI drawing function
pub fn draw(f: &mut Frame, app: &mut App) {
  // Create main layout
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(1), // Title bar
      Constraint::Min(10),   // Main content
      Constraint::Length(3), // Status/info area
    ])
    .split(f.area());

  // Draw title bar
  draw_title_bar(f, chunks[0], app);

  // Draw main content based on view mode
  match app.view_mode {
    ViewMode::Browse => draw_browse_mode(f, chunks[1], app),
    ViewMode::Preview => draw_preview_mode(f, chunks[1], app),
    ViewMode::Help => draw_help_mode(f, chunks[1], app),
  }

  // Draw status bar
  draw_status_bar(f, chunks[2], app);

  // Draw loading overlay if needed
  if app.is_loading {
    draw_loading_overlay(f, f.area());
  }

  // Draw error popup if needed
  if let Some(ref error) = app.error_message {
    draw_error_popup(f, f.area(), error);
  }
}

/// Draw the title bar
fn draw_title_bar(f: &mut Frame, area: Rect, app: &App) {
  let title = match app.view_mode {
    ViewMode::Browse => "🌊 wallflow - Browser",
    ViewMode::Preview => "🌊 wallflow - Preview",
    ViewMode::Help => "🌊 wallflow - Help",
  };

  let title_paragraph = Paragraph::new(title)
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .block(Block::default());

  f.render_widget(title_paragraph, area);
}

/// Draw browse mode UI
fn draw_browse_mode(f: &mut Frame, area: Rect, app: &mut App) {
  // Split into sidebar and main content
  let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Percentage(70), // Wallpaper list
      Constraint::Percentage(30), // Details panel
    ])
    .split(area);

  // Draw wallpaper list
  draw_wallpaper_list(f, chunks[0], app);

  // Draw details panel with thumbnail
  draw_details_panel(f, chunks[1], app);
}

/// Draw wallpaper list widget
fn draw_wallpaper_list(f: &mut Frame, area: Rect, app: &App) {
  let items: Vec<ListItem> = app
    .wallpapers
    .iter()
    .enumerate()
    .map(|(i, wallpaper)| {
      let style = if i == app.selected {
        Style::default().bg(Color::Blue).fg(Color::White)
      } else if wallpaper.is_current {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
      } else {
        Style::default()
      };

      let mut content = vec![Span::from(wallpaper.name.clone())];

      // Add format indicator
      if let Some(ref format) = wallpaper.format {
        content.push(Span::styled(format!(" ({})", format), Style::default().fg(Color::Yellow)));
      }

      // Add current wallpaper indicator
      if wallpaper.is_current {
        content.push(Span::styled(" ●", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
      }

      ListItem::new(Line::from(content)).style(style)
    })
    .collect();

  let title = format!("Wallpapers ({}/{})", app.selected + 1, app.wallpapers.len());
  let list = List::new(items)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(Color::Cyan)),
    )
    .highlight_style(Style::default().bg(Color::Blue))
    .highlight_symbol("▶ ");

  // Calculate list state for scrolling
  let mut list_state = ListState::default();
  list_state.select(Some(app.selected));

  f.render_stateful_widget(list, area, &mut list_state);
}

/// Draw details panel for selected wallpaper
fn draw_details_panel(f: &mut Frame, area: Rect, app: &mut App) {
  // Split details panel into thumbnail, wallpaper details, and config
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Percentage(40), // Thumbnail preview
      Constraint::Percentage(25), // Wallpaper details
      Constraint::Percentage(35), // Config summary
    ])
    .split(area);

  // Thumbnail preview
  draw_thumbnail(f, chunks[0], app);

  // Wallpaper details
  let wallpaper_content = if let Some(wallpaper) = app.selected_wallpaper() {
    format_wallpaper_details(wallpaper)
  } else {
    "No wallpaper selected".to_string()
  };

  let wallpaper_details = Paragraph::new(wallpaper_content)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("Details")
        .title_style(Style::default().fg(Color::Cyan)),
    )
    .wrap(Wrap { trim: true });

  f.render_widget(wallpaper_details, chunks[1]);

  // Config summary
  let config_content = format_config_summary(app);
  let config_panel = Paragraph::new(config_content)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("Config [e to edit]")
        .title_style(Style::default().fg(Color::Green)),
    )
    .wrap(Wrap { trim: true });

  f.render_widget(config_panel, chunks[2]);
}

/// Draw thumbnail preview
fn draw_thumbnail(f: &mut Frame, area: Rect, app: &mut App) {
  let block = Block::default()
    .borders(Borders::ALL)
    .title("Preview")
    .title_style(Style::default().fg(Color::Magenta));

  let inner = block.inner(area);
  f.render_widget(block, area);

  // Try to render the image thumbnail
  if let Some(ref mut image_state) = app.thumbnail_state {
    let image_widget = StatefulImage::new(None);
    f.render_stateful_widget(image_widget, inner, image_state);
  } else if app.is_thumbnail_loading() {
    // Image is being loaded in background
    let placeholder = Paragraph::new("⏳ Loading...")
      .style(Style::default().fg(Color::Yellow))
      .alignment(Alignment::Center);
    f.render_widget(placeholder, inner);
  } else if app.supports_images() {
    // Image picker available but no image loaded yet
    let placeholder = Paragraph::new("No preview")
      .style(Style::default().fg(Color::DarkGray))
      .alignment(Alignment::Center);
    f.render_widget(placeholder, inner);
  } else {
    // Terminal doesn't support image rendering
    let placeholder = Paragraph::new("🖼️ Image preview\nnot supported\nin this terminal")
      .style(Style::default().fg(Color::DarkGray))
      .alignment(Alignment::Center);
    f.render_widget(placeholder, inner);
  }
}

/// Format wallpaper details for display
fn format_wallpaper_details(wallpaper: &WallpaperItem) -> String {
  let mut details = vec![];
  details.push(format!("📍 Path: {}", wallpaper.path.display()));

  if let Some(size) = wallpaper.size {
    details.push(format!("📏 Size: {}", format_file_size(size)));
  }

  if let Some((width, height)) = wallpaper.dimensions {
    details.push(format!("🖼️  Resolution: {}×{}", width, height));
    let aspect_ratio = width as f32 / height as f32;
    details.push(format!("📐 Aspect: {:.2}:1", aspect_ratio));
  }

  if let Some(ref format) = wallpaper.format {
    details.push(format!("🎨 Format: {}", format));
  }

  if wallpaper.is_current {
    details.push("✅ Currently active".to_string());
  }

  details.join("\n")
}

/// Format config summary for display
fn format_config_summary(app: &App) -> String {
  let mut details = vec![];
  details.push(format!("🎯 Source: {}", app.config.sources.default));
  details.push(format!("⏱️  Interval: {}m", app.config.timer.interval));
  details.push(format!("🔀 Randomize: {}", app.config.timer.randomize));
  details.push(format!("🎨 Pywal: {}", if app.config.integration.pywal.enabled { "on" } else { "off" }));
  details.push(format!("📂 Recursive: {}", if app.config.sources.local.recursive { "yes" } else { "no" }));
  details.join("\n")
}

/// Draw preview mode UI - full screen image preview
fn draw_preview_mode(f: &mut Frame, area: Rect, app: &mut App) {
  let block = Block::default()
    .borders(Borders::ALL)
    .title("Preview - Press ENTER to apply, ESC to return")
    .title_style(Style::default().fg(Color::Cyan));

  let inner = block.inner(area);
  f.render_widget(block, area);

  // Try to render the full image preview
  if let Some(ref mut image_state) = app.thumbnail_state {
    let image_widget = StatefulImage::new(None);
    f.render_stateful_widget(image_widget, inner, image_state);
  } else if app.supports_images() {
    let placeholder = Paragraph::new("Loading preview...")
      .style(Style::default().fg(Color::DarkGray))
      .alignment(Alignment::Center);
    f.render_widget(placeholder, inner);
  } else {
    // Fallback for terminals without graphics support
    let preview_text = if let Some(wallpaper) = app.selected_wallpaper() {
      format!(
        "🖼️  {}\n\nImage preview not supported in this terminal.\n\nPress ENTER to apply, ESC to return",
        wallpaper.name
      )
    } else {
      "No wallpaper selected".to_string()
    };

    let preview = Paragraph::new(preview_text)
      .style(Style::default().fg(Color::White))
      .alignment(Alignment::Center)
      .wrap(Wrap { trim: true });

    f.render_widget(preview, inner);
  }
}

/// Draw help mode UI
fn draw_help_mode(f: &mut Frame, area: Rect, _app: &App) {
  let help_text = "🌊 wallflow Keybindings\n\n\
        Navigation:\n\
        j, ↓        Move selection down\n\
        k, ↑        Move selection up\n\
        J           Jump down 10 items\n\
        K           Jump up 10 items\n\
        g           Go to first item\n\
        G           Go to last item\n\n\
        Actions:\n\
        ENTER, SPC  Apply selected wallpaper\n\
        p           Preview mode\n\
        e           Edit config in $EDITOR\n\
        r           Refresh wallpaper list\n\
        c           Clear messages\n\n\
        Modes:\n\
        ?           Show this help\n\
        ESC         Return to browse mode\n\n\
        Global:\n\
        q           Quit (from browse mode)\n\
        Ctrl+C      Force quit\n\n\
        Press ESC to return to browser";

  let help = Paragraph::new(help_text)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("Help")
        .title_style(Style::default().fg(Color::Cyan)),
    )
    .wrap(Wrap { trim: true });

  f.render_widget(help, area);
}

/// Draw status bar at bottom
fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
  // Split status bar into sections
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(1), // Main status line
      Constraint::Length(1), // Message line
      Constraint::Length(1), // Info line
    ])
    .split(area);

  // Status line
  let status = Paragraph::new(app.status_info()).style(Style::default().fg(Color::Gray));
  f.render_widget(status, chunks[0]);

  // Message line
  if let Some(ref message) = app.status_message {
    let msg = Paragraph::new(message.as_str()).style(Style::default().fg(Color::Green));
    f.render_widget(msg, chunks[1]);
  }

  // Keybinding hints
  let hints = match app.view_mode {
    ViewMode::Browse => "j/k: navigate | ENTER: apply | p: preview | e: edit config | ?: help | q: quit",
    ViewMode::Preview => "j/k: navigate | ENTER: apply | ESC: back",
    ViewMode::Help => "ESC: back",
  };

  let hint_paragraph = Paragraph::new(hints).style(Style::default().fg(Color::DarkGray));
  f.render_widget(hint_paragraph, chunks[2]);
}

/// Draw loading overlay
fn draw_loading_overlay(f: &mut Frame, area: Rect) {
  let popup_area = centered_rect(30, 7, area);

  f.render_widget(Clear, popup_area);

  let loading_block = Block::default()
    .borders(Borders::ALL)
    .title("Loading")
    .title_style(Style::default().fg(Color::Yellow));

  let inner = loading_block.inner(popup_area);
  f.render_widget(loading_block, popup_area);
  let loading_text = Paragraph::new("Loading wallpapers...")
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Yellow));

  f.render_widget(loading_text, inner);

  // Add a simple progress bar
  let progress_area = Rect {
    x: inner.x,
    y: inner.y + 2,
    width: inner.width,
    height: 1,
  };

  let progress = Gauge::default()
    .block(Block::default())
    .gauge_style(Style::default().fg(Color::Yellow))
    .percent(50); // Indeterminate progress

  f.render_widget(progress, progress_area);
}

/// Draw error popup
fn draw_error_popup(f: &mut Frame, area: Rect, error: &str) {
  let popup_area = centered_rect(60, 20, area);

  f.render_widget(Clear, popup_area);

  let error_block = Block::default()
    .borders(Borders::ALL)
    .title("Error")
    .title_style(Style::default().fg(Color::Red));

  let inner = error_block.inner(popup_area);
  f.render_widget(error_block, popup_area);
  let error_text = Paragraph::new(format!("❌ {}\n\nPress 'c' to clear", error))
    .alignment(Alignment::Left)
    .style(Style::default().fg(Color::Red))
    .wrap(Wrap { trim: true });

  f.render_widget(error_text, inner);
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
  let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Percentage((100 - percent_y) / 2),
      Constraint::Percentage(percent_y),
      Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

  Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Percentage((100 - percent_x) / 2),
      Constraint::Percentage(percent_x),
      Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
