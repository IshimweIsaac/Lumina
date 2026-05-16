<!-- Source: introduction.md -->

# Chapter 1: The Lumina Philosophy 🛰️

_"Describe what is true. Lumina figures out what to do."_

At its core, Lumina is not just a programming language; it is a **Reactivity Engine** designed to synchronize high-level logic with the physical or digital state of a system. 

### 1.1 The Imperative Trap
Most software today is built using **imperative** patterns. You describe a sequence of steps to change state:
1. If the temperature is > 100...
2. And if the cooling system is off...
3. Then turn on the cooling system.
4. And send an alert.

This approach works for simple scripts but collapses under the weight of complex, real-time systems. Why? Because state changes are messy. A sensor might flicker, a network might delay an update, or multiple rules might try to change the same state simultaneously leading to **race conditions** and **stale data**.

### 1.2 The Lumina Way: Truth, Not Procedure
Lumina flips the script. Instead of telling the computer *how* to change state, you tell it *what relationships must always be true*.

In Lumina, you don't "set" a variable. You **declare a derived field**:
```lumina
entity Reactor {
  temp: Number
  isOverheating := temp > 100
}
```
Here, `isOverheating` isn't a flag you manually toggle. It is a **mathematical truth** derived from `temp`. Whenever `temp` changes whether by 1 unit or 100 Lumina's engine guarantees that `isOverheating` is updated **before** any logic that relies on it is executed.

### 1.3 Key Concepts

#### **The Reactive Graph**
Lumina treats your code as a **Directed Acyclic Graph (DAG)**. Every field and rule is a node. When an input (a "stored field") changes, Lumina performs a **Topological Sort** to determine the exact order in which dependent nodes must be updated. This ensures that you never read a "stale" value.

#### **Atomic Ticks**
The engine operates in discrete "ticks". During a tick, all external inputs are gathered, the entire system state is re-propagated, and only *after* everything is consistent are the changes "committed." If a calculation fails (e.g., division by zero), the engine performs a **zero-cost rollback**, ensuring the system never enters an invalid state.

#### **Historical Awareness**
Most languages only know the "now." Lumina natively understands the "then." By using the `prev()` operator, you can reason about transitions over time without manually caching old values:
```lumina
drift := temp - prev(temp)
```

### 1.4 Who is Lumina For?
Lumina is designed for systems where **correctness**, **high availability**, and **deterministic reactivity** are paramount:
*  **Target Infrastructure**: Managing the desired state of data centers, server fleets, and complex digital ecosystems.
*  **IoT & Edge Computing**: Synchronizing thousands of physical sensors with high-level logic in real-time.
*  **System Simulation**: Building high-performance, verifiable models of complex environments.

Welcome to Lumina. Let's stop writing procedures and start describing reality.


---

<!-- Source: zero-to-hero.md -->

# Chapter 2: The Zero-to-Hero Curriculum 🚀

Welcome to the definitive guide to becoming a Lumina expert. This curriculum builds from foundational concepts to advanced reactive systems through a series of hands-on tutorials.

---

## 2.1 The Foundations ( Concepts)

### **Lesson 1: Your First Entity**
In Lumina, everything starts with an `entity`. An entity is a template for state.
```lumina
entity SmartHome {
  name: String
  temp: Number
  targetTemp: Number
}
```
Open the **Playground**, paste this code, and click "Run". You can now create an instance of `SmartHome` and see its fields.

### **Lesson 2: Derived Fields (The Core)**
Derived fields use the `:=` operator. They are automatically calculated.
```lumina
entity SmartHome {
  temp: Number
  targetTemp: Number
  
  # This is a derived field!
  isCoolingRequired := temp > targetTemp
}
```
Try changing the `temp` in the Playground. Notice how `isCoolingRequired` updates instantly.

---

## 2.2 Adding Reactive Logic (v1.5/1.6)

### **Lesson 3: The `rule` of Law**
Rules allow you to take action when state changes. Rules use **triggers**.
```lumina
rule "Cooling On" when SmartHome.isCoolingRequired becomes true {
  alert severity: "info", message: "{SmartHome.name}: Turning on AC!"
}
```
The `becomes` keyword is crucial. It only fires when the condition *transitions* to true.

### **Lesson 4: Historical Context (`prev`)**
Lumina stores the previous state automatically. Use it to detect trends.
```lumina
entity SmartHome {
  temp: Number
  # Detect a rapid temperature spike!
  tempSpike := temp - prev(temp) > 5
}

rule "Emergency" when SmartHome.tempSpike becomes true {
  alert severity: "critical", message: "CRITICAL TEMP SPIKE DETECTED!"
}
```

---

## 2.3 Advanced Fleet Operations (+)

### **Lesson 5: Aggregates**
What if you have 1,000 smart homes? Use `aggregate` to summarize them.
```lumina
aggregate Neighborhood over SmartHome {
  avgTemp := avg(temp)
  homesInCooling := count(isCoolingRequired)
}
```
Lumina updates these aggregates incrementally (`O(1)` performance), so they are never out of sync.

### **Lesson 6: Fleet Triggers (`any` / `all`)**
Rules can monitor the entire fleet at once.
```lumina
rule "Grid Alert" when any SmartHome.isCoolingRequired becomes true {
  alert severity: "warning", message: "Grid load increasing!"
}
```

---

## 2.4 Mastering Experience Features

### **Lesson 7: Durable Logic (Stabilization)**
In the real world, sensors are noisy. Use `for duration` to stabilize your rules.
```lumina
rule "Stable Cooling" when SmartHome.temp > 25 for 10m {
  alert severity: "info", message: "Temp HAS been high for 10 minutes. Activating."
}
```

### **Lesson 8: The Diagnostic System**
V1.7 introduces "Teaching" error messages. Try creating a circular dependency:
```lumina
entity Loop {
  a := b + 1
  b := a + 1
}
```
The compiler (Analyzer) will now give a detailed, human-readable explanation:
> **L004: Circular Dependency Detected**
> I noticed that 'a' depends on 'b', which in turn depends back on 'a'. Lumina requires a Directed Acyclic Graph (DAG) for evaluation. To fix this, break the circuit...

---

## 2.5 Summary: Your Journey Has Begun
You have mastered:
1. Declaring state with **Entities** and **Stored Fields**.
2. Describing truth with **Derived Fields**.
3. Automating actions with **Rules** and **Triggers**.
4. Reasoning about time with **`prev()`** and **`for duration`**.
5. Summarizing fleets with **Aggregates**.

You are now ready to dive deep into Chapter 3: Engine Internals.


---

<!-- Source: engine-internals.md -->

# Chapter 3: Engine Internals ⚙️

To build truly robust systems in Lumina, it helps to understand what's happening "under the hood." This chapter dives deep into the compiler pipeline and the Snapshot-based Virtual Machine.

---

## 3.1 The Compiler Pipeline

Lumina processes your code in four distinct stages. Each stage is designed for maximum performance and rigorous safety.

### **1. Lexical Analysis (`lumina-lexer`)**
The lexer converts your source text into a stream of tokens (e.g., `ENTITY`, `IDENT("Reactor")`, `LBRACE`). 
*  **Technology**: We use the `logos` crate, which generates a highly optimized Finite State Automaton (DFA) at compile time.
*  **Performance**: Lexing is extremely fast, typically processing millions of lines per second.

### **2. Syntax Analysis (`lumina-parser`)**
The parser takes the token stream and builds an **Abstract Syntax Tree (AST)**.
*  **Top-Level**: Uses **Recursive Descent** to handle declarations like `entity`, `rule`, and `fn`.
*  **Expressions**: Uses **Pratt Parsing** (Precedence Climbing). This allows Lumina to handle complex mathematical and logical expressions with correct operator precedence (e.g., `a + b * c` is parsed as `a + (b * c)`).

### **3. Semantic Analysis (`lumina-analyzer`)**
This is where the "magic" happens. The analyzer performs two passes:
1. **Symbol Registration**: It discovers all entities, fields, and functions.
2. **Type Checking & DAG Building**: It validates that types match and, crucially, builds a **Dependency Graph**.

#### **Topological Sorting**
Lumina requires that derived fields form a **Directed Acyclic Graph (DAG)**.
*  If `a := b + 1`, then `b` must be calculated before `a`.
*  The analyzer performs a **Topological Sort** to find a valid execution order.
*  If it detects a cycle (e.g., `a := b` and `b := a`), it throws **L004: Circular Dependency** and prevents the program from running.

---

## 3.2 The Reactive Runtime (`lumina-runtime`)

The Lumina VM is built around the concept of an **Atomic Tick**.

### **The Evaluation Cycle**
Every time an input changes (a sensor update, a timer firing), the engine executes a tick:
1. **Snapshot**: The VM takes a bit-level copy of the current state.
2. **Propagation**: The engine iterates through the topologically sorted fields, updating each one.
3. **Rule Evaluation**: Rules are checked against the new state.
4. **Edge Detection**: The engine compares the new state to the snapshot to detect `becomes` transitions and `on clear` events.
5. **Commit**: If everything succeeded, the snapshot is replaced by the new state.

### **Rollback Safety**
What happens if a rule causes an error? (e.g., a division by zero in an action).
Lumina uses its **Snapshot Stack** to perform a **zero-cost rollback**. The engine simply discards the in-progress state and reverts to the last known good snapshot. It then returns a structured **Diagnostic** explaining exactly what went wrong and how to fix it.

---

## 3.3 Temporal Logic & Stabilization

Lumina introduced an advanced **TimerHeap** to handle time-based reactivity.
*  **`for duration`**: When you write `when temp > 100 for 5m`, Lumina doesn't just sleep. It schedules a "potential firing" in the TimerHeap. If the condition becomes false before 5 minutes pass, the timer is cancelled.
*  **`every`**: Periodic rules are managed by the same high-precision scheduler, ensuring they fire with microsecond accuracy regardless of system load.

---

## 3.4 Summary: Deterministic by Design
By combining a DAG-based compiler with a Snapshot-based VM, Lumina guarantees that:
1. **State is always consistent**: You never see a partially-updated system.
2. **Logic is deterministic**: The same inputs and history always produce the same outputs.
3. **Failures are isolated**: A single runtime error cannot corrupt the entire system state.

In Chapter 4, we'll look at how we bridge this high-performance Rust core to the outside world via **WASM and FFI**.


---

<!-- Source: wasm-and-ffi.md -->

# Chapter 4: WASM & Polyglot FFI 🌐

Lumina's core is written in Rust for performance and safety, but it is designed to be embedded anywhere. This chapter explains how Lumina communicates with the outside world.

---

## 4.1 WebAssembly: Lumina in the Browser

The Lumina Playground runs entirely on the client side thanks to WebAssembly (WASM).

### **The WASM Bridge (`lumina-wasm`)**
Using `wasm-bindgen`, we export the `LuminaRuntime` struct to JavaScript.
*  **Initialization**: When you load a script in the Playground, `new LuminaRuntime(source)` is called. This performs parsing, analysis, and initial state setup in microseconds.
*  **The Tick Loop**: The Playground uses `requestAnimationFrame` to call `runtime.tick()`. This keeps the reactive engine in sync with the browser's refresh rate.
*  **Event Injection**: When you toggle a button in the Playground UI, it calls `runtime.apply_event(instance, field, value)`. This triggers a new atomic tick within the WASM module.

### **Performance Benefits**
Because Lumina compiles to optimized WASM, it can handle thousands of reactive entities with sub-millisecond propagation latency, even on mobile devices.

---

## 4.2 Polyglot FFI: The Stable C ABI

For server-side and embedded applications, Lumina provides a **Stable C ABI** (`lumina_ffi`). This allows any language that can call C functions to interact with the Lumina engine.

### **The Memory Contract**
FFI requires careful memory management. Lumina follows a strict **Caller-Owns** policy for returned data:
*  **Allocation**: Lumina allocates strings on the Rust heap and returns a raw pointer (`*mut c_char`).
*  **Deallocation**: You *must* pass this pointer back to `lumina_free_string(ptr)` to prevent memory leaks. **Do not use the standard C `free()`**, as it may use a different allocator than Rust.

### **Key API Functions**
*  `lumina_create(source)`: Initializes a new runtime from Lumina source code.
*  `lumina_apply_event(...)`: Injects external data into the engine.
*  `lumina_tick(runtime)`: Advances the engine by one tick.
*  `lumina_export_state(runtime)`: Returns the current system state as a JSON string.

---

## 4.3 Official Wrappers

Building on the C ABI, we provide high-level, "idiomatic" wrappers for popular languages:

### **Python (`lumina-python`)**
Uses `ctypes` to wrap the shared library (`.so` / `.dll`).
```python
from lumina import Runtime

rt = Runtime.from_file("logic.lum")
rt.apply_event("Machine", "temp", 42.5)
state = rt.export_state()
print(state["Machine"]["isHot"])
```

### **Go (`lumina-go`)**
Uses `cgo` to provide a high-performance, type-safe interface.
```go
import "github.com/lumina-lang/lumina-go"

rt, _ := lumina.NewRuntime(source)
rt.ApplyEvent("Machine", "temp", 42.5)
```

---

## 4.4 Summary: Engine as a Library
Lumina is not a monolith; it is a library. Whether you are building a React-based monitoring dashboard (WASM) or a high-performance IoT gateway (FFI/Go), the same deterministic engine powers your logic.

In Chapter 5, we'll look at the **Diagnostic System** and how it helps you debug complex reactive failures.


---

<!-- Source: diagnostics-reference.md -->

# Chapter 5: Diagnostics & Error Reference 🚨

Lumina introduces a "Teaching" diagnostic system. Instead of cryptic codes, the engine provides human-readable explanations and suggested fixes for every failure.

---

## 5.1 The Physiology of an Error

Every Lumina error contains:
1. **Code**: A unique identifier (e.g., `L004`, `R002`) for looking up in this reference.
2. **Message**: A concise explanation of what went wrong.
3. **Location**: The exact file, line, and column where the error was detected.
4. **Help**: A suggested fix or educational tip (the "Teaching" component).

---

## 5.2 Analyzer Errors (L-Codes)

These are detected during the **Analysis Phase**, before your code ever runs.

| Code | Name | Description | Suggested Fix |
| :--- | :--- | :--- | :--- |
| **L001** | Unknown Identifier | You're referencing a variable or entity that hasn't been declared. | Check for typos or ensure the entity is imported. |
| **L002** | Type Mismatch | You're trying to perform an operation on incompatible types (e.g., `Number + Text`). | Ensure both operands are of the same expected type. |
| **L004** | Circular Dependency | A derived field depends on itself through a chain of other fields. | Break the cycle by using a stored field or a different logic path. |
| **L011** | Duplicate Function | You've declared two functions with the same name. | Rename one of the functions to be unique. |
| **L015** | Purity Violation | A pure function (`fn`) is trying to access entity state. | Pass the required state as an argument to the function instead. |
| **L035** | Trigger Limit | A `when` trigger has more than 3 `and` conditions. | Split the logic into multiple rules or use a derived Boolean field. |
| **L041** | Impure Derived Field | You're using `now()` inside a derived field (`:=`). | Derived fields must be deterministic. Use a rule to update a stored field with `now()` instead. |
| **L042** | Invalid Accessor | Using something other than `.age` on a Timestamp. | Timestamps only support `.age` which returns a Duration. |

---

## 5.3 Runtime Errors (R-Codes)

These occur while the engine is running and usually trigger a **State Rollback**.

| Code | Name | Description | suggested Fix |
| :--- | :--- | :--- | :--- |
| **R001** | Null Access | Attempting to access a field on an instance that was deleted. | Add a check to ensure the instance exists before accessing it. |
| **R002** | Math Error | Division by zero or invalid numeric operation. | Guard your division with an `if divisor != 0` check. |
| **R003** | Recursion Limit | Rules are triggering each other in an infinite loop. | Ensure rules have mutually exclusive conditions or use a `cooldown`. |
| **R004** | Out of Bounds | Accessing a list index that doesn't exist. | Check the `len(list)` before performing indexing. |
| **R006** | Range Violation | A value assigned to a field violates its `@range` metadata. | Validate the input value before calling `update`. |
| **R009** | Read-Only Violation | Attempting to `update` a derived field (`:=`). | Derived fields are read-only. Update the source stored fields instead. |

---

## 5.4 Parser & Lexer Errors

These occur when your code doesn't follow Lumina's grammar.

*  **Unexpected Token**: "I expected to see keyword 'then' but found an identifier."
*  **Unclosed Brace**: "You started an entity block on line 10 but never closed it."
*  **Invalid Character**: "I don't recognize the character '@' in this context."

---

## 5.5 Summary: Built-in Mentorship
Lumina's diagnostics are designed to make you a better reactive programmer. If you encounter an error not listed here, or if the "Help" message is unclear, please report it on our GitHub.


---

