# Lumina Error Encyclopedia

Every error code in Lumina with the BAD code that causes it, what the error message says, and the FIXED code.

---

## Analyzer Errors (L-Codes) — Caught Before Execution

These errors are detected during the analysis phase. Your program will NOT run until they are fixed.

---

### L001: Unknown Identifier

**What happened**: You used a name that hasn't been declared.

```lumina
-- ❌ BAD:
entity Server {
    status := unknown_field + 1    -- L001: I don't recognize 'unknown_field'
}
```

```lumina
-- ✅ FIX: Declare the field first
entity Server {
    value: Number
    status := value + 1
}
```

---

### L002: Type Mismatch

**What happened**: Types don't match where they should.

```lumina
-- ❌ BAD: when condition must be Boolean (without becomes)
rule "Bad"
when Server.cpu_temp {       -- L002: must be Boolean
    show "?"
}
```

```lumina
-- ✅ FIX: Add a comparison
rule "Good"
when Server.cpu_temp > 80 {
    show "Hot!"
}
```

Also triggered when `becomes` target type doesn't match the expression type.

---

### L004: Circular Dependency (DAG Cycle)

**What happened**: Derived fields depend on each other in a cycle.

```lumina
-- ❌ BAD:
entity Loop {
    a := b + 1
    b := a + 1    -- L004: Circular dependency: a → b → a
}
```

```lumina
-- ✅ FIX: Break the cycle with a stored field
entity Loop {
    a: Number              -- stored (manual update only)
    b := a + 1             -- derived (auto-computed from a)
}

rule "Update A"
every 1s {
    update Loop.a = Loop.b
}
```

---

### L005: Duplicate Entity Name

**What happened**: Two entities share the same name.

```lumina
-- ❌ BAD:
entity Server { cpu: Number }
entity Server { ram: Number }    -- L005: Duplicate entity name: Server
```

```lumina
-- ✅ FIX: Use unique names
entity CpuServer { cpu: Number }
entity RamServer { ram: Number }
```

---

### L006: Duplicate Field Name

**What happened**: Two fields in the same entity share a name.

```lumina
-- ❌ BAD:
entity Server {
    temp: Number
    temp: Text    -- L006: Duplicate field name: temp
}
```

---

### L011: Duplicate Function Name

```lumina
-- ❌ BAD:
fn calc(x: Number) -> Number { x + 1 }
fn calc(x: Number) -> Number { x * 2 }    -- L011: Duplicate function 'calc()'
```

---

### L014: Function Return Type Mismatch

```lumina
-- ❌ BAD:
fn get_name(x: Number) -> Text {
    x + 1    -- L014: body returns Number, signature says Text
}
```

```lumina
-- ✅ FIX: Match the return type
fn get_name(x: Number) -> Number {
    x + 1
}
```

---

### L015: Function Accessing Entity Fields

```lumina
-- ❌ BAD:
fn bad_func() -> Number {
    Server.cpu_temp    -- L015: fn body cannot access entity fields
}
```

```lumina
-- ✅ FIX: Pass data as a parameter
fn good_func(temp: Number) -> Number {
    temp * 2
}
```

---

### L024: prev() on Derived Field

```lumina
-- ❌ BAD:
entity Server {
    cpu: Number
    is_hot := cpu > 80
    was_hot := prev(is_hot)    -- L024: prev() on derived field
}
```

```lumina
-- ✅ FIX: Use prev() on stored fields only
entity Server {
    cpu: Number
    is_hot := cpu > 80
    was_cooler := prev(cpu) <= 80
}
```

---

### L025: Nested prev()

```lumina
-- ❌ BAD:
entity Server {
    cpu: Number
    change := prev(prev(cpu))    -- L025: Nested prev() call
}
```

---

### L026: Unknown Entity in Rule Parameter

```lumina
-- ❌ BAD:
rule "Check" for (x: UnknownEntity)    -- L026: Unknown entity 'UnknownEntity'
when x.value > 0 {
    show "?"
}
```

---

### L027: Fleet Trigger on Non-Boolean Field

```lumina
-- ❌ BAD:
rule "Bad Fleet"
when any Server.cpu_temp becomes 80 {    -- L027: field must be Boolean
    show "?"
}
```

```lumina
-- ✅ FIX: Use a Boolean derived field
entity Server {
    cpu_temp: Number
    is_hot := cpu_temp > 80
}

rule "Good Fleet"
when any Server.is_hot becomes true {
    show "At least one server is hot"
}
```

---

### L035: Too Many Rule Conditions

```lumina
-- ❌ BAD: Max 3 conditions
rule "Overloaded"
when a > 1 and b > 2 and c > 3 and d > 4 {    -- L035: max 3 clauses
    show "?"
}
```

```lumina
-- ✅ FIX: Combine into derived fields
entity System {
    a: Number
    b: Number
    c: Number
    d: Number
    all_bad := a > 1 and b > 2 and c > 3 and d > 4
}

rule "Combined"
when System.all_bad becomes true {
    show "All conditions met"
}
```

---

### L036: Unknown Entity in ref

```lumina
-- ❌ BAD:
entity Server {
    cooling: ref NonExistentEntity    -- L036: Unknown entity in ref
}
```

---

### L038: write on Non-External Entity

```lumina
-- ❌ BAD:
entity Server { cpu: Number }
let s = Server { cpu: 50 }
write s.cpu = 90    -- L038: write used on non-external entity
```

```lumina
-- ✅ FIX: Use update for regular entities
update s.cpu = 90
```

---

### L039: Frequency Count Too Low

```lumina
-- ❌ BAD:
rule "Bad"
when x > 0 frequency 1 times within 5s {    -- L039: must be >= 2
    show "?"
}
```

---

### L041: now() in Derived Field

```lumina
-- ❌ BAD:
entity Timer {
    start: Timestamp
    elapsed := now() - start    -- L041: now() in derived field
}
```

```lumina
-- ✅ FIX: Compute in a rule
entity Timer {
    start: Timestamp
    elapsed_secs: Number
}

rule "Tick"
every 1s {
    update timer.elapsed_secs = timer.elapsed_secs + 1
}
```

---

### L051: Secret in Derived Field

```lumina
-- ❌ BAD:
entity Config {
    key := env("API_KEY")    -- L051: derived field resolves to Secret
}
```

```lumina
-- ✅ FIX: Store the secret in a stored field
entity Config {
    key: Secret
}
let c = Config { key: env("API_KEY") }
```

---

## Runtime Errors (R-Codes) — Caught During Execution

These errors occur while the program is running. They trigger **atomic rollback** — the state reverts to before the failed operation.

---

### R001: Instance Not Found

```
Access to deleted instance: 'server1'
```

**Cause**: Referencing an instance that doesn't exist or was deleted.

---

### R002: Division by Zero

```
Division by zero
```

```lumina
-- ❌ BAD:
let result = 100 / 0
```

```lumina
-- ✅ FIX: Guard against zero
let result = if divisor != 0 then 100 / divisor else 0
```

---

### R003: Recursion Limit (MAX_DEPTH = 100)

```
Circular Dependency Detected: Rule re-entrancy limit exceeded (100)
```

**Cause**: A rule's action re-triggers the same rule (or a chain of rules that loops back).

```lumina
-- ❌ BAD: Self-triggering rule
rule "Infinite"
when Node.value becomes 100 {
    update Node.value = 100    -- re-triggers itself!
}
```

**Result**: Full rollback. The state returns to before the update.

---

### R004: List Index Out of Bounds

```
List index out of bounds: 5 of 3
```

```lumina
-- ❌ BAD:
let items = [10, 20, 30]
let bad = at(items, 5)    -- R004: index 5, length 3
```

Also triggered by `head()`, `tail()`, `min()`, `max()` on empty lists.

---

### R005: Null Field Access

```
Null field access: 'server1.nonexistent'
```

---

### R006: Range Violation

```
@range violation: cpu_temp = 200, expected 0–150
```

```lumina
-- ❌ BAD:
entity Server {
    @range 0 to 150 cpu_temp: Number
}
let s = Server { cpu_temp: 50 }
update s.cpu_temp = 200    -- R006: 200 outside 0-150
```

**Result**: Full rollback. cpu_temp stays at 50.

---

### R009: Cannot Update Derived Field

```
Cannot update derived field 'is_hot' — it is computed automatically
```

```lumina
-- ❌ BAD:
entity Server {
    cpu_temp: Number
    is_hot := cpu_temp > 80
}
let s = Server { cpu_temp: 50 }
update s.is_hot = true    -- R009
```

```lumina
-- ✅ FIX: Update the stored field that drives it
update s.cpu_temp = 90    -- is_hot becomes true automatically
```

---

### R018: Type Mismatch in Operation

```
Type mismatch: cannot apply '+' to Number and Boolean
```

```lumina
-- ❌ BAD:
let result = 42 + true    -- R018
```

---

### R019: Snapshot Stack Corruption

```
Internal error: snapshot stack corrupted during rollback. This is a bug.
```

This is an engine bug. If you encounter it, report it.
