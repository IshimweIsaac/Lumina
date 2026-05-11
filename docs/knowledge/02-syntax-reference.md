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
