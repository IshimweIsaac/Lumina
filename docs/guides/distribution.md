# Lumina v1.8: Distribution & Installation 🌍

Lumina is designed to be accessible everywhere. This guide explains how to install the engine and its ecosystem on your preferred platform.

---

## 1. Quick Install (Shell)

For most developers on Linux or macOS, the easiest way to install Lumina is using the official shell script:
```bash
curl -sSL https://lumina-lang.web.app/install.sh | bash
```
This script automatically detects your OS and architecture, fetches the latest binary, and adds it to your path.

---

## 2. Package Managers

### **Homebrew (macOS / Linux)**
We maintain an official Homebrew formula. Use it to keep Lumina updated easily:
```bash
brew tap lumina-lang/tap
brew install lumina
```

### **NPM (JavaScript / Node.js)**
For developers building web or Node.js applications, use the `@lumina-lang/core` package:
```bash
npm install @lumina-lang/core
npx lumina --version
```
This package uses a post-install hook to fetch the correct native binary for your system.

---

## 3. Containerization (Docker)

For reproducible environments or IoT edge deployments, use our official Docker images:
```bash
docker pull luminalang/core:v1.8.0
```
Or use the provided `docker-compose.yml` to start an isolated development environment.

---

## 4. IDE Integration: VS Code

The official **Lumina Extension** provides syntax highlighting, real-time diagnostics, and "Go to Definition" support through the built-in Language Server (LSP).
1.  Open VS Code.
2.  Search for "Lumina Language" in the Extensions marketplace.
3.  Click "Install".
Note: The extension automatically looks for the `lumina` binary in your system path.

---

## 5. Summary: One Engine, Everywhere
Whether you are running on a Raspberry Pi at the edge or a powerful workstation, Lumina v1.8 provides the tools you need for a seamless developer experience.
