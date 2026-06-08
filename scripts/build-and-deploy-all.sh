#!/bin/bash
# scripts/build-and-deploy-all.sh
set -e

# Sourcing Cargo env
source "$HOME/.cargo/env"

echo "=========================================================="
echo "   Lumina OS Multi-Platform Build & Deploy Tool (v2.1.3)"
echo "=========================================================="

# Define Targets and remote naming
# Matrix structure: TARGET | OS_NAME | EXTENSION
TARGETS=(
  "x86_64-unknown-linux-gnu|linux-x64|"
  "aarch64-unknown-linux-gnu|linux-arm64|"
  "x86_64-pc-windows-gnu|windows-x64|.exe"
)

# Loop and build/deploy each target
for entry in "${TARGETS[@]}"; do
  IFS='|' read -r TARGET OS_NAME EXT <<< "$entry"
  
  echo "----------------------------------------------------------"
  echo "🚀 Building for target: $TARGET ($OS_NAME)"
  echo "----------------------------------------------------------"
  
  # Determine compiler: standard cargo for native linux, cargo zigbuild for cross
  if [ "$TARGET" = "x86_64-unknown-linux-gnu" ]; then
    cargo build --release --target "$TARGET" -p lumina-cli
    cargo build --release --target "$TARGET" -p lumina-lsp
  else
    cargo zigbuild --release --target "$TARGET" -p lumina-cli
    cargo zigbuild --release --target "$TARGET" -p lumina-lsp
  fi
  
  CLI_BIN="target/$TARGET/release/lumina$EXT"
  LSP_BIN="target/$TARGET/release/lumina-lsp$EXT"
  
  CLI_REMOTE="lumina-$OS_NAME$EXT"
  LSP_REMOTE="lumina-$OS_NAME-lsp$EXT"
  
  # Generate checksum files
  sha256sum "$CLI_BIN" | cut -d' ' -f1 > "$CLI_BIN.sha256"
  sha256sum "$LSP_BIN" | cut -d' ' -f1 > "$LSP_BIN.sha256"
  
  echo "⬆️ Deploying CLI to Supabase Storage..."
  ./scripts/upload-binaries.sh "$CLI_BIN" "$CLI_REMOTE"
  ./scripts/upload-binaries.sh "$CLI_BIN.sha256" "$CLI_REMOTE.sha256"
  
  echo "⬆️ Deploying LSP to Supabase Storage..."
  ./scripts/upload-binaries.sh "$LSP_BIN" "$LSP_REMOTE"
  ./scripts/upload-binaries.sh "$LSP_BIN.sha256" "$LSP_REMOTE.sha256"
  
  # Cleanup local checksum files
  rm -f "$CLI_BIN.sha256" "$LSP_BIN.sha256"
  
  echo "✓ Successfully deployed target: $TARGET"
done

# Upload version.txt so the CLI updater can see it
echo "v2.1.3" > version.txt
echo "⬆️ Deploying version.txt to Supabase Storage..."
./scripts/upload-binaries.sh version.txt "version.txt"
rm version.txt

echo "=========================================================="
echo "🎉 All targets successfully compiled & deployed to Supabase!"
echo "=========================================================="
