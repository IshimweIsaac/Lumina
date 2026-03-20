# Lumina: A Declarative and Reactive Programming Language for State-Driven Systems

## Abstract

Lumina is a statically typed, declarative, and reactive programming language engineered in Rust, designed specifically for modeling complex state-driven systems. Unlike traditional imperative languages where state transitions are manually coordinated, Lumina employs a continuous evaluation model. Domain entities are defined with intrinsic storage and derived mathematical relationships. Reactive rule cascades autonomously compute state changes, temporal triggers enforce duration and interval-based logic, and invariant assertions ensure computational consistency. The resulting execution model provides a robust framework for asynchronous and deterministic system state resolution, suitable for both embedded logic and high-level behavioral orchestration.

---

## 1. Introduction

Modern software architecture frequently struggles with the synchronization of distributed state and the unintended consequences of imperative control flow across varying subsystems. Lumina mitigates these systemic challenges by inverting the control paradigm. Instead of explicitly instructing the system when and how to update context, developers declare the inherent relationships between properties and the subsequent reactive triggers that must fire upon specific state variations.

### 1.1 Key Features in v1.5
*   **Declarative Entity Modeling**: Fields are partitioned into basal and derived categories. Derived fields (`:=`) maintain a strictly guaranteed relationship with their dependencies through continuous topological re-evaluation.
*   **Fleet-Level Triggers**: Support for `when any` and `when all` conditions across entity instances (e.g., `when any Moto.battery becomes < 10`).
*   **Historical State Access**: Use the `prev()` keyword to access a field's value from the previous engine tick.
*   **External Entities & Adapters**: Native support for synchronizing state with external sources (e.g., Supabase, MQTT) via the Adapter protocol.
*   **List & Collection Types**: First-class support for list types (`type[]`), list literals (`[...]`), and element indexing (`list[0]`).
*   **Built-in Aggregates**: High-performance built-in functions for collection analysis (`len`, `sum`, `min`, `max`, etc.).
*   **Pure Functions (`fn`)**: Stateless, side-effect-free functions for complex logic encapsulation.
*   **Language Server Protocol (LSP)**: Full IDE support with real-time error squiggles, hover tooltips, and document symbol navigation.
*   **Temporal Logic Triggers**: `for` and `every` reactive clauses enable interval-based and sustained-duration logic.

---

## 2. Architecture and Execution Model

The Lumina compiler and runtime pipeline is implemented in Rust, exploiting zero-cost abstractions and strict memory safety guarantees. Every program flows through this pipeline:

1.  **`lumina-lexer`**: A high-throughput deterministic finite automaton tokenizer.
2.  **`lumina-parser`**: A hybrid recursive-descent and Pratt parser.
3.  **`Module Loader`**: Resolves `import` statements and builds a unified AST.
4.  **`lumina-analyzer`**: Enforces static type safety and validates evaluation order.
5.  **`lumina-runtime`**: The core Snapshot VM (`Evaluator`) handling state allocation, fleet tracking, and temporal scheduling.
6.  **`lumina-lsp`**: Provides real-time developer feedback and cross-file navigation.

---

## 3. Language Specification (v1.5 Example)

### 3.1 Entity Schemas & Initialization
Entities define the structure of state contexts.

```lua
import "types.lum"

fn calculate_priority(battery: Number) -> Number {
  if battery < 10 then 1 else 2
}

entity Moto {
  @doc "Battery capacity measured in watt-hours (Wh)"
  @range 0 to 100
  battery: Number
  isBusy: Boolean
  logs: Text[]
  
  -- Derived fields autonomously calculate their value topologically
  priority    := calculate_priority(battery)
  isCritical  := battery < 5
  
  -- Access historical state with prev()
  wasCharged  := battery > prev(battery)
}

-- Instantiate with explicit basal fields
let moto1 = Moto { battery: 80, isBusy: false, logs: ["init"] }
```

### 3.2 List Operations & Built-ins
Lumina provides native operations for handling collections of data.

```lua
let numbers = [10, 20, 30, 40]
let first = numbers[0]
let total = sum(numbers)
let count = len(numbers)

-- List functions
let lowest = min(numbers)
let highest = max(numbers)
let tail_list = tail(numbers)
```

### 3.3 Fleet Triggers & Lifecycle
Rules can monitor the entire fleet of entities or manage their existence.

```lua
-- Fires when ANY instance becomes critical
rule "Single Unit Critical" {
  when any Moto.isCritical becomes true
  then show "Critical unit detected."
}

-- Create and Delete instances dynamically
rule "Cleanup" {
  when any Moto.battery becomes 0
  then delete moto1
}
```

---

## 4. v1.5 Roadmap (Planned Features)
The following features are part of the v1.5 design specification and are currently under development:

*   **`aggregate` Blocks**: Top-level structural blocks for defining named fleet-wide facts (e.g., `aggregate FleetStatus over Moto { avgBattery := avg(battery) }`).
*   **`alert` Action**: Specialized rule action for structured signals with severity levels and metadata.
*   **`on clear` Blocks**: Recovery logic that fires when a rule condition is no longer met.
*   **Rule `cooldown`**: Managing alert volume by enforcing wait periods between rule firings.

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

# Interactive REPL
cargo run -p lumina-cli -- repl
```

### 5.2 Language Server (LSP)
Install the LSP globally to enable IDE features in VS Code:
```bash
cargo install --path crates/lumina-lsp
```

---

## 6. Foreign Function Interface (FFI)

Lumina implements a secure C ABI for integration into external environments.

```bash
# Build the shared library
cargo build --release -p lumina-ffi
```

---

## 7. WebAssembly Integration

Lumina cross-compiles to WebAssembly, operating inside standard JavaScript runtimes.

```bash
cd crates/lumina-wasm
wasm-pack build --target web --release
```

---

## 8. Development & Testing
Before committing to the codebase, read our detailed [CONTRIBUTING.md](./CONTRIBUTING.md) guide.

Run the full specification regression suite:
```bash
cargo test --workspace
```

## License
This software and associated documentation files are distributed under the MIT License. Reference [`LICENSE`](LICENSE) for complete legal stipulations.
