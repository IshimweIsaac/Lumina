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
