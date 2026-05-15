# Lumina Knowledge Base — Index

**Version: 2.0.0-GOLD | Status: AI Training Reference**

This directory is the **single source of truth** for writing, debugging, and architecting Lumina programs. It is designed to be read by AI assistants that do NOT have access to the Lumina Rust codebase.

---

## Quick Reference: Which File Do I Need?

| If you need to...                           | Read this file                                                |
|---------------------------------------------|---------------------------------------------------------------|
| Understand what Lumina IS and how to think   | [01-mental-model.md](01-mental-model.md)                      |
| Look up syntax for ANY construct             | [02-syntax-reference.md](02-syntax-reference.md)              |
| Understand types, operators, coercion        | [03-type-system.md](03-type-system.md)                        |
| Find a built-in function signature           | [04-built-in-functions.md](04-built-in-functions.md)          |
| Write or debug a reactive rule               | [05-rules-and-triggers.md](05-rules-and-triggers.md)          |
| See complete, runnable example programs      | [06-patterns-cookbook.md](06-patterns-cookbook.md)              |
| Diagnose an error code (L-codes / R-codes)   | [07-error-encyclopedia.md](07-error-encyclopedia.md)          |
| Use clusters, FFI, external entities, secrets| [08-advanced-features.md](08-advanced-features.md)            |
| Check if something is supported or broken    | [09-known-limitations.md](09-known-limitations.md)            |
| Architect a full project from scratch        | [10-project-templates.md](10-project-templates.md)            |
| Install, update, or use the CLI              | [11-cli-reference.md](11-cli-reference.md)                    |

---

## About Lumina (One Paragraph)

Lumina is a **Distributed Reactive Language (DRL)** for agentless infrastructure orchestration. Instead of writing procedural scripts that describe *how* to reach a state, you write declarative rules that describe *what is true* about your system. The Lumina engine continuously maintains a **Directed Acyclic Graph (DAG)** of state, reacting atomically to changes. If a sensor value changes, every derived field, rule, and aggregate that depends on it is recomputed in a single atomic tick — or rolled back entirely if anything fails.

## File Format

Lumina source files use the `.lum` extension. Comments start with `--`.

## CLI Quick Start

```bash
# Install Lumina
curl -fsSL https://lumina-lang.web.app/install.sh | sh && . ~/.lumina/env

# Run a file
lumina run myfile.lum

# Type-check without running
lumina check myfile.lum

# Format source code
lumina fmt myfile.lum

# Start the interactive REPL
lumina repl

# Update to the latest version
lumina update

# Check if an update is available
lumina update --check
```

See [11-cli-reference.md](11-cli-reference.md) for the full command reference.


---

# Lumina Mental Model: Think in Truth, Not in Steps

## The Core Idea

Most programming languages are **step-by-step instructions**. To monitor a server temperature:

```python
# Traditional (procedural)
while True:
    temp = read_sensor()
    if temp > 80:
        start_timer()
        if timer_elapsed(5, "minutes"):
            send_alert("Server overheating!")
    else:
        cancel_timer()
    sleep(1)
```

In Lumina, you skip the steps and **describe the truth**:

```lumina
entity Server {
    cpu_temp: Number
    is_overheating := cpu_temp > 80
}

rule "Thermal Alert"
when Server.is_overheating becomes true for 5m {
    alert severity: "critical", message: "Server overheating!"
}
```

Lumina handles the reading, comparison, timing, state cleanup, and recovery **automatically**. This is **Declarative Reactivity**.

---

## The 4-Tier Layered Architecture

Every Lumina program is built from four layers, each with different properties:

### Tier 1: Stored Fields — The Raw Facts

Stored fields are the foundation. They hold values that only change when:
- An external event pushes new data (sensors, APIs)
- A rule explicitly `update`s them

```lumina
entity Thermometer {
    current_temp: Number       -- only changes via update or external push
    location: Text             -- static unless explicitly changed
    threshold: Number          -- configurable
}
```

### Tier 2: Derived Fields — The Living Logic

Derived fields compute themselves instantly whenever their dependencies change. They use the `:=` operator. They are ALWAYS consistent with their inputs — you never need to worry about stale data.

```lumina
entity Thermometer {
    current_temp: Number
    threshold: Number
    is_overheating := current_temp > threshold   -- auto-computed
    status := if is_overheating then "CRITICAL" else "OK"
}
```

**Key rule**: Derived fields form a DAG (Directed Acyclic Graph). If `A := B + 1` and `B := A + 1`, the compiler catches this circular dependency (error L004) before any code runs.

### Tier 3: Rules — The Reactive Actions

Rules watch for **moments of transition** in your state. They fire when conditions change, NOT continuously.

```lumina
rule "Safety Trip"
when Thermometer.is_overheating becomes true {
    alert severity: "critical", message: "Emergency cooling required!"
}
on clear {
    show "Temperature stabilized."
}
```

The `becomes true` is critical: the rule fires **once** when the value transitions from false to true, not every time the engine evaluates it while it's true.

### Tier 4: The Cluster Mesh — The Network (v2.0)

State isn't confined to a single node. Lumina supports multi-node clusters with native UDP gossip, leader election, and workload migration.

```lumina
rule "Failover"
when MainServer.is_unhealthy becomes true {
    migrate { workloads: "critical_db", target: "backup_node" }
}
```

---

## The Atomic Tick: How State Changes

Every state change in Lumina happens in an **Atomic Tick**:

1. **Snapshot**: The engine saves a copy of the current state
2. **Mutation**: The update is applied to the stored field
3. **Propagation**: Every derived field is recomputed in topological order (DAG order)
4. **Rule Evaluation**: Rules check if conditions have transitioned
5. **Validation**: `@range` constraints and invariants are checked
6. **Commit or Rollback**: If everything succeeds, the state is committed. If ANY rule fails or an invariant is breached, the engine **rolls back to the snapshot** — the system is never "half-updated"

This means: **Lumina state is either fully consistent or unchanged.** There is no corrupt intermediate state.

---

## Key Mental Shifts

| Procedural Thinking | Lumina Thinking |
|---------------------|-----------------|
| "Run this function when X happens" | "This field IS true when X" |
| "Poll every 5 seconds and check" | "React when the value transitions" |
| "Update variable A, then B, then C" | "A depends on B depends on C — the DAG handles ordering" |
| "Try/catch and hope state is ok" | "Atomic tick: all-or-nothing commit" |
| "Write a loop to aggregate" | "Declare an aggregate — engine computes incrementally" |

---

## What Lumina is NOT

- **Not a general-purpose language**: No loops, no classes, no file I/O. It's a reactive state machine.
- **Not a scripting language**: You don't write procedures. You declare truth.
- **Not eventually consistent**: State changes are atomic. Derived fields are always up-to-date after a tick.
- **Not a database**: Entities are in-memory reactive objects, not persisted rows.


---

# Lumina Syntax Reference (v2.0-GOLD)

This is the exhaustive syntax reference for every Lumina construct. Every example here is valid, runnable code.

---

## Comments

```lumina
-- This is a single-line comment
-- Lumina only has single-line comments using --
```

---

## Entity Declaration

Entities are the core building blocks. They define a type with fields.

### Basic Entity (Stored Fields Only)

```lumina
entity Server {
    cpu_temp: Number
    hostname: Text
    is_online: Boolean
}
```

### Entity with Derived Fields

Derived fields use `:=` and auto-compute whenever their dependencies change.

```lumina
entity Server {
    cpu_temp: Number
    threshold: Number
    is_overheating := cpu_temp > threshold
    status := if is_overheating then "CRITICAL" else "OK"
}
```

### Entity with Metadata

```lumina
@doc "A physical server in the datacenter"
entity Server {
    @range 0 to 150 cpu_temp: Number
    @doc "Hostname identifier" hostname: Text
    is_online: Boolean
}
```

- `@doc "description"` — Documentation annotation
- `@range min to max` — Runtime constraint. Violating it causes R006 rollback.

### Entity with References (ref)

References link entities. The referenced entity MUST be declared before the referencing one.

```lumina
entity CoolingUnit {
    name: Text
    is_active: Boolean
    power_watts: Number
    is_failing := power_watts > 50 and not is_active
}

entity Server {
    name: Text
    cpu_temp: Number
    cooling: ref CoolingUnit
    needs_emergency := cpu_temp > 90 and cooling.is_failing
}
```

Access referenced fields with dot notation: `cooling.is_failing`, `cooling.name`.

---

## Field Types

| Type        | Syntax         | Examples                    |
|-------------|----------------|-----------------------------|
| Number      | `Number`       | `42`, `3.14`, `-7`          |
| Text        | `Text`         | `"hello"`, `"temp: {val}"`  |
| Boolean     | `Boolean`      | `true`, `false`             |
| Timestamp   | `Timestamp`    | Created by `now()`          |
| Duration    | (via literals) | `5s`, `10m`, `2h`, `1d`     |
| Secret      | `Secret`       | Created by `env("KEY")`     |
| List        | (via literals) | `[1, 2, 3]`                 |

---

## Instance Creation (let)

### Creating Entity Instances

```lumina
let server1 = Server {
    cpu_temp: 45,
    hostname: "web-01",
    is_online: true
}
```

All stored fields must be provided. Derived fields are computed automatically.

### Creating Simple Values

```lumina
let my_list = [10, 20, 30, 40, 50]
let threshold = 80
let label = "Production"
```

---

## Rules

Rules are the reactive heart of Lumina. They fire when conditions change.

### Basic Rule (when condition is true)

```lumina
rule "Overheat Alert"
when Server.is_overheating becomes true {
    show "Server is overheating!"
    alert severity: "warning", message: "Temperature exceeded threshold"
}
```

### Rule with Parameter Binding

The `for` clause binds each instance of an entity type to a parameter:

```lumina
rule "Thermal Warning" for (s: Server)
when Server.is_overheating becomes true {
    alert severity: "warning", message: "Server {Server.name} is hot: {Server.cpu_temp}C"
}
```

**Important**: Inside a rule with `for (s: Server)`, both `s.cpu_temp` and `Server.cpu_temp` resolve to the same thing — the current instance being evaluated.

### Rule with Named String

Rule names can be plain identifiers or quoted strings:

```lumina
rule OverheatAlert
when Server.is_overheating becomes true {
    show "Alert fired!"
}

rule "Overheat Alert"
when Server.is_overheating becomes true {
    show "Alert fired!"
}
```

### Level-Triggered Rule (whenever)

While `when` triggers only on the exact moment a condition becomes true (edge-triggered), `whenever` fires repeatedly every engine tick as long as the condition remains true (level-triggered).

```lumina
rule "Constant Heat Warning"
whenever (Server.cpu_temp > 90) {
    show "WARNING: Server is STILL too hot!"
}
```
*Note: The engine natively supports auto-deduplication to prevent log spam, but `whenever` is best used for continuous state reconciliation.*

### Temporal Rule (every)

Fires repeatedly on a fixed interval:

```lumina
rule Heartbeat
every 2s {
    update server1.cpu_temp = server1.cpu_temp + 1
    show "Tick: {server1.cpu_temp}"
}
```

Duration units: `s` (seconds), `m` (minutes), `h` (hours), `d` (days).

### Stabilization (for duration)

The condition must remain true for the specified duration before firing:

```lumina
rule "Sustained Overheat"
when Server.is_overheating becomes true for 5m {
    alert severity: "critical", message: "Overheating for 5 minutes!"
}
```

If the condition becomes false before the timer elapses, the timer is cancelled.

### Frequency Trigger

Fires when the condition becomes true N times within a window:

```lumina
rule "Flapping Alert" for (s: Server)
when Server.cpu_temp > 80
frequency 3 times within 10s {
    show "Flapping detected on {Server.name}"
}
```

Constraints: N must be >= 2 (L039), window must be > 0 (L040).

### Cooldown

Prevents a rule from re-firing too quickly:

```lumina
rule "Rate Limited Alert" for (s: Server)
when Server.is_overheating becomes true {
    alert severity: "warning", message: "Hot!"
}
cooldown 30s
```

### On Clear (Recovery)

Fires when the condition returns to false after being true:

```lumina
rule "CRITICAL: Server Down" for (s: Server)
when Server.is_overheating becomes true {
    alert severity: "critical", message: "EMERGENCY: {Server.name} overheating!"
}
on clear {
    show "System stable: {Server.name} thermal state resolved."
}
```

### Multi-Condition Rules (and)

Up to 3 conditions can be combined with `and`:

```lumina
rule "Compound Alert" for (s: Server)
when Server.cpu_temp > 90 and Server.is_online becomes true {
    alert severity: "critical", message: "Online server is dangerously hot"
}
```

Maximum 3 conditions per rule (L035).

### Fleet Triggers (any/all)

Watch across ALL instances of an entity type:

```lumina
rule "Any Server Hot"
when any Server.is_overheating becomes true {
    alert severity: "warning", message: "At least one server is overheating"
}

rule "All Servers Hot"
when all Server.is_overheating becomes true {
    alert severity: "critical", message: "EVERY server is overheating!"
}
```

The field used in fleet triggers MUST be Boolean (L027).

---

## Actions

Actions are what rules DO when they fire.

### show — Print Output

```lumina
show "Hello, world!"
show "Temperature is {server1.cpu_temp}C"
show "Sum: " + "value"
```

### update — Mutate a Stored Field

```lumina
update server1.cpu_temp = 85
update server1.cpu_temp = server1.cpu_temp + 1
```

**Important**: Use `=` syntax. The `to` keyword is legacy.
**Cannot update derived fields** — they are computed automatically (R009).

### write — Mutate an External Entity Field

```lumina
write sensor.threshold = 90
```

`write` is for external entities only. Using `write` on a regular entity raises L038.

### alert — Send a Structured Alert

```lumina
alert severity: "warning", message: "Server hot!"

alert
    severity: "critical",
    message: "EMERGENCY: {Server.name} overheating!",
    source: "ThermalMonitor"
```

Parameters: `severity` (required), `message` (required), `source` (optional), `code` (optional).

### create — Create a New Instance at Runtime

```lumina
create Server {
    cpu_temp: 30,
    hostname: "new-server",
    is_online: true
}
```

### delete — Remove an Instance

```lumina
delete server1
```

### Infrastructure Actions (v2.1)

Lumina v2.1 introduces native actions for external infrastructure lifecycle management via Adapters (like Docker, Kubernetes).

```lumina
-- Provision a new external resource based on its entity definition
provision worker_2

-- Tear down an existing external resource
terminate worker_2
-- (destroy is an alias for terminate)
destroy worker_2

-- Manually trigger a drift-detection poll for a specific Entity type
reconcile Service
```

---

## Aggregates

Aggregates compute fleet-level metrics across all instances of an entity.

```lumina
aggregate RackStats over Server {
    avg_temp := avg(cpu_temp)
    max_temp := max(cpu_temp)
    total_servers := count(is_online)
    any_hot := any(is_overheating)
    all_healthy := all(is_online)
}
```

### Aggregate Functions

| Function     | Input Type | Output Type | Description                    |
|-------------|------------|-------------|--------------------------------|
| `avg(field)` | Number     | Number      | Average of all instances       |
| `min(field)` | Number     | Number      | Minimum value                  |
| `max(field)` | Number     | Number      | Maximum value                  |
| `sum(field)` | Number     | Number      | Sum of all values              |
| `count(field)`| Boolean   | Number      | Count where field is true      |
| `count()`    | —          | Number      | Count all instances            |
| `any(field)` | Boolean    | Boolean     | True if ANY instance is true   |
| `all(field)` | Boolean    | Boolean     | True if ALL instances are true |

### Using Aggregate Values

Access aggregate fields with dot notation:

```lumina
show "Average temp: {RackStats.avg_temp}"
show "Hot servers: {RackStats.any_hot}"

rule "Fleet Alert"
when RackStats.any_hot becomes true {
    alert severity: "warning", message: "At least one server overheating"
}
```

---

## User-Defined Functions

Pure functions for reusable calculations. They CANNOT access entity fields directly (L015).

```lumina
fn clamp(val: Number, lo: Number, hi: Number) -> Number {
    if val < lo then lo else if val > hi then hi else val
}

fn fahrenheit_to_celsius(f: Number) -> Number {
    (f - 32) * 5 / 9
}
```

Use in derived fields or rules:

```lumina
entity Sensor {
    raw_temp: Number
    clamped := clamp(raw_temp, 0, 100)
}
```

---

## String Interpolation

Use `{expr}` inside double-quoted strings:

```lumina
show "Temperature: {server1.cpu_temp}C"
show "Status: {if server1.is_online then "UP" else "DOWN"}"
show "Sum: {1 + 2 + 3}"
```

---

## Expressions

### Arithmetic

```lumina
1 + 2          -- 3
10 - 3         -- 7
4 * 5          -- 20
10 / 3         -- 3.333...
10 mod 3       -- 1
-x             -- negation
```

### Comparison

```lumina
x == y         -- equal
x != y         -- not equal
x > y          -- greater than
x < y          -- less than
x >= y         -- greater or equal
x <= y         -- less or equal
```

### Logical

```lumina
a and b        -- logical AND (short-circuits)
a or b         -- logical OR (short-circuits)
not a          -- logical NOT
```

### Conditional (if/then/else)

Always requires both `then` and `else` branches:

```lumina
if temp > 80 then "HOT" else "OK"
if x > 0 then x else -x    -- absolute value
```

### Lists

```lumina
let nums = [10, 20, 30]
let first = head(nums)        -- 10
let rest = tail(nums)          -- [20, 30]
let third = at(nums, 2)       -- 30
let count = len(nums)          -- 3
let bigger = append(nums, 40)  -- [10, 20, 30, 40]
```

### Previous Value

Access the value a field had BEFORE the current update:

```lumina
prev(cpu_temp)    -- previous value of cpu_temp
```

Constraints: Only works on stored fields (L024). Cannot nest: `prev(prev(x))` is invalid (L025).

### Timestamp and Age

```lumina
let start = now()              -- current engine time
-- later:
let elapsed = start.age        -- Duration since start was set
```

`now()` cannot be used in derived fields (L041). Use it in rules or `let` statements.

---

## External Entities

External entities represent data from outside Lumina (sensors, APIs, databases):

```lumina
external entity Sensor {
    temperature: Number
    sync: "mqtt://broker:1883"
    on: realtime
    sync on temperature
}
```

Sync strategies:
- `realtime` — Push-based, data arrives as events
- `poll` — Lumina pulls data at intervals
- `webhook` — External system calls Lumina

---

## Cluster Configuration (v2.0)

```lumina
cluster {
    node_id: "node-1"
    bind_addr: "0.0.0.0:7777"
    peers: ["10.0.0.2:7777", "10.0.0.3:7777"]
    quorum: 2
}
```

---

## Orchestration Actions (v2.0)

```lumina
-- Migrate workloads to a specific node
migrate([instance1, instance2], to: "node-2")

-- Evacuate all instances of an entity type
evacuate("Server")

-- Deploy workloads
deploy("deployment-spec")
```

---

## Imports

```lumina
-- Import another .lum file
import "other_file.lum"

-- Import a standard LSL schema
import "LSL::datacenter::Server"
import "LSL::network::Switch"
import "LSL::k8s::Pod"
```

Available LSL namespaces: `datacenter`, `network`, `k8s`, `power`.

---

## Program Structure

A complete Lumina program follows this order (recommended, not enforced):

```lumina
-- 1. Imports (if any)
import "LSL::datacenter::Server"

-- 2. Entity definitions
entity MyEntity { ... }

-- 3. Aggregates
aggregate Stats over MyEntity { ... }

-- 4. Functions
fn helper(x: Number) -> Number { x * 2 }

-- 5. Rules
rule "Name" when ... { ... }

-- 6. Instance creation
let instance1 = MyEntity { ... }

-- 7. Simulation actions (for testing)
show "--- Running simulation ---"
update instance1.field = value
```


---

# Lumina Type System

This document covers every type in Lumina, what operations work on each, and how types interact.

---

## The Seven Types

### Number

64-bit floating point. All numeric values in Lumina are Numbers — there is no integer type.

```lumina
entity Stats {
    count: Number        -- 42
    ratio: Number        -- 0.75
    negative: Number     -- -10
}
```

Whole numbers display without decimals: `42` not `42.0`.

### Text

UTF-8 strings with interpolation support.

```lumina
entity Server {
    hostname: Text       -- "web-01"
    label: Text          -- "Production Server"
}

-- String interpolation uses {expr} inside quotes
show "Host: {server1.hostname}, Temp: {server1.cpu_temp}C"
```

Concatenation uses `+`:
```lumina
let full = "Hello" + " " + "World"    -- "Hello World"
```

### Boolean

Logical true/false.

```lumina
entity Server {
    is_online: Boolean       -- true or false
    is_overheating := cpu_temp > 80    -- derived Boolean
}
```

### Timestamp

Represents a point in time as epoch milliseconds (f64 internally). Created by `now()`.

```lumina
let start_time = now()
```

**Cannot be used in derived fields** (error L041). The `.age` accessor returns the Duration since the timestamp was set:

```lumina
-- In a rule:
rule "Stale Check"
when start_time.age > 60s {
    show "More than 60 seconds have passed"
}
```

### Duration

Represents a length of time. Created with duration literals.

```lumina
5s        -- 5 seconds
10m       -- 10 minutes (600 seconds internally)
2h        -- 2 hours (7200 seconds internally)
1d        -- 1 day (86400 seconds internally)
```

Durations can be compared with `>`, `<`, `>=`, `<=`, `==`, `!=`.

Display format adapts to magnitude: `5s`, `2m 30s`, `1h 15m`, `3d 6h`.

### Secret

Encrypted-at-rest values. Created by `env()`. Always displayed as `***SECRET***`.

```lumina
let api_key = env("API_KEY")
show "Key: {api_key}"          -- Output: Key: ***SECRET***
```

Secrets can only be passed to `write` actions and external adapters. Derived fields cannot resolve to Secret type (error L051).

### List

Ordered collection of values.

```lumina
let nums = [10, 20, 30, 40]
let names = ["alpha", "beta", "gamma"]
let mixed = [1, "two", true]          -- mixed types allowed at runtime
```

---

## Operator Compatibility Matrix

### Arithmetic Operators

| Operator | Number + Number | Text + Text | Other combinations |
|----------|----------------|-------------|-------------------|
| `+`      | ✅ Addition     | ✅ Concatenation | ❌ R018 type error |
| `-`      | ✅ Subtraction   | ❌           | ❌ R018 |
| `*`      | ✅ Multiplication| ❌           | ❌ R018 |
| `/`      | ✅ Division (R002 if divisor is 0) | ❌ | ❌ R018 |
| `mod`    | ✅ Modulo (R002 if divisor is 0)   | ❌ | ❌ R018 |

### Comparison Operators

| Operator   | Number | Text | Boolean | Duration | Timestamp |
|-----------|--------|------|---------|----------|-----------|
| `==` `!=` | ✅     | ✅   | ✅      | ✅       | ✅        |
| `>` `<`   | ✅     | ❌   | ❌      | ✅       | ❌        |
| `>=` `<=` | ✅     | ❌   | ❌      | ✅       | ❌        |

### Logical Operators

| Operator | Boolean | Other types |
|----------|---------|-------------|
| `and`    | ✅ (short-circuits) | ❌ R018 |
| `or`     | ✅ (short-circuits) | ❌ R018 |
| `not`    | ✅      | ❌ R018 |

### Unary Operators

| Operator | Number | Boolean | Other |
|----------|--------|---------|-------|
| `-` (negate) | ✅ | ❌      | ❌ R018 |
| `not`    | ❌     | ✅      | ❌ R018 |

---

## Type Errors

When you use an operator with incompatible types, you get **R018**:

```
Type mismatch: cannot apply '+' to Number and Boolean
```

This is a runtime error that triggers an atomic rollback.

---

## The Unknown Type

`Unknown` is an internal type used for external entity fields that haven't received data yet. You cannot create Unknown values directly. Check for it defensively in rules that depend on external data.

---

## Default Values by Type

When an external entity is registered, its stored fields get these defaults:

| Type      | Default Value    |
|-----------|-----------------|
| Number    | `0`             |
| Text      | `""`            |
| Boolean   | `false`         |
| Timestamp | `0`             |
| Duration  | `0s`            |
| Secret    | `""` (empty)    |
| List      | `[]`            |

---

## Expression Precedence (Highest to Lowest)

1. Primary: literals, identifiers, `(expr)`, function calls
2. Unary: `-`, `not`
3. Multiplicative: `*`, `/`, `mod`
4. Additive: `+`, `-`
5. Comparison: `==`, `!=`, `>`, `<`, `>=`, `<=`
6. Logical NOT: `not`
7. Logical AND: `and`
8. Logical OR: `or`
9. Transition: `becomes`


---

# Lumina Built-in Functions Reference

Every built-in function available in the Lumina runtime, with signatures, examples, and edge cases.

---

## List Functions

### `len(list) → Number`

Returns the number of elements in a list.

```lumina
let items = [10, 20, 30]
show "Count: {len(items)}"        -- Output: Count: 3

let empty = []
show "Count: {len(empty)}"        -- Output: Count: 0
```

### `min(list) → Number`

Returns the smallest number in a list.

```lumina
let temps = [72, 45, 88, 31]
show "Coldest: {min(temps)}"      -- Output: Coldest: 31
```

⚠️ **Errors on empty list** (R004: index out of bounds).
Only operates on Number values — non-numeric elements are silently ignored.

### `max(list) → Number`

Returns the largest number in a list.

```lumina
let temps = [72, 45, 88, 31]
show "Hottest: {max(temps)}"      -- Output: Hottest: 88
```

⚠️ **Errors on empty list** (R004).

### `sum(list) → Number`

Returns the sum of all numbers in a list.

```lumina
let values = [10, 20, 30]
show "Total: {sum(values)}"       -- Output: Total: 60

let empty = []
show "Total: {sum(empty)}"        -- Output: Total: 0
```

Returns `0` for empty lists (does NOT error).

### `head(list) → Value`

Returns the first element of a list.

```lumina
let items = [10, 20, 30]
show "First: {head(items)}"       -- Output: First: 10
```

⚠️ **Errors on empty list** (R004).

### `tail(list) → List`

Returns a new list with all elements except the first.

```lumina
let items = [10, 20, 30]
show "Rest: {tail(items)}"        -- Output: Rest: [20, 30]
```

⚠️ **Errors on empty list** (R004).

### `at(list, index) → Value`

Returns the element at the given index (0-based).

```lumina
let items = [10, 20, 30, 40]
show "Third: {at(items, 2)}"      -- Output: Third: 30
show "First: {at(items, 0)}"      -- Output: First: 10
```

⚠️ **Errors if index >= length** (R004: List index out of bounds).

### `append(list, value) → List`

Returns a NEW list with the value added at the end. Does not mutate the original.

```lumina
let items = [10, 20]
let bigger = append(items, 30)
show "Result: {bigger}"           -- Output: Result: [10, 20, 30]
```

---

## Time Functions

### `now() → Timestamp`

Returns the current engine time as a Timestamp value.

```lumina
let start = now()
show "Started at: {start}"        -- Output: Started at: Timestamp(0)
```

⚠️ **Cannot be used in derived fields** (error L041). The engine would need to recompute it continuously, causing a recomputation storm. Use it in `let` statements or rule actions instead.

**The `.age` accessor**: Returns a Duration representing time elapsed since the timestamp was set:

```lumina
let created = now()
-- In a rule:
rule "Stale Data"
when created.age > 300s {
    show "Data is older than 5 minutes"
}
```

---

## Environment Functions

### `env(name) → Secret`

Reads an operating system environment variable and returns it as a Secret value.

```lumina
let api_key = env("API_KEY")
let user = env("USER")
show "User: {user}"               -- Output: User: ***SECRET***
```

The returned value is ALWAYS a Secret — it displays as `***SECRET***` in output. If the environment variable doesn't exist, returns an empty Secret.

---

## Aggregate Functions (Entity-Level)

These functions are used ONLY inside `aggregate` declarations, NOT as standalone expressions.

```lumina
aggregate FleetStats over Server {
    avg_temp := avg(cpu_temp)         -- Average of cpu_temp across all Server instances
    min_temp := min(cpu_temp)         -- Minimum cpu_temp
    max_temp := max(cpu_temp)         -- Maximum cpu_temp
    total_power := sum(power_watts)   -- Sum of power_watts
    online := count(is_online)        -- Count where is_online == true
    total := count()                  -- Count all instances (no field needed)
    any_hot := any(is_overheating)    -- true if ANY instance has is_overheating == true
    all_ok := all(is_online)          -- true if ALL instances have is_online == true
}
```

| Function   | Input Field Type | Output Type | Empty Fleet Result |
|-----------|-----------------|-------------|-------------------|
| `avg(f)`  | Number          | Number      | `0`               |
| `min(f)`  | Number          | Number      | `Infinity`        |
| `max(f)`  | Number          | Number      | `-Infinity`       |
| `sum(f)`  | Number          | Number      | `0`               |
| `count(f)`| Boolean         | Number      | `0`               |
| `count()` | —               | Number      | `0`               |
| `any(f)`  | Boolean         | Boolean     | `false`           |
| `all(f)`  | Boolean         | Boolean     | `false`           |

---

## User-Defined Functions

You can define pure functions with `fn`. They cannot access entity fields directly — all data must be passed as parameters.

```lumina
fn clamp(val: Number, lo: Number, hi: Number) -> Number {
    if val < lo then lo else if val > hi then hi else val
}

fn to_celsius(f: Number) -> Number {
    (f - 32) * 5 / 9
}

fn label(temp: Number) -> Text {
    if temp > 80 then "HOT" else if temp > 50 then "WARM" else "COOL"
}
```

Usage:

```lumina
entity Sensor {
    raw_temp: Number
    safe_temp := clamp(raw_temp, -40, 150)
    temp_label := label(raw_temp)
}
```

Function constraints:
- Must have a return type annotation (`-> Type`)
- Body must be a single expression (no statements)
- Cannot access entity fields directly (L015)
- Cannot have duplicate names (L011)
- Return type must match body expression type (L014)


---

# Lumina Rules & Triggers Deep Dive

Rules are the hardest part of Lumina to get right. This document covers every trigger type, how edge detection works, and the most common mistakes.

---

## How Rules Work: The Mental Model

A rule is NOT an event handler. It's a **reactive invariant**. The engine evaluates rules after every state change and fires them only when conditions **transition**.

```
State Change → Propagate Derived Fields → Evaluate Rules → Execute Actions
```

---

## Trigger Type 1: `when` + `becomes` (Edge Detection)

This is the most common trigger. It fires **once** when a value transitions to the target.

```lumina
entity Server {
    cpu_temp: Number
    is_overheating := cpu_temp > 80
}

let s1 = Server { cpu_temp: 50 }

rule "Alert"
when Server.is_overheating becomes true {
    show "ALERT: Server overheating!"
}

-- This fires the rule (transition: false → true):
update s1.cpu_temp = 90

-- This does NOT fire again (already true, no transition):
update s1.cpu_temp = 95

-- This does NOT fire (still true → true):
update s1.cpu_temp = 91

-- After cooling down and heating up again, it WILL fire:
update s1.cpu_temp = 50    -- becomes false
update s1.cpu_temp = 90    -- becomes true again → FIRES
```

### Key insight: `becomes` compares the PREVIOUS value to the CURRENT value

The engine keeps a snapshot of the previous state. After an update:
1. It evaluates the condition against the new state
2. If it matches the `becomes` target, it evaluates the same condition against the previous state
3. If the previous state did NOT match, it's a genuine transition → fire the rule

### `when` without `becomes` (Simple Condition)

Without `becomes`, the condition must evaluate to a Boolean. But it still only fires on transitions (false → true), NOT continuously while true.

```lumina
rule "Temp Check"
when Server.cpu_temp > 80 {
    show "Server is above 80 degrees"
}
```

This fires once when `cpu_temp` crosses above 80, NOT every tick while it's above 80.

---

## Trigger Type 2: `every` (Temporal Interval)

Fires repeatedly on a fixed interval. No condition checking — it just ticks.

```lumina
rule Heartbeat
every 1s {
    show "Tick!"
}

rule SlowPoll
every 5m {
    show "5-minute check"
}
```

Duration units: `s` (seconds), `m` (minutes), `h` (hours), `d` (days).

`every` rules start ticking immediately when the program begins. They are registered in the timer heap at startup.

### Combining `every` with updates (Reactive Loops)

This is the canonical pattern for creating loops in Lumina:

```lumina
entity Counter {
    value: Number
}

let c = Counter { value: 0 }

rule Increment
every 2s {
    update c.value = c.value + 1
    show "Counter: {c.value}"
}

rule Reset
when c.value >= 10 {
    show "Resetting counter!"
    update c.value = 0
}
```

The `every` rule increments the counter. The `when` rule resets it. Together they form a self-resetting loop.

---

## Trigger Type 3: `for` Duration (Stabilization)

The condition must remain true for the entire duration before firing.

```lumina
rule "Sustained Alert"
when Server.is_overheating becomes true for 5m {
    alert severity: "critical", message: "Overheating for 5 minutes!"
}
```

How it works internally:
1. When condition becomes true → start a timer
2. If condition becomes false before timer elapses → cancel the timer
3. If timer elapses while condition is still true → fire the rule

This prevents false alarms from momentary spikes.

---

## Trigger Type 4: `frequency` (Rate Detection)

Fires when the condition becomes true N times within a time window.

```lumina
rule "Flapping" for (s: Server)
when Server.is_overheating becomes true
frequency 3 times within 10s {
    show "Server {Server.name} is flapping! (3 overheat events in 10s)"
}
```

Constraints:
- Count must be >= 2 (error L039)
- Window duration must be > 0 (error L040)

---

## Trigger Type 5: `cooldown` (Throttling)

Prevents a rule from re-firing too quickly after it last fired.

```lumina
rule "Rate Limited" for (s: Server)
when Server.is_overheating becomes true {
    alert severity: "warning", message: "Hot!"
}
cooldown 30s
```

After firing, the rule ignores transitions for 30 seconds. Cooldown is tracked per rule+instance pair.

---

## Recovery: `on clear`

The `on clear` block fires when the condition returns to false after being true.

```lumina
rule "Thermal Alert" for (s: Server)
when Server.is_overheating becomes true {
    alert severity: "critical", message: "{Server.name} overheating!"
}
on clear {
    show "RESOLVED: {Server.name} temperature normalized"
}
```

Timeline:
```
cpu_temp: 50 → 90 (becomes true → ALERT fires)
cpu_temp: 90 → 50 (becomes false → ON CLEAR fires)
```

---

## Parameter Binding: `for (param: Entity)`

When a rule has `for (s: Server)`, the engine evaluates it once per Server instance.

```lumina
rule "Per-Server Check" for (s: Server)
when Server.is_overheating becomes true {
    show "Server {Server.name} is hot: {Server.cpu_temp}C"
}
```

### The Entity-Name Resolution Gotcha

Inside a parameterized rule, you can reference fields two ways:

```lumina
rule "Check" for (s: Server)
when Server.is_overheating becomes true {
    -- Both of these refer to the SAME instance:
    show "Via entity: {Server.cpu_temp}"
    show "Via param: {s.cpu_temp}"        -- ALSO works (alias)
}
```

The engine resolves `Server.cpu_temp` to the current instance `s` because it sees that `s` is bound to `Server`. This is a convenience feature — use whichever style you prefer.

---

## Fleet Triggers: `any` and `all`

Watch across ALL instances of an entity type:

### `any` — True if at least one instance matches

```lumina
rule "Any Hot"
when any Server.is_overheating becomes true {
    alert severity: "warning", message: "At least one server overheating"
}
```

### `all` — True if every instance matches

```lumina
rule "All Hot"
when all Server.is_overheating becomes true {
    alert severity: "critical", message: "EVERY server is overheating!"
}
```

**The field MUST be Boolean** (error L027). You cannot use `any Server.cpu_temp becomes 80`.

---

## Multi-Condition Rules (`and`)

Combine up to 3 conditions:

```lumina
rule "Compound" for (s: Server)
when Server.cpu_temp > 90 and Server.is_online becomes true {
    alert severity: "critical", message: "Online server dangerously hot"
}
```

**Maximum 3 conditions** per rule (error L035).

---

## Rule Execution Order

1. Rules are evaluated in declaration order
2. If a rule's action triggers another rule (via `update`), the second rule fires during the same tick
3. If recursion exceeds 100 depth → **R003 rollback** (the entire tick is undone)

---

## Common Mistakes

### ❌ Updating a derived field

```lumina
entity Server {
    is_hot := cpu_temp > 80
}
-- ERROR R009: Cannot update derived field 'is_hot'
update s1.is_hot = true
```

**Fix**: Update the stored field that drives the derived field:
```lumina
update s1.cpu_temp = 90    -- is_hot becomes true automatically
```

### ❌ Creating infinite rule loops

```lumina
rule "Loop" for (n: Node)
when Node.value becomes 100 {
    update Node.value = 100    -- re-triggers itself!
}
```

**Result**: R003 at depth 100, full rollback. Fix by making the action change the value to something that doesn't re-trigger.

### ❌ Using `now()` in derived fields

```lumina
entity Timer {
    elapsed := now() - start    -- ERROR L041
}
```

**Fix**: Use a rule with `every` instead:
```lumina
rule "Update Elapsed"
every 1s {
    update timer.elapsed_secs = timer.elapsed_secs + 1
}
```

### ❌ Non-Boolean `when` without `becomes`

```lumina
rule "Bad"
when Server.cpu_temp {    -- ERROR L002: must be Boolean
    show "?"
}
```

**Fix**: Add a comparison or use `becomes`:
```lumina
rule "Good"
when Server.cpu_temp > 80 {
    show "Hot!"
}
```


---

# Lumina Patterns Cookbook

Complete, runnable Lumina programs. Each program demonstrates a real pattern.

---

## Pattern 1: Hello World — Minimal Entity

```lumina
-- The simplest possible Lumina program
entity Greeter {
    message: Text
}

let g = Greeter { message: "Hello, Lumina!" }
show g.message
```

**Expected output:**
```
Hello, Lumina!
```

---

## Pattern 2: Temperature Monitor — Derived Fields + Rules

```lumina
entity Sensor {
    temperature: Number
    threshold: Number
    is_hot := temperature > threshold
    status := if is_hot then "OVERHEATING" else "NORMAL"
}

let s1 = Sensor { temperature: 45, threshold: 80 }

rule "Heat Alert" for (s: Sensor)
when Sensor.is_hot becomes true {
    show "ALERT: Temperature {Sensor.temperature}C exceeds {Sensor.threshold}C!"
}

show "--- Initial State ---"
show "Status: {s1.status}"

show "--- Heating Up ---"
update s1.temperature = 90
show "Status: {s1.status}"
```

**Expected output:**
```
--- Initial State ---
Status: NORMAL
--- Heating Up ---
ALERT: Temperature 90C exceeds 80C!
Status: OVERHEATING
```

---

## Pattern 3: Self-Resetting Loop — every + when

```lumina
entity LoopNode {
    iteration: Number
}

let n = LoopNode { iteration: 0 }

rule Heartbeat
every 2s {
    update n.iteration = n.iteration + 1
    show ">>> Current Iteration: {n.iteration}"
}

rule ResetLoop
when n.iteration >= 10 {
    show "--- Milestone: 10 reached. Resetting loop... ---"
    update n.iteration = 0
}
```

**Behavior**: Counter increments by 1 every 2 seconds. When it reaches 10, it resets to 0 and starts again.

---

## Pattern 4: Health Dashboard — Multiple Entities + Refs + Aggregates

```lumina
entity CoolingUnit {
    name: Text
    power_watts: Number
    is_active: Boolean
    is_failing := power_watts > 50 and not is_active
}

entity Server {
    name: Text
    cpu_temp: Number
    is_online: Boolean
    cooling: ref CoolingUnit
    is_overheating := cpu_temp > 80
    is_critical := is_overheating and cooling.is_failing
}

aggregate RackStats over Server {
    avg_temp := avg(cpu_temp)
    online_count := count(is_online)
}

rule "Cooling Recovery" for (c: CoolingUnit)
when CoolingUnit.is_active becomes true {
    show "Cooling {CoolingUnit.name}: ONLINE"
}

rule "Cooling Failure" for (c: CoolingUnit)
when CoolingUnit.is_active becomes false {
    show "Cooling {CoolingUnit.name}: OFFLINE"
}

rule "Thermal Warning" for (s: Server)
when Server.is_overheating becomes true {
    alert
        severity: "warning",
        message: "Server {Server.name} overheating: {Server.cpu_temp}C",
        source: "ThermalMonitor"
}

rule "CRITICAL: Cooling Failure" for (s: Server)
when Server.is_critical becomes true {
    alert
        severity: "critical",
        message: "EMERGENCY: {Server.name} overheating AND cooling failing!",
        source: "SovereignRuntime"
}
on clear {
    show "System stable: {Server.name} thermal state resolved."
}

-- Initialize
let MainCooler = CoolingUnit {
    name: "Main Cooler (Rack A)",
    power_watts: 100,
    is_active: true
}

let S1 = Server {
    name: "Web-01",
    cpu_temp: 45,
    is_online: true,
    cooling: MainCooler
}

let S2 = Server {
    name: "DB-01",
    cpu_temp: 50,
    is_online: true,
    cooling: MainCooler
}

-- Simulate
show "--- INITIAL STATE ---"
show "Avg Temp: {RackStats.avg_temp}C"

show "--- SIMULATING THERMAL SPIKE ---"
update S1.cpu_temp = 85

show "--- SIMULATING COOLING FAILURE ---"
update MainCooler.is_active = false

show "--- SIMULATING RECOVERY ---"
update MainCooler.is_active = true
update S1.cpu_temp = 45
```

---

## Pattern 5: Fleet Monitoring — Multiple Instances + Aggregate Rules

```lumina
entity Worker {
    name: Text
    cpu_percent: Number
    is_healthy: Boolean
    is_stressed := cpu_percent > 90
}

aggregate ClusterHealth over Worker {
    avg_cpu := avg(cpu_percent)
    stressed_count := count(is_stressed)
    all_healthy := all(is_healthy)
}

rule "Worker Stressed" for (w: Worker)
when Worker.is_stressed becomes true {
    show "WARNING: {Worker.name} CPU at {Worker.cpu_percent}%"
}

rule "Cluster Unhealthy"
when ClusterHealth.all_healthy becomes false {
    alert severity: "critical", message: "Not all workers are healthy!"
}

let w1 = Worker { name: "worker-1", cpu_percent: 45, is_healthy: true }
let w2 = Worker { name: "worker-2", cpu_percent: 55, is_healthy: true }
let w3 = Worker { name: "worker-3", cpu_percent: 30, is_healthy: true }

show "--- Cluster Status ---"
show "Avg CPU: {ClusterHealth.avg_cpu}%"
show "Stressed: {ClusterHealth.stressed_count}"

show "--- Simulating Load Spike ---"
update w1.cpu_percent = 95
update w2.cpu_percent = 92

show "Stressed count: {ClusterHealth.stressed_count}"
```

---

## Pattern 6: Security Quarantine — write + on clear

```lumina
entity NetworkNode {
    name: Text
    security_score: Number
    vlan: Number
    is_quarantined := vlan == 999
}

rule "Zero-Trust Quarantine" for (n: NetworkNode)
when NetworkNode.security_score < 50 {
    show "QUARANTINE: {NetworkNode.name} (score: {NetworkNode.security_score})"
    update NetworkNode.vlan = 999
}
on clear {
    show "RESTORED: {NetworkNode.name} returned to production VLAN"
    update NetworkNode.vlan = 1
}

let node1 = NetworkNode { name: "db-prod-01", security_score: 85, vlan: 1 }

show "--- Initial: score = 85 ---"
show "VLAN: {node1.vlan}, Quarantined: {node1.is_quarantined}"

show "--- Lowering security score ---"
update node1.security_score = 30

show "--- Restoring security score ---"
update node1.security_score = 90
```

---

## Pattern 7: Chaos Test — Rollback + Range Violations

```lumina
entity Node {
    @range 0 to 500 value: Number
}

entity Vault {
    @range 0 to 1000 balance: Number
}

aggregate GlobalStress over Node {
    total := sum(value)
}

-- This rule triggers infinite recursion (R003)
rule "Self-Trigger" for (n: Node)
when Node.value becomes 100 {
    update Node.value = 100
}

-- This rule violates @range (R006) — balance 2000 > max 1000
rule "Range Violation" for (v: Vault)
when Vault.balance becomes 500 {
    update Vault.balance = 2000
}

let N1 = Node { value: 0 }
let N2 = Node { value: 0 }
let MyVault = Vault { balance: 100 }

show "--- TEST 1: Recursion (MAX_DEPTH) ---"
update N1.value = 100

show "--- TEST 2: Rollback (Atomic Transaction) ---"
show "Balance before: {MyVault.balance}"
update MyVault.balance = 500
show "Balance after failure (should be 100): {MyVault.balance}"

show "--- TEST 3: Aggregate ---"
update N2.value = 300
show "Total: {GlobalStress.total}"
```

**Key behavior**: 
- Test 1: R003 — rollback, N1.value stays 0
- Test 2: R006 — rollback, balance stays 100
- Test 3: Aggregate computes correctly

---

## Pattern 8: User-Defined Functions

```lumina
fn clamp(val: Number, lo: Number, hi: Number) -> Number {
    if val < lo then lo else if val > hi then hi else val
}

fn severity_label(temp: Number) -> Text {
    if temp > 90 then "CRITICAL"
    else if temp > 70 then "WARNING"
    else "NORMAL"
}

entity Sensor {
    raw_reading: Number
    safe_reading := clamp(raw_reading, -40, 150)
    severity := severity_label(raw_reading)
}
```

---

## Pattern 9: Functions as Constructors (Boilerplate Reduction)

Because Lumina strictly requires all stored fields to be initialized when creating an entity (to prevent undefined state crashes), large schemas like `LSL::docker::Container` can lead to boilerplate when you only care about a few fields. Use functions to create "constructors" that provide safe default values:

```lumina
import "LSL::docker::Container"

// A helper function that handles all the boilerplate for you
fn create_worker(worker_name: Text) -> Container {
  return Container {
    name: worker_name,
    image: "nginx:alpine",
    port: 0,
    target_port: 80,
    env_vars: "NONE",
    status: "unknown",
    verified: false,
    tier: "app"
  }
}

// Now you can just use the fields you care about!
let worker_1 = create_worker("hydra-app-01")
let worker_2 = create_worker("hydra-app-02")
```

let s = Sensor { raw_reading: 95 }
show "Raw: {s.raw_reading}"
show "Safe: {s.safe_reading}"
show "Severity: {s.severity}"

update s.raw_reading = -100
show "Raw: {s.raw_reading}"
show "Safe: {s.safe_reading}"
show "Severity: {s.severity}"
```

---

## Pattern 9: List Operations

```lumina
let temps = [72, 88, 45, 91, 33]

show "Count: {len(temps)}"
show "Min: {min(temps)}"
show "Max: {max(temps)}"
show "Sum: {sum(temps)}"
show "First: {head(temps)}"
show "Rest: {tail(temps)}"
show "Third: {at(temps, 2)}"

let extended = append(temps, 100)
show "Extended: {extended}"
show "New length: {len(extended)}"
```

---

## Pattern 10: Multi-Entity Datacenter Simulation

```lumina
entity PDU {
    name: Text
    total_kw: Number
    max_kw: Number
    is_overloaded := total_kw > max_kw * 0.9
}

entity Rack {
    name: Text
    server_count: Number
    pdu: ref PDU
    power_draw_kw: Number
    is_powered := not pdu.is_overloaded
}

entity Server {
    name: Text
    cpu_temp: Number
    rack_name: Text
    is_online: Boolean
    is_overheating := cpu_temp > 80
}

aggregate DatacenterStats over Server {
    avg_temp := avg(cpu_temp)
    hot_count := count(is_overheating)
    online_count := count(is_online)
}

-- Rules
rule "PDU Overload" for (p: PDU)
when PDU.is_overloaded becomes true {
    alert severity: "critical", message: "PDU {PDU.name} overloaded: {PDU.total_kw}kW / {PDU.max_kw}kW"
}

rule "Server Overheat" for (s: Server)
when Server.is_overheating becomes true {
    alert severity: "warning", message: "{Server.name} at {Server.cpu_temp}C"
}

rule "Fleet Status"
when DatacenterStats.hot_count > 0 {
    show "FLEET ALERT: {DatacenterStats.hot_count} servers overheating. Avg: {DatacenterStats.avg_temp}C"
}

-- Initialize
let pdu1 = PDU { name: "PDU-A1", total_kw: 15, max_kw: 20 }
let rack1 = Rack { name: "Rack-A", server_count: 3, pdu: pdu1, power_draw_kw: 5 }
let srv1 = Server { name: "web-01", cpu_temp: 45, rack_name: "Rack-A", is_online: true }
let srv2 = Server { name: "web-02", cpu_temp: 50, rack_name: "Rack-A", is_online: true }
let srv3 = Server { name: "db-01", cpu_temp: 55, rack_name: "Rack-A", is_online: true }

-- Simulate
show "=== INITIAL STATE ==="
show "Avg temp: {DatacenterStats.avg_temp}C"
show "Hot servers: {DatacenterStats.hot_count}"

show "=== THERMAL EVENT ==="
update srv1.cpu_temp = 85
update srv2.cpu_temp = 82

show "=== PDU OVERLOAD ==="
update pdu1.total_kw = 19

show "=== RECOVERY ==="
update srv1.cpu_temp = 45
update srv2.cpu_temp = 48
update pdu1.total_kw = 12
```

---

## Pattern 11: String Interpolation Showcase

```lumina
entity Report {
    server_name: Text
    cpu: Number
    memory: Number
    disk: Number
    health := if cpu < 80 and memory < 80 and disk < 90 then "HEALTHY" else "DEGRADED"
}

let r = Report {
    server_name: "prod-api-01",
    cpu: 65,
    memory: 72,
    disk: 45
}

show "╔══════════════════════════════╗"
show "║  Server: {r.server_name}"
show "║  CPU:    {r.cpu}%"
show "║  Memory: {r.memory}%"
show "║  Disk:   {r.disk}%"
show "║  Health: {r.health}"
show "╚══════════════════════════════╝"
```

---

## Pattern 12: Conditional Logic Chains

```lumina
fn risk_level(temp: Number, load: Number) -> Text {
    if temp > 90 and load > 90 then "CRITICAL"
    else if temp > 80 or load > 85 then "HIGH"
    else if temp > 60 or load > 70 then "MEDIUM"
    else "LOW"
}

entity Machine {
    temp: Number
    load: Number
    risk := risk_level(temp, load)
}

let m = Machine { temp: 50, load: 40 }
show "Risk: {m.risk}"

update m.temp = 85
show "Risk: {m.risk}"

update m.load = 95
show "Risk: {m.risk}"
```

---

## Pattern 13: Previous Value Detection

```lumina
entity Metric {
    value: Number
}

let m = Metric { value: 100 }

rule "Big Jump"
when m.value > prev(value) * 1.5 {
    show "SPIKE: value jumped from {prev(value)} to {m.value}"
}

rule "Big Drop"
when m.value < prev(value) * 0.5 {
    show "DROP: value fell from {prev(value)} to {m.value}"
}

update m.value = 200
update m.value = 50
```

---

## Pattern 14: IoT Sensor with External Entity

```lumina
external entity Sensor {
    temperature: Number
    sync: "mqtt://broker:1883"
    on: realtime
    sync on temperature
}

entity System {
    isOverheating := Sensor.temperature > 80
}

rule TemperatureAlert for (s: System)
when System.isOverheating becomes true {
    show "Critical: Temperature is {Sensor.temperature}"
    alert severity: "critical", message: "System is overheating!"
}

rule TemperatureResolved for (s: System)
when System.isOverheating becomes false {
    show "Status: Temperature stabilized at {Sensor.temperature}"
    alert severity: "info", message: "Temperature stabilized."
}
```


---

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


---

# Lumina Advanced Features

Cluster networking, external entities, FFI integration, and secrets.

---

## Cluster Configuration (v2.0)

Lumina supports multi-node clusters with native UDP gossip, leader election, and workload migration.

### Cluster Block

```lumina
cluster {
    node_id: "node-1"
    bind_addr: "0.0.0.0:7777"
    peers: ["10.0.0.2:7777", "10.0.0.3:7777"]
    quorum: 2
}
```

| Field         | Type   | Description |
|--------------|--------|-------------|
| `node_id`    | Text   | Unique identifier for this node (L060 if empty) |
| `bind_addr`  | Text   | Address to listen on for gossip |
| `peers`      | List   | Addresses of other nodes (L061 if empty) |
| `quorum`     | Number | Minimum nodes for consensus (L062 if > total nodes) |

### Gossip Protocol

Nodes communicate via UDP on port 7777 (configurable via `bind_addr`). The gossip protocol is SWIM-inspired:

- **Health monitoring**: Nodes cycle through `Alive`, `Suspect`, `Dead` states
- **State sync**: Merkle Tree anti-entropy — nodes exchange state roots every 5 seconds
- **Delta sync**: Only changed fields are gossiped, not full state

### Leader Election (Raft-lite)

For sensitive operations (`migrate`, `deploy`), a leader is elected:

- Requires `quorum` (N/2 + 1) votes
- Leader orchestrates cross-node actions
- If quorum is lost → **Safe Mode** (all writes frozen)

### Orchestration Actions

```lumina
-- Migrate specific instances to a target node
migrate([instance1, instance2], to: "node-2")

-- Evacuate ALL instances of entity types to alive peers
evacuate("Server")

-- Deploy (simplified in v2.0 — spec is evaluated, leader broadcasts)
deploy("deployment-spec")
```

### Accessing Cluster State

```lumina
-- Access a remote node's state
cluster.node_id.field_name
```

Errors:
- R012: Node not found in cluster state
- R014: Cross-node entity reference unresolvable
- R015: Orchestration write target unreachable

### Aggregate Scoping

Aggregates can operate at different scopes:

```lumina
-- Default: Local node only
aggregate LocalStats over Server {
    avg_temp := avg(cpu_temp)
}

-- Cluster-wide (v2.0)
-- Computed by exchanging pre-aggregates via gossip
-- The scope is set in the AST but uses the default Local scope in syntax
```

---

## External Entities

External entities represent data sources outside of Lumina — sensors, APIs, databases, MQTT brokers.

### Declaration

```lumina
external entity Sensor {
    temperature: Number
    humidity: Number
    sync: "mqtt://broker:1883"
    on: realtime
    sync on temperature
}
```

### Sync Strategies

| Strategy   | Keyword    | Description |
|-----------|-----------|-------------|
| Realtime  | `realtime` | Push-based: data arrives as events via adapter |
| Poll      | `poll`     | Lumina pulls data at regular intervals |
| Webhook   | `webhook`  | External system calls Lumina's HTTP endpoint |

### Sync Fields

`sync on field_name` specifies which field triggers reactive propagation when updated externally.

### Poll Interval (for poll strategy)

```lumina
external entity APIData {
    value: Number
    sync: "https://api.example.com/data"
    on: poll
    poll_interval: 30s
    sync on value
}
```

### Default Instance

When an external entity is declared, a default instance is automatically created with default values (0 for Number, "" for Text, false for Boolean, etc.).

### Using `write` with External Entities

The `write` action sends mutations to the external system via the adapter:

```lumina
rule "Adjust Threshold"
when Sensor.temperature > 100 {
    write Sensor.threshold = 120
}
```

`write` ONLY works on external entities (L038 on regular entities).

---

## Providers (JSON-RPC 2.0)

Providers are external processes that connect Lumina to the world. They communicate via JSON-RPC 2.0 over stdin/stdout.

### Provider Protocol

1. **Handshake**: Lumina sends `lumina_hello` → Provider responds with name and managed entities
2. **Schema sync**: `lumina_get_schema` → Provider sends field definitions
3. **State stream**: Provider pushes `state_update` notifications
4. **Write/Rollback**: Lumina sends `lumina_write` for side effects, `lumina_rollback` if transaction fails

### Provider Declaration

```lumina
provider "json-rpc" {
    endpoint: "tcp://localhost:9000"
}
```

Provider must have an `endpoint` config (L053 if missing).

---

## Secrets & Security

### Creating Secrets

```lumina
let api_key = env("API_KEY")          -- reads environment variable as Secret
let db_pass = env("DATABASE_PASSWORD")
```

### Secret Behavior

- Display: Always shown as `***SECRET***` in `show` output
- Storage: Encrypted at rest (when persisted)
- Restrictions:
  - Cannot be used in derived fields (L051)
  - Can be passed to `write` actions and external adapters
  - Secrets in alert payloads are redacted

### Entity Fields with Secret Type

```lumina
entity Credentials {
    api_key: Secret
    db_password: Secret
}

let creds = Credentials {
    api_key: env("API_KEY"),
    db_password: env("DB_PASS")
}

show "Key: {creds.api_key}"    -- Output: Key: ***SECRET***
```

---

## FFI (Foreign Function Interface)

The `lumina_ffi` crate provides a C-compatible API for embedding Lumina in other languages.

### Core C API Functions

```c
// Create a runtime from source code
LuminaRuntime* lumina_create(const char* source);

// Inject an external event (JSON payload)
void lumina_apply_event(LuminaRuntime* rt, const char* instance, const char* field, const char* json);

// Trigger temporal recomputations (tick the timers)
void lumina_tick(LuminaRuntime* rt);

// Free a Lumina-allocated string — NEVER use C free()!
void lumina_free_string(char* ptr);

// Export current state as JSON
char* lumina_export_state(LuminaRuntime* rt);
```

### Critical Memory Rule

**ALWAYS** call `lumina_free_string(ptr)` for strings returned by Lumina. Lumina strings are Rust-owned — using C `free()` will corrupt memory.

### Supported Host Languages

- C / C++ (direct FFI)
- Python (via ctypes/cffi)
- Go (via cgo)
- Node.js (via ffi-napi)

---

## WASM Support

Lumina can run in browsers via WebAssembly with ~90% native performance.

### Key Details

- The exact same Rust engine runs in WASM — simulation parity is guaranteed
- `EntityStore` maps to WASM linear memory for O(1) JS access
- Sub-millisecond event injection from React or vanilla JS

### Limitations in WASM Mode

- `import` statements are disabled (L018)
- File I/O adapters are unavailable
- Cluster features are unavailable
- `env()` returns empty strings

---

## LSP & VS Code Extension

### Features

- **Live diagnostics**: Real-time type checking and cycle detection
- **Semantic highlighting**: Visually distinguishes `stored` vs `derived` fields
- **Go to Definition**: Navigate to entity and function declarations
- **Find All References**: Find all usages of a field or entity
- **Incremental parsing**: Only re-parses modified AST branches

### Installation

The extension is available in the VS Code Marketplace as "Lumina LSL".

---

## Standard Library: LSL Registry

Lumina includes pre-defined entity schemas for common infrastructure:

### Available Namespaces

```lumina
import "LSL::datacenter::Server"     -- Fields: temp, power, status
import "LSL::datacenter::Rack"       -- Fields: total_kw, used_u
import "LSL::datacenter::PDU"
import "LSL::datacenter::CRAC"

import "LSL::network::Switch"        -- Fields: packet_loss
import "LSL::network::Router"        -- Fields: bgp_peers
import "LSL::network::Firewall"

import "LSL::k8s::Pod"               -- Fields: cpu_mcore, restarts
import "LSL::k8s::Node"              -- Fields: disk_pressure
import "LSL::k8s::Deployment"

import "LSL::power::UPS"
import "LSL::power::Generator"
```

Using an unknown namespace raises L054.


---

# Lumina Known Limitations (v2.0-GOLD)

This document lists what does NOT work, what is partially implemented, and what will cause AI hallucinations if assumed to exist. **Read this before generating code.**

---

## Syntax & Language

### ❌ No `trend()` built-in function

The old documentation mentions `trend(field, duration)` in multiple patterns. **This function does NOT exist in the runtime.** It is not implemented in `engine.rs`.

**Workaround**: Use `prev()` to compare current vs. previous values:

```lumina
entity Sensor {
    temp: Number
    is_rising := temp > prev(temp)
}
```

For more complex trend detection, use a stored field and accumulate deltas in a rule.

### ❌ `update` uses `=`, not `to`

The EBNF spec (`SPEC.md`) still mentions `update path 'to' expr`. The v2.0-GOLD standard uses `=`:

```lumina
-- ✅ Correct (v2.0):
update server1.cpu_temp = 90

-- ⚠️ Legacy (may still parse but not recommended):
update server1.cpu_temp to 90
```

Always use `=` in new code.

### ❌ No general-purpose loops

Lumina has no `for`, `while`, or `loop` constructs. Loops are achieved through reactive feedback:

```lumina
-- Reactive loop pattern:
rule Tick every 1s { update counter.value = counter.value + 1 }
rule Reset when counter.value >= 10 { update counter.value = 0 }
```

### ❌ No string methods

There are no built-in functions like `uppercase()`, `lowercase()`, `split()`, `trim()`, `replace()`, or `substring()`. Text manipulation is limited to concatenation (`+`) and interpolation (`{expr}`).

### ❌ No map/dictionary type

Lumina only has Lists. There is no key-value map or dictionary type. Use entity fields for structured data.

### ❌ No variable reassignment outside rules

`let` bindings are immutable outside of rule contexts. You cannot do:

```lumina
let x = 5
x = 10    -- NOT VALID — use entity fields with update instead
```

### ❌ No multi-line strings

Strings must be on a single line. There is no `"""` or heredoc syntax.

### ❌ Max 3 conditions per rule (L035)

Multi-condition rules with `and` are limited to 3 clauses. Combine complex logic into derived fields.

### ❌ No `else if` in rule triggers

You cannot chain conditions in `when`:

```lumina
-- ❌ NOT VALID:
rule "Check"
when x > 90 { ... }
else when x > 80 { ... }
```

Use separate rules instead.

---

## Runtime Behavior

### ❌ Recursion limit: 100 (hard limit)

If a rule's action triggers another rule that triggers another... after 100 depth, R003 fires and the ENTIRE tick is rolled back. This is by design and cannot be configured.

### ❌ No persistent storage

Entity instances live in memory only. When the process exits, all state is lost. External persistence requires a provider or adapter.

### ❌ Aggregates recompute fully on every change

Despite documentation mentioning "incremental O(1) aggregates," the current implementation rescans all instances of the entity on every recompute. This works fine for hundreds of instances but may slow down with 100K+ instances.

### ❌ `for` duration timers require `--live` mode

The `for 5m` stabilization syntax only works when running with `lumina run --live`. In batch execution mode, temporal triggers don't tick.

### ❌ `every` timers don't fire in batch mode

Similarly, `every 2s` rules only fire in `--live` mode. In batch mode, only `when` conditions evaluate against the sequential updates in your file.

### ❌ `.id` property is informal

The engine resolves `.id` to the instance name at runtime (engine.rs line 318), but `.id` is NOT declared in the entity schema. The analyzer may not know about it. You can use it, but it's safer to add a `name: Text` field explicitly.

```lumina
-- Safer pattern:
entity Server {
    name: Text
    cpu_temp: Number
}
let web01 = Server { name: "web-01", cpu_temp: 45 }
show "Server: {web01.name}"

-- Also works at runtime but less safe:
show "Server: {web01.id}"
```

---

## Cluster Features

### ⚠️ Cluster is functional but early

The cluster networking (gossip, leader election, state mesh) is implemented and works, but:

- Migration is best-effort (no guaranteed delivery)
- `evacuate` picks the first alive peer (no load-balancing)
- `deploy` is simplified (evaluates spec but doesn't have full deployment logic)

### ❌ No automatic cluster discovery

You must manually list all `peers` in the cluster block. There is no mDNS or DNS-SD discovery.

### ❌ No cross-node rule execution

Rules only execute on the local node. A rule on node-1 cannot directly trigger actions on node-2. Use `migrate` for workload movement.

---

## Features Mentioned in Old Docs That Don't Exist

These features appear in the old `master_knowledge.md` or pattern descriptions but are **NOT implemented** in the v2.0 runtime:

| Feature | Status | Notes |
|---------|--------|-------|
| `trend(field, duration)` | ❌ Not implemented | Use `prev()` as workaround |
| `@secret` taint analysis (L050) | ❌ Not implemented | L051 (secret in derived) exists, but no flow analysis |
| `Secret` leak detection to alerts | ❌ Not implemented | Secrets display as `***SECRET***` but no sink analysis |
| KMS integration | ❌ Not implemented | `env()` reads OS environment variables directly |
| eBPF integration | ❌ Roadmap (v3.0) | Not available |
| Natural Language compiler | ❌ Roadmap (v3.0) | Not available |
| Edge Federation | ❌ Roadmap (v3.0) | Not available |
| Blake3 Truth Log | ⚠️ Designed, not wired | The concept exists but WAL isn't cryptographically chained in runtime |
| `@affects` metadata | ⚠️ Parsed, not enforced | The field is in the AST but has no runtime effect |

---

## Type System Gaps

### ❌ No automatic type coercion

`42 + "hello"` is a runtime error (R018), not a string concatenation. You must explicitly convert:

```lumina
-- ❌ BAD:
let result = 42 + " items"    -- R018

-- ✅ FIX: Use interpolation
let result = "{42} items"
```

### ❌ No null/nil/optional type

Fields always have a value. External entity fields default to `Unknown` until data arrives, but there's no way to explicitly create or check for Unknown in user code.

---

## Performance Notes

- **Designed for**: Hundreds to low thousands of entity instances
- **Tested at**: ~1000 instances with responsive performance
- **Theoretical limit**: FxHashMap allows 100K+ entities, but aggregates recompute fully
- **WASM performance**: ~90% of native speed
- **Timer resolution**: Millisecond-level (not microsecond)


---

# Lumina Project Templates

Full project architectures for common use cases. Use these as starting points for real projects.

---

## Template 1: IoT Sensor Fleet Monitor

**Use case**: Monitor a fleet of temperature/humidity sensors, alert on anomalies, track fleet health.

```lumina
-- ============================================================
-- IoT Sensor Fleet Monitor
-- Monitors N sensors, alerts on overheating, tracks fleet stats
-- ============================================================

-- 1. ENTITIES

entity Sensor {
    name: Text
    location: Text
    temperature: Number
    humidity: Number
    is_online: Boolean
    is_overheating := temperature > 80
    is_humid := humidity > 85
    status := if not is_online then "OFFLINE"
              else if is_overheating then "CRITICAL"
              else if is_humid then "WARNING"
              else "OK"
}

-- 2. AGGREGATES

aggregate FleetHealth over Sensor {
    avg_temp := avg(temperature)
    max_temp := max(temperature)
    min_temp := min(temperature)
    online_count := count(is_online)
    hot_count := count(is_overheating)
    any_critical := any(is_overheating)
    all_online := all(is_online)
}

-- 3. HELPER FUNCTIONS

fn temp_label(t: Number) -> Text {
    if t > 90 then "DANGER"
    else if t > 80 then "HIGH"
    else if t > 60 then "NORMAL"
    else "COOL"
}

-- 4. RULES

rule "Sensor Overheat" for (s: Sensor)
when Sensor.is_overheating becomes true {
    alert
        severity: "critical",
        message: "{Sensor.name} at {Sensor.location}: {Sensor.temperature}C",
        source: "ThermalMonitor"
}
on clear {
    show "RESOLVED: {Sensor.name} temperature normalized"
}

rule "Sensor Offline" for (s: Sensor)
when Sensor.is_online becomes false {
    alert
        severity: "warning",
        message: "{Sensor.name} went offline",
        source: "ConnectivityMonitor"
}
on clear {
    show "RECOVERED: {Sensor.name} back online"
}

rule "Fleet Emergency"
when FleetHealth.hot_count > 2 {
    alert
        severity: "critical",
        message: "Multiple sensors overheating! Count: {FleetHealth.hot_count}",
        source: "FleetMonitor"
}

-- 5. INSTANCES

let s1 = Sensor {
    name: "Sensor-A1", location: "Warehouse North",
    temperature: 45, humidity: 60, is_online: true
}
let s2 = Sensor {
    name: "Sensor-A2", location: "Warehouse South",
    temperature: 50, humidity: 55, is_online: true
}
let s3 = Sensor {
    name: "Sensor-B1", location: "Server Room",
    temperature: 55, humidity: 40, is_online: true
}
let s4 = Sensor {
    name: "Sensor-B2", location: "Server Room",
    temperature: 48, humidity: 42, is_online: true
}

-- 6. SIMULATION

show "=== FLEET STATUS ==="
show "Online: {FleetHealth.online_count}"
show "Avg Temp: {FleetHealth.avg_temp}C"
show "Max Temp: {FleetHealth.max_temp}C"

show "=== THERMAL EVENT ==="
update s3.temperature = 88
update s4.temperature = 85

show "Hot sensors: {FleetHealth.hot_count}"

show "=== SENSOR OFFLINE ==="
update s1.is_online = false

show "=== RECOVERY ==="
update s3.temperature = 55
update s4.temperature = 50
update s1.is_online = true
```

### How to extend:
- **Add new sensor**: Copy a `let` statement, change name/location/values
- **Add humidity alerts**: Create a `rule "Humidity Alert"` with `when Sensor.is_humid becomes true`
- **Add external data**: Replace `entity Sensor` with `external entity Sensor` and add sync config

---

## Template 2: Server Rack Health System

**Use case**: Monitor servers with cooling units, cross-entity references, power management.

```lumina
-- ============================================================
-- Server Rack Health System
-- Entities: CoolingUnit → Server → Rack (with refs)
-- ============================================================

-- 1. ENTITIES (in dependency order — referenced entities first)

entity CoolingUnit {
    name: Text
    power_watts: Number
    is_active: Boolean
    efficiency := if is_active then 100 - power_watts else 0
    is_failing := power_watts > 50 and not is_active
}

entity Server {
    name: Text
    cpu_temp: Number
    ram_usage: Number
    is_online: Boolean
    cooling: ref CoolingUnit
    is_overheating := cpu_temp > 80
    is_ram_full := ram_usage > 90
    is_critical := is_overheating and cooling.is_failing
    health := if is_critical then "CRITICAL"
              else if is_overheating then "WARNING"
              else if not is_online then "OFFLINE"
              else "HEALTHY"
}

-- 2. AGGREGATES

aggregate RackMetrics over Server {
    avg_temp := avg(cpu_temp)
    avg_ram := avg(ram_usage)
    online := count(is_online)
    critical_count := count(is_critical)
}

-- 3. RULES

rule "Thermal Warning" for (s: Server)
when Server.is_overheating becomes true {
    alert severity: "warning",
        message: "Server {Server.name}: {Server.cpu_temp}C (cooling: {Server.cooling.name})"
}

rule "CRITICAL ALERT" for (s: Server)
when Server.is_critical becomes true {
    alert severity: "critical",
        message: "EMERGENCY: {Server.name} overheating AND {Server.cooling.name} failing!"
}
on clear {
    show "RESOLVED: {Server.name} critical state cleared"
}

rule "Cooling Failure" for (c: CoolingUnit)
when CoolingUnit.is_failing becomes true {
    alert severity: "critical",
        message: "Cooling unit {CoolingUnit.name} is failing!"
}

rule "Rack Stress"
when RackMetrics.critical_count > 0 {
    show "RACK ALERT: {RackMetrics.critical_count} servers in critical state"
    show "Avg temp: {RackMetrics.avg_temp}C, Online: {RackMetrics.online}"
}

-- 4. INSTANCES

let cooler_a = CoolingUnit { name: "CRAC-A", power_watts: 30, is_active: true }
let cooler_b = CoolingUnit { name: "CRAC-B", power_watts: 25, is_active: true }

let web1 = Server {
    name: "web-01", cpu_temp: 45, ram_usage: 60,
    is_online: true, cooling: cooler_a
}
let web2 = Server {
    name: "web-02", cpu_temp: 50, ram_usage: 55,
    is_online: true, cooling: cooler_a
}
let db1 = Server {
    name: "db-01", cpu_temp: 55, ram_usage: 70,
    is_online: true, cooling: cooler_b
}

-- 5. SIMULATION

show "=== INITIAL STATE ==="
show "Rack avg temp: {RackMetrics.avg_temp}C"
show "Online servers: {RackMetrics.online}"

show "=== THERMAL SPIKE ON web-01 ==="
update web1.cpu_temp = 88

show "=== COOLING FAILURE ON CRAC-A ==="
update cooler_a.is_active = false

show "=== FULL RECOVERY ==="
update cooler_a.is_active = true
update web1.cpu_temp = 45
```

---

## Template 3: Security Monitoring System

**Use case**: Zero-trust network monitoring with quarantine and recovery.

```lumina
-- ============================================================
-- Zero-Trust Security Monitor
-- Auto-quarantine compromised nodes, auto-restore on recovery
-- ============================================================

entity NetworkNode {
    name: Text
    ip_address: Text
    security_score: Number
    vlan: Number
    failed_logins: Number
    is_quarantined := vlan == 999
    is_compromised := security_score < 40
    is_suspicious := security_score < 60 and security_score >= 40
    threat_level := if is_compromised then "CRITICAL"
                    else if is_suspicious then "ELEVATED"
                    else "NORMAL"
}

aggregate SecurityOverview over NetworkNode {
    avg_score := avg(security_score)
    quarantined_count := count(is_quarantined)
    compromised_count := count(is_compromised)
    any_compromised := any(is_compromised)
}

-- Auto-quarantine compromised nodes
rule "Auto-Quarantine" for (n: NetworkNode)
when NetworkNode.is_compromised becomes true {
    show "QUARANTINE: {NetworkNode.name} ({NetworkNode.ip_address}) score={NetworkNode.security_score}"
    update NetworkNode.vlan = 999
    alert severity: "critical",
        message: "Node {NetworkNode.name} quarantined: security score {NetworkNode.security_score}"
}
on clear {
    show "RESTORED: {NetworkNode.name} returned to production"
    update NetworkNode.vlan = 1
}

-- Alert on suspicious activity
rule "Suspicious Activity" for (n: NetworkNode)
when NetworkNode.is_suspicious becomes true {
    alert severity: "warning",
        message: "Elevated threat: {NetworkNode.name} score={NetworkNode.security_score}"
}

-- Fleet-wide breach alert
rule "Breach Alert"
when SecurityOverview.compromised_count > 1 {
    alert severity: "critical",
        message: "MULTI-NODE BREACH: {SecurityOverview.compromised_count} nodes compromised!"
}

-- Instances
let node1 = NetworkNode {
    name: "web-prod-01", ip_address: "10.0.1.10",
    security_score: 92, vlan: 1, failed_logins: 0
}
let node2 = NetworkNode {
    name: "db-prod-01", ip_address: "10.0.1.20",
    security_score: 88, vlan: 1, failed_logins: 0
}
let node3 = NetworkNode {
    name: "api-prod-01", ip_address: "10.0.1.30",
    security_score: 95, vlan: 1, failed_logins: 0
}

-- Simulation
show "=== INITIAL STATE ==="
show "Avg security: {SecurityOverview.avg_score}"
show "Quarantined: {SecurityOverview.quarantined_count}"

show "=== ATTACK SIMULATION ==="
update node1.security_score = 35
update node1.failed_logins = 50

show "Node1 VLAN: {node1.vlan} (should be 999)"
show "Node1 quarantined: {node1.is_quarantined}"

show "=== SECOND NODE COMPROMISED ==="
update node2.security_score = 25

show "=== RECOVERY ==="
update node1.security_score = 85
show "Node1 VLAN after recovery: {node1.vlan} (should be 1)"
```

---

## Template 4: Application Health Checker (Simple)

**Use case**: Quick health check for a web application stack.

```lumina
-- ============================================================
-- Application Health Checker
-- Simple stack: LB → App → DB
-- ============================================================

entity Service {
    name: Text
    response_time_ms: Number
    error_rate: Number
    is_healthy: Boolean
    is_slow := response_time_ms > 500
    is_degraded := error_rate > 5
}

aggregate StackHealth over Service {
    avg_response := avg(response_time_ms)
    avg_errors := avg(error_rate)
    all_healthy := all(is_healthy)
    any_degraded := any(is_degraded)
}

fn health_icon(healthy: Boolean) -> Text {
    if healthy then "✅" else "❌"
}

rule "Service Degraded" for (s: Service)
when Service.is_degraded becomes true {
    alert severity: "warning",
        message: "{Service.name} error rate: {Service.error_rate}%"
}

rule "Stack Unhealthy"
when StackHealth.all_healthy becomes false {
    alert severity: "critical", message: "Stack health compromised!"
}

let lb = Service { name: "Load Balancer", response_time_ms: 12, error_rate: 0.1, is_healthy: true }
let app = Service { name: "App Server", response_time_ms: 45, error_rate: 0.5, is_healthy: true }
let db = Service { name: "Database", response_time_ms: 8, error_rate: 0.0, is_healthy: true }

show "╔═══════════════════════════════╗"
show "║  Stack Health Dashboard       ║"
show "╠═══════════════════════════════╣"
show "║ {health_icon(lb.is_healthy)} LB:  {lb.response_time_ms}ms  err:{lb.error_rate}%"
show "║ {health_icon(app.is_healthy)} App: {app.response_time_ms}ms  err:{app.error_rate}%"
show "║ {health_icon(db.is_healthy)} DB:  {db.response_time_ms}ms  err:{db.error_rate}%"
show "╠═══════════════════════════════╣"
show "║ Avg Response: {StackHealth.avg_response}ms"
show "║ All Healthy: {StackHealth.all_healthy}"
show "╚═══════════════════════════════╝"

show "=== SIMULATING DB ISSUES ==="
update db.error_rate = 12
update db.is_healthy = false
```

---

## Project Architecture Guidelines

### File Organization

For larger projects, split across multiple `.lum` files:

```
my_project/
├── entities.lum        -- All entity definitions
├── aggregates.lum      -- All aggregate declarations
├── functions.lum       -- Helper functions
├── rules/
│   ├── thermal.lum     -- Temperature-related rules
│   ├── security.lum    -- Security rules
│   └── fleet.lum       -- Fleet-wide rules
├── instances.lum       -- Instance creation
└── main.lum            -- Imports + simulation
```

Use `import "path/to/file.lum"` to connect them.

### Naming Conventions

| Item       | Convention       | Example                    |
|-----------|-----------------|----------------------------|
| Entity    | PascalCase       | `Server`, `CoolingUnit`    |
| Field     | snake_case       | `cpu_temp`, `is_online`    |
| Instance  | camelCase or descriptive | `mainServer`, `web01` |
| Rule      | Quoted descriptive | `"Thermal Warning"`       |
| Function  | snake_case       | `clamp`, `severity_label`  |
| Aggregate | PascalCase       | `RackStats`, `FleetHealth` |

### Entity Definition Order

Define entities in dependency order — referenced entities FIRST:

```lumina
-- ✅ CORRECT ORDER:
entity CoolingUnit { ... }          -- referenced by Server
entity Server { cooling: ref CoolingUnit ... }  -- references CoolingUnit

-- ❌ WRONG ORDER:
entity Server { cooling: ref CoolingUnit ... }  -- CoolingUnit not defined yet!
entity CoolingUnit { ... }
```

### Rule Organization

1. **Alert rules** first (most important)
2. **Recovery rules** (`on clear`) attached to alert rules
3. **Periodic rules** (`every`) for maintenance
4. **Fleet rules** last (aggregate-dependent)

### Testing Pattern

Always include a simulation section at the bottom:

```lumina
-- ============ SIMULATION ============
show "=== INITIAL STATE ==="
-- show initial values

show "=== EVENT 1: Description ==="
update instance.field = value
-- show state after change

show "=== RECOVERY ==="
-- restore to normal
-- verify on clear fired
```


---

# Lumina CLI Reference

Every command available in the `lumina` binary, with usage, flags, and examples.

---

## Installation

### Linux / macOS

```bash
curl -fsSL https://lumina-lang.web.app/install.sh | sh && . ~/.lumina/env
```

### Windows (PowerShell)

```powershell
iwr https://lumina-lang.web.app/install.ps1 -useb | iex
```

### Homebrew (macOS)

```bash
brew tap IshimweIsaac/lumina
brew install lumina
```

### Verify Installation

```bash
lumina --version
```

---

## Commands

### `lumina run <file.lum>`

Execute a Lumina program.

```bash
lumina run myfile.lum
```

**Flags:**
- `--node-id <id>` — Override the cluster node ID (for multi-node testing)

The program runs sequentially through all statements. If `every` or `for` timers are present, the engine enters live mode and ticks in real-time until interrupted with `Ctrl+C`.

---

### `lumina check <file.lum>`

Type-check and analyze a program without running it.

```bash
lumina check myfile.lum
```

Output: `✓ myfile.lum — no errors found` on success, or detailed diagnostics on failure.

Use this for CI pipelines and pre-commit hooks.

---

### `lumina fmt <file.lum>`

Format a Lumina source file in-place using the canonical style.

```bash
lumina fmt myfile.lum
```

Output: `✓ myfile.lum — formatted`

---

### `lumina repl`

Start an interactive Read-Eval-Print Loop.

```bash
lumina repl
```

Type Lumina expressions and statements interactively. Multi-line input is supported with brace-depth tracking. Type `:help` to see inspector commands.

---

### `lumina update`

Update Lumina to the latest version. This replaces both `lumina` and `lumina-lsp` binaries in-place.

```bash
lumina update
```

**Flags:**
- `--check` — Only check if an update is available, don't download
- `--force` — Re-download and reinstall even if already on the latest version (useful for repairing corrupted installs)

**How it works:**
1. Queries the GitHub Releases API for the latest version tag
2. Compares against the currently installed version
3. Downloads the correct platform-specific binaries (with SHA256 verification)
4. Atomically replaces the running binaries

**Examples:**

```bash
# Check if there's a new version
lumina update --check

# Update to the latest version
lumina update

# Force-reinstall the current version
lumina update --force
```

**Notes:**
- Requires `curl` to be available on your system
- On Windows, the current binary is renamed to `.old` before replacement (standard self-update pattern)
- On macOS, the quarantine flag is automatically removed

---

### `lumina setup`

Automated IDE and environment setup. Detects installed editors and installs the Lumina extension.

```bash
lumina setup
```

This command runs automatically during installation. It scans for:
- **VS Code-compatible editors**: VS Code, VSCodium, Cursor, Windsurf, Positron, Code Insiders, Code OSS
- **Neovim**: Auto-generates a zero-config LSP plugin at `~/.config/nvim/plugin/lumina.lua`

The extension provides syntax highlighting, live diagnostics, go-to-definition, and find-all-references via the built-in `lumina-lsp` language server.

---

### `lumina uninstall`

Remove Lumina from your system.

```bash
lumina uninstall
```

This command:
1. Uninstalls the VS Code extension from all detected editors
2. Removes the `~/.lumina` directory (binaries and environment)
3. Cleans PATH entries from shell profiles (`.bashrc`, `.zshrc`, `.profile`, etc.)

---

### `lumina get documentation`

Output the master knowledge atlas to the current directory for AI-assisted development.

```bash
lumina get documentation
```

Creates `master_knowledge.md` in the current working directory. This file contains the full Lumina technical reference — designed to be ingested by AI coding assistants for context-aware code generation.

---

### `lumina query <expression>`

Evaluate a standalone Lumina expression.

```bash
lumina query "1 + 2 + 3"
```

Useful for quick calculations and testing expressions without creating a full `.lum` file.

---

### `lumina provider <command>`

Manage external data providers.

```bash
lumina provider list          # List installed providers
lumina provider install <name>   # Install a provider (registry coming soon)
```

Built-in protocol support: `redfish`, `snmp`, `modbus`.

---

### `lumina cluster <command>`

Cluster management for distributed Lumina nodes.

```bash
lumina cluster start <spec.lum> [node_id]   # Start a cluster node
lumina cluster status                        # Show cluster status
lumina cluster join <address>                # Join an existing cluster
```

See [08-advanced-features.md](08-advanced-features.md) for cluster configuration details.

---

### `lumina --version`

Print the version string.

```bash
lumina --version
# Output: Lumina v2.0.0: The Cluster Release
```

Also available as `lumina version` and `lumina -v`.

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `LUMINA_HOME` | `~/.lumina` | Root directory for Lumina installation |
| `LUMINA_SKIP_CHECKSUM` | `0` | Set to `1` to skip SHA256 verification during install |

---

## File Locations

| Path | Contents |
|------|----------|
| `~/.lumina/bin/lumina` | CLI binary |
| `~/.lumina/bin/lumina-lsp` | Language server binary |
| `~/.lumina/env` | Shell environment file (sourced by your shell profile) |


---

# Infrastructure Patterns (v2.1)

With the introduction of external adapters and the `provision`/`terminate` actions in v2.1, Lumina can now manage real-world infrastructure lifecycles autonomously.

## The Hydra Fleet Pattern (Auto-Scaling & Self-Healing)

The "Hydra Fleet" is a robust architectural pattern for building autonomous infrastructure that scales up during traffic peaks, scales down to save costs, and heals itself if nodes fail.

### Step 1: Define the Infrastructure Entities

```lumina
resource entity Service {
  name: String
  image: String
  status: String
  verified: Boolean
  tier: String
}

entity TrafficSensor {
  load: Number
  active_workers: Number
}
```

### Step 2: Establish the Initial State (Genesis)

```lumina
let database = Service { name: "hydra-db", status: "unknown", verified: false, tier: "db" }
let worker_1 = Service { name: "hydra-app-01", status: "unknown", verified: false, tier: "app" }
let worker_2 = Service { name: "hydra-app-02", status: "unknown", verified: false, tier: "app" }

let cluster_monitor = TrafficSensor { load: 10, active_workers: 1 }

rule "Genesis" when (database.status == "unknown") {
  update database.status = "starting"
  update worker_1.status = "starting"
  provision database
  provision worker_1
}
```

### Step 3: Implement Self-Healing

The Self-Healing law ensures core services remain online. We use the `whenever` trigger to continuously enforce this state, while intentionally ignoring dynamic nodes (like `worker_2`) that are managed by the auto-scaler.

```lumina
rule "Self-Healing" for (s: Service)
whenever (s.status == "stopped" and s.name != "hydra-app-02") {
  show "HEALING: {s.name} went down! Re-initializing..."
  update s.status = "running"
  provision s
}
```

### Step 4: Implement Auto-Scaling

The scaling rules use the level-triggered `whenever` block to detect traffic states and scale the dynamic workers accordingly.

```lumina
rule "Scale Up" cooldown 30s
whenever (cluster_monitor.load > 80 and worker_2.status != "running") {
  show "MONITOR: High Load detected. Provisioning worker_2..."
  update worker_2.status = "running"
  update cluster_monitor.active_workers = 2
  provision worker_2
}

rule "Scale Down" cooldown 30s
whenever (cluster_monitor.load < 30 and worker_2.status == "running") {
  show "MONITOR: Low Load detected. Terminating worker_2..."
  update worker_2.status = "stopped"
  update cluster_monitor.active_workers = 1
  terminate worker_2
}
```

### Step 5: Global Reconciliation

Finally, keep the engine synchronized with reality by polling the external adapter (e.g., Docker) periodically.

```lumina
global rule "Fleet Sync" every 5s {
  reconcile Service
}
```

