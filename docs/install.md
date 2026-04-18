# Installing Lumina

Lumina v2.0 "Sovereign Cluster" offers multiple ways to get started on your machine.

## 1. Automated Installer (Recommended)

The easiest way to install Lumina on Linux and macOS is via the one-line installer script. It automatically detects your platform and verifies the download.

```bash
curl -fsSL https://lumina-lang.dev/install.sh | sh
```

## 2. Homebrew (macOS)

If you use Homebrew, you can install Lumina using our official tap:

```bash
brew tap IshimweIsaac/lumina
brew install lumina
```

This installs both the `lumina` CLI and the `lumina-lsp` (Language Server Protocol) for IDE support.

## 3. Manual Binary Downloads

You can download the binaries directly from our [GitHub Releases](https://github.com/IshimweIsaac/Lumina/releases).

| Platform | Binary | Architecture |
| --- | --- | --- |
| **Linux** | [lumina-linux-x64](https://github.com/IshimweIsaac/Lumina/releases/latest/download/lumina-linux-x64) | x86_64 |
| **Linux** | [lumina-linux-arm64](https://github.com/IshimweIsaac/Lumina/releases/latest/download/lumina-linux-arm64) | ARM64 |
| **macOS** | [lumina-macos-x64](https://github.com/IshimweIsaac/Lumina/releases/latest/download/lumina-macos-x64) | x86_64 (Intel) |
| **macOS** | [lumina-macos-arm64](https://github.com/IshimweIsaac/Lumina/releases/latest/download/lumina-macos-arm64) | ARM64 (Apple Silicon) |
| **Windows** | [lumina-windows-x64.exe](https://github.com/IshimweIsaac/Lumina/releases/latest/download/lumina-windows-x64.exe) | x86_64 |

### Verification

After downloading, verify the checksum to ensure the file has not been tampered with:

```bash
# Example for Linux x64
curl -LO https://github.com/IshimweIsaac/Lumina/releases/latest/download/lumina-linux-x64.sha256
sha256sum -c lumina-linux-x64.sha256
```

## 4. Verification

Once installed, verify the installation by checking the version:

```bash
lumina --version
```

Expected output: `Lumina v2.0.0` (or similar).
