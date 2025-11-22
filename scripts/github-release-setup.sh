#!/bin/bash
# Setup GitHub releases with multiple package formats

echo "🚀 Setting up GitHub releases for wallflow distribution"

# Create GitHub Actions workflow for releases
mkdir -p .github/workflows

cat > .github/workflows/release.yml << 'WORKFLOW_EOF'
name: Release

on:
  release:
    types: [published]
  push:
    tags:
      - 'v*'

jobs:
  build-packages:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install packaging tools
        run: |
          cargo install cargo-deb
          cargo install cargo-generate-rpm
          sudo apt-get update
          sudo apt-get install -y createrepo-c

      - name: Build release binary
        run: cargo build --release

      - name: Build .deb package
        run: cargo deb

      - name: Build .rpm package
        run: cargo generate-rpm

      - name: Create AppImage
        run: |
          # Basic AppImage creation
          APPDIR="wallflow.AppDir"
          mkdir -p "$APPDIR/usr/bin"
          cp target/release/wallflow "$APPDIR/usr/bin/"

          # Create AppRun
          cat > "$APPDIR/AppRun" << 'EOF'
          #!/bin/bash
          HERE="$(dirname "$(readlink -f "${0}")")"
          exec "${HERE}/usr/bin/wallflow" "$@"
          EOF
          chmod +x "$APPDIR/AppRun"

          # Create desktop file
          mkdir -p "$APPDIR/usr/share/applications"
          cat > "$APPDIR/usr/share/applications/wallflow.desktop" << 'EOF'
          [Desktop Entry]
          Name=wallflow
          Exec=wallflow
          Type=Application
          Categories=Graphics;
          EOF
          cp "$APPDIR/usr/share/applications/wallflow.desktop" "$APPDIR/"

          # Package as tar.gz for now (would need appimagetool for real AppImage)
          tar czf wallflow.AppImage.tar.gz -C . "$APPDIR"

      - name: Upload Release Assets
        uses: softprops/action-gh-release@v1
        with:
          files: |
            target/debian/*.deb
            target/generate-rpm/*.rpm
            wallflow.AppImage.tar.gz
            target/release/wallflow
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Create installation instructions
        run: |
          cat > INSTALL.md << 'EOF'
          # wallflow Installation

          ## Fedora/RHEL/CentOS
          ```bash
          # Download RPM from releases
          wget https://github.com/YOUR_USERNAME/wallflow/releases/latest/download/wallflow-*.rpm
          sudo dnf install ./wallflow-*.rpm
          ```

          ## Debian/Ubuntu
          ```bash
          # Download DEB from releases
          wget https://github.com/YOUR_USERNAME/wallflow/releases/latest/download/wallflow_*.deb
          sudo dpkg -i wallflow_*.deb
          sudo apt-get install -f  # Fix any dependency issues
          ```

          ## Universal (Any Linux)
          ```bash
          # Download AppImage
          wget https://github.com/YOUR_USERNAME/wallflow/releases/latest/download/wallflow.AppImage.tar.gz
          tar xzf wallflow.AppImage.tar.gz
          chmod +x wallflow.AppDir/AppRun
          ./wallflow.AppDir/AppRun --help
          ```

          ## Build from source
          ```bash
          git clone https://github.com/YOUR_USERNAME/wallflow
          cd wallflow
          cargo build --release
          sudo cp target/release/wallflow /usr/local/bin/
          ```
          EOF
WORKFLOW_EOF

echo "✅ Created GitHub Actions workflow for releases"

# Create installation script for users
cat > install.sh << 'INSTALL_EOF'
#!/bin/bash
# wallflow installation script

set -e

REPO="YOUR_USERNAME/wallflow"
LATEST_URL="https://api.github.com/repos/$REPO/releases/latest"

echo "🌊 Installing wallflow..."

# Detect distribution
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$ID
else
    echo "❌ Cannot detect Linux distribution"
    exit 1
fi

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" != "x86_64" ]; then
    echo "❌ Only x86_64 architecture is supported currently"
    exit 1
fi

# Get latest release info
echo "📡 Fetching latest release information..."
LATEST_JSON=$(curl -s "$LATEST_URL")
VERSION=$(echo "$LATEST_JSON" | grep '"tag_name":' | cut -d'"' -f4)

echo "📦 Latest version: $VERSION"

case "$DISTRO" in
    fedora|rhel|centos|almalinux|rocky)
        echo "🔴 Installing RPM package for $DISTRO..."
        RPM_URL=$(echo "$LATEST_JSON" | grep '"browser_download_url":' | grep '\.rpm"' | cut -d'"' -f4)
        wget -O wallflow.rpm "$RPM_URL"
        sudo dnf install -y ./wallflow.rpm
        rm wallflow.rpm
        ;;
    ubuntu|debian)
        echo "🔵 Installing DEB package for $DISTRO..."
        DEB_URL=$(echo "$LATEST_JSON" | grep '"browser_download_url":' | grep '\.deb"' | cut -d'"' -f4)
        wget -O wallflow.deb "$DEB_URL"
        sudo dpkg -i wallflow.deb
        sudo apt-get install -f -y
        rm wallflow.deb
        ;;
    *)
        echo "📦 Installing AppImage for $DISTRO..."
        APPIMAGE_URL=$(echo "$LATEST_JSON" | grep '"browser_download_url":' | grep 'AppImage' | cut -d'"' -f4)
        wget -O wallflow.AppImage.tar.gz "$APPIMAGE_URL"
        tar xzf wallflow.AppImage.tar.gz
        chmod +x wallflow.AppDir/AppRun

        # Optionally move to /usr/local/bin
        read -p "Install to /usr/local/bin? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            sudo cp wallflow.AppDir/usr/bin/wallflow /usr/local/bin/
            echo "✅ Installed to /usr/local/bin/wallflow"
        else
            echo "✅ Extracted to ./wallflow.AppDir/AppRun"
        fi

        rm wallflow.AppImage.tar.gz
        ;;
esac

echo "🎉 wallflow installation complete!"
echo "Run 'wallflow examples' to get started"
INSTALL_EOF

chmod +x install.sh

echo "✅ Created install.sh script for users"
echo ""
echo "📋 Distribution strategy:"
echo "1. Users run: curl -sSL https://github.com/YOUR_USERNAME/wallflow/raw/main/install.sh | bash"
echo "2. Or download packages manually from GitHub Releases"
echo "3. GitHub Actions automatically builds packages on git tags"
echo ""
echo "🚀 To create a release:"
echo "git tag v0.1.0"
echo "git push origin v0.1.0"
echo ""
echo "📝 Don't forget to update YOUR_USERNAME in the files!"