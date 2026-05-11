# Lumina System Architecture (v2.0)

The Lumina runtime engine is designed for absolute correctness and deterministic reactivity. Version 2.0, the "Cluster Release," transforms the language from a single-node engine into a distributed mesh orchestrator, supporting high-availability and workload migration.

---

## 1. Compiler Pipeline Overview

Lumina programs are processed through a strictly ordered pipeline. Every stage is designed for zero-allocation performance and strong safety guarantees.

### 1.1 Lexical Analysis (`lumina-lexer`)
Tokenization is performed via the `logos` crate, generating a high-speed Deterministic Finite Automaton (DFA). Version 2.0 adds optimized tokenization for multi-line strings and duration literals.

### 1.2 Syntax Analysis (`lumina-parser`)
The parser maps tokens into a structured Abstract Syntax Tree (AST):
*   **Recursive Descent**: Handles declarative constructs (`entity`, `rule`, `fn`).
*   **Pratt Parsing**: For expressions, managing complex operator precedence and accessor logic.

### 1.3 Semantic Analysis (`lumina-analyzer`)
The analyzer performs two distinct passes:
1.  **Declaration Registration**: Records all entities, fields, and pure functions.
2.  **Structural Integrity & Typecheck**: Validates expressions and constructs a topological `DependencyGraph`.
3.  **Cyclic Dependency Detection**: Ensures all derived fields form a Directed Acyclic Graph (DAG) for deterministic propagation.

---

## 2. The Reactive Engine (`lumina-runtime`)

### 2.1 Snapshot-Based Virtual Machine
Lumina implements a **Self-Healing Guarantee**. Before any destructive action:
1.  The VM takes a complete memory **Snapshot**.
2.  Evaluation proceeds. If a recursion limit (100) or invariant is breached, the runtime **Automatically Rolls Back** to the snapshot.
3.  **Diagnostic Reporting**: Instead of crashing, the engine returns a structured `Diagnostic` object to the host system.

### 2.2 Incremental Aggregates
Fleet-level summaries (`avg`, `sum`, `count`) are updated **incrementally**.
*   **O(1) Evaluation**: When an instance updates, the aggregate counters are adjusted in constant time, rather than re-scanning the entire fleet.
*   **Reactive Flow**: Aggregates are integrated into the main dependency graph, allowing derived fields to depend on fleet-level metrics.

### 2.3 Temporal Engine & Stabilization
The v2.0 temporal engine is the most stable version yet.
*   **Unified TimerHeap**: Manages both `every` intervals and `for` duration stabilization.
*   **Edge Detection**: The engine maintains a transition cache, firing rules only on precise state transitions (`becomes`).

### 2.4 The Distributed Cluster Engine (`lumina-cluster`)
Version 2.0 introduces native multi-node orchestration directly integrated into the language runtime.
*   **Gossip Protocol (`UdpTransport`)**: Nodes discover each other and broadcast state changes using a custom, high-speed UDP transport layer. This enables real-time peer health monitoring and message routing.
*   **StateMesh**: A conflict-free replicated data structure. When nodes broadcast state updates, the `StateMesh` resolves conflicts using Last-Write-Wins (LWW) and version vectors, ensuring eventual consistency across the cluster.
*   **Raft-Inspired Election**: The cluster requires a defined `quorum` to elect a leader. This leader orchestrates cross-node actions like `migrate` and `deploy`.
*   **Memory Scaling (`FxHashMap`)**: To support high-throughput network syncing, the core indexers and entity stores have been migrated from standard cryptographically secure maps to `FxHashMap`, allowing for O(1) tracking of 100k+ entities with negligible RAM bloat.

---

## 3. Platform & Distribution

### 3.1 WASM Bridge (`lumina-wasm`)
The WASM layer provides a high-performance interface for browser embedding.
*   **Deterministic Evaluation**: The exact same Rust engine runs in the browser, ensuring simulation parity.
*   **JS Integration**: Optimized serialization allow for sub-millisecond event injection from React or Vanilla JS frontends.

### 3.2 Polyglot FFI (`lumina_ffi`)
The stable C ABI enables integration with any language:
*   **C-Compatible Interface**: Exports functions for creation, ticking, and state export.
*   **Memory Safety**: Enforces strict ownership rules across the FFI boundary to prevent leaks.

### 3.3 LSP v2 (`lumina-lsp`)
The v2.0 Language Server provides production-grade IDE support:
*   **Live Diagnostics**: Real-time type checking and cycle detection.
*   **Navigation**: "Go to Definition" and "Find All References" for complex data flows.

---

## 4. Technical Stack
*   **Rust**: Deterministic performance and memory safety.
*   **Logos**: DFA-based high-performance lexing.
*   **Pratt Parsing**: Precedence-climbing expression evaluation.
*   **Snapshot VM**: Atomic state transitions with guaranteed rollback.
*   **Serde**: Efficient serialization for WASM and FFI boundaries.
