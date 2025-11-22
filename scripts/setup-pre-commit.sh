#!/bin/bash
# Setup pre-commit hooks for wallflow development

set -e

echo "🪝 Setting up pre-commit hooks for wallflow"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if pre-commit is installed
if ! command -v pre-commit &> /dev/null; then
    echo -e "${YELLOW}📦 Installing pre-commit...${NC}"

    # Try pip first, then package manager
    if command -v pip3 &> /dev/null; then
        pip3 install --user pre-commit
    elif command -v pip &> /dev/null; then
        pip install --user pre-commit
    elif command -v dnf &> /dev/null; then
        echo "Installing via dnf..."
        sudo dnf install -y pre-commit
    elif command -v apt &> /dev/null; then
        echo "Installing via apt..."
        sudo apt update && sudo apt install -y pre-commit
    else
        echo -e "${YELLOW}⚠️  Please install pre-commit manually:${NC}"
        echo "pip install pre-commit"
        echo "OR visit: https://pre-commit.com/#installation"
        exit 1
    fi
fi

# Install additional tools needed by hooks
echo -e "${BLUE}🔧 Installing additional tools...${NC}"

# Install cargo-audit for security scanning
if ! command -v cargo-audit &> /dev/null; then
    echo "Installing cargo-audit..."
    cargo install cargo-audit
fi

# Install yamllint
if ! command -v yamllint &> /dev/null; then
    pip3 install --user yamllint || pip install --user yamllint
fi

# Install markdownlint-cli (requires Node.js)
if command -v npm &> /dev/null; then
    if ! command -v markdownlint &> /dev/null; then
        echo "Installing markdownlint-cli..."
        npm install -g markdownlint-cli || echo "⚠️ markdownlint installation failed (run as sudo if needed)"
    fi
else
    echo -e "${YELLOW}⚠️ npm not found. Markdown linting will be skipped.${NC}"
    echo "To enable: install Node.js and run 'npm install -g markdownlint-cli'"
fi

# Install shellcheck
if ! command -v shellcheck &> /dev/null; then
    if command -v dnf &> /dev/null; then
        sudo dnf install -y shellcheck || echo "⚠️ shellcheck installation failed"
    elif command -v apt &> /dev/null; then
        sudo apt install -y shellcheck || echo "⚠️ shellcheck installation failed"
    else
        echo -e "${YELLOW}⚠️ Please install shellcheck manually for shell script linting${NC}"
    fi
fi

echo -e "${BLUE}🔗 Installing pre-commit hooks...${NC}"

# Install the pre-commit hooks
pre-commit install

# Also install commit message hook
pre-commit install --hook-type commit-msg

echo -e "${GREEN}✅ Pre-commit setup complete!${NC}"
echo ""
echo -e "${BLUE}📋 What was installed:${NC}"
echo "🔧 Code Formatting:"
echo "  • rustfmt - Automatic Rust code formatting"
echo "  • clippy - Rust linting (catches bugs and style issues)"
echo ""
echo "🛡️ Security & Quality:"
echo "  • cargo-audit - Security vulnerability scanning"
echo "  • shellcheck - Shell script analysis"
echo "  • yamllint - YAML file validation"
echo "  • markdownlint - Markdown formatting"
echo ""
echo "📝 Git Hygiene:"
echo "  • Trailing whitespace removal"
echo "  • End-of-file newline enforcement"
echo "  • Large file detection"
echo "  • Merge conflict detection"
echo "  • Conventional commit message format"
echo ""
echo -e "${BLUE}🚀 Usage:${NC}"
echo "  • Hooks run automatically on git commit"
echo "  • Run manually: pre-commit run --all-files"
echo "  • Update hooks: pre-commit autoupdate"
echo "  • Skip for emergency: git commit --no-verify"
echo ""
echo -e "${GREEN}Happy coding! 🦀${NC}"
