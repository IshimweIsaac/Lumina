# Lumina System Architecture (v1.3)

The Lumina runtime engine is designed to ensure absolute correctness and reliability. It combines a closed evaluation reactive model with an intentional Runtime I/O boundary.

## 1. Compiler Pipeline Overview

Lumina features a multi-stage pipeline, transforming declarative syntax into a reactive execution graph:

### 2.1 Lexical Analysis (`lumina-lexer`)
Powered by the `logos` crate, this phase generates a high-performance Deterministic Finite Automaton (DFA) that tokenizes incoming UTF-8 streams natively at GB/s throughputs without allocations.

### 2.2 Syntax Analysis (`lumina-parser`)
The parser maps the token stream into an AST utilizing:
*   **Recursive Descent**: For declarative blocks (Entities, Rules).
*   **Pratt Parsing**: For expression evaluations, dynamically sorting variable binding powers (like `and`/`or`/`==`) without stack limitations.

### 2.3 Semantic Analysis (`lumina-analyzer`)
Calculates cyclic checks to avoid infinite reactivity loops.
*   Constructs a flat integer DAG via Kahn's algorithm mapping every Entity and Field.
*   Enforces type safety completely statically so `Boolean` cannot be assigned to `Number`.

## 3. The Reactive Engine (`lumina-runtime`)

### 3.1 Closed Evaluation Model
Lumina is closed by default. It executes in deterministic dependency topological order.
*   Updates on stored fields trigger a DAG traversal marking nodes dirty via BitVecs.
*   Reactively recomputes any derived `:=` field in O(1) topological resolution.
*   Evaluates all `rule` transition boundaries (`becomes true`), scheduling actions sequentially.

### 3.2 State Snapshots & Self-Healing
A catastrophic logic flow in dynamic systems cannot cause a crash. Lumina implements a **Self-Healing Guarantee**:
*   Before any state-changing manipulation (`update`, `create`, `delete`), the VM caches a complete **Snapshot**.
*   If a rule cascades infinitely or an `@range` bound is violently breached, the runtime **Automatically Rolls Back** to the snapshot.
*   It emits structured diagnostic logs pointing to the exact rule and constraint, quarantining logic if needed.

## 4. Host Communications & The I/O Boundary

To keep the runtime pure, it explicitly narrows communication to the physical host via exactly two methods:
1. `export_state()`: Serializes the totality of the entity contexts into a JSON payload.
2. `apply_event(json)`: Pushes foreign inputs to the AST safely, evaluating all rule cascades, and returning the updated `export_state()`.

## 4. Technical Stack Considerations (Why Rust?)
Rust was deliberately formalized for the runtime due to:
*   **Borrow Checker**: The `evaluator` operates unthreaded in cache-tight memory limits, ensuring safe memory access without undefined memory races.
*   **WASM & FFI Targets**: Single-binary drop-ins into Javascript (V8 via `wasm-bindgen`) or Python deployments via `cbindgen` exposing C ABI arrays.
