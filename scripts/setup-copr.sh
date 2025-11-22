#!/bin/bash
# Setup COPR repository for wallflow distribution

echo "🌊 Setting up COPR for wallflow distribution"

# COPR setup instructions
cat << 'EOF'
🗄️ COPR (Community Package Repository) Setup

COPR is Fedora's official way to distribute community packages.
It's free, trusted, and integrates perfectly with dnf.

📋 Steps to distribute wallflow via COPR:

1. Create COPR account:
   - Go to: https://copr.fedorainfracloud.org/
   - Login with FAS (Fedora Account System)

2. Create new project:
   - Project name: wallflow
   - Description: "Elegant wallpaper management with smooth transitions"
   - Instructions: "A modern Rust-based wallpaper manager"

3. Setup auto-builds from Git:
   - Source Type: "SCM"
   - Clone URL: https://github.com/YOUR_USERNAME/wallflow
   - Subdirectory: "" (root)
   - Spec file: wallflow.spec
   - Targets: fedora-39-x86_64, fedora-40-x86_64

4. Users will install with:
   sudo dnf copr enable YOUR_USERNAME/wallflow
   sudo dnf install wallflow

✅ Benefits:
   - Official Fedora infrastructure
   - Automatic builds on git push
   - Trusted by users (signed packages)
   - Free for open source projects

📦 What you need to provide:
   - wallflow.spec file (RPM build instructions)
   - Git repository with source code
   - Valid Fedora account

EOF

# Check if we have the spec file template
if [ ! -f "wallflow.spec" ]; then
    echo "📄 Creating wallflow.spec template..."

    cat > wallflow.spec << 'SPEC_EOF'
Name:           wallflow
Version:        0.1.0
Release:        1%{?dist}
Summary:        Elegant wallpaper management with smooth transitions

License:        MIT
URL:            https://github.com/YOUR_USERNAME/wallflow
Source0:        %{url}/archive/v%{version}/%{name}-%{version}.tar.gz

BuildRequires:  rust
BuildRequires:  cargo

Requires:       awww
Recommends:     python3-pywal

%description
wallflow brings fluidity to your desktop with beautiful wallpaper
transitions, dynamic color schemes, and seamless desktop integration.
Supports multiple wallpaper sources including local files, Wallhaven,
and Picsum, with automatic color scheme generation via pywal.

%prep
%autosetup

%build
cargo build --release

%install
install -Dm755 target/release/wallflow %{buildroot}%{_bindir}/wallflow
install -Dm644 config.example.yml %{buildroot}%{_docdir}/%{name}/config.example.yml

%files
%license LICENSE
%doc README.md
%{_bindir}/wallflow
%{_docdir}/%{name}/config.example.yml

%changelog
* $(date "+%a %b %d %Y") Your Name <your.email@example.com> - 0.1.0-1
- Initial package
SPEC_EOF

    echo "✅ Created wallflow.spec"
    echo "   Edit the URL and your details before using"
fi

echo ""
echo "🚀 Next steps:"
echo "1. Edit wallflow.spec with your GitHub URL"
echo "2. Create COPR account and project"
echo "3. Push to git and watch auto-builds!"
