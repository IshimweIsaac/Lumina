# Lumina: A Reactive Language for Target Infrastructure and Distributed State

## Abstract

Lumina is a statically typed, declarative, and reactive programming language engineered in Rust, designed specifically for modeling and managing **Target Infrastructure**. Unlike traditional imperative languages where state transitions are manually coordinated, Lumina employs a continuous evaluation model to synchronize desired state with physical and digital reality. Domain entities are defined with intrinsic storage and derived mathematical relationships, allowing for autonomous infrastructure orchestration, fleet-wide monitoring, and deterministic system state resolution.

---

## 1. Introduction

Modern software architecture frequently struggles with the synchronization of distributed state and the unintended consequences of imperative control flow across varying subsystems. Lumina mitigates these systemic challenges by inverting the control paradigm. Instead of explicitly instructing the system when and how to update context, developers declare the inherent relationships between properties and the subsequent reactive triggers that must fire upon specific state variations.

### 1.1 Evolution & Feature History

Lumina has evolved from a simple reactive engine into a **Sovereign Infrastructure Language**.

#### **v1.9: The Metal Release (Latest Stable)**
*   **Lumina Standard Library (LSL)**: Pre-defined entity schemas for datacenter, network, Kubernetes, and power infrastructure — composed, not inherited.
*   **Native Southbound Protocols**: Agentless hardware polling via `provider` blocks (Redfish, SNMP v3, Modbus TCP).
*   **Declarative Security**: Security as a structural truth in the DAG. Write operations are blocked when auth context evaluates false (`L039`).
*   **`env()` Built-in**: Secure environment variable access, returning `Secret` values.
*   **Query Interface (`lumina query`)**: Interrogate the truth store from the CLI.
*   **Provider Management (`lumina provider`)**: Install and manage southbound protocol adapters.

#### **v1.8: The Ecosystem Release**
*   **Plugin System (`import plugin`)**: Dynamically load external adapters and components.
*   **Secret Management (`Secret`)**: Secure handling of credentials, preventing accidental leakage.
*   **Distributed State Consistency**: Introduces `timeout`, `fallible`, and `unknown` states to handle unreliable infrastructure.
*   **Opinionated Formatter (`lumina fmt`)**: Standardized canonical formatting for all Lumina source files.

#### **v1.8: The Experience Release**
*   **Zero-Configuration Installer**: Native `.deb`, `.exe`, and Homebrew support with automated `lumina setup`.
*   **"Teaching" Diagnostics**: Rewritten compiler errors that provide mentoring and actionable hints.
*   **Professional Branding**: Dedicated documentation site and high-fidelity VS Code extension.
*   **Performance Optimization**: WASM runtime and CLI binaries optimized for scale and speed.

#### **v1.6: The Infrastructure Release**
*   **Entity Relationships (`ref`)**: Structural truth declaration—one entity can reference another.
*   **Structural Traversal**: Traverse relationships in rules and derived fields.
*   **Multi-Condition Triggers (`and`)**: Fire rules only when compound truths are met.
*   **Write Capabilities (`write`)**: Send commands back to the physical world.
*   **Frequency Conditions**: Detect flapping with `N times within <duration>` triggers.
*   **Temporal Truth**: Native `Timestamp` type with `.age` accessor and `now()` function.
*   **LSP v2**: Production-grade IDE support with rename, references, and code actions.

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

*   **[Lumina Complete Guide (v1.8)](./docs/Lumina_Complete_Guide.md)**: The technical bible of the Lumina language.
*   **[Language Specification](./docs/SPEC.md)**: EBNF grammar and v1.8 syntax reference.
*   **[Architecture Overview](./docs/ARCHITECTURE.md)**: Deep dive into the reactive engine, adapters, and the write-back cycle.

---

## 3. Architecture and Execution Model

The Lumina compiler and runtime pipeline is implemented in Rust. Every program flows through this pipeline:

1.  **`lumina-lexer`**: High-throughput DFA tokenizer.
2.  **`lumina-parser`**: Hybrid recursive-descent and Pratt parser.
3.  **`lumina-analyzer`**: Static type safety, ref-integrity, and cycle detection.
4.  **`lumina-runtime`**: The reactive Snapshot VM. Handles temporal scheduling and external adapter synchronization.
5.  **`lumina-lsp`**: Multi-protocol language server providing real-time IDE feedback.

---

## 4. Language Specification (v1.8 Example)

```lua
-- Infrastructure modeling with v1.6 features
external entity CoolingUnit {
  isRunning: Boolean
  isFailing := not isRunning
} sync on isRunning

entity Server {
  cpuTemp: Number
  cooling: ref CoolingUnit -- Structural relationship
  
  isOverheating := cpuTemp > 85
  isAtRisk := isOverheating and cooling.isFailing
}

-- Multi-condition trigger with write action
rule EmergencyShutdown for (s: Server)
  when s.isOverheating becomes true
  and s.cooling.isFailing becomes true {
    write s.throttle = 10 -- Command the physical world
    alert severity: "critical", message: "Emergency throttle applied"
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
Run the full regression suite to verify v1.8 compliance:
```bash
cargo test --workspace
```

## License
This software is distributed under the MIT License. Reference [`LICENSE`](LICENSE) for details.
