#═══════════════════════════════════════════════════════════════════════════════
#  Lumina — Project Makefile
#
#  Common commands for development, testing, and releasing.
#
#  Usage:
#    make help              Show all available targets
#    make version           Print the current version
#    make check             Run fmt + clippy + test
#    make release           Bump version (dry-run by default)
#    make release-execute   Bump version and apply changes
#    make sync-docs         Sync docs/ → website/public/docs/
#═══════════════════════════════════════════════════════════════════════════════

.PHONY: help version check release release-execute build test fmt clippy sync-docs clean

# ─── Default ──────────────────────────────────────────────────────────────────
help: ## Show this help message
	@echo ""
	@echo "  Lumina v$$(cat VERSION) — Available Commands"
	@echo "  ─────────────────────────────────────────"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""

# ─── Info ─────────────────────────────────────────────────────────────────────
version: ## Print the current Lumina version
	@echo "v$$(cat VERSION)"

# ─── Quality ──────────────────────────────────────────────────────────────────
fmt: ## Run cargo fmt on the whole workspace
	cargo fmt --all

clippy: ## Run cargo clippy with all features
	cargo clippy --all-features -- -D warnings

test: ## Run all tests with all features
	cargo test --all-features

check: fmt clippy test ## Run fmt + clippy + test (full quality suite)
	@echo ""
	@echo "  ✅ All quality checks passed"

# ─── Build ────────────────────────────────────────────────────────────────────
build: ## Build all crates in release mode
	cargo build --release

# ─── Release ──────────────────────────────────────────────────────────────────
# Usage: make release V=2.1.4 NAME="Adapter Hardening"
release: ## Dry-run a version bump (preview only)
ifndef V
	@echo "Usage: make release V=<version> NAME=\"<release name>\""
	@echo "Example: make release V=2.1.4 NAME=\"Adapter Hardening\""
	@exit 1
endif
	./scripts/release.sh $(V) "$(NAME)"

release-execute: ## Apply a version bump (modifies files)
ifndef V
	@echo "Usage: make release-execute V=<version> NAME=\"<release name>\""
	@echo "Example: make release-execute V=2.1.4 NAME=\"Adapter Hardening\""
	@exit 1
endif
	./scripts/release.sh $(V) "$(NAME)" --execute

# ─── Docs ─────────────────────────────────────────────────────────────────────
sync-docs: ## Sync docs/ → website/public/docs/ (one-way copy)
	@echo "Syncing docs to website..."
	@cp docs/VERSION_MAP.md website/public/docs/VERSION_MAP.md 2>/dev/null || true
	@cp docs/README.md website/public/docs/README.md 2>/dev/null || true
	@cp -r docs/guides/* website/public/docs/guides/ 2>/dev/null || true
	@echo "  ✅ Docs synced: docs/ → website/public/docs/"

# ─── Cleanup ──────────────────────────────────────────────────────────────────
clean: ## Remove build artifacts
	cargo clean
	@echo "  ✅ Build artifacts cleaned"
