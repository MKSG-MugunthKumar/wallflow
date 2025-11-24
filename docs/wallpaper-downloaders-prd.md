# Wallpaper Downloaders PRD

## Executive Summary

This PRD outlines the implementation of a modular wallpaper downloader system for wallflow, inspired by the Variety wallpaper manager plugin ecosystem. The goal is to create a clean, extensible architecture that supports multiple wallpaper sources while maintaining the educational focus of the wallflow project.

## Current State Analysis

### Existing Implementation (src/wallpaper.rs)

- **Current Sources**: Wallhaven, Picsum, Local files
- **Architecture Issues**:
  - Mixed concerns (HTTP client, file operations, wallpaper application)
  - Hardcoded source implementations
  - No error handling abstraction
  - Limited extensibility

### Unused Configuration Options

- `config.cleanup.*` - Auto-cleanup functionality not implemented
- `config.logging.*` - Custom logging configuration not used
- `config.advanced.*` - Parallel downloads, retry logic, timeouts not implemented
- `config.integration.desktop.notify_completion` - Desktop notifications not implemented
- `config.integration.desktop.icon_theme_sync` - Icon theme syncing not implemented

## Proposed Architecture

### 1. Modular Downloader System

```rust
// Core trait for all downloaders
pub trait WallpaperDownloader {
    async fn download(&self, config: &DownloaderConfig) -> Result<DownloadedWallpaper>;
    fn source_name(&self) -> &'static str;
    fn requires_api_key(&self) -> bool;
}

// Common download result
pub struct DownloadedWallpaper {
    pub file_path: PathBuf,
    pub source_url: Option<String>,
    pub metadata: WallpaperMetadata,
}
```

### 2. Implementable Downloaders

#### Tier 1: High Priority (Proven in Variety)

| Source                   | Description                  | API Details                      | Educational Value                    |
| ------------------------ | ---------------------------- | -------------------------------- | ------------------------------------ |
| **NASA APOD** ✅         | Astronomy Picture of the Day | Public API, no key needed        | JSON parsing, date handling          |
| **Bing Daily** ✅        | Microsoft Bing daily images  | JSON endpoint, no auth           | REST APIs, image metadata            |
| **Google Earth View** ✅ | Satellite imagery wallpapers | Public JSON API                  | Geographic data, image URLs          |
| **Reddit** ✅            | Subreddit wallpapers         | Public JSON (add .json to URL)   | Social media APIs, content filtering |
| **Unsplash** ⚠️          | High-quality photography     | Has hardcoded API key in Variety | Rate limiting, attribution           |
| **MediaRSS** ✅          | RSS feed-based sources       | RSS/XML parsing                  | Feed processing, content validation  |

#### Tier 2: Medium Priority (Implementation Available)

| Source                     | Description              | API Details           | Implementation Notes            |
| -------------------------- | ------------------------ | --------------------- | ------------------------------- |
| **ArtStation** ✅          | Digital art community    | Public API endpoints  | Complex pagination, art-focused |
| **National Geographic** ✅ | Professional photography | Web scraping approach | Ethical scraping practices      |
| **Wallhaven Legacy** ✅    | Wallpaper community      | Older API version     | Backward compatibility patterns |

#### Tier 3: Advanced/Future (Avoid for Now)

| Source                    | Description              | Why Avoid                      | Alternative                |
| ------------------------- | ------------------------ | ------------------------------ | -------------------------- |
| **Unsplash with own key** | High-quality photography | Requires user API registration | Use other free sources     |
| **Flickr**                | Photo sharing platform   | Complex OAuth required         | Use Reddit or APOD         |
| **500px**                 | Photography community    | Paid API access                | Use Google Earth View      |
| **Local Network Shares**  | SMB/NFS directories      | Complex network protocols      | Focus on HTTP sources      |
| **Dynamic Generators**    | Procedural wallpapers    | Complex image generation       | Use existing image sources |

#### Recommended Implementation Order

**Phase 1 (Week 1)**: Easy wins

1. **NASA APOD** - Simple JSON API, educational content
2. **Bing Daily** - Straightforward endpoint, reliable images
3. **Google Earth View** - Beautiful imagery, simple JSON

**Phase 2 (Week 2)**: Social/Community 4. **Reddit** - Learn social media APIs, content filtering 5. **MediaRSS** - RSS processing, feed validation

**Phase 3 (Week 3)**: Advanced 6. **ArtStation** - Complex pagination, art-specific filtering 7. **National Geographic** - Ethical web scraping techniques

### 3. Enhanced Configuration Schema

```yaml
# Enhanced configuration supporting new downloaders
sources:
  downloaders:
    unsplash:
      enabled: true
      categories: ["nature", "landscape"]
      orientation: "landscape" # landscape, portrait, squarish
      featured: true

    nasa_apod:
      enabled: true
      hd: true
      date_range: "recent" # recent, random, specific

    bing_daily:
      enabled: true
      market: "en-US"

    reddit:
      enabled: false
      subreddits: ["EarthPorn", "wallpapers"]
      sort_by: "hot" # hot, new, top
      time_period: "day" # hour, day, week, month

# New advanced configuration usage
advanced:
  parallel_downloads: 3 # Used: concurrent downloads
  retry_attempts: 3 # Used: failure resilience
  timeout: 30 # Used: request timeouts
  cache_duration: 3600 # New: cache downloaded metadata

# Enhanced cleanup configuration
cleanup:
  keep_count: 10 # Used: limit downloaded files
  auto_cleanup: true # Used: automatic cleanup
  cleanup_schedule: "daily" # New: when to run cleanup

# Enhanced logging configuration
logging:
  enabled: true # Used: enable/disable logging
  level: "info" # Used: log level
  file: "~/.local/share/wallflow/wallflow.log" # Used: log file
  timestamp: true # Used: timestamp format
  structured: false # New: JSON structured logs

# Enhanced desktop integration
integration:
  desktop:
    notify_completion: true # Used: desktop notifications
    icon_theme_sync: false # Used: sync with icon themes
    update_terminal_colors: false # Used: terminal color updates
```

## Implementation Phases

### Phase 1: Architecture Refactoring (Week 1)

**Goal**: Clean separation of concerns

1. **Create downloader abstraction layer**

   ```rust
   // src/downloaders/mod.rs - New module
   // src/downloaders/traits.rs - Core traits
   // src/downloaders/registry.rs - Downloader registry
   ```

2. **Extract HTTP client utilities**

   ```rust
   // src/http/client.rs - Reusable HTTP client
   // src/http/cache.rs - Response caching
   // src/http/retry.rs - Retry logic
   ```

3. **Implement unused config options**
   - Cleanup functionality
   - Advanced download settings
   - Enhanced logging

### Phase 2: Core Downloaders (Week 2-3)

**Goal**: Implement Tier 1 downloaders

1. **Easy wins** (1-2 days each):

   - NASA APOD
   - Bing Daily
   - Pexels

2. **Medium complexity** (2-3 days each):
   - Unsplash
   - Pixabay
   - Interfacelift

### Phase 3: Social/Community Sources (Week 4)

**Goal**: Implement Tier 2 downloaders

1. **Reddit integration**

   - JSON API usage
   - Subreddit filtering
   - Content safety

2. **Flickr integration**
   - Public API
   - Search capabilities

### Phase 4: Advanced Features (Week 5+)

**Goal**: Implement Tier 3 and polish

1. **RSS feed support**
2. **Local network shares**
3. **Plugin system for custom sources**

## Technical Specifications

### Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum DownloaderError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("Rate limited: retry after {seconds}s")]
    RateLimit { seconds: u64 },

    #[error("Invalid configuration: {0}")]
    Configuration(String),
}
```

### Caching Strategy

- **Metadata caching**: API responses cached for 1 hour
- **Image caching**: Downloaded images managed by cleanup config
- **Rate limit caching**: Store rate limit status per source

### Testing Strategy

```rust
// Integration tests for each downloader
#[cfg(test)]
mod tests {
    // Mock HTTP responses
    // Configuration validation
    // Error handling scenarios
}
```

## Educational Benefits

### Learning Opportunities

1. **HTTP Client Patterns**: Different API styles (REST, GraphQL, XML)
2. **Error Handling**: Robust error management and user feedback
3. **Async Programming**: Concurrent downloads and rate limiting
4. **Configuration Management**: Type-safe config with validation
5. **Plugin Architecture**: Extensible system design
6. **Testing**: Mock services and integration testing

### Code Quality Improvements

1. **Separation of Concerns**: Clean architecture boundaries
2. **Type Safety**: Strong typing for all configurations
3. **Error Recovery**: Graceful degradation and retry logic
4. **Documentation**: Comprehensive examples for each downloader

## Risk Mitigation

### API Reliability

- **Multiple fallback sources**: If one API fails, try others
- **Rate limit respect**: Implement proper backoff strategies
- **Terms of Service**: Ensure all usage complies with TOS

### Performance Considerations

- **Parallel downloads**: Respect source rate limits
- **Memory usage**: Stream large files instead of loading in memory
- **Disk space**: Implement effective cleanup strategies

### Security Considerations

- **Input validation**: Sanitize all external inputs
- **File type validation**: Verify downloaded content is actually images
- **Path traversal**: Prevent malicious file paths

## Success Metrics

### Technical Metrics

- **Downloader success rate**: >95% successful downloads
- **Response time**: <30s average download time
- **Error recovery**: <5% unrecoverable errors

### Educational Metrics

- **Code complexity**: Maintainable modules (<200 lines each)
- **Documentation coverage**: 100% public API documentation
- **Test coverage**: >80% code coverage

## Deliverables

### Phase 1 Deliverables

- [ ] Refactored `src/wallpaper.rs` into modular system
- [ ] New `src/downloaders/` module structure
- [ ] Implemented cleanup functionality
- [ ] Enhanced logging system
- [ ] Configuration validation

### Phase 2 Deliverables

- [ ] 6 working Tier 1 downloaders
- [ ] Integration tests for each downloader
- [ ] Updated documentation with examples
- [ ] CLI commands for testing individual downloaders

### Phase 3+ Deliverables

- [ ] Reddit and Flickr downloaders
- [ ] RSS feed support
- [ ] Plugin system architecture
- [ ] Performance optimization
- [ ] Cross-platform testing

## Future Considerations

### TUI Integration

- **Preview mode**: Show wallpaper thumbnails before download
- **Source browser**: Browse available categories/collections
- **Download progress**: Real-time download status

### Advanced Features

- **Machine learning**: Auto-categorization of downloaded wallpapers
- **Color analysis**: Download wallpapers matching current color scheme
- **Weather integration**: Download weather-appropriate wallpapers
- **Time-based**: Different wallpapers for different times of day

---

This PRD provides a comprehensive roadmap for transforming wallflow from a simple wallpaper setter into a robust, educational wallpaper management ecosystem while maintaining its focus on learning Rust through practical development.
