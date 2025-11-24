//! UI rendering for the TUI application
//!
//! This module demonstrates modern terminal UI development with ratatui,
//! including:
//! - Responsive layout design with constraints
//! - List widgets with scrolling and selection
//! - Status bars and information displays
//! - Color schemes and styling
//! - Error and loading state visualization

use ratatui::{
  prelude::*,
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};

use super::app::{App, ViewMode, WallpaperItem, format_file_size};

/// Main UI drawing function
pub fn draw(f: &mut Frame, app: &App) {
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
    ViewMode::Settings => draw_settings_mode(f, chunks[1], app),
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
    ViewMode::Settings => "🌊 wallflow - Settings",
    ViewMode::Help => "🌊 wallflow - Help",
  };

  let title_paragraph = Paragraph::new(title)
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .block(Block::default());

  f.render_widget(title_paragraph, area);
}

/// Draw browse mode UI
fn draw_browse_mode(f: &mut Frame, area: Rect, app: &App) {
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

  // Draw details panel
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
fn draw_details_panel(f: &mut Frame, area: Rect, app: &App) {
  // Split details panel into wallpaper details and daemon status
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Percentage(70), // Wallpaper details
      Constraint::Percentage(30), // Daemon status
    ])
    .split(area);

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

  f.render_widget(wallpaper_details, chunks[0]);

  // Daemon status
  let daemon_content = format_daemon_status(app);
  let daemon_details = Paragraph::new(daemon_content)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("Daemon Status")
        .title_style(Style::default().fg(Color::Yellow)),
    )
    .wrap(Wrap { trim: true });

  f.render_widget(daemon_details, chunks[1]);
}

/// Format wallpaper details for display
fn format_wallpaper_details(wallpaper: &WallpaperItem) -> String {
  let mut details = vec![];

  details.push(format!("📁 Name: {}", wallpaper.name));
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

  details.join("\n\n")
}

/// Format daemon status for display
fn format_daemon_status(app: &App) -> String {
  if let Some(ref status) = app.daemon_status {
    let mut details = vec![];

    if status.is_stale() {
      details.push("🔴 Status: Offline (stale)".to_string());
    } else {
      details.push("🟢 Status: Online".to_string());

      let time_remaining = status.time_remaining_formatted();
      details.push(format!("⏰ Next rotation: {}", time_remaining));

      details.push(format!("🔄 Interval: {}m", status.config.interval_minutes));

      if let Some(ref current) = status.current_wallpaper {
        let name = std::path::Path::new(current).file_name().and_then(|n| n.to_str()).unwrap_or("Unknown");
        details.push(format!("🖼️  Current: {}", name));
      }

      details.push(format!("🎯 Source: {}", status.config.source));
      details.push(format!("📂 Category: {}", status.config.category));
    }

    details.join("\n")
  } else {
    "❓ Daemon status unknown\n\nNo daemon status file found.\nStart daemon with:\nwallflow daemon".to_string()
  }
}

/// Draw preview mode UI
fn draw_preview_mode(f: &mut Frame, area: Rect, app: &App) {
  // For now, show a placeholder for image preview
  // Future implementation will use Kitty graphics protocol
  let preview_text = if let Some(wallpaper) = app.selected_wallpaper() {
    format!(
      "🖼️  Preview Mode\n\n{}\n\n[Image preview will be implemented with Kitty graphics protocol]\n\nPress ENTER to apply, ESC to return",
      wallpaper.name
    )
  } else {
    "No wallpaper selected".to_string()
  };

  let preview = Paragraph::new(preview_text)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("Preview")
        .title_style(Style::default().fg(Color::Cyan)),
    )
    .style(Style::default().fg(Color::White))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });

  f.render_widget(preview, area);
}

/// Draw settings mode UI
fn draw_settings_mode(f: &mut Frame, area: Rect, app: &App) {
  let settings_text = format!(
    "⚙️  Settings\n\n\
        Local Path: {}\n\
        Downloads: {}\n\
        Default Source: {}\n\
        Category: {}\n\
        Formats: {}\n\
        Recursive: {}\n\n\
        [Settings editing will be implemented in future versions]\n\n\
        Press ESC to return",
    app.config.paths.local,
    app.config.paths.downloads,
    app.config.sources.default,
    app.config.sources.category,
    app.config.sources.local.formats.join(", "),
    app.config.sources.local.recursive
  );

  let settings = Paragraph::new(settings_text)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("Settings")
        .title_style(Style::default().fg(Color::Cyan)),
    )
    .wrap(Wrap { trim: true });

  f.render_widget(settings, area);
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
        r           Refresh wallpaper list\n\
        c           Clear messages\n\n\
        Modes:\n\
        ?           Show this help\n\
        ,           Settings mode\n\
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
    ViewMode::Browse => "j/k: navigate | ENTER: apply | p: preview | r: refresh | ?: help | q: quit",
    ViewMode::Preview => "j/k: navigate | ENTER: apply | ESC: back",
    ViewMode::Settings => "ESC: back",
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
