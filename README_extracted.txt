# Lumina
**The Sovereignty of Infrastructure**

Lumina is a **Distributed Reactive Language (DRL)** for high-availability infrastructure orchestration. It replaces fragile, event-driven scripts with a deterministic state-resolution engine. By defining infrastructure logic as a **Directed Acyclic Graph (DAG)** of dependencies, Lumina automatically propagates state changes and executes side-effects without the need for manual event-handling boilerplate.


[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Lumina-v2.1.2-green.svg)](CHANGELOG.md)

---

## The Philosophy: "Describe What is True"

Traditional infrastructure management is **imperative**: "If X happens, do Y, then check Z." This leads to race conditions and "split-brain" states.

Lumina is **declarative-reactive**. You define the *state* you want, and the engine maintains it through a Directed Acyclic Graph (DAG). If a sensor temperature rises, every dependent field, aggregate, and rule re-evaluates instantly and atomically across the entire cluster.

---

## Lumina in 30 Seconds

```lumina
-- Define a self-healing server rack
entity Server {
  cpu_temp: Number
  is_online: Boolean
  is_hot := cpu_temp > 80
}

aggregate RackStats over Server {
  avg_temp := avg(cpu_temp)
  critical_count := count(is_hot)
}

rule "Emergency Cooling"
when RackStats.critical_count > 2 {
  alert severity: "critical", message: "Fleet overheating: {RackStats.avg_temp}C"
}
on clear {
  show "Thermal levels stabilized."
}
```

---

## Core Pillars

### ⚡ Distributed Reactivity
A high-performance reactive engine written in Rust. Every state change is a transaction—updates either propagate fully through the graph or roll back entirely.

### 🌐 Architecting
Native cluster networking with UDP gossip, leader election, and state mesh synchronization. No external database or message broker required.

### 🔌 Universal Embedding
Lumina runs everywhere. Embed it in C, Python, Go, or Node.js via the **C-FFI**, or run it in the browser at native speed via **WebAssembly**.

### 🛠️ Developer Experience
- **Human-Centric Diagnostics**: Error messages that don't just report bugs—they teach the language.
- **LSP Support**: Full IDE integration for VS Code, VSCodium, Cursor, and Neovim.
- **Stateful REPL**: Experiment with reactive logic in real-time.

---

## Quick Start

### 1. Install Lumina

**Linux / macOS**
```bash
curl -fsSL https://lumina.rw/install.sh | sh && . ~/.lumina/env
```

**Windows (PowerShell)**
```powershell
iwr https://lumina.rw/install.ps1 -useb | iex
```

**NPM**
```bash
npm install -g @lumina-lang/core
```

### 2. Run Your First Program
Create `hello.lum`:
```lumina
entity World {
  status: Text
}
let w = World { status: "Quiet" }
rule "Greet" when World.status becomes "Loud" { show "Hello, Lumina!" }
update w.status = "Loud"
```

Execute it:
```bash
lumina run hello.lum
```

---

## Project Structure

### Core Engine
- **[`crates/`](./crates/)**: The Rust workspace containing all core crates:
  - [`lumina-lexer`](./crates/lumina-lexer/) — Tokenizer for Lumina source code.
  - [`lumina-parser`](./crates/lumina-parser/) — Parser producing the AST from tokens.
  - [`lumina-analyzer`](./crates/lumina-analyzer/) — Semantic analysis, type checking, and DAG construction.
  - [`lumina-diagnostics`](./crates/lumina-diagnostics/) — Human-centric, teaching-style error reporting.
  - [`lumina-runtime`](./crates/lumina-runtime/) — The reactive execution engine.
  - [`lumina-cluster`](./crates/lumina-cluster/) — Distributed clustering: gossip protocol, leader election, and state mesh.
  - [`lumina-cli`](./crates/lumina-cli/) — The `lumina` command-line interface.
  - [`lumina-lsp`](./crates/lumina-lsp/) — Language Server Protocol implementation for IDE support.
  - [`lumina-wasm`](./crates/lumina-wasm/) — WebAssembly compilation target for browser execution.
  - [`lumina_ffi`](./crates/lumina_ffi/) — C-FFI bindings for embedding Lumina in other languages.

### Ecosystem
- **[`website/`](./website/)**: The official Lumina website (Vite), including the interactive WASM playground and documentation pages.
- **[`playground/`](./playground/)**: Standalone WASM playground app (Vite + TypeScript).
- **[`extensions/`](./extensions/lumina-vscode/)**: VS Code / VSCodium language extension.
- **[`docs/`](./docs/)**: Documentation portal with guides, language specification, and version history.

### Distribution & Deployment
- **[`installers/`](./installers/)**: Native installers for Linux, macOS, and Windows.
- **[`Formula/`](./Formula/)**: Homebrew formula (`lumina.rb`) for macOS/Linux.
- **[`deploy/`](./deploy/)**: Docker configuration (`Dockerfile`, `docker-compose.yml`) for containerized environments.
- **[`scripts/`](./scripts/)**: Build, deployment, and stress-testing scripts.
- **[`.github/workflows/`](./.github/workflows/)**: CI/CD pipelines for releases, Firebase deployment, and installer publishing.

### Quality & Assets
- **[`tests/`](./tests/)**: Spec files (`.lum`) and oracle-based expected outputs for end-to-end testing.
- **[`Assets/`](./Assets/)**: Lumina branding icons and imagery.

---

## Documentation

- **[Documentation Portal](./docs/README.md)**: Entry point to all Lumina docs.
- **[Core Reference](./docs/guides/core_reference.md)**: Exhaustive guide to Lumina syntax and standard library.
- **[Language Specification](./docs/guides/language_spec.md)**: Formal language grammar and semantics.
- **[Tutorials & Getting Started](./docs/guides/tutorials.md)**: Quickstarts and setup guides.
- **[Operations & Cluster Guide](./docs/guides/operations.md)**: Running Lumina in production and distributed mode.
- **[Developer Guide](./docs/guides/developer_guide.md)**: Contributing to the Lumina codebase.
- **[The Lumina Book](./docs/book.md)**: In-depth exploration of the language design.
- **[Version Map](./docs/VERSION_MAP.md)**: Full history and patch notes across all versions.

---

## Contributing

We welcome contributions to the core engine and the standard library. Please see [CONTRIBUTING.md](CONTRIBUTING.md) for architecture guidelines.

## License

Lumina is open-source software licensed under the [MIT License](LICENSE).
