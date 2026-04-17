# Chapter 4: WASM & Polyglot FFI 🌐

Lumina's core is written in Rust for performance and safety, but it is designed to be embedded anywhere. This chapter explains how Lumina communicates with the outside world.

---

## 4.1 WebAssembly: Lumina in the Browser

The Lumina Playground runs entirely on the client side thanks to WebAssembly (WASM).

### **The WASM Bridge (`lumina-wasm`)**
Using `wasm-bindgen`, we export the `LuminaRuntime` struct to JavaScript.
*   **Initialization**: When you load a script in the Playground, `new LuminaRuntime(source)` is called. This performs parsing, analysis, and initial state setup in microseconds.
*   **The Tick Loop**: The Playground uses `requestAnimationFrame` to call `runtime.tick()`. This keeps the reactive engine in sync with the browser's refresh rate.
*   **Event Injection**: When you toggle a button in the Playground UI, it calls `runtime.apply_event(instance, field, value)`. This triggers a new atomic tick within the WASM module.

### **Performance Benefits**
Because Lumina compiles to optimized WASM, it can handle thousands of reactive entities with sub-millisecond propagation latency, even on mobile devices.

---

## 4.2 Polyglot FFI: The Stable C ABI

For server-side and embedded applications, Lumina provides a **Stable C ABI** (`lumina_ffi`). This allows any language that can call C functions to interact with the Lumina engine.

### **The Memory Contract**
FFI requires careful memory management. Lumina follows a strict **Caller-Owns** policy for returned data:
*   **Allocation**: Lumina allocates strings on the Rust heap and returns a raw pointer (`*mut c_char`).
*   **Deallocation**: You *must* pass this pointer back to `lumina_free_string(ptr)` to prevent memory leaks. **Do not use the standard C `free()`**, as it may use a different allocator than Rust.

### **Key API Functions**
*   `lumina_create(source)`: Initializes a new runtime from Lumina source code.
*   `lumina_apply_event(...)`: Injects external data into the engine.
*   `lumina_tick(runtime)`: Advances the engine by one tick.
*   `lumina_export_state(runtime)`: Returns the current system state as a JSON string.

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
