#!/bin/sh
#
# Lumina Toolchain Installer -- lumina-lang.web.app
# Usage: curl -fsSL https://lumina-lang.web.app/install.sh | sh
#

set -e

# --- Configuration ---
LUMINA_HOME="${LUMINA_HOME:-$HOME/.lumina}"
BIN_DIR="$LUMINA_HOME/bin"
BASE_URL="https://lumina-lang.web.app"
SKIP_CHECKSUM="${LUMINA_SKIP_CHECKSUM:-0}"

# --- Output Formatting ---
BOLD="$(tput bold 2>/dev/null || echo '')"
RESET="$(tput sgr0 2>/dev/null || echo '')"
GREEN="$(tput setaf 2 2>/dev/null || echo '')"
RED="$(tput setaf 1 2>/dev/null || echo '')"
BLUE="$(tput setaf 4 2>/dev/null || echo '')"

log_info() { echo "${BLUE}[INFO]${RESET} $1"; }
log_success() { echo "${GREEN}[SUCCESS]${RESET} ${BOLD}$1${RESET}"; }
log_error() { echo "${RED}[ERROR]${RESET} $1"; exit 1; }
log_warn() { echo "${RED}[WARNING]${RESET} $1"; }

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

# --- Download & Verify ──────────────────────────────────
log_info "Installing Lumina v2.0.0 for $PLATFORM..."
mkdir -p "$BIN_DIR"

download_binary() {
    NAME="$1"
    BIN_NAME="lumina"
    URL_SUFFIX="$PLATFORM"
    
    if [ "$NAME" = "lsp" ]; then
        BIN_NAME="lumina-lsp"
        URL_SUFFIX="$PLATFORM-lsp"
    fi
    
    # Notice the hosted URL structure based on the online script
    URL="$BASE_URL/lumina-$URL_SUFFIX"
    DEST="$BIN_DIR/$BIN_NAME"
    
    log_info "Downloading $BIN_NAME..."
    if ! curl -fsSL "$URL" -o "$DEST"; then
        log_error "Failed to download $BIN_NAME from $URL"
    fi

    # Checksum Verification
    if [ "$SKIP_CHECKSUM" = "1" ]; then
        log_warn "Skipping checksum verification for $BIN_NAME due to LUMINA_SKIP_CHECKSUM=1"
    else
        log_info "Verifying checksum for $BIN_NAME..."
        if curl -fsSL "$URL.sha256" -o "$DEST.sha256"; then
            EXPECTED=$(cat "$DEST.sha256" | cut -d" " -f1)
            
            if command -v sha256sum >/dev/null 2>&1; then
                ACTUAL=$(sha256sum "$DEST" | cut -d" " -f1)
            else
                ACTUAL=$(shasum -a 256 "$DEST" | cut -d" " -f1)
            fi

            if [ "$EXPECTED" != "$ACTUAL" ]; then
                log_error "Checksum verification failed for $DEST\nExpected: $EXPECTED\nActual:   $ACTUAL\n\nIf you believe this is an error, you can rerun the script with LUMINA_SKIP_CHECKSUM=1 curl ..."
                rm -f "$DEST" "$DEST.sha256"
                exit 1
            fi
            rm -f "$DEST.sha256"
            log_info "Checksum verified."
        else
            log_warn "Could not download checksum file for $BIN_NAME. Proceeding anyway..."
        fi
    fi
    
    chmod +x "$DEST"
}

# Install core components
download_binary core
download_binary lsp

# --- Path Injection ─────────────────────────────────────
add_to_path() {
    EXPORT_LINE="export PATH=\"$BIN_DIR:\$PATH\""
    ADDED=false
    for PROFILE in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.profile"; do
        if [ -f "$PROFILE" ]; then
            if ! grep -q "$BIN_DIR" "$PROFILE" 2>/dev/null; then
                log_info "Adding $BIN_DIR to $PROFILE"
                echo "" >> "$PROFILE"
                echo "$EXPORT_LINE" >> "$PROFILE"
                ADDED=true
            fi
        fi
    done
    
    if [ "$ADDED" = "false" ]; then
        log_info "$BIN_DIR is already in your PATH or no profile found."
    else
        log_info "Please run: source $PROFILE (or restart your terminal)"
    fi
    export PATH="$BIN_DIR:$PATH"
}

add_to_path

# --- Verify & Setup ─────────────────────────────────────
log_success "Lumina successfully installed!"

if [ -x "$BIN_DIR/lumina" ]; then
    VERSION=$("$BIN_DIR/lumina" --version 2>/dev/null || echo "v2.0.0")
    echo ""
    log_info "Running: Lumina $VERSION"
    
    # ── Zero-Config Hook ────────────────────────────────────
    log_info "Running automated environment setup..."
    "$BIN_DIR/lumina" setup || true
    echo ""

    log_success "Setup complete! Start coding:"
    echo "  Run:   lumina run your-program.lum"
    echo "  Check: lumina check your-program.lum"
    echo "  Docs:  https://lumina-lang.web.app/docs"
    echo ""
fi
