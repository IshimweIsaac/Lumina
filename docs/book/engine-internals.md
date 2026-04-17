# Chapter 3: Engine Internals ⚙️

To build truly robust systems in Lumina, it helps to understand what's happening "under the hood." This chapter dives deep into the compiler pipeline and the Snapshot-based Virtual Machine.

---

## 3.1 The Compiler Pipeline

Lumina processes your code in four distinct stages. Each stage is designed for maximum performance and rigorous safety.

### **1. Lexical Analysis (`lumina-lexer`)**
The lexer converts your source text into a stream of tokens (e.g., `ENTITY`, `IDENT("Reactor")`, `LBRACE`). 
*   **Technology**: We use the `logos` crate, which generates a highly optimized Finite State Automaton (DFA) at compile time.
*   **Performance**: Lexing is extremely fast, typically processing millions of lines per second.

### **2. Syntax Analysis (`lumina-parser`)**
The parser takes the token stream and builds an **Abstract Syntax Tree (AST)**.
*   **Top-Level**: Uses **Recursive Descent** to handle declarations like `entity`, `rule`, and `fn`.
*   **Expressions**: Uses **Pratt Parsing** (Precedence Climbing). This allows Lumina to handle complex mathematical and logical expressions with correct operator precedence (e.g., `a + b * c` is parsed as `a + (b * c)`).

### **3. Semantic Analysis (`lumina-analyzer`)**
This is where the "magic" happens. The analyzer performs two passes:
1.  **Symbol Registration**: It discovers all entities, fields, and functions.
2.  **Type Checking & DAG Building**: It validates that types match and, crucially, builds a **Dependency Graph**.

#### **Topological Sorting**
Lumina requires that derived fields form a **Directed Acyclic Graph (DAG)**.
*   If `a := b + 1`, then `b` must be calculated before `a`.
*   The analyzer performs a **Topological Sort** to find a valid execution order.
*   If it detects a cycle (e.g., `a := b` and `b := a`), it throws **L004: Circular Dependency** and prevents the program from running.

---

## 3.2 The Reactive Runtime (`lumina-runtime`)

The Lumina VM is built around the concept of an **Atomic Tick**.

### **The Evaluation Cycle**
Every time an input changes (a sensor update, a timer firing), the engine executes a tick:
1.  **Snapshot**: The VM takes a bit-level copy of the current state.
2.  **Propagation**: The engine iterates through the topologically sorted fields, updating each one.
3.  **Rule Evaluation**: Rules are checked against the new state.
4.  **Edge Detection**: The engine compares the new state to the snapshot to detect `becomes` transitions and `on clear` events.
5.  **Commit**: If everything succeeded, the snapshot is replaced by the new state.

### **Rollback Safety**
What happens if a rule causes an error? (e.g., a division by zero in an action).
Lumina uses its **Snapshot Stack** to perform a **zero-cost rollback**. The engine simply discards the in-progress state and reverts to the last known good snapshot. It then returns a structured **Diagnostic** explaining exactly what went wrong and how to fix it.

---

## 3.3 Temporal Logic & Stabilization

Lumina v1.8 introduced an advanced **TimerHeap** to handle time-based reactivity.
*   **`for duration`**: When you write `when temp > 100 for 5m`, Lumina doesn't just sleep. It schedules a "potential firing" in the TimerHeap. If the condition becomes false before 5 minutes pass, the timer is cancelled.
*   **`every`**: Periodic rules are managed by the same high-precision scheduler, ensuring they fire with microsecond accuracy regardless of system load.

---

## 3.4 Summary: Deterministic by Design
By combining a DAG-based compiler with a Snapshot-based VM, Lumina guarantees that:
1.  **State is always consistent**: You never see a partially-updated system.
2.  **Logic is deterministic**: The same inputs and history always produce the same outputs.
3.  **Failures are isolated**: A single runtime error cannot corrupt the entire system state.

In Chapter 4, we'll look at how we bridge this high-performance Rust core to the outside world via **WASM and FFI**.
