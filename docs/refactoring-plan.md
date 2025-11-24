# Wallpaper.rs Refactoring Plan

## Current State Analysis

### Problems with Current `src/wallpaper.rs`

1. **Mixed Responsibilities**: Single file handles:

   - HTTP client management
   - JSON parsing for specific APIs
   - File I/O operations
   - Wallpaper application logic
   - System integration (KDE, pywal)

2. **Hardcoded Implementations**:

   - Wallhaven and Picsum logic baked into specific functions
   - No abstraction for adding new sources
   - Duplicate HTTP client creation

3. **Unused Configuration**:

   - `config.cleanup.*` - Not implemented
   - `config.advanced.*` - Parallel downloads, retries, timeouts ignored
   - `config.logging.*` - Custom logging not used

4. **Limited Error Handling**:
   - Generic error types
   - No retry logic
   - No rate limiting

## Proposed Module Structure

```
src/
├── wallpaper/               # Main wallpaper module
│   ├── mod.rs              # Public API
│   ├── apply.rs            # Wallpaper application logic
│   └── manager.rs          # High-level orchestration
├── downloaders/            # Downloader system
│   ├── mod.rs              # Public API & registry
│   ├── traits.rs           # Core traits
│   ├── registry.rs         # Downloader registry
│   ├── apod.rs             # NASA APOD downloader
│   ├── bing.rs             # Bing Daily downloader
│   ├── earthview.rs        # Google Earth View
│   ├── reddit.rs           # Reddit downloader
│   └── wallhaven.rs        # Existing Wallhaven (refactored)
├── http/                   # HTTP utilities
│   ├── mod.rs              # Public API
│   ├── client.rs           # Shared HTTP client
│   ├── cache.rs            # Response caching
│   └── retry.rs            # Retry logic with backoff
├── filesystem/             # File management
│   ├── mod.rs              # Public API
│   ├── cleanup.rs          # Implement config.cleanup logic
│   └── local.rs            # Local file scanning (extracted)
└── integration/            # System integration
    ├── mod.rs              # Public API
    ├── pywal.rs            # Color scheme integration
    └── desktop.rs          # Desktop environment integration
```

## Implementation Plan

### Phase 1: Core Abstractions (3-4 days)

#### Step 1.1: Create Downloader Trait

**File**: `src/downloaders/traits.rs`

```rust
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub category: Option<String>,
    pub resolution: Option<(u32, u32)>,
    pub quality: Option<String>,
}

#[derive(Debug)]
pub struct DownloadedWallpaper {
    pub file_path: PathBuf,
    pub source_url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub metadata: WallpaperMetadata,
}

#[derive(Debug)]
pub struct WallpaperMetadata {
    pub dimensions: Option<(u32, u32)>,
    pub file_size: Option<u64>,
    pub format: Option<String>,
    pub source: String,
    pub downloaded_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait WallpaperDownloader {
    /// Download a wallpaper based on the request
    async fn download(&self, request: &DownloadRequest) -> Result<DownloadedWallpaper>;

    /// Get the source name for this downloader
    fn source_name(&self) -> &'static str;

    /// Check if this source requires authentication
    fn requires_auth(&self) -> bool { false }

    /// Validate configuration for this downloader
    fn validate_config(&self, config: &crate::config::Config) -> Result<()> { Ok(()) }
}
```

#### Step 1.2: HTTP Client Utilities

**File**: `src/http/client.rs`

```rust
use anyhow::{Context, Result};
use reqwest::{Client, Response};
use std::time::Duration;

pub struct HttpClient {
    client: Client,
    timeout: Duration,
    retry_attempts: u32,
}

impl HttpClient {
    pub fn new(timeout_secs: u32, retry_attempts: u32) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs as u64))
            .user_agent("wallflow/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            timeout: Duration::from_secs(timeout_secs as u64),
            retry_attempts,
        }
    }

    pub async fn get_json<T>(&self, url: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // Implementation with retry logic
    }

    pub async fn download_file(&self, url: &str, path: &Path) -> Result<()> {
        // Implementation with progress tracking
    }
}
```

#### Step 1.3: Downloader Registry

**File**: `src/downloaders/registry.rs`

```rust
use super::traits::WallpaperDownloader;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

pub struct DownloaderRegistry {
    downloaders: HashMap<String, Arc<dyn WallpaperDownloader + Send + Sync>>,
}

impl DownloaderRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            downloaders: HashMap::new(),
        };

        // Register built-in downloaders
        registry.register_builtin_downloaders();
        registry
    }

    fn register_builtin_downloaders(&mut self) {
        // Register each downloader
    }

    pub fn get_downloader(&self, source: &str) -> Result<Arc<dyn WallpaperDownloader + Send + Sync>> {
        self.downloaders.get(source)
            .cloned()
            .ok_or_else(|| anyhow!("Unknown wallpaper source: {}", source))
    }

    pub fn list_sources(&self) -> Vec<String> {
        self.downloaders.keys().cloned().collect()
    }
}
```

### Phase 2: Migrate Existing Functionality (2-3 days)

#### Step 2.1: Extract Wallpaper Application Logic

**File**: `src/wallpaper/apply.rs`

```rust
// Move apply_wallpaper, set_wallpaper_awww, set_wallpaper_kde from wallpaper.rs
// Clean interface focusing just on applying downloaded wallpapers
```

#### Step 2.2: Implement Cleanup Functionality

**File**: `src/filesystem/cleanup.rs`

```rust
use anyhow::Result;
use std::path::Path;
use crate::config::CleanupConfig;

pub struct CleanupManager<'a> {
    config: &'a CleanupConfig,
    download_dir: &'a Path,
}

impl<'a> CleanupManager<'a> {
    pub fn new(config: &'a CleanupConfig, download_dir: &'a Path) -> Self {
        Self { config, download_dir }
    }

    pub async fn cleanup_if_needed(&self) -> Result<u32> {
        if !self.config.auto_cleanup {
            return Ok(0);
        }

        let removed = self.remove_old_wallpapers().await?;
        if removed > 0 {
            info!("🧹 Cleaned up {} old wallpapers", removed);
        }

        Ok(removed)
    }

    async fn remove_old_wallpapers(&self) -> Result<u32> {
        // Implementation: keep only the most recent config.keep_count files
    }
}
```

#### Step 2.3: Refactor Existing Downloaders

Move Wallhaven and Picsum into the new structure:

**File**: `src/downloaders/wallhaven.rs`

```rust
use super::traits::*;
use crate::http::HttpClient;

pub struct WallhavenDownloader {
    client: HttpClient,
}

#[async_trait]
impl WallpaperDownloader for WallhavenDownloader {
    async fn download(&self, request: &DownloadRequest) -> Result<DownloadedWallpaper> {
        // Refactored from existing set_wallhaven function
    }

    fn source_name(&self) -> &'static str {
        "wallhaven"
    }
}
```

### Phase 3: New Downloaders (5-7 days)

#### Step 3.1: NASA APOD (1 day)

**File**: `src/downloaders/apod.rs`

```rust
pub struct ApodDownloader {
    client: HttpClient,
}

#[derive(serde::Deserialize)]
struct ApodResponse {
    date: String,
    title: String,
    explanation: String,
    hdurl: Option<String>,
    url: String,
    media_type: String,
}

#[async_trait]
impl WallpaperDownloader for ApodDownloader {
    async fn download(&self, request: &DownloadRequest) -> Result<DownloadedWallpaper> {
        // NASA APOD API: https://api.nasa.gov/planetary/apod?api_key=DEMO_KEY
        // Educational: JSON parsing, optional fields handling
    }
}
```

#### Step 3.2: Bing Daily (1 day)

**File**: `src/downloaders/bing.rs`

```rust
#[derive(serde::Deserialize)]
struct BingResponse {
    images: Vec<BingImage>,
}

#[derive(serde::Deserialize)]
struct BingImage {
    url: String,
    title: String,
    copyright: String,
}

// Educational: XML/JSON hybrid APIs, URL construction
```

#### Step 3.3: Google Earth View (1 day)

**File**: `src/downloaders/earthview.rs`

```rust
// Educational: Geographic data, large image handling
```

#### Step 3.4: Reddit (2 days)

**File**: `src/downloaders/reddit.rs`

```rust
// Educational: Social media APIs, content filtering, JSON traversal
```

#### Step 3.5: MediaRSS (2 days)

**File**: `src/downloaders/mediarss.rs`

```rust
// Educational: RSS/XML parsing, feed validation
```

### Phase 4: Integration & Polish (2-3 days)

#### Step 4.1: Update Main Wallpaper Module

**File**: `src/wallpaper/mod.rs`

```rust
use crate::config::Config;
use crate::downloaders::DownloaderRegistry;
use anyhow::Result;

pub async fn set_wallpaper_by_source(config: &Config) -> Result<()> {
    let registry = DownloaderRegistry::new();
    let downloader = registry.get_downloader(&config.sources.default)?;

    let request = build_download_request(config)?;
    let wallpaper = downloader.download(&request).await?;

    // Apply wallpaper
    apply::apply_wallpaper(&wallpaper.file_path, config).await?;

    // Cleanup old downloads
    let cleanup_manager = crate::filesystem::cleanup::CleanupManager::new(
        &config.cleanup,
        Path::new(&config.paths.downloads)
    );
    cleanup_manager.cleanup_if_needed().await?;

    Ok(())
}
```

#### Step 4.2: Enhanced Configuration Usage

Implement all the unused config options:

```rust
// Use config.advanced.* in HTTP client
// Use config.cleanup.* in cleanup manager
// Use config.logging.* for structured logging
```

#### Step 4.3: Update CLI Commands

Add commands for testing individual downloaders:

```bash
cargo run -- test-downloader apod
cargo run -- test-downloader bing
cargo run -- list-sources
```

## Educational Benefits

### Rust Learning Outcomes

1. **Trait System**: Abstract interfaces for downloaders
2. **Async Programming**: Concurrent downloads, timeouts
3. **Error Handling**: Custom error types, graceful degradation
4. **Module Organization**: Large project structure
5. **Configuration Management**: Type-safe config validation
6. **HTTP Clients**: Real-world API integration patterns
7. **File I/O**: Streaming downloads, cleanup logic
8. **Testing**: Mock HTTP responses, integration tests

### Design Patterns

1. **Registry Pattern**: Pluggable downloader system
2. **Builder Pattern**: HTTP client configuration
3. **Strategy Pattern**: Different download strategies per source
4. **Factory Pattern**: Creating downloaders based on source type

## Migration Strategy

### Backward Compatibility

- Keep existing CLI commands working
- Maintain current config file format
- Gradual migration of functionality

### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_apod_downloader() {
        // Mock NASA API response
        // Test successful download
        // Test error handling
    }
}
```

### Risk Mitigation

- Implement one downloader at a time
- Keep existing wallpaper.rs as fallback during migration
- Extensive testing of each component
- Clear rollback plan if issues arise

## Success Criteria

### Technical Metrics

- [ ] All existing functionality preserved
- [ ] At least 5 new wallpaper sources implemented
- [ ] 90%+ test coverage for new modules
- [ ] Clean separation of concerns
- [ ] All unused config options implemented

### Educational Metrics

- [ ] Clear, documented module boundaries
- [ ] Comprehensive examples for each pattern
- [ ] Step-by-step implementation guides
- [ ] Error handling best practices demonstrated

This refactoring transforms wallflow from a simple wallpaper setter into a comprehensive, educational codebase demonstrating modern Rust patterns while maintaining the project's learning-focused mission.
