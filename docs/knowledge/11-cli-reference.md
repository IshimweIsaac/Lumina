# Lumina CLI Reference

Every command available in the `lumina` binary, with usage, flags, and examples.

---

## Installation

### Linux / macOS

```bash
curl -fsSL https://lumina-lang.web.app/install.sh | sh && . ~/.lumina/env
```

### Windows (PowerShell)

```powershell
iwr https://lumina-lang.web.app/install.ps1 -useb | iex
```

### Homebrew (macOS)

```bash
brew tap IshimweIsaac/lumina
brew install lumina
```

### Verify Installation

```bash
lumina --version
```

---

## Commands

### `lumina run <file.lum>`

Execute a Lumina program.

```bash
lumina run myfile.lum
```

**Flags:**
- `--node-id <id>` — Override the cluster node ID (for multi-node testing)

The program runs sequentially through all statements. If `every` or `for` timers are present, the engine enters live mode and ticks in real-time until interrupted with `Ctrl+C`.

---

### `lumina check <file.lum>`

Type-check and analyze a program without running it.

```bash
lumina check myfile.lum
```

Output: `✓ myfile.lum — no errors found` on success, or detailed diagnostics on failure.

Use this for CI pipelines and pre-commit hooks.

---

### `lumina fmt <file.lum>`

Format a Lumina source file in-place using the canonical style.

```bash
lumina fmt myfile.lum
```

Output: `✓ myfile.lum — formatted`

---

### `lumina repl`

Start an interactive Read-Eval-Print Loop.

```bash
lumina repl
```

Type Lumina expressions and statements interactively. Multi-line input is supported with brace-depth tracking. Type `:help` to see inspector commands.

---

### `lumina update`

Update Lumina to the latest version. This replaces both `lumina` and `lumina-lsp` binaries in-place.

```bash
lumina update
```

**Flags:**
- `--check` — Only check if an update is available, don't download
- `--force` — Re-download and reinstall even if already on the latest version (useful for repairing corrupted installs)

**How it works:**
1. Queries the GitHub Releases API for the latest version tag
2. Compares against the currently installed version
3. Downloads the correct platform-specific binaries (with SHA256 verification)
4. Atomically replaces the running binaries

**Examples:**

```bash
# Check if there's a new version
lumina update --check

# Update to the latest version
lumina update

# Force-reinstall the current version
lumina update --force
```

**Notes:**
- Requires `curl` to be available on your system
- On Windows, the current binary is renamed to `.old` before replacement (standard self-update pattern)
- On macOS, the quarantine flag is automatically removed

---

### `lumina setup`

Automated IDE and environment setup. Detects installed editors and installs the Lumina extension.

```bash
lumina setup
```

This command runs automatically during installation. It scans for:
- **VS Code-compatible editors**: VS Code, VSCodium, Cursor, Windsurf, Positron, Code Insiders, Code OSS
- **Neovim**: Auto-generates a zero-config LSP plugin at `~/.config/nvim/plugin/lumina.lua`

The extension provides syntax highlighting, live diagnostics, go-to-definition, and find-all-references via the built-in `lumina-lsp` language server.

---

### `lumina uninstall`

Remove Lumina from your system.

```bash
lumina uninstall
```

This command:
1. Uninstalls the VS Code extension from all detected editors
2. Removes the `~/.lumina` directory (binaries and environment)
3. Cleans PATH entries from shell profiles (`.bashrc`, `.zshrc`, `.profile`, etc.)

---

### `lumina get documentation`

Output the master knowledge atlas to the current directory for AI-assisted development.

```bash
lumina get documentation
```

Creates `master_knowledge.md` in the current working directory. This file contains the full Lumina technical reference — designed to be ingested by AI coding assistants for context-aware code generation.

---

### `lumina query <expression>`

Evaluate a standalone Lumina expression.

```bash
lumina query "1 + 2 + 3"
```

Useful for quick calculations and testing expressions without creating a full `.lum` file.

---

### `lumina provider <command>`

Manage external data providers.

```bash
lumina provider list          # List installed providers
lumina provider install <name>   # Install a provider (registry coming soon)
```

Built-in protocol support: `redfish`, `snmp`, `modbus`.

---

### `lumina cluster <command>`

Cluster management for distributed Lumina nodes.

```bash
lumina cluster start <spec.lum> [node_id]   # Start a cluster node
lumina cluster status                        # Show cluster status
lumina cluster join <address>                # Join an existing cluster
```

See [08-advanced-features.md](08-advanced-features.md) for cluster configuration details.

---

### `lumina --version`

Print the version string.

```bash
lumina --version
# Output: Lumina v2.0.0: The Cluster Release
```

Also available as `lumina version` and `lumina -v`.

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `LUMINA_HOME` | `~/.lumina` | Root directory for Lumina installation |
| `LUMINA_SKIP_CHECKSUM` | `0` | Set to `1` to skip SHA256 verification during install |

---

## File Locations

| Path | Contents |
|------|----------|
| `~/.lumina/bin/lumina` | CLI binary |
| `~/.lumina/bin/lumina-lsp` | Language server binary |
| `~/.lumina/env` | Shell environment file (sourced by your shell profile) |
