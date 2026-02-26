Name:           wallflow
Version:        0.5.2
Release:        1%{?dist}
Summary:        Elegant wallpaper management with smooth transitions

License:        MIT
URL:            https://github.com/MKSG-MugunthKumar/wallflow
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.75
BuildRequires:  cargo
BuildRequires:  gcc
BuildRequires:  openssl-devel
BuildRequires:  perl-FindBin
BuildRequires:  systemd-rpm-macros

# Optional runtime dependencies
Recommends:     python3-pywal

%description
wallflow brings fluidity to your desktop with beautiful wallpaper
transitions, dynamic color schemes, and seamless desktop integration.

Features:
- Multiple wallpaper sources (local, Wallhaven, Picsum, Bing, Reddit, etc.)
- Smooth animated transitions via awww
- Built-in daemon for automatic rotation
- Optional pywal integration for dynamic color schemes
- Self-update capability

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release

%install
install -Dm755 target/release/wallflow %{buildroot}%{_bindir}/wallflow
install -Dm644 systemd/wallflow.service %{buildroot}%{_userunitdir}/wallflow.service
install -Dm644 config.example.yml %{buildroot}%{_docdir}/%{name}/config.example.yml
install -Dm644 README.md %{buildroot}%{_docdir}/%{name}/README.md

%post
%systemd_user_post wallflow.service

%preun
%systemd_user_preun wallflow.service

%files
%license LICENSE
%{_bindir}/wallflow
%{_userunitdir}/wallflow.service
%{_docdir}/%{name}/

%changelog
* Wed Feb 26 2026 Mugunth Kumar <mk@mk.sg> - 0.5.2-1
- Fix systemd service missing environment variables for GNOME
- Add DBUS_SESSION_BUS_ADDRESS to PassEnvironment for gsettings
- Generated service now uses graphical-session.target

* Wed Dec 04 2024 Mugunth Kumar <mk@mk.sg> - 0.1.0-1
- Initial package release
- Self-update command
- Multiple wallpaper sources
