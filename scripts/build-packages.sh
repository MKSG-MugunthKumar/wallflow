#!/bin/bash
# Build script for wallflow packages
# Builds .deb, .rpm, and AppImage packages

set -e

echo "🌊 Building wallflow packages..."

# Ensure we're in the right directory
cd "$(dirname "$0")/.."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install cargo tools if needed
install_cargo_tools() {
    echo -e "${BLUE}📦 Installing cargo packaging tools...${NC}"

    if ! command_exists cargo-deb; then
        echo "Installing cargo-deb..."
        cargo install cargo-deb
    fi

    if ! command_exists cargo-generate-rpm; then
        echo "Installing cargo-generate-rpm..."
        cargo install cargo-generate-rpm
    fi

    if ! command_exists cargo-appimage; then
        echo "Installing cargo-appimage..."
        cargo install cargo-appimage
    fi
}

# Build release binary
build_release() {
    echo -e "${BLUE}🔨 Building release binary...${NC}"
    cargo build --release
}

# Build Debian package
build_deb() {
    echo -e "${BLUE}📦 Building .deb package...${NC}"

    if ! command_exists cargo-deb; then
        echo -e "${RED}cargo-deb not installed. Run with --install-tools${NC}"
        return 1
    fi

    cargo deb

    DEB_FILE=$(find target/debian -name "*.deb" | head -1)
    if [ -f "$DEB_FILE" ]; then
        echo -e "${GREEN}✅ Debian package: $DEB_FILE${NC}"

        # Show package info
        echo "Package info:"
        dpkg -I "$DEB_FILE"
    fi
}

# Build RPM package
build_rpm() {
    echo -e "${BLUE}📦 Building .rpm package...${NC}"

    if ! command_exists cargo-generate-rpm; then
        echo -e "${RED}cargo-generate-rpm not installed. Run with --install-tools${NC}"
        return 1
    fi

    cargo generate-rpm

    RPM_FILE=$(find target/generate-rpm -name "*.rpm" | head -1)
    if [ -f "$RPM_FILE" ]; then
        echo -e "${GREEN}✅ RPM package: $RPM_FILE${NC}"

        # Show package info
        echo "Package info:"
        rpm -qip "$RPM_FILE"
    fi
}

# Build AppImage
build_appimage() {
    echo -e "${BLUE}📦 Building AppImage...${NC}"

    # AppImage requires different approach - let's create a basic one
    APPDIR="target/wallflow.AppDir"
    mkdir -p "$APPDIR/usr/bin"
    mkdir -p "$APPDIR/usr/share/applications"
    mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"

    # Copy binary
    cp target/release/wallflow "$APPDIR/usr/bin/"

    # Create desktop file
    cat > "$APPDIR/usr/share/applications/wallflow.desktop" << EOF
[Desktop Entry]
Name=wallflow
Exec=wallflow
Icon=wallflow
Type=Application
Categories=Graphics;
Comment=Elegant wallpaper management with smooth transitions
EOF

    # Create a simple icon (you'd want a real icon here)
    cat > "$APPDIR/usr/share/icons/hicolor/256x256/apps/wallflow.png" << EOF
# Placeholder - add real icon file
EOF

    # Create AppRun script
    cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
exec "${HERE}/usr/bin/wallflow" "$@"
EOF
    chmod +x "$APPDIR/AppRun"

    # Create .desktop file in root
    cp "$APPDIR/usr/share/applications/wallflow.desktop" "$APPDIR/"

    echo -e "${GREEN}✅ AppImage directory: $APPDIR${NC}"
    echo "To create final AppImage, use: appimagetool $APPDIR wallflow.AppImage"
}

# Show package comparison
show_comparison() {
    echo -e "${BLUE}📊 Package Format Comparison:${NC}"
    echo ""
    echo "🔷 Debian (.deb):"
    echo "  ✅ Native APT integration"
    echo "  ✅ Automatic updates via 'apt upgrade'"
    echo "  ✅ Dependency resolution"
    echo "  ✅ System service integration"
    echo "  ❌ Debian/Ubuntu only"
    echo ""
    echo "🔷 RPM (.rpm):"
    echo "  ✅ Native DNF/YUM integration"
    echo "  ✅ Automatic updates via 'dnf upgrade'"
    echo "  ✅ Dependency resolution"
    echo "  ✅ System service integration"
    echo "  ❌ Red Hat/Fedora/SUSE only"
    echo ""
    echo "🔷 AppImage:"
    echo "  ✅ Universal Linux compatibility"
    echo "  ✅ No installation required"
    echo "  ✅ Portable (USB drive friendly)"
    echo "  ❌ No automatic updates"
    echo "  ❌ No system integration"
    echo "  ❌ Larger file size (bundled deps)"
}

# Main execution
case "${1:-all}" in
    deb)
        build_release
        build_deb
        ;;
    rpm)
        build_release
        build_rpm
        ;;
    appimage)
        build_release
        build_appimage
        ;;
    all)
        build_release
        build_deb
        build_rpm
        build_appimage
        show_comparison
        ;;
    --install-tools)
        install_cargo_tools
        ;;
    --help)
        echo "Usage: $0 [deb|rpm|appimage|all|--install-tools|--help]"
        echo ""
        echo "  deb         Build only .deb package"
        echo "  rpm         Build only .rpm package"
        echo "  appimage    Build only AppImage"
        echo "  all         Build all packages (default)"
        echo "  --install-tools  Install required cargo tools"
        echo "  --help      Show this help"
        ;;
    *)
        echo "Unknown option: $1"
        echo "Run '$0 --help' for usage information"
        exit 1
        ;;
esac

echo -e "${GREEN}🌊 Package build complete!${NC}"