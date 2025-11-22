# Pre-commit Guide for wallflow

## 🪝 Pre-commit for Rust vs Python

Coming from Python, here's how Rust pre-commit hooks compare:

| Python Tool | Rust Equivalent | Purpose |
|-------------|-----------------|---------|
| `black` | `rustfmt` | Code formatting |
| `flake8`/`pylint` | `clippy` | Linting & code analysis |
| `isort` | Built into `rustfmt` | Import organization |
| `bandit` | `cargo-audit` | Security scanning |
| `mypy` | Built into `rustc` | Type checking (compile-time) |

## 🔧 What's Configured

### Core Rust Tools
```yaml
- rustfmt: Formats code according to Rust style guide
- clippy: Catches bugs, performance issues, and style violations
- cargo-audit: Scans for known security vulnerabilities
```

### General Quality
```yaml
- YAML/TOML validation: Config file correctness
- Markdown linting: Documentation quality
- Shell script checking: Script safety
- Git hygiene: Whitespace, large files, merge conflicts
```

### Custom Checks
```yaml
- No debug prints: Catches leftover println!/dbg! statements
- TODO detection: Flags TODO/FIXME comments (optional)
- Cargo.lock validation: Ensures dependencies are locked
```

## 🚀 Installation & Usage

### Setup (One-time)
```bash
# Automated installation
./scripts/setup-pre-commit.sh

# Manual installation
pip install pre-commit
pre-commit install
pre-commit install --hook-type commit-msg
```

### Daily Usage
```bash
# Automatic (runs on git commit)
git add .
git commit -m "feat: add wallpaper rotation"  # Hooks run automatically

# Manual (run on all files)
pre-commit run --all-files

# Update hook versions
pre-commit autoupdate
```

### Emergency Override
```bash
# Skip hooks in emergency (use sparingly!)
git commit --no-verify -m "hotfix: critical bug"
```

## 🎯 Hook Categories

### **Always Enabled (Critical)**
- `rustfmt` - Code formatting consistency
- `clippy` - Bug prevention and style
- `cargo-audit` - Security vulnerabilities
- `trailing-whitespace` - Clean commits
- `check-yaml` - Config file validity

### **Quality of Life**
- `markdownlint` - Documentation quality
- `shellcheck` - Script safety
- `conventional-pre-commit` - Consistent commit messages

### **Strict Mode (Optional)**
```yaml
# Uncomment in .pre-commit-config.yaml to enable:
- no-debug-prints: Prevents debugging code in commits
- check-todos: Flags TODO comments (good for releases)
- cargo-deny: Advanced dependency analysis
```

## 🔧 Customization

### Adjust Rustfmt Rules
Edit `rustfmt.toml`:
```toml
max_width = 150  # Match your preference
tab_spaces = 2   # Your current 2-space setup
```

### Skip Specific Hooks
```bash
# Skip clippy for this commit (not recommended)
SKIP=clippy git commit -m "wip: work in progress"

# Skip multiple hooks
SKIP=clippy,cargo-audit git commit -m "draft changes"
```

### Configure Hook Behavior

**Make TODOs non-blocking:**
```yaml
# In .pre-commit-config.yaml, add:
- id: check-todos
  name: Check for TODOs (warning only)
  entry: 'grep -r "TODO\|FIXME" src/ || true'  # Always pass
```

**Adjust Clippy strictness:**
```yaml
- id: clippy
  entry: cargo clippy --all-targets --all-features -- -W warnings  # Warnings instead of errors
```

## 🎨 Integration with IDEs

### VS Code
```json
// settings.json
{
  "rust-analyzer.check.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### Neovim/Vim
Pre-commit hooks complement rust-analyzer LSP integration.

## 🚦 CI/CD Integration

The hooks also run in GitHub Actions:
```yaml
# .github/workflows/ci.yml includes:
- name: Run pre-commit
  run: pre-commit run --all-files
```

## 🔍 Troubleshooting

### Common Issues

**Hook installation fails:**
```bash
# Update pre-commit itself
pip install --upgrade pre-commit

# Clear cache and reinstall
pre-commit clean && pre-commit install
```

**Clippy fails with complex errors:**
```bash
# Run clippy standalone to see full errors
cargo clippy --all-targets --all-features

# Fix issues one by one, then commit
```

**Formatting conflicts:**
```bash
# Run rustfmt to see what it wants to change
cargo fmt --check

# Apply formatting
cargo fmt
```

**Commit message format:**
```bash
# Use conventional commit format:
git commit -m "feat: add new wallpaper source"
git commit -m "fix: resolve daemon crash on exit"
git commit -m "docs: update installation guide"
```

## 🎯 Rust-Specific Benefits

Unlike Python projects, Rust pre-commit hooks provide extra value because:

1. **Compile-time guarantees**: Hooks catch issues before compilation
2. **Zero runtime overhead**: All checks happen at dev time
3. **Consistency**: rustfmt eliminates formatting debates
4. **Security focus**: cargo-audit catches vulnerabilities early
5. **Performance**: Clippy suggests optimizations

This setup ensures wallflow maintains professional code quality while you focus on building awesome features! 🌊