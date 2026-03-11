# Contributing to Lumina

Welcome to the Lumina Systems team! Lumina is built for rigorous safety, deterministic reactions, and temporal stability. Because it is used as a backend reactive engine, we employ strict developmental methodologies that you must adhere to.

This document serves as the **deep technical guide** for contributing to the Lumina v1.3 codebase. It is based directly on the authoritative `lumina-1.3.md` language specification.

---

## 1. Local Workstation Setup

If contributing to the production engine, ensure you have:
* The Rust Toolchain (`>= 1.70.0`)
* The Cargo dependency manager.
* `wasm-pack` (for WASM target compilation)
* `python3` (for testing the C FFI)

---

## 2. Architecture & Key Patterns

Every `.lum` program flows through a strict 4-stage pipeline. Each stage is a separate crate with a clean API boundary:

1. **`lumina-lexer`**: Converts source `&str` to `Vec<SpannedToken>` using the high-performance `logos` crate.
2. **`lumina-parser`**: Maps tokens to a typed Abstract Syntax Tree (AST).
3. **`lumina-analyzer`**: Performs semantic validation, constructs the dependency graph, and strictly enforces static typing.
4. **`lumina-runtime`**: The core Snapshot VM (`Evaluator`) that evaluates rules and commands.

### 2.1 The NodeId Pattern (Kahn's Algorithm)
Lumina guarantees acyclic evaluation using topological sorting. 
* The dependency graph uses flat `u32` indices (`NodeId`) instead of pointers or String keys.
* Each `(entity_name, field_name)` pair is interned to a `NodeId`.
* **Never** change this to use `String` keys or `HashMap<String, Node>` in the reactive loop—it breaks the topo sort efficiency and O(1) dirty marking capability.

### 2.2 Re-entrancy Guard and Cascades
Rules can trigger updates which trigger other rules. A re-entrancy guard ensures safe rule cascading:
* The `Evaluator` tracks recursive `apply_update` calls via a `depth` counter.
* If `depth > MAX_DEPTH` (100), the runtime returns `R003` and rolls back to safety.
* A rule can only fire once per propagation cycle to prevent oscillation.

### 2.3 `becomes` Detection
The `becomes` keyword detects *transitions*, not just current state.
* The `store` maintains both `prev_fields` and `fields`.
* `becomes` is only `true` when the *current* resolves to the target AND the *previous* did not.

---

## 3. What Never to Break (Core Invariants)

These invariants keep the runtime mathematically sound. If violated, they silently corrupt state correctness.

### ⚠️ DO NOT BREAK: `commit_all()` Timing
`store.commit_all()` MUST only be called at the outermost `apply_update` (`depth == 1`). It syncs `prev_fields` with `fields`. If called mid-propagation during a nested update, `becomes` edge-transition detection is fundamentally broken.

### ⚠️ DO NOT BREAK: Snapshot Before Every Mutation
Lumina guarantees self-healing. Every function that modifies `EntityStore` must take a snapshot *before* mutating:
* Normal updates (`apply_update`)
* Timer expirations (`tick()` for-timer firing and every-timer firing)
If you add a new mutating operation, it must follow the snapshot/restore pattern.

### ⚠️ DO NOT BREAK: Topo Order is Read-Only
`graph.topo_order` is computed once by Kahn's algorithm in the Analyzer. It must never be mutated during runtime execution.

### ⚠️ DO NOT BREAK: `lumina_free_string`
All strings returned by C FFI functions are owned by Rust. They *must* be freed with `lumina_free_string()`, not the system `free()`. Double-frees cause undefined behavior.

---

## 4. How to Extend the Runtime

Before adding new functionality, follow these strict pipelines:

### 4.1 Adding a New Keyword
1. Add the token variant to `Token` enum in `lumina-lexer/src/token.rs`.
2. Add the `logos` pattern to the lexer in `lumina-lexer/src/lib.rs`.
3. Add an AST node to `lumina-parser/src/ast.rs`.
4. Add parsing logic in `lumina-parser/src/parser.rs`.
5. Add type-checking in `lumina-analyzer/src/analyzer.rs`.
6. Add evaluation in `lumina-runtime/src/engine.rs`.
7. Write at least 2 tests (parsing and analyzer verification).

### 4.2 Adding a New Action
1. Add a variant to the `Action` enum in `lumina-parser/src/ast.rs`.
2. Map it in `parse_action()` in the parser.
3. Validate it in `check_action()` in the analyzer.
4. Execute it in `exec_action()` in `engine.rs`.
   * `exec_action` MUST return `Result<Vec<FiredEvent>, RuntimeError>`.
   * **Crucial:** If the action mutates state, take a snapshot before, and restore on error.

### 4.3 Adding New Error Codes
* **Compile-time** errors use **L-codes** (`L011`, `L012`, etc.) in `lumina-analyzer`. Pass spans accurately.
* **Runtime** errors use **R-codes** (`R010`, `R011`, etc.) in `lumina-runtime`. They must be added to the `RuntimeError` enum and explicitly mapped in `Diagnostic::from_runtime_error()`.

### 4.4 Adding a New FFI Function
Follow the strict `unsafe` memory bounds:
1. Null-check all pointer arguments.
2. Convert C strings to Rust strings safely using `CStr`.
3. Execute inner `Evaluator` logics.
4. Return as `CString::into_raw()`.
5. Document clearly that the caller *must* invoke `lumina_free_string`.
6. Add bindings to `crates/lumina-ffi/lumina_py.py` and test in `test_ffi.py`.

---

## 5. Regression Test Checklist

Run this full suite before *any* pull request. All must pass:

1. **Full workspace:** `cargo test --workspace`
2. **CLI E2E:** `cargo build --release` && `cargo run --bin lumina -- run tests/spec/fleet.lum`
3. **Python FFI:** `cargo build --release -p lumina-ffi` && `cd crates/lumina-ffi && python test_ffi.py`
4. **WASM target:** `cd crates/lumina-wasm && wasm-pack build --target web --out-dir pkg --release`

## Welcome to the Engine!
By following these strict constraints, you help maintain Lumina as a fault-tolerant, mathematically sound engine. Thank you for contributing!
