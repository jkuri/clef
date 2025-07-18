#!/bin/bash

# PNRS Installation Script
# Installs PNRS as a systemd service

set -e

INSTALL_DIR="/opt/pnrs"
SERVICE_FILE="/etc/systemd/system/pnrs.service"
USER="pnrs"

echo "🚀 Installing PNRS..."
echo "===================="

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo "❌ This script must be run as root (use sudo)"
   exit 1
fi

# Build the project
echo "📦 Building PNRS..."
cargo build --release

# Create user
echo "👤 Creating user '$USER'..."
if ! id "$USER" &>/dev/null; then
    useradd -r -s /bin/false -d "$INSTALL_DIR" "$USER"
    echo "✅ User '$USER' created"
else
    echo "ℹ️  User '$USER' already exists"
fi

# Create installation directory
echo "📁 Creating installation directory..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR/logs"

# Copy binary
echo "📋 Installing binary..."
cp target/release/pnrs "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/pnrs"

# Set ownership
chown -R "$USER:$USER" "$INSTALL_DIR"

# Install systemd service
echo "⚙️  Installing systemd service..."
cp scripts/pnrs.service "$SERVICE_FILE"

# Reload systemd and enable service
systemctl daemon-reload
systemctl enable pnrs

echo ""
echo "✅ Installation complete!"
echo ""
echo "🎯 Next steps:"
echo "   1. Configure environment variables in $SERVICE_FILE"
echo "   2. Start the service: sudo systemctl start pnrs"
echo "   3. Check status: sudo systemctl status pnrs"
echo "   4. View logs: sudo journalctl -u pnrs -f"
echo ""
echo "🌐 Default configuration:"
echo "   - Host: 0.0.0.0 (all interfaces)"
echo "   - Port: 8000"
echo "   - Upstream: https://registry.npmjs.org"
echo ""
echo "📝 To customize configuration, edit: $SERVICE_FILE"
