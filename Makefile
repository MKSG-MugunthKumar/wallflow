# Wallflow Makefile

CARGO := cargo
SYSTEMD_USER_DIR := $(HOME)/.config/systemd/user
AUTOSTART_DIR := $(HOME)/.config/autostart

.PHONY: build release install install-service uninstall-service enable-service disable-service \
        install-autostart uninstall-autostart

# Build targets
build:
	$(CARGO) build

release:
	$(CARGO) build --release

# Install binary to ~/.cargo/bin (via cargo install)
install: release
	$(CARGO) install --path .

# Linux systemd service targets
install-service: install $(SYSTEMD_USER_DIR)/wallflow.service
	@echo "Service installed. Run 'make enable-service' to enable it."

$(SYSTEMD_USER_DIR)/wallflow.service: systemd/wallflow.service
	@mkdir -p $(SYSTEMD_USER_DIR)
	@install -m 644 $< $@
	@systemctl --user daemon-reload
	@echo "Installed wallflow.service to $(SYSTEMD_USER_DIR)"

uninstall-service:
	@systemctl --user stop wallflow.service 2>/dev/null || true
	@systemctl --user disable wallflow.service 2>/dev/null || true
	@rm -f $(SYSTEMD_USER_DIR)/wallflow.service
	@systemctl --user daemon-reload
	@echo "Uninstalled wallflow.service"

enable-service:
	@systemctl --user import-environment WAYLAND_DISPLAY XDG_SESSION_TYPE DISPLAY XDG_CURRENT_DESKTOP
	@systemctl --user enable --now wallflow.service
	@echo "Wallflow service enabled and started"

disable-service:
	@systemctl --user disable --now wallflow.service
	@echo "Wallflow service disabled and stopped"

status:
	@systemctl --user status wallflow.service

logs:
	@journalctl --user -u wallflow.service -f

# XDG autostart targets (alternative to systemd, simpler environment handling)
install-autostart: install $(AUTOSTART_DIR)/wallflow.desktop
	@echo "Autostart installed. Wallflow will start on next login."

$(AUTOSTART_DIR)/wallflow.desktop: systemd/wallflow.desktop
	@mkdir -p $(AUTOSTART_DIR)
	@install -m 644 $< $@
	@echo "Installed wallflow.desktop to $(AUTOSTART_DIR)"

uninstall-autostart:
	@rm -f $(AUTOSTART_DIR)/wallflow.desktop
	@echo "Uninstalled wallflow.desktop"
