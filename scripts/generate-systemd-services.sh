#!/usr/bin/env bash
# Generate systemd user services from wallflow configuration templates

set -euo pipefail

# Get target directory from argument or default
SYSTEMD_USER_DIR="${1:-${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user}"
CONFIG_FILE="${XDG_CONFIG_HOME:-$HOME/.config}/wallflow/config.yml"

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WALLFLOW_DIR="$(dirname "$SCRIPT_DIR")"

# Template files
SERVICE_TEMPLATE="$WALLFLOW_DIR/systemd/wallflow.service.template"
TIMER_TEMPLATE="$WALLFLOW_DIR/systemd/wallflow.timer.template"

# Output files
SERVICE_FILE="$SYSTEMD_USER_DIR/wallflow.service"
TIMER_FILE="$SYSTEMD_USER_DIR/wallflow.timer"

echo "🔧 Generating wallflow systemd services..."
echo "Config file: $CONFIG_FILE"
echo "Output directory: $SYSTEMD_USER_DIR"

# Function to extract config value with fallback
get_config_value() {
    local key="$1"
    local default="$2"
    local value

    if [[ -f "$CONFIG_FILE" ]]; then
        # Try to extract value from YAML
        value=$(grep -A5 "$key" "$CONFIG_FILE" | head -1 | sed 's/.*:[[:space:]]*//' | tr -d '"' || echo "$default")
        # Clean up the value
        value=$(echo "$value" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
        # Use default if empty
        if [[ -z "$value" || "$value" =~ ^[[:space:]]*$ ]]; then
            value="$default"
        fi
    else
        value="$default"
    fi

    echo "$value"
}

# Extract configuration values
TIMER_INTERVAL=$(get_config_value "interval:" "30")
TIMER_RANDOMIZE=$(get_config_value "randomize:" "5m")
TIMER_START_DELAY=$(get_config_value "start_delay:" "1m")
SOURCES_DEFAULT=$(get_config_value "default:" "wallhaven")
SOURCES_CATEGORY=$(get_config_value "category:" "nature")

echo "Configuration values:"
echo "  Timer interval: $TIMER_INTERVAL minutes"
echo "  Timer randomization: $TIMER_RANDOMIZE"
echo "  Start delay: $TIMER_START_DELAY"
echo "  Default source: $SOURCES_DEFAULT"
echo "  Default category: $SOURCES_CATEGORY"

# Create output directory
mkdir -p "$SYSTEMD_USER_DIR"

# Generate service file
if [[ -f "$SERVICE_TEMPLATE" ]]; then
    sed "s/{{SOURCES_DEFAULT}}/$SOURCES_DEFAULT/g; s/{{SOURCES_CATEGORY}}/$SOURCES_CATEGORY/g" \
        "$SERVICE_TEMPLATE" > "$SERVICE_FILE"
    echo "✅ Generated: $SERVICE_FILE"
else
    echo "⚠️ Service template not found: $SERVICE_TEMPLATE"
    echo "Creating minimal service file..."
    cat > "$SERVICE_FILE" << EOF
[Unit]
Description=wallflow - Set wallpaper with configurable sources and transitions
After=graphical-session.target

[Service]
Type=oneshot
Environment="DISPLAY=:0"
Environment="WAYLAND_DISPLAY=wayland-0"
Environment="XDG_RUNTIME_DIR=%i"
Environment="PATH=%h/.local/bin:/usr/local/bin:/usr/bin:/bin"
ExecStart=%h/.local/bin/wallflow $SOURCES_DEFAULT $SOURCES_CATEGORY

[Install]
WantedBy=default.target
EOF
fi

# Generate timer file
if [[ -f "$TIMER_TEMPLATE" ]]; then
    sed "s/{{TIMER_INTERVAL}}/$TIMER_INTERVAL/g; s/{{TIMER_RANDOMIZE}}/$TIMER_RANDOMIZE/g; s/{{TIMER_START_DELAY}}/$TIMER_START_DELAY/g" \
        "$TIMER_TEMPLATE" > "$TIMER_FILE"
    echo "✅ Generated: $TIMER_FILE"
else
    echo "⚠️ Timer template not found: $TIMER_TEMPLATE"
    echo "Creating minimal timer file..."
    cat > "$TIMER_FILE" << EOF
[Unit]
Description=wallflow timer - Automatic wallpaper rotation (every $TIMER_INTERVAL minutes)
Requires=wallflow.service

[Timer]
OnCalendar=*:0/$TIMER_INTERVAL
RandomizedDelaySec=$TIMER_RANDOMIZE
OnBootSec=$TIMER_START_DELAY
Persistent=true
Unit=wallflow.service

[Install]
WantedBy=timers.target
EOF
fi

echo ""
echo "🔄 Reloading systemd user daemon..."
if systemctl --user daemon-reload 2>/dev/null; then
    echo "✅ Systemd daemon reloaded successfully"
else
    echo "⚠️ Failed to reload systemd daemon - you may need to run 'systemctl --user daemon-reload' manually"
fi

echo ""
echo "🎯 Next steps:"
echo "  systemctl --user enable --now wallflow.timer"
echo ""
echo "💡 Tips:"
echo "  Check status: systemctl --user status wallflow.timer"
echo "  View logs: journalctl --user -u wallflow.service -f"
echo "  Regenerate after config changes: $0"