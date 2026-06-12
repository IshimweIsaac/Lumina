#!/bin/bash
#═══════════════════════════════════════════════════════════════════════════════
#  Lumina Release Script — Single-command version propagation
#
#  Usage:
#    ./scripts/release.sh <NEW_VERSION> "<RELEASE_NAME>"         # Dry run
#    ./scripts/release.sh <NEW_VERSION> "<RELEASE_NAME>" --execute  # Apply
#
#  Examples:
#    ./scripts/release.sh 2.1.4 "Adapter Hardening"
#    ./scripts/release.sh 2.1.4 "Adapter Hardening" --execute
#
#  This script replaces the manual "Friday Release Checklist" in SPRINTS.md.
#  It bumps version strings across all 20+ locations, syncs docs, and
#  optionally runs formatting/tests.
#═══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

# ─── Constants ────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION_FILE="$PROJECT_ROOT/VERSION"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
RESET='\033[0m'

# ─── Helpers ──────────────────────────────────────────────────────────────────
info()    { echo -e "${BLUE}[INFO]${RESET}    $1"; }
success() { echo -e "${GREEN}[SUCCESS]${RESET} $1"; }
warn()    { echo -e "${YELLOW}[WARNING]${RESET} $1"; }
error()   { echo -e "${RED}[ERROR]${RESET}   $1"; exit 1; }
step()    { echo -e "\n${CYAN}━━━ $1 ━━━${RESET}"; }
changed() { echo -e "  ${DIM}↳${RESET} $1"; CHANGED_FILES+=("$1"); }

# Track what we change
declare -a CHANGED_FILES=()

# ─── Parse Arguments ─────────────────────────────────────────────────────────
if [ $# -lt 2 ]; then
    echo -e "${BOLD}Lumina Release Script${RESET}"
    echo ""
    echo "Usage: $0 <NEW_VERSION> \"<RELEASE_NAME>\" [--execute]"
    echo ""
    echo "Arguments:"
    echo "  NEW_VERSION     Semver version (e.g., 2.1.4)"
    echo "  RELEASE_NAME    Human name for the release (e.g., \"Adapter Hardening\")"
    echo "  --execute       Apply changes (default is dry-run)"
    echo ""
    echo "Examples:"
    echo "  $0 2.1.4 \"Adapter Hardening\"           # Preview changes"
    echo "  $0 2.1.4 \"Adapter Hardening\" --execute  # Apply changes"
    exit 1
fi

NEW_VERSION="$1"
RELEASE_NAME="$2"
DRY_RUN=true

if [ "${3:-}" = "--execute" ]; then
    DRY_RUN=false
fi

# ─── Validate ────────────────────────────────────────────────────────────────

# Check we're in the right directory
if [ ! -f "$VERSION_FILE" ]; then
    error "VERSION file not found at $VERSION_FILE. Are you in the Lumina project root?"
fi

# Read current version
CURRENT_VERSION=$(tr -d '[:space:]' < "$VERSION_FILE")
info "Current version: ${BOLD}$CURRENT_VERSION${RESET}"
info "New version:     ${BOLD}$NEW_VERSION${RESET}"
info "Release name:    ${BOLD}$RELEASE_NAME${RESET}"

# Validate semver format (major.minor.patch)
if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    error "Invalid version format '$NEW_VERSION'. Must be semver (e.g., 2.1.4)"
fi

# Validate version is different
if [ "$NEW_VERSION" = "$CURRENT_VERSION" ]; then
    error "New version ($NEW_VERSION) is the same as current version ($CURRENT_VERSION)"
fi

# Simple forward-only check (compare as strings — works for dotted semver in same major)
if [ "$(printf '%s\n' "$CURRENT_VERSION" "$NEW_VERSION" | sort -V | head -n1)" != "$CURRENT_VERSION" ]; then
    error "New version ($NEW_VERSION) is not greater than current version ($CURRENT_VERSION)"
fi

# ─── Pre-flight checks ──────────────────────────────────────────────────────
step "Pre-flight Checks"

# Check for clean git working tree (warn, don't block)
if command -v git &>/dev/null && [ -d "$PROJECT_ROOT/.git" ]; then
    if ! git -C "$PROJECT_ROOT" diff --quiet 2>/dev/null; then
        warn "Git working tree has uncommitted changes. Consider committing first."
    fi
    
    CURRENT_BRANCH=$(git -C "$PROJECT_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
    info "Current branch: $CURRENT_BRANCH"
fi

if $DRY_RUN; then
    echo ""
    echo -e "${YELLOW}${BOLD}══════════════════════════════════════════════${RESET}"
    echo -e "${YELLOW}${BOLD}  DRY RUN — No files will be modified${RESET}"
    echo -e "${YELLOW}${BOLD}  Pass --execute to apply changes${RESET}"
    echo -e "${YELLOW}${BOLD}══════════════════════════════════════════════${RESET}"
fi

# ─── Sed Wrapper (respects dry-run) ──────────────────────────────────────────
# Usage: safe_sed "s/old/new/g" /path/to/file
safe_sed() {
    local pattern="$1"
    local file="$2"
    
    if ! [ -f "$file" ]; then
        warn "File not found, skipping: $file"
        return
    fi
    
    if $DRY_RUN; then
        # Show what would change (but don't apply)
        if grep -qP "$(echo "$pattern" | sed 's|^s/\(.*\)/.*/$|\1|; s|^s/\(.*\)/.*$|\1|')" "$file" 2>/dev/null || true; then
            changed "$file ${DIM}(would update)${RESET}"
        fi
    else
        sed -i "$pattern" "$file"
        changed "$file"
    fi
}

# ─── Step 1: VERSION file ───────────────────────────────────────────────────
step "Step 1/8: Update VERSION file"

if $DRY_RUN; then
    changed "$VERSION_FILE ${DIM}(would update: $CURRENT_VERSION → $NEW_VERSION)${RESET}"
else
    echo "$NEW_VERSION" > "$VERSION_FILE"
    changed "$VERSION_FILE"
fi

# ─── Step 2: Cargo.toml files ───────────────────────────────────────────────
step "Step 2/8: Update Cargo.toml files (all crates)"

CRATE_DIRS=(
    "lumina-lexer"
    "lumina-parser"
    "lumina-analyzer"
    "lumina-diagnostics"
    "lumina-runtime"
    "lumina_ffi"
    "lumina-cli"
    "lumina-wasm"
    "lumina-lsp"
    "lumina-cluster"
)

for crate in "${CRATE_DIRS[@]}"; do
    CARGO_FILE="$PROJECT_ROOT/crates/$crate/Cargo.toml"
    safe_sed "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_FILE"
done

# Also update internal dependency versions in lumina-cluster
CLUSTER_CARGO="$PROJECT_ROOT/crates/lumina-cluster/Cargo.toml"
if [ -f "$CLUSTER_CARGO" ]; then
    safe_sed "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/g" "$CLUSTER_CARGO"
fi

# ─── Step 3: package.json ───────────────────────────────────────────────────
step "Step 3/8: Update package.json"

safe_sed "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" "$PROJECT_ROOT/package.json"

# ─── Step 4: install.sh ─────────────────────────────────────────────────────
step "Step 4/8: Update install.sh"

safe_sed "s/Lumina v$CURRENT_VERSION/Lumina v$NEW_VERSION/g" "$PROJECT_ROOT/install.sh"

# ─── Step 5: Build & Deploy script ──────────────────────────────────────────
step "Step 5/8: Update scripts/build-and-deploy-all.sh"

BUILD_SCRIPT="$PROJECT_ROOT/scripts/build-and-deploy-all.sh"
safe_sed "s/v$CURRENT_VERSION/v$NEW_VERSION/g" "$BUILD_SCRIPT"

# ─── Step 6: Website HTML files ──────────────────────────────────────────────
step "Step 6/8: Update website version badges"

safe_sed "s/v$CURRENT_VERSION/v$NEW_VERSION/g" "$PROJECT_ROOT/website/index.html"
safe_sed "s/v$CURRENT_VERSION/v$NEW_VERSION/g" "$PROJECT_ROOT/website/docs.html"

# ─── Step 7/8: README.md & docs README ──────────────────────────────────────
step "Step 7/8: Update README.md + docs/README.md"

safe_sed "s/Lumina-v$CURRENT_VERSION/Lumina-v$NEW_VERSION/g" "$PROJECT_ROOT/README.md"

# Update the "Current Version" line in docs/README.md
safe_sed "s/v$CURRENT_VERSION/v$NEW_VERSION/g" "$PROJECT_ROOT/docs/README.md"

# ─── Step 8/8: VERSION_MAP.md (both copies) ─────────────────────────────────
step "Step 8/8: Update VERSION_MAP.md"

# Update current active version line
safe_sed "s/v$CURRENT_VERSION-ARCHITECT/v$NEW_VERSION-ARCHITECT/g" "$PROJECT_ROOT/docs/VERSION_MAP.md"
safe_sed "s/v$CURRENT_VERSION-ARCHITECT/v$NEW_VERSION-ARCHITECT/g" "$PROJECT_ROOT/website/public/docs/VERSION_MAP.md"

# Note: crates/lumina-wasm/pkg/package.json is auto-generated by `wasm-pack build`
# from the Cargo.toml version, so it doesn't need manual bumping here.

# Sync docs → website (keep them identical)
if ! $DRY_RUN; then
    # Sync VERSION_MAP
    if [ -f "$PROJECT_ROOT/docs/VERSION_MAP.md" ] && [ -d "$PROJECT_ROOT/website/public/docs" ]; then
        cp "$PROJECT_ROOT/docs/VERSION_MAP.md" "$PROJECT_ROOT/website/public/docs/VERSION_MAP.md"
        changed "website/public/docs/VERSION_MAP.md ${DIM}(synced from docs/)${RESET}"
    fi
    
    # Sync docs README
    if [ -f "$PROJECT_ROOT/docs/README.md" ] && [ -d "$PROJECT_ROOT/website/public/docs" ]; then
        cp "$PROJECT_ROOT/docs/README.md" "$PROJECT_ROOT/website/public/docs/README.md"
        changed "website/public/docs/README.md ${DIM}(synced from docs/)${RESET}"
    fi
    
    # Sync guides
    if [ -d "$PROJECT_ROOT/docs/guides" ] && [ -d "$PROJECT_ROOT/website/public/docs/guides" ]; then
        cp -r "$PROJECT_ROOT/docs/guides/"* "$PROJECT_ROOT/website/public/docs/guides/"
        changed "website/public/docs/guides/ ${DIM}(synced from docs/guides/)${RESET}"
    fi
fi

# ─── Summary ─────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}══════════════════════════════════════════════════════════════${RESET}"
if $DRY_RUN; then
    echo -e "${YELLOW}${BOLD}  DRY RUN COMPLETE${RESET}"
    echo -e "  ${#CHANGED_FILES[@]} files would be modified"
    echo -e "  Run with ${BOLD}--execute${RESET} to apply"
else
    echo -e "${GREEN}${BOLD}  ✅ VERSION BUMP COMPLETE: $CURRENT_VERSION → $NEW_VERSION${RESET}"
    echo -e "  ${#CHANGED_FILES[@]} files updated"
fi
echo -e "${BOLD}══════════════════════════════════════════════════════════════${RESET}"

# ─── Post-bump reminders ────────────────────────────────────────────────────
if ! $DRY_RUN; then
    echo ""
    step "Remaining Manual Steps"
    echo -e "  ${BOLD}1.${RESET} Update ${CYAN}CHANGELOG.md${RESET} with this week's features"
    echo -e "  ${BOLD}2.${RESET} Update ${CYAN}SPRINTS.md${RESET} — check off completed tasks"
    echo -e "  ${BOLD}3.${RESET} Run quality checks:"
    echo -e "     ${DIM}\$ make check${RESET}  ${DIM}(or: cargo fmt && cargo clippy && cargo test --all-features)${RESET}"
    echo -e "  ${BOLD}4.${RESET} Commit and tag:"
    echo -e "     ${DIM}\$ git add -A && git commit -m \"release: v$NEW_VERSION — $RELEASE_NAME\"${RESET}"
    echo -e "     ${DIM}\$ git tag v$NEW_VERSION${RESET}"
    echo -e "     ${DIM}\$ git push origin main --tags${RESET}"
    echo ""
fi

# ─── Optional: Run cargo checks ─────────────────────────────────────────────
if ! $DRY_RUN; then
    echo ""
    read -r -p "$(echo -e "${CYAN}Run cargo fmt + clippy + test now? [y/N]:${RESET} ")" RUN_CHECKS
    if [[ "$RUN_CHECKS" =~ ^[Yy]$ ]]; then
        step "Running cargo fmt"
        (cd "$PROJECT_ROOT" && cargo fmt --all)
        success "Formatting complete"
        
        step "Running cargo clippy"
        (cd "$PROJECT_ROOT" && cargo clippy --all-features -- -D warnings) || warn "Clippy found warnings"
        
        step "Running cargo test"
        (cd "$PROJECT_ROOT" && cargo test --all-features) || warn "Some tests failed"
        
        success "All quality checks completed"
    fi
fi
