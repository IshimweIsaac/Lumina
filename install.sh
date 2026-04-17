#!/bin/sh
#
# Lumina Toolchain Installer -- lumina-lang.web.app
# Usage: curl -fsSL https://lumina-lang.web.app/install.sh | sh
#

set -e

# --- Configuration ---
LUMINA_HOME="${LUMINA_HOME:-$HOME/.lumina}"
BIN_DIR="$LUMINA_HOME/bin"
BASE_URL="https://lumina-lang.web.app/releases"

# --- Output Formatting ---
BOLD="$(tput bold 2>/dev/null || echo '')"
RESET="$(tput sgr0 2>/dev/null || echo '')"
GREEN="$(tput setaf 2 2>/dev/null || echo '')"
RED="$(tput setaf 1 2>/dev/null || echo '')"
BLUE="$(tput setaf 4 2>/dev/null || echo '')"

log_info() { echo -e "${BLUE}[INFO]${RESET} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${RESET} ${BOLD}$1${RESET}"; }
log_error() { echo -e "${RED}[ERROR]${RESET} $1"; exit 1; }

# --- Platform Detection ---
detect_platform() {
    OS=$(uname -s)
    ARCH=$(uname -m)
    case "$OS" in
        Linux)
            case "$ARCH" in
                x86_64) echo "linux-x64" ;;
                aarch64) echo "linux-arm64" ;;
                *) return 1 ;;
            esac ;;
        Darwin)
            case "$ARCH" in
                arm64) echo "macos-arm64" ;;
                x86_64) echo "macos-x64" ;;
                *) return 1 ;;
            esac ;;
        *) return 1 ;;
    esac
}

PLATFORM=$(detect_platform)
if [ -z "$PLATFORM" ]; then
    log_error "Unsupported platform ($OS $ARCH). Please download manually from: https://lumina-lang.dev/docs/install"
fi

# --- Installation ---
log_info "Installing Lumina v1.8.0 for $PLATFORM..."
mkdir -p "$BIN_DIR"

download_binary() {
    NAME="$1"
    BIN_NAME="lumina"
    [ "$NAME" = "lsp" ] && BIN_NAME="lumina-lsp"
    
    URL="$BASE_URL/$BIN_NAME-$PLATFORM"
    DEST="$BIN_DIR/$BIN_NAME"
    
    log_info "Downloading $BIN_NAME..."
    curl -fsSL "$URL" -o "$DEST"
    chmod +x "$DEST"
}

# Install core components
download_binary core
download_binary lsp

# --- Path Injection ---
case $SHELL in
    */zsh) PROFILE="$HOME/.zshrc" ;;
    */bash) PROFILE="$HOME/.bashrc" ;;
    *) PROFILE="$HOME/.profile" ;;
esac

if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$PROFILE"
    log_info "Added $BIN_DIR to your PATH in $PROFILE."
    log_info "Please run: source $PROFILE"
else
    log_info "Lumina is already in your PATH."
fi

log_success "Lumina v1.8.0 successfully installed!"
log_success "Try running: lumina --version"
