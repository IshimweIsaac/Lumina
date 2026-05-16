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
