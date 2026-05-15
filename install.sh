#!/bin/sh
#
# Lumina Toolchain Installer -- lumina-lang.web.app
# Usage: curl -fsSL https://lumina-lang.web.app/install.sh | sh && . ~/.lumina/env
#

set -e

# --- Configuration ---
LUMINA_HOME="${LUMINA_HOME:-$HOME/.lumina}"
BIN_DIR="$LUMINA_HOME/bin"
BASE_URL="https://woijupkxzzakmkneyxwk.supabase.co/storage/v1/object/public/Lumina"
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
# --- Download & Verify ──────────────────────────────────
log_info "Installing Lumina v2.0.0 (The Cluster Release) for $PLATFORM..."
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
    
    # macOS: remove quarantine flag if present (no developer account needed)
    if [ "$(uname -s)" = "Darwin" ]; then
        xattr -d com.apple.quarantine "$DEST" 2>/dev/null || true
    fi
}

# Install core components
download_binary core
download_binary lsp

# --- Environment File & Path Injection ──────────────────
ENV_FILE="$LUMINA_HOME/env"

# Create the env file (like rustup's ~/.cargo/env)
cat > "$ENV_FILE" << 'ENVEOF'
# Lumina environment
# This file is sourced by your shell profile and by the install command.
export PATH="$HOME/.lumina/bin:$PATH"
ENVEOF

log_info "Created $ENV_FILE"

# Add sourcing of env file to shell profiles
add_to_path() {
    SOURCE_LINE=". \"$ENV_FILE\""
    ADDED=false
    for PROFILE in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.profile"; do
        if [ -f "$PROFILE" ]; then
            if ! grep -q ".lumina/env" "$PROFILE" 2>/dev/null; then
                log_info "Adding Lumina to $PROFILE"
                echo "" >> "$PROFILE"
                echo "# Lumina" >> "$PROFILE"
                echo "$SOURCE_LINE" >> "$PROFILE"
                ADDED=true
            fi
        fi
    done
    
    if [ "$ADDED" = "false" ]; then
        log_info "Lumina is already in your shell profiles."
    fi
    # Make PATH available for the rest of this script
    export PATH="$BIN_DIR:$PATH"
}

add_to_path

# --- Verify & Setup ─────────────────────────────────────
log_success "Lumina successfully installed!"

if [ -x "$BIN_DIR/lumina" ]; then
    # ── Zero-Config Hook ────────────────────────────────────
    "$BIN_DIR/lumina" setup || true
    echo ""

    log_success "Setup complete! Start coding:"
    echo "  Run:   lumina run your-program.lum"
    echo "  Check: lumina check your-program.lum"
    echo "  Docs:  https://lumina-lang.web.app/docs"
    echo ""
    echo "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    echo "${BOLD}  Run this to activate Lumina in your current terminal:${RESET}"
    echo ""
    echo "    ${GREEN}. ~/.lumina/env${RESET}"
    echo ""
    echo "  Or if you installed with the full command, it's already active:"
    echo "    curl -fsSL https://lumina-lang.web.app/install.sh | sh ${GREEN}&& . ~/.lumina/env${RESET}"
    echo "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
fi

