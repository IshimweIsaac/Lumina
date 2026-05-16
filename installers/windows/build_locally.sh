#!/bin/bash
# Lumina Windows Installer Local Build Script
# This script is a fallback for when GitHub Actions is unavailable.
# It cross-compiles from Linux to Windows using the GNU toolchain.

set -e

# Colors for output
BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

TARGET="x86_64-pc-windows-gnu"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Lumina Windows Installer Builder (Local Fallback)  ${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# 1. Check for dependencies
echo -e "Step 1: Checking dependencies..."
if ! command -v makensis &> /dev/null; then
    echo -e "${RED}[ERROR] NSIS (makensis) not found. Please install it (e.g., sudo apt install nsis).${NC}"
    exit 1
fi

if ! rustup target list --installed | grep -q "$TARGET"; then
    echo -e "${RED}[ERROR] Rust target $TARGET not installed.${NC}"
    echo -e "Run: rustup target add $TARGET"
    exit 1
fi

if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo -e "${RED}[ERROR] MinGW cross-compiler not found.${NC}"
    echo -e "Run: sudo apt install gcc-mingw-w64-x86-64"
    exit 1
fi

# 2. Build binaries
echo -e "Step 2: Building Windows binaries (release)..."
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
ROOT_DIR="$SCRIPT_DIR/../.."
cd "$ROOT_DIR"

cargo build --release --target "$TARGET" -p lumina-cli
cargo build --release --target "$TARGET" -p lumina-lsp

# 3. Build installer
echo -e "Step 3: Generating installer with NSIS..."
cd "$SCRIPT_DIR"

# Remove old installers
rm -f Lumina-v*-x64-Setup.exe LuminaSetup.exe

makensis -DTARGET="$TARGET" lumina.nsi

# 4. Finalize
INSTALLER="Lumina-v2.1.0-x64-Setup.exe"
if [ -f "$INSTALLER" ]; then
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}[SUCCESS] Windows Installer created: $INSTALLER${NC}"
    echo -e "  Location: $(pwd)/$INSTALLER"
    echo -e "  Size: $(du -h "$INSTALLER" | cut -f1)"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
else
    echo -e "${RED}[ERROR] Failed to generate $INSTALLER${NC}"
    exit 1
fi
