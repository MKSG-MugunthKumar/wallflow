# Downloaders - Next Steps

## New Backends to Implement

- [ ] Wallhaven (API with search/categories)
- [ ] Bing Daily (simple JSON API)
- [ ] Reddit (r/wallpapers, r/earthporn)
- [ ] Unsplash (requires API key)

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
