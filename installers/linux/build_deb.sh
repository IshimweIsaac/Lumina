#!/bin/bash
set -e

VERSION=${1:-"1.7.0"}
TARGET=${2:-"target/x86_64-unknown-linux-gnu/release"}
DEB_DIR="lumina_${VERSION}_amd64"

echo "Building Debian package for Lumina $VERSION..."

# 1. Create directory structure
mkdir -p "$DEB_DIR/DEBIAN"
mkdir -p "$DEB_DIR/usr/local/bin"

# 2. Create control file
cat <<EOF > "$DEB_DIR/DEBIAN/control"
Package: lumina
Version: $VERSION
Section: devel
Priority: optional
Architecture: amd64
Maintainer: Isaac Ishimwe <ishimwe@example.com>
Description: Lumina Programming Language
 Declarative reactive language for IoT and infrastructure monitoring.
EOF

# 3. Copy binaries and setup scripts
cp "../../$TARGET/lumina-cli" "$DEB_DIR/usr/local/bin/lumina"
cp "../../$TARGET/lumina-lsp" "$DEB_DIR/usr/local/bin/lumina-lsp"
cp "postinst" "$DEB_DIR/DEBIAN/postinst"

chmod 755 "$DEB_DIR/usr/local/bin/lumina"
chmod 755 "$DEB_DIR/usr/local/bin/lumina-lsp"
chmod 755 "$DEB_DIR/DEBIAN/postinst"

# 4. Build package
dpkg-deb --build "$DEB_DIR"
mv "${DEB_DIR}.deb" "../../lumina-setup-linux-amd64.deb"

# 5. Clean up
rm -rf "$DEB_DIR"

echo "Successfully created lumina-setup-linux-amd64.deb"
