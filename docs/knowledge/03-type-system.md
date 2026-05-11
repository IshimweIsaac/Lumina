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
