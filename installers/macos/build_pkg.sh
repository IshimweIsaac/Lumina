#!/bin/bash
set -e

VERSION=${1:-"1.7.0"}
TARGET=${2:-"target/aarch64-apple-darwin/release"}
PKG_NAME="lumina-setup-macos.pkg"
PAYLOAD_DIR="payload"

echo "Building macOS PKG for Lumina $VERSION..."

# 1. Create payload structure
mkdir -p "$PAYLOAD_DIR/usr/local/bin"

# 2. Copy binaries
cp "../../$TARGET/lumina" "$PAYLOAD_DIR/usr/local/bin/"
cp "../../$TARGET/lumina-lsp" "$PAYLOAD_DIR/usr/local/bin/"
chmod 755 "$PAYLOAD_DIR/usr/local/bin/lumina"
chmod 755 "$PAYLOAD_DIR/usr/local/bin/lumina-lsp"

# 3. Build component package
pkgbuild \
  --root "$PAYLOAD_DIR" \
  --identifier "dev.lumina-lang.lumina" \
  --version "$VERSION" \
  --install-location "/" \
  "../../$PKG_NAME"

# 4. Clean up
rm -rf "$PAYLOAD_DIR"

echo "Successfully created $PKG_NAME"
