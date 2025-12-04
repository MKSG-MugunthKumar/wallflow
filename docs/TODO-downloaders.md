# Downloaders - Implementation Status

## Implemented Sources

- [x] **Local** - Local wallpaper collection
- [x] **Wallhaven** - API with search/categories, optional API key
- [x] **Picsum** - Random photos from Lorem Picsum
- [x] **APOD** - NASA Astronomy Picture of the Day
- [x] **Bing** - Bing Photo of the Day (daily curated images)
- [x] **Reddit** - Subreddit wallpapers (r/wallpapers, r/earthporn, etc.)
- [x] **Earth View** - Google satellite imagery
- [x] **Unsplash** - High-resolution photos (requires API key)

## CLI Usage

```bash
# Sources that accept search queries
wallflow wallhaven nature mountains
wallflow reddit earthporn
wallflow unsplash architecture

# Sources with daily/curated content (no query needed)
wallflow bing
wallflow earthview
wallflow apod
wallflow picsum
```

## Configuration

Sources can be configured in `~/.config/wallflow/config.yml`:

```yaml
sources:
  default: local

  wallhaven:
    api_key: "your-api-key" # Optional, for NSFW content
    purity: [sfw]
    categories: [general]

  unsplash:
    api_key: "your-api-key" # Required

  reddit:
    subreddit: "wallpapers" # Default subreddit
```

## Plugin Architecture

### Current: Compile-time (Traits + Generics)

- `WallpaperDownloader` trait in `traits.rs`
- Register in `registry.rs`
- Requires recompilation to add new sources

### Future: Runtime Plugins

**Option A: Dynamic Libraries (`libloading`)**

- Load `.so`/`.dylib` at runtime
- Fast, native performance
- Platform-specific binaries

**Option B: WASM Plugins**

- Sandboxed, cross-platform
- Slightly slower than native
- Single binary works everywhere

### References

- `libloading` crate: https://docs.rs/libloading
- `wasmtime` crate: https://docs.rs/wasmtime
- Plugin pattern example: https://michael-f-bryan.github.io/rust-ffi-guide/
