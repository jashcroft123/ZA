#!/bin/bash

# Configuration
PROJECT_DIR=$(pwd)
BIN_NAME="zmq_gui"
ICON_NAME="zmq_icon.png"
LOCAL_BIN="$HOME/.local/bin"
LOCAL_APPS="$HOME/.local/share/applications"
LOCAL_ICONS="$HOME/.local/share/icons"

echo "🚀 Installing ZeroMQ Testing App..."

# 1. Ensure binary is built
if [ ! -f "$BIN_NAME" ]; then
    echo "⚠️ Binary $BIN_NAME not found. Building with make..."
    make
fi

# 2. Create directories
mkdir -p "$LOCAL_BIN"
mkdir -p "$LOCAL_APPS"
mkdir -p "$LOCAL_ICONS"

# 3. Copy binary
echo "📁 Copying binary to $LOCAL_BIN..."
cp "$BIN_NAME" "$LOCAL_BIN/"
chmod +x "$LOCAL_BIN/$BIN_NAME"

# 4. Copy icon
echo "🖼️  Copying icon to $LOCAL_ICONS..."
cp "$ICON_NAME" "$LOCAL_ICONS/zmq-test.png"

# 5. Copy desktop entries
echo "📎 Installing desktop shortcuts..."
cp zmq_server.desktop "$LOCAL_APPS/"
cp zmq_client.desktop "$LOCAL_APPS/"

# Update desktop database
update-desktop-database "$LOCAL_APPS" 2>/dev/null

echo "✅ Done! You can now find 'ZeroMQ Publisher' and 'ZeroMQ Subscriber' in your application menu."
