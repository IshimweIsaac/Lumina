# Lumina
**The Sovereignty of Infrastructure**

Lumina is a **Distributed Reactive Language (DRL)** for high-availability infrastructure orchestration. It replaces fragile, event-driven scripts with a deterministic state-resolution engine. By defining infrastructure logic as a **Directed Acyclic Graph (DAG)** of dependencies, Lumina automatically propagates state changes and executes side-effects without the need for manual event-handling boilerplate.


[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Lumina-v2.0.0-green.svg)](CHANGELOG.md)

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

### 🌐 Sovereign Clustering
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
curl -fsSL https://lumina-lang.web.app/install.sh | sh && . ~/.lumina/env
```

**Windows (PowerShell)**
```powershell
iwr https://lumina-lang.web.app/install.ps1 -useb | iex
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

- **[`crates/`](./crates/)**: The core engine (Lexer, Parser, Analyzer, Runtime).
- **[`docs/knowledge/`](./docs/knowledge/)**: High-fidelity AI training suite and deep-dives.
- **[`examples/`](./examples/)**: Runnable architectures for IoT, Infrastructure, and Security.
- **[`extensions/`](./extensions/)**: VS Code language support.
- **[`website/`](./website/)**: The interactive WASM playground.

---

## Documentation

- **[Mental Model](./docs/knowledge/01-mental-model.md)**: Transitioning from procedural to reactive thinking.
- **[Syntax Reference](./docs/knowledge/02-syntax-reference.md)**: Exhaustive guide to the Lumina grammar.
- **[Patterns Cookbook](./docs/knowledge/06-patterns-cookbook.md)**: 14 complete, runnable infrastructure patterns.
- **[FFI & Embedding](./docs/knowledge/08-advanced-features.md)**: Using Lumina from other languages.

---

## Contributing

We welcome contributions to the core engine and the LSL standard library. Please see [CONTRIBUTING.md](CONTRIBUTING.md) for architecture guidelines.

## License

Lumina is open-source software licensed under the [MIT License](LICENSE).
