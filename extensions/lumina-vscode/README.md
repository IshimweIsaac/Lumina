# Lumina for VS Code

Bring the "Describe what is true" experience to your favorite editor. This official extension provides first-class support for the Lumina language, including syntax highlighting, snippets, and deep integration with the Lumina Language Server (LSP).

![Lumina Banner](https://lumina-lang.web.app/logo.png)

## Features

- **🚀 Professional Syntax Highlighting**: Accurate, performant colorization for `.lum` files powered by a native TextMate grammar.
- **✨ Smart Snippets**: Rapidly scaffold entities, rules, and aggregates with intelligent tab-stops.
- **🔍 Language Server Integration**: Real-time error reporting and semantic analysis (requires `lumina-lsp` installed).
- **🛠 Configuration Support**: Bracket matching, indentation rules, and auto-closing pairs tailored for IoT workflows.

## Installation

### 1. Install the CLI
Lumina works best when the `lumina` CLI and `lumina-lsp` are in your PATH.

```bash
# macOS (Homebrew)
brew install luminalang/core/lumina

# Linux/macOS (Shell)
curl -fsSL https://lumina-lang.web.app/install.sh | sh
```

### 2. Install the Extension
Search for **Lumina** in the VS Code Extensions view (`Ctrl+Shift+X`) and click Install.

## Quick Start

Create a new file named `monitor.lum`:

```lumina
entity Sensor {
  temperature: Number
}

rule "High Temp"
for (s: Sensor)
when s.temperature > 40 {
  show "Warning: ${s.id} is overheating!"
}
```

## Settings

This extension provides the following settings:

* `lumina.serverPath`: Path to the `lumina-lsp` executable (Default: `lumina-lsp`).
* `lumina.trace.server`: Trace level for the language server communications.

## About Lumina

Lumina is a reactive, declarative language specifically designed for the IoT and Infrastructure age. Instead of writing code that *does* things, you write code that *describes* what is true.

Learn more at [lumina-lang.web.app](https://lumina-lang.web.app).

---
Developed with ❤️ by the Lumina Team.
Licensed under [MIT](LICENSE).
