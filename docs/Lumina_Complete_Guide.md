# Lumina: The Definitive Guide (v1.8) 🛰️

_"Describe what is true. Lumina figures out what to do."_

Welcome to the official documentation for Lumina v1.8. This guide serves as your entry point into the Lumina ecosystem, providing everything from a beginner's introduction to deep technical internals.

> [!TIP]
> **New to Lumina?** Start with our **[Conceptual Getting Started](docs/GET_STARTED.md)** to understand the purpose, use cases, and problems Lumina solves.

---

## 📖 The Lumina Book (Deep Dives)

For a structured, deep-dive learning experience, we recommend following the Lumina Book chapters:

1.  **[Introduction & Philosophy](docs/book/introduction.md)**: Understand the "Truth vs Procedure" mental model.
2.  **[Zero-to-Hero Curriculum](docs/book/zero-to-hero.md)**: A step-by-step tutorial from basic entities to complex fleet operations.
3.  **[Engine Internals](docs/book/engine-internals.md)**: Deep dive into the Compiler, Analyzer, and Snapshot-based VM.
4.  **[WASM & Polyglot FFI](docs/book/wasm-and-ffi.md)**: How to embed Lumina in the Browser, Python, Go, and C.
5.  **[Diagnostics Reference](docs/book/diagnostics-reference.md)**: A complete guide to the L-codes and R-codes.

---

## 🛠️ Quick Start

### **Installation**
Get Lumina running on your machine in seconds:
```bash
curl -sSL https://lumina-lang.web.app/install.sh | bash
```
For more installation options (Homebrew, NPM, Docker), see the **[Distribution Guide](docs/guides/distribution.md)**.

### **The CLI**
The `lumina` binary is your main entry point. 
*   `lumina check <file>`: Validates syntax and types with deep diagnostics.
*   `lumina run <file>`: Executes a program and exports final state.
*   `lumina repl`: Launches the interactive shell.

---

## 🚀 Key Features of v1.8

*   **Teaching Diagnostics**: Beautiful, Rust-style error reports with actionable help messages.
*   **Stabilized WASM Engine**: Sub-millisecond reactive propagation in the browser.
*   **Fleet Aggregates**: Incremental, O(1) performance for summarizing large populations of entities.
*   **Deterministic Snapshots**: Guaranteed safety with zero-cost rollbacks on failure.

---

## 🤝 Community & Support

*   **[GitHub Repository](https://github.com/lumina-lang/lumina)**: Report bugs, request features, or contribute.
*   **[Official Website](https://lumina-lang.web.app)**: News, downloads, and the interactive Playground.

_Lumina v1.8 Experience Release | 2026 | Isaac Ishimwe_
