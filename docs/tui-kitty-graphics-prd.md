# Wallflow TUI with Kitty Graphics Protocol PRD

## Executive Summary

This PRD outlines the implementation of a Terminal User Interface (TUI) for wallflow using ratatui, enhanced with cross-platform image display via the Kitty graphics protocol. The goal is to create an intuitive, visual wallpaper management interface that maintains wallflow's educational focus while providing a modern, interactive user experience.

## Vision Statement

Transform wallflow from a CLI-only tool into a visually rich TUI application where users can:

- Browse wallpaper collections with inline image previews
- Manage multiple wallpaper sources from a unified interface
- Preview wallpapers before applying them
- Navigate with vim-like keybindings for power users
- Experience smooth, responsive interactions across platforms

## Current State Analysis

### Existing CLI Limitations

- **Text-Only Interface**: No visual wallpaper previews
- **Command-Based Workflow**: Requires memorizing commands and options
- **Sequential Operations**: Cannot browse and preview simultaneously
- **Limited Discovery**: Hard to explore available wallpapers
- **No Visual Feedback**: Cannot see wallpaper before applying

### TUI Opportunities

- **Visual Browsing**: See wallpapers before selecting them
- **Interactive Navigation**: Mouse and keyboard navigation
- **Multi-Panel Interface**: Simultaneous browsing and configuration
- **Real-Time Feedback**: Instant preview of changes
- **Cross-Platform Consistency**: Same interface on all platforms

## Kitty Graphics Protocol Research

### Protocol Capabilities

The Kitty graphics protocol enables rich image display in terminal environments:

**Core Features**:

- **Image Formats**: PNG, JPEG, GIF, WebP support
- **Transparency**: Full alpha channel support
- **Positioning**: Precise pixel-level positioning
- **Scaling**: Built-in image scaling and aspect ratio preservation
- **Animation**: GIF animation support
- **Memory Efficient**: Streaming and caching capabilities

**Cross-Platform Support**:

- **Native**: Kitty terminal (Linux, macOS, Windows via WSL)
- **Compatible**: WezTerm, foot, konsole (partial support)
- **Fallback**: ASCII art representation for unsupported terminals
- **Detection**: Runtime protocol capability detection

### Implementation Approaches

**Direct Protocol Implementation**:

```rust
// Send image via Kitty graphics protocol
pub fn display_image(path: &Path, x: u16, y: u16, width: u16, height: u16) -> Result<()> {
    let image_data = encode_image_base64(path)?;
    let protocol_command = format!(
        "\x1b_Gf=100,s={},v={},c={},r={};{}\x1b\\",
        width, height, x, y, image_data
    );
    print!("{}", protocol_command);
    Ok(())
}
```

**ratatui Integration**:

```rust
// Custom widget for image display
pub struct ImageWidget {
    image_path: PathBuf,
    dimensions: (u16, u16),
}

impl Widget for ImageWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if supports_kitty_graphics() {
            display_kitty_image(&self.image_path, area);
        } else {
            render_ascii_placeholder(area, buf);
        }
    }
}
```

## Proposed TUI Architecture

### Layout Design

```
┌─────────────────────────────────────────────────────────────────┐
│ wallflow v0.2.0 │ [Platform: Linux/Wayland] │ [Backend: swww]  │
├─────────────────┼──────────────────────────────┬──────────────────┤
│                 │                              │                  │
│   Sources       │        Image Browser         │    Preview       │
│                 │                              │                  │
│ ▶ Local (142)   │  ┌─────┬─────┬─────┬─────┐   │ ┌──────────────┐ │
│   Wallhaven     │  │     │     │     │     │   │ │              │ │
│   NASA APOD     │  │ 🖼️  │ 🖼️  │ 🖼️  │ 🖼️  │   │ │   Selected   │ │
│   Bing Daily    │  │     │     │     │     │   │ │   Wallpaper  │ │
│   Reddit        │  └─────┴─────┴─────┴─────┘   │ │              │ │
│   Google Earth  │  ┌─────┬─────┬─────┬─────┐   │ │              │ │
│                 │  │     │     │     │     │   │ └──────────────┘ │
│ [+] Add Source  │  │ 🖼️  │ 🖼️  │ 🖼️  │ 🖼️  │   │                  │
│                 │  │     │     │     │     │   │ Resolution:      │
│                 │  └─────┴─────┴─────┴─────┘   │ 1920x1080        │
│                 │                              │                  │
│                 │  [↑↓] Navigate [Enter] Apply │ Source: Local    │
│                 │  [Space] Preview [q] Quit    │ Format: PNG      │
│                 │                              │ Size: 2.4MB      │
├─────────────────┴──────────────────────────────┴──────────────────┤
│ Status: Ready │ Last Applied: nature_001.jpg │ [F1] Help         │
└─────────────────────────────────────────────────────────────────┘
```

### Key UI Components

#### 1. Source Panel (Left)

- **Tree View**: Hierarchical source listing
- **Status Indicators**: Source availability and item counts
- **Quick Actions**: Add/remove sources, refresh
- **Filtering**: Search and filter sources

#### 2. Image Browser (Center)

- **Grid Layout**: Thumbnail grid with Kitty graphics
- **Lazy Loading**: Progressive image loading for performance
- **Pagination**: Handle large collections efficiently
- **Selection States**: Visual selection indicators

#### 3. Preview Panel (Right)

- **Large Preview**: Full-size wallpaper preview
- **Metadata Display**: Resolution, format, source information
- **Action Buttons**: Apply, save, download
- **History**: Recently viewed/applied wallpapers

#### 4. Status Bar (Bottom)

- **Current Status**: Operation status and progress
- **Keybinding Hints**: Context-sensitive help
- **System Information**: Platform and backend info

### Keybinding Scheme

**Vim-Inspired Navigation**:

```
Global Navigation:
  h/j/k/l     ← ↓ ↑ →    Move cursor
  gg/G        Home/End   Go to first/last
  /           Search     Search wallpapers
  n/N         Next/Prev  Search navigation
  q           Quit       Exit application

Source Panel:
  Enter       Expand     Expand/collapse source
  r           Refresh    Refresh source
  +           Add        Add new source
  -           Remove     Remove source

Image Browser:
  Space       Preview    Toggle preview
  Enter       Apply      Apply selected wallpaper
  d           Download   Download wallpaper
  s           Save       Save to favorites
  f           Filter     Filter images

Preview Panel:
  Tab         Cycle      Cycle preview modes
  i           Info       Show detailed info
  c           Configure  Configure application
```

**Mouse Support**:

- Click to select images
- Scroll to navigate grids
- Drag to resize panels
- Right-click context menus

## Technical Implementation

### Core TUI Framework

**Dependencies**:

```toml
[dependencies]
# TUI framework
ratatui = "0.26"
crossterm = "0.27"

# Image handling
image = "0.24"
imagesize = "0.12"

# Terminal graphics
kitty-image = { git = "https://github.com/benjajaja/kitty-image" }

# Async runtime for image loading
tokio = { version = "1.0", features = ["full"] }
tokio-stream = "0.1"

# Configuration and state
serde = { version = "1.0", features = ["derive"] }
dirs = "5.0"
```

### Application Architecture

```rust
// Main TUI application structure
pub struct WallflowTui {
    // Application state
    state: AppState,

    // UI components
    source_panel: SourcePanel,
    browser_panel: BrowserPanel,
    preview_panel: PreviewPanel,
    status_bar: StatusBar,

    // Graphics support
    graphics_backend: GraphicsBackend,

    // Background services
    image_loader: ImageLoader,
    wallpaper_service: WallpaperService,
}

#[derive(Debug)]
pub struct AppState {
    pub current_source: String,
    pub selected_image: Option<ImageInfo>,
    pub images: Vec<ImageInfo>,
    pub preview_mode: PreviewMode,
    pub filter_text: String,
    pub loading_state: LoadingState,
}

pub enum PreviewMode {
    Thumbnail,
    Preview,
    Fullscreen,
}
```

### Graphics Backend Abstraction

```rust
pub trait GraphicsBackend {
    /// Check if graphics are supported in current terminal
    fn is_supported(&self) -> bool;

    /// Display image at specified coordinates
    async fn display_image(&self, path: &Path, area: Rect) -> Result<()>;

    /// Clear image from display
    fn clear_image(&self, area: Rect) -> Result<()>;

    /// Get optimal image size for display area
    fn optimal_size(&self, image_size: (u32, u32), display_area: Rect) -> (u16, u16);
}

pub struct KittyGraphicsBackend {
    supported: bool,
    image_cache: LruCache<PathBuf, CachedImage>,
}

impl GraphicsBackend for KittyGraphicsBackend {
    fn is_supported(&self) -> bool {
        // Detect Kitty graphics protocol support
        // Check TERM_PROGRAM, test protocol response
        detect_kitty_graphics_support()
    }

    async fn display_image(&self, path: &Path, area: Rect) -> Result<()> {
        let image_data = self.load_and_scale_image(path, area).await?;
        let protocol_cmd = build_kitty_command(&image_data, area);
        execute_protocol_command(&protocol_cmd)?;
        Ok(())
    }
}

// Fallback for non-graphics terminals
pub struct AsciiGraphicsBackend;

impl GraphicsBackend for AsciiGraphicsBackend {
    fn is_supported(&self) -> bool { true }

    async fn display_image(&self, path: &Path, area: Rect) -> Result<()> {
        let ascii_art = generate_ascii_art(path, area).await?;
        display_ascii_art(&ascii_art, area);
        Ok(())
    }
}
```

### Image Management System

```rust
pub struct ImageLoader {
    cache: Arc<Mutex<LruCache<PathBuf, LoadedImage>>>,
    loading_queue: Arc<Mutex<VecDeque<LoadRequest>>>,
    worker_handles: Vec<JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub struct LoadedImage {
    pub data: Vec<u8>,
    pub format: ImageFormat,
    pub dimensions: (u32, u32),
    pub thumbnail: Option<Vec<u8>>,
}

impl ImageLoader {
    pub async fn load_thumbnail(&self, path: &Path, size: (u16, u16)) -> Result<LoadedImage> {
        // Check cache first
        if let Some(cached) = self.cache.lock().await.get(path) {
            return Ok(cached.clone());
        }

        // Load and process image
        let image = image::open(path)?;
        let thumbnail = image.thumbnail(size.0 as u32, size.1 as u32);

        // Convert to appropriate format for Kitty protocol
        let processed = self.process_for_display(thumbnail).await?;

        // Cache result
        self.cache.lock().await.put(path.to_path_buf(), processed.clone());

        Ok(processed)
    }

    pub fn preload_images(&self, paths: &[PathBuf], priority: LoadPriority) {
        // Queue images for background loading
        for path in paths {
            self.loading_queue.lock().unwrap().push_back(LoadRequest {
                path: path.clone(),
                priority,
                timestamp: std::time::Instant::now(),
            });
        }
    }
}
```

### Panel Implementation

```rust
// Source panel for wallpaper source management
pub struct SourcePanel {
    sources: Vec<WallpaperSource>,
    selected_index: usize,
    expanded_sources: HashSet<String>,
    scroll_offset: usize,
}

impl SourcePanel {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Sources")
            .borders(Borders::ALL)
            .border_style(if self.is_focused() {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Render source tree
        let items: Vec<ListItem> = self.sources
            .iter()
            .enumerate()
            .map(|(i, source)| {
                let style = if i == self.selected_index {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                let prefix = if self.expanded_sources.contains(&source.name) {
                    "▼ "
                } else {
                    "▶ "
                };

                ListItem::new(format!("{}{} ({})", prefix, source.name, source.count))
                    .style(style)
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner);
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.sources.len() - 1 {
                    self.selected_index += 1;
                }
            }
            KeyCode::Enter => {
                self.toggle_expansion().await?;
            }
            KeyCode::Char('r') => {
                self.refresh_selected_source().await?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

// Image browser with grid layout and Kitty graphics
pub struct BrowserPanel {
    images: Vec<ImageInfo>,
    selected_index: Option<usize>,
    grid_dimensions: (usize, usize), // columns, rows
    scroll_offset: usize,
    graphics_backend: Arc<dyn GraphicsBackend + Send + Sync>,
    image_loader: Arc<ImageLoader>,
}

impl BrowserPanel {
    pub async fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Browser")
            .borders(Borders::ALL);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Calculate grid layout
        let cell_width = inner.width / self.grid_dimensions.0 as u16;
        let cell_height = inner.height / self.grid_dimensions.1 as u16;

        // Render image grid
        for (i, image) in self.visible_images().enumerate() {
            let col = i % self.grid_dimensions.0;
            let row = i / self.grid_dimensions.0;

            let cell_area = Rect {
                x: inner.x + col as u16 * cell_width,
                y: inner.y + row as u16 * cell_height,
                width: cell_width.saturating_sub(1),
                height: cell_height.saturating_sub(1),
            };

            // Render image or loading placeholder
            if let Some(loaded_image) = self.image_loader.get_cached(&image.path).await {
                self.graphics_backend.display_image(&image.path, cell_area).await?;
            } else {
                // Show loading placeholder
                self.render_loading_placeholder(frame, cell_area);
                // Queue for loading
                self.image_loader.load_thumbnail(
                    &image.path,
                    (cell_area.width, cell_area.height)
                ).await;
            }

            // Highlight selection
            if Some(self.scroll_offset + i) == self.selected_index {
                self.render_selection_border(frame, cell_area);
            }
        }
    }

    fn visible_images(&self) -> impl Iterator<Item = &ImageInfo> {
        let start = self.scroll_offset;
        let visible_count = self.grid_dimensions.0 * self.grid_dimensions.1;
        self.images.iter().skip(start).take(visible_count)
    }
}
```

## Cross-Platform Considerations

### Terminal Compatibility Matrix

| Terminal             | Kitty Graphics  | Alternative    |
| -------------------- | --------------- | -------------- |
| **Kitty**            | ✅ Full support | -              |
| **WezTerm**          | ✅ Compatible   | -              |
| **foot**             | ⚠️ Partial      | ASCII fallback |
| **Alacritty**        | ❌ No support   | ASCII art      |
| **GNOME Terminal**   | ❌ No support   | ASCII art      |
| **Windows Terminal** | ❌ No support   | ASCII art      |
| **iTerm2**           | ❌ No support   | ASCII art      |

### Platform-Specific Features

**Linux**:

- Full Kitty protocol support in compatible terminals
- Wayland/X11 detection for optimal performance
- Multiple terminal emulator support

**macOS**:

- Kitty.app native support
- iTerm2 fallback to ASCII
- Terminal.app fallback to ASCII

**Windows**:

- WSL + Kitty for graphics support
- Windows Terminal ASCII fallback
- PowerShell ASCII fallback

### Runtime Detection Strategy

```rust
pub fn detect_graphics_capabilities() -> GraphicsCapabilities {
    // Check environment variables
    let term_program = env::var("TERM_PROGRAM").unwrap_or_default();
    let term = env::var("TERM").unwrap_or_default();

    match term_program.as_str() {
        "kitty" => test_kitty_graphics(),
        "WezTerm" => test_kitty_graphics(), // WezTerm supports Kitty protocol
        _ => {
            if term.contains("kitty") {
                test_kitty_graphics()
            } else {
                GraphicsCapabilities::AsciiOnly
            }
        }
    }
}

fn test_kitty_graphics() -> GraphicsCapabilities {
    // Send test query and wait for response
    print!("\x1b_Gi=1,t=q,q=2;\x1b\\");

    // Parse response to determine capabilities
    if read_graphics_response().is_ok() {
        GraphicsCapabilities::KittyProtocol
    } else {
        GraphicsCapabilities::AsciiOnly
    }
}
```

## Educational Implementation Path

### Phase 1: Basic TUI Foundation (Week 1-2)

**Learning Goals**: ratatui fundamentals, event handling

```rust
// Simple TUI with panels (no images yet)
pub struct BasicTui {
    sources: Vec<String>,
    selected: usize,
}

// Key concepts:
// - Frame rendering
// - Event loops
// - State management
// - Layout system
```

**Deliverables**:

- [ ] Basic three-panel layout
- [ ] Keyboard navigation
- [ ] Source listing
- [ ] Status bar

### Phase 2: Graphics Integration (Week 3-4)

**Learning Goals**: Terminal graphics, protocol implementation

```rust
// Add image display capabilities
pub fn display_kitty_image(path: &Path, area: Rect) -> Result<()> {
    // Protocol message construction
    // Base64 encoding
    // Terminal escape sequences
}

// Key concepts:
// - Terminal protocols
// - Image encoding
// - Memory management
// - Performance optimization
```

**Deliverables**:

- [ ] Kitty protocol implementation
- [ ] ASCII art fallback
- [ ] Image caching system
- [ ] Performance optimization

### Phase 3: Advanced Features (Week 5-6)

**Learning Goals**: Async programming, state management

```rust
// Background image loading
pub struct ImageLoader {
    // Work queues
    // Caching strategies
    // Progress tracking
}

// Key concepts:
// - Producer/consumer patterns
// - Cache algorithms
// - Progress tracking
// - Error recovery
```

**Deliverables**:

- [ ] Lazy loading system
- [ ] Search and filtering
- [ ] Preview modes
- [ ] Configuration interface

### Phase 4: Integration & Polish (Week 7-8)

**Learning Goals**: Integration patterns, user experience

```rust
// Connect to wallpaper backend system
pub async fn apply_wallpaper_from_tui(selection: &ImageInfo) -> Result<()> {
    // Backend integration
    // Progress feedback
    // Error handling
}

// Key concepts:
// - Service integration
// - User feedback
// - Error UX
// - Testing strategies
```

**Deliverables**:

- [ ] Wallpaper backend integration
- [ ] Help system
- [ ] Configuration persistence
- [ ] Cross-platform testing

## User Experience Design

### Workflow Examples

#### First-Time User Experience:

1. **Launch**: `wallflow tui`
2. **Welcome**: Brief overlay explaining layout
3. **Auto-discovery**: Automatically find local wallpapers
4. **Visual Tour**: Highlight key features and controls
5. **First Action**: Guide user through applying a wallpaper

#### Power User Workflow:

1. **Quick Launch**: `wallflow tui --source reddit`
2. **Instant Navigation**: vim keys to browse quickly
3. **Preview**: Space to preview without applying
4. **Batch Operations**: Select multiple for download
5. **Custom Sources**: Add new sources on-the-fly

#### Discovery Workflow:

1. **Browse Sources**: Expand different wallpaper sources
2. **Filter/Search**: Find wallpapers matching criteria
3. **Preview Modes**: Different viewing modes for exploration
4. **Save Favorites**: Mark wallpapers for later use
5. **Share**: Copy URLs or paths for sharing

### Accessibility Features

**Visual**:

- High contrast themes
- Configurable color schemes
- Large text options
- ASCII art fallbacks

**Motor**:

- Full keyboard navigation
- Configurable key bindings
- Mouse support optional
- Command palette

**Cognitive**:

- Clear visual hierarchy
- Consistent navigation patterns
- Help system always available
- Undo/redo support

## Performance Considerations

### Image Loading Strategy

- **Lazy Loading**: Load images only when visible
- **Progressive Enhancement**: Low-res first, then high-res
- **Smart Caching**: LRU cache with memory limits
- **Background Preloading**: Predict user navigation

### Memory Management

- **Streaming**: Large files streamed, not loaded entirely
- **Compression**: On-disk cache compression
- **Cleanup**: Automatic cache eviction
- **Monitoring**: Memory usage tracking and limits

### Responsiveness

- **Non-blocking**: All operations asynchronous
- **Progressive Rendering**: Show partial results immediately
- **Debouncing**: Limit rapid navigation updates
- **Progress Feedback**: Visual loading indicators

## Success Metrics

### Technical Metrics

- **Startup Time**: <500ms cold start
- **Image Load Time**: <200ms for thumbnails
- **Memory Usage**: <100MB for 1000 thumbnails
- **Responsiveness**: <16ms frame time for 60fps

### User Experience Metrics

- **Discovery**: Average time to find desired wallpaper
- **Efficiency**: Keystrokes to complete common tasks
- **Satisfaction**: User preference vs CLI interface
- **Adoption**: Usage frequency after implementation

### Educational Metrics

- **Code Clarity**: Maintainable, well-documented modules
- **Pattern Demonstration**: Clear examples of Rust patterns
- **Learning Path**: Progressive complexity introduction
- **Documentation**: Comprehensive learning materials

## Risk Mitigation

### Technical Risks

- **Graphics Compatibility**: Fallback strategies for all terminals
- **Performance Issues**: Profiling and optimization plans
- **Memory Leaks**: Automated testing and monitoring
- **Cross-Platform Bugs**: Platform-specific testing

### User Experience Risks

- **Complexity**: Progressive disclosure of features
- **Learning Curve**: Built-in help and tutorials
- **Terminal Incompatibility**: Clear capability communication
- **Performance Degradation**: Graceful degradation strategies

## Future Enhancements

### Advanced Graphics Features

- **Video Support**: MP4/GIF wallpaper previews
- **3D Previews**: Interactive wallpaper exploration
- **AR Integration**: Wallpaper preview in actual environment
- **Color Analysis**: Automatic color scheme generation

### AI-Powered Features

- **Smart Recommendations**: ML-based wallpaper suggestions
- **Auto-Categorization**: Intelligent wallpaper organization
- **Content Understanding**: Natural language wallpaper search
- **Style Transfer**: Generate wallpapers in specific styles

### Community Features

- **Sharing**: Share favorite wallpaper collections
- **Rating**: Community wallpaper rating system
- **Comments**: Wallpaper discussions and reviews
- **Playlists**: Curated wallpaper collections

---

This TUI implementation transforms wallflow from a simple CLI tool into a rich, visual wallpaper management experience while maintaining its educational mission. The progressive implementation path ensures learning opportunities at every step, from basic TUI concepts to advanced graphics programming and async systems design.
