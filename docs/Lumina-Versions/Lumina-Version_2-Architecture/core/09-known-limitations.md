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
