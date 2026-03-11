# Lumina: A Declarative and Reactive Programming Language for State-Driven Systems

## Abstract

Lumina is a statically typed, declarative, and reactive programming language engineered in Rust, designed specifically for modeling complex state-driven systems. Unlike traditional imperative languages where state transitions are manually coordinated, Lumina employs a continuous evaluation model. Domain entities are defined with intrinsic storage and derived mathematical relationships. Reactive rule cascades autonomously compute state changes, temporal triggers enforce duration and interval-based logic, and invariant assertions ensure computational consistency. The resulting execution model provides a robust framework for asynchronous and deterministic system state resolution, suitable for both embedded logic and high-level behavioral orchestration.

---

## 1. Introduction

Modern software architecture frequently struggles with the synchronization of distributed state and the unintended consequences of imperative control flow across varying subsystems. Lumina mitigates these systemic challenges by inverting the control paradigm. Instead of explicitly instructing the system *when* and *how* to update context, developers declare the inherent relationships between properties and the subsequent reactive triggers that must fire upon specific state variations.

### 1.1 Key Features in v1.3
*   **Declarative Entity Modeling**: Fields are partitioned into basal and derived categories. Derived fields (`:=`) maintain a strictly guaranteed relationship with their dependencies through continuous topological re-evaluation using Kahn's algorithm.
*   **Deterministic Reactive Automata**: State mutations are constrained to atomic intervals. The runtime leverages a directed acyclic evaluation graph to cascade variable updates without triggering divergent recursion limits.
*   **Temporal Logic Triggers**: The inclusion of `for` and `every` reactive clauses enables the execution of interval-based and sustained-duration logic, natively offloading manual timer management to the runtime orchestrator.
*   **Robust Boundary and Type Checking**: Compile-time semantic analysis combined with runtime bounds constraints (`@range`) minimizes invalid state allocations.
*   **Self-Healing Snapshots**: Before every state-changing operation, the runtime takes a deep-copy snapshot. If anything fails (like a recursion limit breach or range violation), the snapshot is restored instantly, guaranteeing a stable state without crashing.

---

## 2. Architecture and Execution Model

The Lumina compiler and runtime pipeline is implemented in Rust, exploiting zero-cost abstractions and strict memory safety guarantees. Every `.lum` program flows through this exact 4-stage pipeline:

1.  **`lumina-lexer`**: A high-throughput deterministic finite automaton tokenizer powered by `logos` that generates `Vec<SpannedToken>`.
2.  **`lumina-parser`**: A hybrid recursive-descent and Pratt parser optimized for context-free grammar evaluation and operator precedence (`Ast`).
3.  **`lumina-analyzer`**: Constructs a dependency graph natively as flat `u32` `NodeId` arrays, enforces static type safety, and validates evaluation order statically.
4.  **`lumina-runtime`**: The core Snapshot VM (`Evaluator`) handling state allocation, `becomes` edge-transition detection, and temporal scheduling.

---

## 3. Language Specification (v1.3 Example)

### 3.1 Entity Schemas & Initialization
Entities define the polymorphic structure of state contexts.

```lua
entity Moto {
  @doc "Battery capacity measured in watt-hours (Wh)"
  @range 0 to 100
  battery: Number
  isBusy: Boolean
  status: Text
  
  -- Derived fields autonomously calculate their value topologically
  isLowBattery := battery < 20
  isCritical   := battery < 5
}

-- Instantiate with explicit basal fields (derived fields compute automatically)
let moto1 = Moto { battery: 80, isBusy: false, status: "available" }
```

### 3.2 Rule Cascades & Temporal Semantics
Control logic relies upon the `rule` keyword. The `becomes` modifier ensures rules execute strictly on edge transitions, minimizing computational redundancy.

```lua
-- Fires exactly once when the condition transitions from false to true
rule "Critical Battery" {
  when Moto.isCritical becomes true
  then update moto1.status to "maintenance"
  then show "CRITICAL: Moto pulled from service"
}

-- Fires after the condition holds continuously for the duration
rule "Auto-lock idle bike" {
  when Moto.isBusy becomes false for 15 m
  then update moto1.status to "locked"
}

-- Fires unconditionally on a scheduled recurring interval
rule "Fleet heartbeat" {
  every 1 h
  then show "Fleet check running..."
}
```

---

## 4. Compilation and Usage

### Prerequisites
* Rust Toolchain (`rustc` >= 1.70.0)
* Cargo Build Manager

### 4.1 Command Line Interface (CLI)
```bash
# Build the CLI
cargo build --release -p lumina-cli

# Execute abstract syntax tree evaluation
cargo run --bin lumina -- run tests/spec/fleet.lum

# Perform static analysis and dependency validation without executing
cargo run --bin lumina -- check tests/spec/fleet.lum

# REPL (Note: State persistence is limited in v1.3; full persistence coming in v1.4)
cargo run --bin lumina -- repl
```

---

## 5. Using Lumina from Python (C FFI)

Lumina implements a highly secure, memory-safe Foreign Function Interface (FFI) allowing native linkage and execution from external environments via a shared library.

### 5.1 FFI Setup
```bash
# 1. Build the shared library (produces liblumina_ffi.so / .dylib / .dll)
cargo build --release -p lumina-ffi

# 2. Add target/release to your LD_LIBRARY_PATH (or run Python from the repo root)
```

### 5.2 Python `lumina_py` Wrapper Example
```python
from crates.lumina_ffi.lumina_py import LuminaRuntime

# Load a Lumina script from source string
rt = LuminaRuntime.from_source("""
entity Iterator { index: Number }
let my_iter = Iterator { index: 0 }
""")

# Apply extrinsic variables explicitly into the evaluation engine
result = rt.apply_event("my_iter", "index", 42)

# Export the entire topological system state as a JSON dictionary
print(rt.export_state()["instances"]["my_iter"]["fields"]["index"]) # 42
```

---

## 6. WebAssembly Integration (Playground)

Lumina cross-compiles to WebAssembly (`wasm32-unknown-unknown`), operating inside standard JavaScript runtimes for browser experiences.

```bash
# Build WASM package
cargo install wasm-pack
rustup target add wasm32-unknown-unknown
cd crates/lumina-wasm
wasm-pack build --target web --out-dir pkg --release

# Serve the locally built WASM playground
cd ../..
python3 -m http.server 8080
# Open http://localhost:8080/playground/index.html
```
*(Note: System-native timers are approximate inside the WASM target; native CLI builds utilize precise `std::time::Instant`.)*

---

## 7. Known Limitations (Targeted for v1.4)
The current v1.3 spec formally notes these boundaries, which define the scope for future versions:
*   **External Entities:** Syntax is parsed but `Supabase` and `Docker` adapters are not yet implemented.
*   **Functions & Lists:** `fn` pure functions and `List<T>` types are lexically parsed but not executed by the runtime.
*   **REPL State Persistence:** The v1.3 REPL rebuilds state on every input; multi-line rule retention will arrive in v1.4.
*   **Source Context in Diagnostics:** Errors display codes (e.g., `L003`) and lines, but lack precise column-caret highlighting.

---

## 8. Development & Testing
Before committing to the codebase, read our detailed [CONTRIBUTING.md](./CONTRIBUTING.md) guide which outlines the strict memory safety, topological ordering rules, and `apply_update` snapshot invariants that define the engine.

Run the full specification regression suite before every PR:
```bash
cargo test --workspace
```

## License
This software and associated documentation files are distributed under the MIT License. Reference [`LICENSE`](LICENSE) for complete legal stipulations.
