#!/usr/bin/env bash
# wallflow installation script
# Quick and easy installation for wallflow

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Helper functions
print_header() {
    echo -e "${BLUE}🌊 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${PURPLE}ℹ️ $1${NC}"
}

# Check dependencies
check_dependencies() {
    print_header "Checking Dependencies"

    local deps_missing=()
    local deps_optional=()

    # Required dependencies
    if ! command -v curl >/dev/null 2>&1; then
        deps_missing+=("curl")
    else
        print_success "curl found"
    fi

    if ! command -v jq >/dev/null 2>&1; then
        deps_missing+=("jq")
    else
        print_success "jq found"
    fi

    # Wallpaper backends (at least one required)
    local backends_found=false

    # Check for awww (required wallpaper daemon)
    if command -v awww >/dev/null 2>&1 && command -v awww-daemon >/dev/null 2>&1; then
        print_success "awww found (Wayland wallpaper daemon)"
        backends_found=true

        # Check if awww-daemon service exists and is properly configured
        if systemctl --user list-unit-files awww-daemon.service >/dev/null 2>&1; then
            print_success "awww-daemon service configured"
        else
            print_warning "awww-daemon service not found - you'll need to set it up"
            print_info "Create systemd service: ~/.config/systemd/user/awww-daemon.service"
        fi
    fi

    if ! $backends_found; then
        print_error "awww not found - wallflow requires awww to function"
        print_info "Install awww from https://codeberg.org/LGFae/awww"
        return 1
    fi

    # Optional dependencies
    if command -v wal >/dev/null 2>&1; then
        print_success "pywal found (color scheme generation)"
    else
        deps_optional+=("pywal")
    fi

    if command -v yq >/dev/null 2>&1; then
        print_success "yq found (YAML processing)"
    else
        deps_optional+=("yq")
    fi

    if command -v systemctl >/dev/null 2>&1; then
        print_success "systemd found (timer support)"
    else
        print_warning "systemd not found - timer functionality will not be available"
    fi

    # Report missing dependencies
    if [[ ${#deps_missing[@]} -gt 0 ]]; then
        print_error "Missing required dependencies: ${deps_missing[*]}"
        echo ""
        print_info "Install missing dependencies:"
        echo "  Fedora/RHEL: sudo dnf install ${deps_missing[*]}"
        echo "  Ubuntu/Debian: sudo apt install ${deps_missing[*]}"
        echo "  Arch: sudo pacman -S ${deps_missing[*]}"
        echo "  macOS: brew install ${deps_missing[*]}"
        return 1
    fi

    if [[ ${#deps_optional[@]} -gt 0 ]]; then
        print_warning "Optional dependencies not found: ${deps_optional[*]}"
        print_info "These provide enhanced functionality but wallflow will work without them"
    fi

    print_success "Dependency check passed"
    return 0
}

# Installation function
install_wallflow() {
    print_header "Installing wallflow"

    # Check if we're in the wallflow directory
    if [[ ! -f "Makefile" ]] || [[ ! -d "bin" ]]; then
        print_error "Installation must be run from the wallflow directory"
        print_info "Clone the repository first: git clone https://github.com/MKSG-MugunthKumar/wallflow.git"
        exit 1
    fi

    # Run make install
    if make install; then
        print_success "wallflow installed successfully"
    else
        print_error "Installation failed"
        exit 1
    fi

    # Test installation
    print_header "Testing Installation"
    if make test; then
        print_success "Installation test passed"
    else
        print_warning "Installation test failed - check the output above"
    fi
}

# Setup function
setup_wallflow() {
    print_header "Setting up wallflow"

    # Initialize configuration if needed
    if command -v wallflow-config >/dev/null 2>&1; then
        if ! wallflow-config validate >/dev/null 2>&1; then
            print_info "Initializing configuration..."
            wallflow-config init
        else
            print_success "Configuration is valid"
        fi
    else
        print_warning "wallflow-config not found in PATH"
        print_info "You may need to restart your shell or run: source ~/.zshrc"
    fi

    # Ask about enabling timer
    echo ""
    read -p "Enable automatic wallpaper rotation? [y/N]: " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Enabling wallflow timer..."
        if make enable; then
            print_success "wallflow timer enabled"
        else
            print_warning "Failed to enable timer - you can enable it later with: make enable"
        fi
    else
        print_info "Automatic rotation not enabled"
        print_info "Enable later with: make enable"
    fi
}

# Usage information
show_usage() {
    print_header "wallflow Quick Start"
    echo ""
    echo "Commands:"
    echo "  wallflow local              # Set random wallpaper from local collection"
    echo "  wallflow wallhaven nature   # Download and set from Wallhaven"
    echo "  wallflow picsum             # Set random photo from Picsum"
    echo "  wallflow config             # Show current configuration"
    echo ""
    echo "Configuration:"
    echo "  wallflow-config edit        # Edit configuration file"
    echo "  wallflow-config validate    # Check configuration"
    echo "  wallflow-config test        # Test wallflow with current config"
    echo ""
    echo "Service management:"
    echo "  make enable                 # Enable automatic wallpaper rotation"
    echo "  make status                 # Check timer status"
    echo "  make logs                   # View service logs"
    echo "  make disable                # Disable automatic rotation"
    echo ""
    echo "Next steps:"
    echo "1. Add wallpapers to ~/Pictures/Wallpapers/ (for local source)"
    echo "2. Edit configuration: wallflow-config edit"
    echo "3. Test manually: wallflow local"
    echo "4. Enable automatic rotation: make enable"
}

# Main execution
main() {
    print_header "wallflow Installation"
    echo "Elegant wallpaper management with smooth transitions"
    echo ""

    # Check dependencies first
    if ! check_dependencies; then
        print_error "Dependency check failed"
        exit 1
    fi

    echo ""

    # Install wallflow
    install_wallflow

    echo ""

    # Setup configuration and services
    setup_wallflow

    echo ""

    # Show usage information
    show_usage

    echo ""
    print_success "wallflow installation complete! 🎉"
}

# Show help if requested
case "${1:-}" in
    -h|--help|help)
        echo "wallflow Installation Script"
        echo ""
        echo "Usage: $0 [OPTIONS]"
        echo ""
        echo "Options:"
        echo "  -h, --help    Show this help message"
        echo ""
        echo "This script will:"
        echo "1. Check for required dependencies"
        echo "2. Install wallflow scripts to ~/.local/bin"
        echo "3. Set up configuration and systemd services"
        echo "4. Optionally enable automatic wallpaper rotation"
        exit 0
        ;;
esac

# Run main installation
main "$@"