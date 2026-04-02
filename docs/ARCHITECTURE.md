# Lumina System Architecture (v1.6)

The Lumina runtime engine is designed for absolute correctness and deterministic reactivity. Version 1.6 completes the "Infrastructure Release," introducing structural entity relationships, multi-condition triggers, and a robust command-action cycle for the physical world.

## 1. Compiler Pipeline Overview

Lumina programs are processed through a strictly ordered pipeline. Every stage is designed for zero-allocation performance and strong safety guarantees.

### 2.1 Lexical Analysis (`lumina-lexer`)
Tokenization is performed via the `logos` crate. Version 1.6 adds specialized keywords for `ref`, `and`, `write`, `Timestamp`, and `.age`.

### 2.2 Syntax Analysis (`lumina-parser`)
The parser maps tokens into an Abstract Syntax Tree (AST):
*   **Recursive Descent**: Handles declarative constructs (`entity`, `rule`, `fn`).
*   **Pratt Parsing**: For expressions, managing operator precedence including specialized accessors like `.age`.

### 2.3 Semantic Analysis (`lumina-analyzer`)
Analysis is performed in two distinct passes:
1.  **Declaration Registration**: Records all entities, fields, and pure functions.
2.  **Structural Integrity & Typecheck**: Validates expressions and constructs a topological `DependencyGraph`. Version 1.6 introduces **Relationship Validation** to detect circular `ref` paths and ensures `write` actions only target external entities.

## 3. The Reactive Engine (`lumina-runtime`)

### 3.1 Structural Relationships (`ref`)
Lumina v1.6 introduces the `ref` keyword, transforming the flat instance map into a directed structural graph.
*   **Graph Traversal**: Evaluators can traverse entity relationships (e.g., `s.cooling.isFailing`) in a single pass.
*   **Reactive Propagation**: When a referenced instance updates, all dependent entities in the structural graph are automatically marked for re-evaluation.

### 3.2 Multi-Condition Triggers (`and`)
Compound truths are handled as single rising-edge events.
*   **Logical Conjunction**: Rules with `when A and B` triggers fire only when both conditions are simultaneously true.
*   **State Cache**: The engine maintains a transition cache for each condition, firing the rule only when the *last* required condition transitions to true.

### 3.3 Frequency Detection (`N times within`)
Temporal patterns are tracked via the `FrequencyTracker`:
*   **Sliding Windows**: The engine maintains a per-rule buffer of transition timestamps.
*   **Threshold Evaluation**: On every rule tick, the engine counts transitions within the sliding duration window to detect "flapping" or "chronic" conditions.

### 3.4 Temporal Engine & Stabilization
The v1.6 temporal engine is the most stable version yet.
*   **`Timestamp` Type**: Native support for temporal truth with the `.age` accessor.
*   **TimerHeap Synchronization**: A unified `TimerHeap` manages both `every` intervals and `for` duration stabilization.
*   **Fleet Stabilization**: Fleet triggers (`any`/`all`) now support full `for duration` stabilization, ensuring noisy sensor fluctuations do not trigger premature rule cascades.

### 3.5 Write-Back Cycle (`write`)
Lumina v1.6 closes the loop between observation and action.
*   **Command Dispatch**: The `write` action sends structured commands to external entity adapters.
*   **Adapter Contract**: Adapters implement the `on_write(field, value)` hook to translate Lumina intent into physical signals (MQTT publish, API call, etc).

### 3.6 State Snapshots & Safety
Lumina implements a **Self-Healing Guarantee**. Before any destructive action:
1.  The VM takes a complete memory **Snapshot**.
2.  Evaluation proceeds. If a recursion limit (100) or invariant is breached, the runtime **Automatically Rolls Back** to the snapshot.

---

## 4. Maintenance & IDE Integration

### 4.1 LSP v2 (`lumina-lsp`)
The v1.6 Language Server is production-grade, providing:
*   **Refactoring**: Global symbol renaming across multiple modules.
*   **Navigation**: "Find All References" to trace data flow through rules and derived fields.
*   **Quick Fixes**: Code actions for common analyzer errors (L001-L042).

### 4.2 Platform Support
*   **FFI**: Stable C ABI for integration with Python, Go, and C.
*   **WASM**: Optimized WebAssembly layer for browser-side simulation.
*   **CLI**: Event-driven output with real-time alert logging.

## 5. Technical Stack Considerations
*   **Rust**: Deterministic performance and memory safety.
*   **Logos**: DFA-based high-performance lexing.
*   **Pratt Parsing**: Precedence-climbing expression evaluation.
*   **Snapshot VM**: Atomic state transitions with guaranteed rollback.
