#!/usr/bin/env bash
set -e

# Dynamically detect project root regardless of folder nesting or spaces
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ICON_SRC="$PROJECT_DIR/icon.png"
APP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"

echo "=========================================================="
echo "    QuTY Quantum Hardware Console - Fedora 42 Installer   "
echo "=========================================================="

echo "=== 1. Compiling QuTY Release Binary ==="
cd "$PROJECT_DIR"
cargo build --release

# Locate the compiled binary (checks QuTY, quty, or first generated executable)
if [ -f "$PROJECT_DIR/target/release/QuTY" ]; then
    BINARY_PATH="$PROJECT_DIR/target/release/QuTY"
elif [ -f "$PROJECT_DIR/target/release/quty" ]; then
    BINARY_PATH="$PROJECT_DIR/target/release/quty"
elif [ -f "$PROJECT_DIR/target/release/QTY" ]; then
    BINARY_PATH="$PROJECT_DIR/target/release/QTY"
else
    # Fallback to the primary release executable
    BINARY_PATH=$(find "$PROJECT_DIR/target/release" -maxdepth 1 -type f -executable ! -name "*.so" ! -name "*.d" | head -n 1)
fi

chmod +x "$BINARY_PATH"
echo "✓ Release binary ready at: $BINARY_PATH"

echo "=== 2. Installing High-DPI Application Icon ==="
mkdir -p "$APP_DIR"
mkdir -p "$ICON_DIR"

if [ -f "$ICON_SRC" ]; then
    cp "$ICON_SRC" "$ICON_DIR/quty.png"
    ICON_PARAM="$ICON_DIR/quty.png"
    echo "✓ Custom QuTY icon installed to $ICON_PARAM"
else
    echo "⚠️ Warning: icon.png not found at $ICON_SRC, falling back to system icon."
    ICON_PARAM="atom"
fi

echo "=== 3. Registering FreeDesktop Launcher ==="
cat <<EOF > "$APP_DIR/quty.desktop"
[Desktop Entry]
Type=Application
Name=QuTY
GenericName=Quantum Hardware Console
Comment=Multi-Platform Simulator: Superconducting Transmons, Trapped Ions, Silicon Emitters & QEC
Exec="$BINARY_PATH"
Icon=$ICON_PARAM
Terminal=false
Categories=Science;Education;Development;Physics;
Keywords=Quantum;Transmon;Trapped Ion;IonQ;QEC;Silicon;Emitters;Physics;Simulation;
StartupWMClass=QuTY
EOF

chmod +x "$APP_DIR/quty.desktop"
echo "✓ Desktop launcher written to $APP_DIR/quty.desktop"

echo "=== 4. Refreshing Fedora 42 Desktop & Icon Databases ==="
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR"
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

echo ""
echo "=== Success! ==="
echo "QuTY is updated and registered as a standalone desktop application."
echo "You can launch it anytime by pressing the 'Super' key and searching 'QuTY'."

# Optional: Auto-launch if '--run' or '-r' flag is passed
if [[ "$1" == "--run" || "$1" == "-r" ]]; then
    echo ""
    echo "=== Launching QuTY ==="
    exec "$BINARY_PATH"
fi
