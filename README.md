# Lumina

> **A declarative, reactive language for infrastructure orchestration.**

Lumina turns your infrastructure into a **living, reactive system**. Define entities, declare rules, and let the engine handle the rest from real-time monitoring to cross-cluster workload migration.

[![Build](https://github.com/IshimweIsaac/Lumina/actions/workflows/rust.yml/badge.svg)](https://github.com/IshimweIsaac/Lumina/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## 1. Version History

Lumina has evolved from a simple reactive engine into a **Sovereign Infrastructure Language**.

#### **v1.9: The Metal Release (Latest Stable)**
*   **Lumina Standard Library (LSL)**: Pre-defined entity schemas for datacenter, network, Kubernetes, and power infrastructure — composed, not inherited.
*   **Native Southbound Protocols**: Agentless hardware polling via `provider` blocks (Redfish, SNMP v3, Modbus TCP).
*   **Declarative Security**: Security as a structural truth in the DAG. Write operations are blocked when auth context evaluates false (`L039`).
*   **`env()` Built-in**: Secure environment variable access, returning `Secret` values.
*   **Query Interface (`lumina query`)**: Interrogate the truth store from the CLI.
*   **Provider Management (`lumina provider`)**: Install and manage southbound protocol adapters.

#### **v1.8: The Ecosystem & Experience Release**
*   **Plugin System (`import plugin`)**: Dynamically load external adapters and components.
*   **Secret Management (`Secret`)**: Secure handling of credentials, preventing accidental leakage.
*   **Distributed State Consistency**: Introduces `timeout`, `fallible`, and `unknown` states to handle unreliable infrastructure.
*   **Opinionated Formatter (`lumina fmt`)**: Standardized canonical formatting for all Lumina source files.
*   **Zero-Configuration Installer**: Native `.deb`, `.exe`, and Homebrew support with automated `lumina setup`.
*   **"Teaching" Diagnostics**: Rewritten compiler errors that provide mentoring and actionable hints.

#### **v1.6: The Infrastructure Release**
*   **Entity Relationships (`ref`)**: Structural truth declaration—one entity can reference another.
*   **Structural Traversal**: Traverse relationships in rules and derived fields.
*   **Multi-Condition Triggers (`and`)**: Fire rules only when compound truths are met.
*   **Write Capabilities (`write`)**: Send commands back to the physical world.
*   **Frequency Conditions**: Detect flapping with `N times within <duration>` triggers.
*   **Temporal Truth**: Native `Timestamp` type with `.age` accessor and `now()` function.

#### **v1.5: The Fleet Release**
*   **Fleet-Level Triggers**: Support for `any` and `all` conditions across entity instances.
*   **Reactive Aggregates**: `aggregate` blocks for `avg`, `sum`, `count` across device fleets.
*   **Structured Alerting**: Native `alert` actions with severity levels and `on clear` recovery logic.
*   **Rule Cooldowns**: Silence periods to prevent alert storms.
*   **Historical State**: The `prev()` keyword for transition-based logic.

#### **v1.4: The Developer Experience Release**
*   **Enhanced Diagnostics**: Rust-style error messages with source context and carets.
*   **Stateful REPL**: Persistent evaluator state across interactive sessions.
*   **Pure Functions (`fn`)**: Stateless logic encapsulation.
*   **Modules (`import`)**: Multi-file program support.
*   **Collections**: First-class list types (`type[]`) and indexing.

#### **v1.3: The Core Engine Release**
*   **Reactive Core**: The basic topological propagation engine.
*   **FFI & WASM**: Native C ABI and WebAssembly compilation targets.
*   **CLI**: Initial `run`, `check`, and `repl` commands.

---

## 2. Documentation

*   **[Lumina Complete Guide](./docs/Lumina_Complete_Guide.md)**: The technical bible of the Lumina language.
*   **[Language Specification](./docs/SPEC.md)**: EBNF grammar and syntax reference.
*   **[Architecture Overview](./docs/ARCHITECTURE.md)**: Deep dive into the reactive engine, adapters, and the write-back cycle.

---

## 3. Architecture and Execution Model

The Lumina compiler and runtime pipeline is implemented in Rust. Every program flows through this pipeline:

1.  **`lumina-lexer`**: High-throughput DFA tokenizer.
2.  **`lumina-parser`**: Hybrid recursive-descent and Pratt parser.
3.  **`lumina-analyzer`**: Static type safety, ref-integrity, and cycle detection.
4.  **`lumina-runtime`**: The reactive Snapshot VM. Handles temporal scheduling and external adapter synchronization.
5.  **`lumina-cluster`**: Distributed state mesh, gossip protocol, and leader election.
6.  **`lumina-lsp`**: Multi-protocol language server providing real-time IDE feedback.

---

## 4. Language Specification (v2.0 Example)

```lua
-- Cluster-aware infrastructure monitoring
cluster {
  node_id: "node-1"
  peers: ["node-2", "node-3"]
  quorum: 2
  election_timeout: 1.5 s
}

entity Workload {
  cpu: Number
  zone: String
  isHot := cpu > 80
}

aggregate NodeStats over Workload {
  total_cpu := sum(cpu)
  avg_cpu   := avg(cpu)
}

rule "NodeOverloaded"
when NodeStats.total_cpu > 100 {
  show "Node CPU is {NodeStats.total_cpu}% — rebalancing"
}
```

---

## 5. Compilation and Usage

### Prerequisites
* Rust Toolchain (`rustc` >= 1.70.0)
* Cargo Build Manager

### 5.1 Command Line Interface (CLI)
```bash
# Build the CLI
cargo build --release -p lumina-cli

# Execute a Lumina program
cargo run -p lumina-cli -- run main.lum
```

### 5.2 Foreign Function Interface (FFI)
```bash
# Build the shared library (.so / .dll / .dylib)
cargo build --release -p lumina-ffi
```

---

## 6. Development & Testing
Run the full regression suite to verify v2.0 compliance:
```bash
cargo test --workspace
```

## License
This software is distributed under the MIT License. Reference [`LICENSE`](LICENSE) for details.
