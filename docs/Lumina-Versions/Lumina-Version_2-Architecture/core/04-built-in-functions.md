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
