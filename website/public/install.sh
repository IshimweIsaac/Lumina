#!/bin/sh
# Lumina installer -- lumina-lang.dev
# Usage: curl -fsSL https://lumina-lang.dev/install.sh | sh

set -e

LUMINA_HOME="${LUMINA_HOME:-$HOME/.lumina}"
BIN_DIR="$LUMINA_HOME/bin"
RELEASES="https://lumina-lang.web.app"

# ── Detect platform ─────────────────────────────────────
detect_platform() {
    OS=$(uname -s)
    ARCH=$(uname -m)
    case "$OS" in
        Linux)
            case "$ARCH" in
                x86_64) echo "linux-x64" ;;
                aarch64) echo "linux-arm64" ;;
                *) echo ""; return 1 ;;
            esac ;;
        Darwin)
            case "$ARCH" in
                arm64) echo "macos-arm64" ;;
                x86_64) echo "macos-x64" ;;
                *) echo ""; return 1 ;;
            esac ;;
        *) echo ""; return 1 ;;
    esac
}

PLATFORM=$(detect_platform)
if [ -z "$PLATFORM" ]; then
    echo "Error: unsupported platform. Download manually from:"
    echo "https://github.com/IshimweIsaac/Lumina/releases"
    exit 1
fi

# ── Download ────────────────────────────────────────────
echo "Installing Lumina for $PLATFORM..."
mkdir -p "$BIN_DIR"

download_binary() {
    NAME="$1"
    if [ "$NAME" = "lsp" ]; then
        URL="$RELEASES/lumina-$PLATFORM-lsp"
        DEST="$BIN_DIR/lumina-lsp"
    else
        URL="$RELEASES/lumina-$PLATFORM"
        DEST="$BIN_DIR/lumina"
    fi
    
    echo "  Downloading $NAME..."
    curl -fsSL "$URL" -o "$DEST"
    curl -fsSL "$URL.sha256" -o "$DEST.sha256"

    # Verify checksum
    EXPECTED=$(cat "$DEST.sha256" | cut -d" " -f1)
    if command -v sha256sum >/dev/null 2>&1; then
        ACTUAL=$(sha256sum "$DEST" | cut -d" " -f1)
    else
        ACTUAL=$(shasum -a 256 "$DEST" | cut -d" " -f1)
    fi

    if [ "$EXPECTED" != "$ACTUAL" ]; then
        echo "Error: checksum verification failed for $DEST"
        rm -f "$DEST" "$DEST.sha256"
        exit 1
    fi
    rm -f "$DEST.sha256"
    chmod +x "$DEST"
}

download_binary lumina
download_binary lsp

# ── Add to PATH ─────────────────────────────────────────
add_to_path() {
    EXPORT_LINE="export PATH=\"$BIN_DIR:\$PATH\""
    ADDED=false
    for PROFILE in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.profile"; do
        if [ -f "$PROFILE" ]; then
            if ! grep -q "$BIN_DIR" "$PROFILE" 2>/dev/null; then
                echo "  Adding $BIN_DIR to $PROFILE"
                echo "" >> "$PROFILE"
                echo "$EXPORT_LINE" >> "$PROFILE"
                ADDED=true
            fi
        fi
    done
    
    if [ "$ADDED" = false ]; then
        echo "  $BIN_DIR already in PATH or no profile found."
    fi
    
    export PATH="$BIN_DIR:$PATH"
}

add_to_path

# ── Verify ──────────────────────────────────────────────
if [ -x "$BIN_DIR/lumina" ]; then
    VERSION=$("$BIN_DIR/lumina" --version 2>/dev/null || echo "v1.8.0")
    echo ""
    echo "Lumina $VERSION installed successfully."
    echo ""
    
    # ── Zero-Config Hook ────────────────────────────────────
    echo "Running automated environment setup..."
    "$BIN_DIR/lumina" setup || true
    echo ""

    echo " Run: lumina run your-program.lum"
    echo " Check: lumina check your-program.lum"
    echo " Docs: https://lumina-lang.web.app/docs"
    echo ""
    echo "Restart your terminal or run: source ~/.zshrc"
else
    echo "Installation complete. Please add $BIN_DIR to your PATH."
fi
