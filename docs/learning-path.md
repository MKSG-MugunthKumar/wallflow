# 🦀 Rust Learning Path with wallflow

A practical, project-based approach to learning Rust through building a real wallpaper management tool.

## 🎯 Why This Learning Approach Works

**Project-Based Learning Benefits:**
- ✅ **Real motivation** - You'll actually use this daily
- ✅ **Practical skills** - Learn by solving real problems
- ✅ **Immediate feedback** - See your progress working
- ✅ **Portfolio piece** - Showcase your Rust skills

**What Makes wallflow Perfect for Learning:**
- **Right complexity** - Not too simple, not overwhelming
- **Multiple concepts** - CLI, async, HTTP, configs, system integration
- **Incremental development** - Add features as you learn
- **Cross-platform target** - Learn portable development

## 📚 Phase 1: Rust Fundamentals (Week 1-2)

### **Day 1-3: Get Started**

**Resources:**
- [The Rust Book](https://doc.rust-lang.org/book/) chapters 1-6
- [Rustlings](https://github.com/rust-lang/rustlings) exercises

**Key Concepts:**
```rust
// Ownership & Borrowing
fn take_ownership(s: String) { }           // Takes ownership
fn borrow_immutable(s: &String) { }        // Borrows immutably
fn borrow_mutable(s: &mut String) { }      // Borrows mutably

// Error Handling
fn might_fail() -> Result<String, std::io::Error> {
    std::fs::read_to_string("file.txt")
}

// Pattern Matching
match might_fail() {
    Ok(contents) => println!("File: {}", contents),
    Err(e) => eprintln!("Error: {}", e),
}
```

**Practice with wallflow:**
- Read and understand `src/config.rs`
- Modify default values in the config
- Add a new field to the configuration struct

### **Day 4-7: CLI & Project Structure**

**Resources:**
- [Command Line Applications in Rust](https://rust-cli.github.io/book/)
- [clap documentation](https://docs.rs/clap/latest/clap/)

**Hands-on Tasks:**
```bash
# 1. Add a new command to the CLI
cargo run -- --help    # See current commands
# Add a "status" command to show daemon state

# 2. Experiment with logging levels
cargo run -- --verbose local
RUST_LOG=debug cargo run -- config

# 3. Modify the CLI structure
# Add flags like --dry-run or --no-cleanup
```

**Key Learning:**
- How `#[derive(Parser)]` works
- Command line argument parsing
- Project structure and modules
- Error propagation with `?`

## 📚 Phase 2: Systems Programming (Week 3-4)

### **HTTP & Async Programming**

**Resources:**
- [Async Book](https://rust-lang.github.io/async-book/)
- [reqwest documentation](https://docs.rs/reqwest/latest/reqwest/)

**Project Tasks:**
1. **Implement Wallhaven API** (see `bin/wallflow-reference` for URLs):
```rust
// In src/wallpaper.rs
pub async fn download_wallhaven(category: &str, resolution: &Resolution) -> Result<PathBuf> {
    let client = reqwest::Client::new();
    let url = format!("https://wallhaven.cc/api/v1/search?q={}&categories=100&purity=100&resolutions={}",
        category, resolution.as_string());

    let response: WallhavenResponse = client
        .get(&url)
        .send()
        .await?
        .json()
        .await?;

    // Download the actual image...
    todo!("Implement image download")
}
```

2. **Add Picsum integration**:
```rust
pub async fn download_picsum(resolution: &Resolution) -> Result<PathBuf> {
    let url = format!("https://picsum.photos/{}/{}?random",
        resolution.width, resolution.height);
    // Implement download...
}
```

**Key Learning:**
- `async`/`await` syntax
- HTTP clients and JSON parsing
- File I/O and path handling
- Error handling in async contexts

### **System Integration & Process Management**

**Project Tasks:**
1. **Wire up display detection**:
```rust
// Use the display.rs functions in wallpaper.rs
let resolution = config.get_wallhaven_resolution()?;
download_wallhaven(category, &resolution).await?;
```

2. **Improve daemon mode**:
```rust
// Add graceful shutdown
use tokio::signal;

async fn setup_signal_handlers() {
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    tokio::spawn(async move {
        sigterm.recv().await;
        info!("Graceful shutdown requested");
        std::process::exit(0);
    });
}
```

3. **Add daemon status checking**:
```bash
# New command to implement
wallflow status    # Show if daemon is running, last change, etc.
```

## 📚 Phase 3: Polish & Advanced Features (Week 5-6)

### **Configuration & Error Handling**

**Project Tasks:**
1. **Better configuration validation**:
```rust
impl Config {
    pub fn validate(&self) -> Result<()> {
        if !Path::new(&self.paths.local).exists() {
            return Err(anyhow!("Local wallpaper directory does not exist: {}", self.paths.local));
        }
        // Add more validation...
        Ok(())
    }
}
```

2. **Configuration migration**:
```rust
// Handle old config formats gracefully
// Add version field and migration logic
```

### **Testing & Documentation**

**Learn Rust Testing:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_wallhaven_api() {
        // Mock HTTP responses for testing
    }
}
```

**Documentation:**
```rust
/// Downloads a wallpaper from Wallhaven API
///
/// # Arguments
/// * `category` - Search category (e.g., "nature", "abstract")
/// * `resolution` - Target resolution for the wallpaper
///
/// # Returns
/// Path to the downloaded wallpaper file
///
/// # Errors
/// Returns error if API request fails or download fails
pub async fn download_wallhaven(category: &str, resolution: &Resolution) -> Result<PathBuf> {
    // Implementation...
}
```

## 📚 Phase 4: Advanced Rust (Week 7-8)

### **Performance & Optimization**

**Project Tasks:**
1. **Parallel downloads**:
```rust
use tokio::task::JoinSet;

async fn download_multiple_wallpapers(urls: Vec<String>) -> Result<Vec<PathBuf>> {
    let mut set = JoinSet::new();

    for url in urls {
        set.spawn(download_single_wallpaper(url));
    }

    let mut results = Vec::new();
    while let Some(res) = set.join_next().await {
        results.push(res??);
    }

    Ok(results)
}
```

2. **Caching & state management**:
```rust
// Add wallpaper cache
// Remember recently used wallpapers
// Smart cleanup based on usage
```

3. **Cross-platform improvements**:
```rust
// Better Windows/macOS support
// Platform-specific optimizations
```

### **Advanced Features**

**Choose Your Adventure:**

**Option A: TUI Interface** (Great for learning):
```rust
// Add ratatui-based interface
// Vim keybindings
// Real-time status updates
```

**Option B: Web API** (Learn web development):
```rust
// Add axum web server
// HTTP API for remote control
// Web dashboard
```

**Option C: Plugin System** (Learn advanced patterns):
```rust
// Dynamic loading of wallpaper sources
// Custom transition effects
// Scriptable automation
```

## 🎯 Learning Milestones & Skills

### **Week 1-2 Checkpoint**
**Skills Gained:**
- ✅ Rust syntax and ownership model
- ✅ Error handling with Result types
- ✅ Project structure and modules
- ✅ CLI argument parsing

**Deliverable:** Working `wallflow config` and `wallflow local` commands

### **Week 3-4 Checkpoint**
**Skills Gained:**
- ✅ Async programming with tokio
- ✅ HTTP clients and JSON parsing
- ✅ File I/O and system integration
- ✅ Process management

**Deliverable:** Working `wallflow wallhaven` and `wallflow picsum` commands

### **Week 5-6 Checkpoint**
**Skills Gained:**
- ✅ Testing strategies
- ✅ Documentation and API design
- ✅ Configuration management
- ✅ Error handling patterns

**Deliverable:** Production-ready daemon with all features

### **Week 7-8 Checkpoint**
**Skills Gained:**
- ✅ Performance optimization
- ✅ Advanced async patterns
- ✅ Cross-platform development
- ✅ Choose-your-own advanced feature

**Deliverable:** Polished tool with advanced features

## 🚀 Beyond wallflow

**After completing this project, you'll be ready for:**

**Systems Programming:**
- Operating system components
- Network services and protocols
- Database engines
- Compilers and interpreters

**Web Development:**
- REST APIs with axum or warp
- WebAssembly applications
- Real-time systems with websockets

**Performance-Critical Applications:**
- Game engines
- Scientific computing
- Cryptocurrency and blockchain
- Embedded systems

## 💡 Tips for Success

**Rust-Specific Tips:**
- **Embrace the compiler** - Error messages are your friend
- **Start simple** - Don't try to optimize too early
- **Use the community** - r/rust, Discord, Stack Overflow
- **Read others' code** - Study popular Rust projects

**Learning Strategy:**
- **Build incrementally** - Get each feature working before adding the next
- **Test frequently** - `cargo check` and `cargo test` often
- **Document as you go** - Write docstrings for your functions
- **Refactor regularly** - As you learn better patterns, improve old code

**Resources to Bookmark:**
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust Standard Library Docs](https://doc.rust-lang.org/std/)
- [Crates.io](https://crates.io/) - Package registry
- [This Week in Rust](https://this-week-in-rust.org/) - Weekly newsletter

---

*"The best way to learn Rust is to build something you'll actually use"* 🦀

**Happy coding!** Feel free to ask questions, share progress, or request help with specific concepts as you work through this learning path.