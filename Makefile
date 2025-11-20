# wallflow - Elegant wallpaper management
# Makefile for installation and service management

.PHONY: help install uninstall install-user install-system enable disable start stop restart status logs clean

# Default target
help:
	@echo "🌊 wallflow - Elegant wallpaper management"
	@echo ""
	@echo "Available targets:"
	@echo "  install       Install wallflow for current user (default)"
	@echo "  install-user  Install wallflow for current user"
	@echo "  install-system Install wallflow system-wide (requires sudo)"
	@echo "  uninstall     Remove wallflow installation"
	@echo "  enable        Enable wallflow systemd timer"
	@echo "  disable       Disable wallflow systemd timer"
	@echo "  start         Start wallflow timer"
	@echo "  stop          Stop wallflow timer"
	@echo "  restart       Restart wallflow timer"
	@echo "  status        Show systemd service status"
	@echo "  logs          Show service logs"
	@echo "  clean         Clean up build artifacts"
	@echo "  test          Test wallflow installation"
	@echo ""
	@echo "Examples:"
	@echo "  make install              # Install for current user"
	@echo "  make enable               # Enable automatic wallpaper rotation"
	@echo "  make status               # Check if service is running"

# Configuration
PREFIX ?= $(HOME)/.local
SYSTEMD_USER_DIR ?= $(HOME)/.config/systemd/user
CONFIG_DIR ?= $(HOME)/.config/wallflow

# File locations
BIN_DIR = $(PREFIX)/bin
SCRIPTS = bin/wallflow bin/wallflow-config
SYSTEMD_TEMPLATES = systemd/wallflow.service.template systemd/wallflow.timer.template
CONFIG_TEMPLATE = config/wallflow.yml

# Default installation (user)
install: install-user

# User installation
install-user:
	@echo "📦 Installing wallflow for current user..."
	@mkdir -p $(BIN_DIR)
	@mkdir -p $(CONFIG_DIR)
	@mkdir -p $(SYSTEMD_USER_DIR)

	@echo "📄 Installing scripts..."
	@cp -f $(SCRIPTS) $(BIN_DIR)/
	@chmod +x $(BIN_DIR)/wallflow $(BIN_DIR)/wallflow-config

	@echo "⚙️ Installing configuration template..."
	@if [ ! -f $(CONFIG_DIR)/config.yml ]; then \
		cp $(CONFIG_TEMPLATE) $(CONFIG_DIR)/config.yml; \
		echo "✅ Created default configuration"; \
	else \
		echo "ℹ️ Configuration file already exists, skipping"; \
	fi

	@echo "🔧 Generating systemd services..."
	@./scripts/generate-systemd-services.sh "$(SYSTEMD_USER_DIR)"

	@echo "🔄 Reloading systemd user daemon..."
	@systemctl --user daemon-reload

	@echo "✅ wallflow installed successfully!"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Run 'wallflow-config test' to test installation"
	@echo "  2. Run 'make enable' to enable automatic wallpaper rotation"
	@echo "  3. Run 'wallflow local' to set a wallpaper manually"

# System installation (requires sudo)
install-system:
	@echo "📦 Installing wallflow system-wide..."
	@if [ "$(USER)" = "root" ]; then \
		echo "⚠️ Running as root - installing to /usr/local"; \
		PREFIX=/usr/local make install-system-files; \
	else \
		echo "🔐 Requesting sudo access for system installation..."; \
		sudo $(MAKE) install-system-files; \
	fi

install-system-files:
	@mkdir -p /usr/local/bin
	@mkdir -p /etc/wallflow
	@cp -f $(SCRIPTS) /usr/local/bin/
	@chmod +x /usr/local/bin/wallflow /usr/local/bin/wallflow-config
	@cp $(CONFIG_TEMPLATE) /etc/wallflow/wallflow.yml
	@echo "✅ wallflow installed system-wide to /usr/local/bin"
	@echo "⚙️ Default configuration available at /etc/wallflow/wallflow.yml"

# Uninstall
uninstall:
	@echo "🗑️ Removing wallflow..."
	@$(MAKE) stop 2>/dev/null || true
	@$(MAKE) disable 2>/dev/null || true
	@rm -f $(BIN_DIR)/wallflow $(BIN_DIR)/wallflow-config
	@rm -f $(SYSTEMD_USER_DIR)/wallflow.service $(SYSTEMD_USER_DIR)/wallflow.timer
	@systemctl --user daemon-reload 2>/dev/null || true
	@echo "✅ wallflow uninstalled"
	@echo "ℹ️ Configuration preserved at $(CONFIG_DIR)"

# Service management
enable:
	@echo "🚀 Enabling wallflow timer..."
	@systemctl --user daemon-reload
	@systemctl --user enable wallflow.timer
	@systemctl --user start wallflow.timer
	@echo "✅ wallflow timer enabled and started"
	@$(MAKE) status

disable:
	@echo "⏹️ Disabling wallflow timer..."
	@systemctl --user stop wallflow.timer 2>/dev/null || true
	@systemctl --user disable wallflow.timer 2>/dev/null || true
	@echo "✅ wallflow timer disabled"

start:
	@echo "▶️ Starting wallflow timer..."
	@systemctl --user start wallflow.timer
	@$(MAKE) status

stop:
	@echo "⏹️ Stopping wallflow timer..."
	@systemctl --user stop wallflow.timer
	@systemctl --user stop wallflow.service 2>/dev/null || true
	@echo "✅ wallflow timer stopped"

restart:
	@echo "🔄 Restarting wallflow timer..."
	@systemctl --user restart wallflow.timer
	@$(MAKE) status

status:
	@echo "📊 wallflow service status:"
	@systemctl --user status wallflow.timer --no-pager -l 2>/dev/null || echo "❌ wallflow timer not found or inactive"
	@echo ""
	@echo "📋 Next wallpaper change:"
	@systemctl --user list-timers wallflow.timer --no-pager 2>/dev/null || echo "❌ Timer not scheduled"

logs:
	@echo "📜 wallflow service logs:"
	@journalctl --user -u wallflow.service -f

# Testing
test:
	@echo "🧪 Testing wallflow installation..."
	@if command -v wallflow >/dev/null 2>&1; then \
		echo "✅ wallflow command found"; \
		wallflow config; \
	else \
		echo "❌ wallflow command not found in PATH"; \
		exit 1; \
	fi
	@if command -v wallflow-config >/dev/null 2>&1; then \
		echo "✅ wallflow-config command found"; \
		wallflow-config validate; \
	else \
		echo "❌ wallflow-config command not found in PATH"; \
		exit 1; \
	fi

# Cleanup
clean:
	@echo "🧹 Cleaning up..."
	@rm -f *.log *.tmp
	@echo "✅ Cleanup complete"

# Development targets
dev-install: install
	@echo "🔧 Development installation complete"
	@echo "Files installed to $(PREFIX)"

dev-test: test
	@echo "🧪 Running development tests..."
	@wallflow-config test || echo "⚠️ Some tests failed"