**LUMINA**

**v1.8 Deep Documentation**

**The Deep Memory & Audit Release**

_lumina-store | Truth Log | Provider Model | Historical Operators | WAL_

_"Describe what is true. Lumina figures out what to do."_

_2026 | Chapters 49-54 | Builds on v1.8 | Designed and authored by Isaac Ishimwe_

**Why v1.8**

**The Deep Memory Release**

_What v1.8 could not yet remember — and why v1.8 builds its memory natively_

By the end of v1.8, Lumina's reactive engine was perfect at evaluating the current tick. It could see the previous tick with `prev()`, and it had a world-class distribution system. But to truly rule the data center, Lumina needs to be able to look back an hour, a day, or a week, without relying on an external time-series database.

The gap was persistence. Lumina could react to a flapping sensor, but it couldn't tell you the average temperature over the last 24 hours unless an external system told it first. Furthermore, there was no immutable record of *why* Lumina took an action.

v1.8 closes this gap by giving Lumina native memory, an audit trail, and a provider ecosystem, ensuring it remains zero-dependency and edge-ready without compromising on its core philosophy.

# **The Three Gaps v1.8 Closes**

| **Gap in v1.8** | **v1.8 solution** |
| --- | --- |
| Relies on external DBs for long-term trends | `lumina-store` — Embedded time-series ring-buffers |
| No cryptographic proof of rule evaluations | Truth Log — Immutable audit trail of state transitions |
| Adapters are hardcoded in Rust source | Lumina Provider Model — A stable external WASM plugin API |

---

## **Chapter 49**
## **lumina-store: Native Memory**

_"Lumina doesn't use a database. It has native memory."_

A huge philosophical breakthrough in v1.8 is that Lumina's storage is dictated by its rules. Because the `lumina-analyzer` knows exactly what time windows the rules require at compile time (`avgOver(temp, 4h)`), `lumina-store` knows exactly how much data to persist via the `RetentionManifest`.

### **The Architecture**
`lumina-store` is built around per-entity **fixed-size circular arrays (Ring Buffers)**. Memory allocation is perfectly bounded *before* the runtime even starts.

```rust
store.write(entity_id, field, value, timestamp);
store.latest(entity_id, field) -> Value;
store.avg_over(entity_id, field, duration) -> f64;
store.trend(entity_id, field, duration) -> f64;
```

**The Crate Structure:**
```
lumina-store/
  ├── retention.rs    // Parses the RetentionManifest from the compiler
  ├── ring_buffer.rs  // O(1) sliding window data structures
  ├── window.rs       // Computation for max/min/avg/trend
  ├── wal.rs          // Write-Ahead Log for crash survival
  ├── compaction.rs   // Handles TTL
  └── store.rs        // Main runtime API
```

---

## **Chapter 50**
## **The Truth Log**

_Immutable proof of reality._

When a chiller is commanded to shut down, engineers need to know *why*. The Truth Log is an append-only, structured file (embedded SQLite/DuckDB) recording the exact mathematical proof of every action.

### **The Cryptographic Proof**
Every entry contains a Blake3 hash that includes the signature of the *previous* entry. If an attacker tampers with a log file to hide a write action, every subsequent checksum breaks.

```rust
TruthLogEntry {
  timestamp_ms:     u64,
  rule_id:          String,
  action_type:      Enum(Write, Alert, SecurityViolation),
  target_entity:    String,
  target_field:     Option<String>,
  new_value:        Option<Value>,
  dag_snapshot:     Vec<FieldValue>,  // The PERFECT snapshot of reality
  checksum:         Blake3Hash,       // Cryptographic proof + chained hash
}
```

### **Querying the Truth Log**
Lumina provides native CLI commands to audit the log:
```bash
lumina truth query --rule thermal_pressure --from 24h
lumina truth verify --from 24h  # cryptographic verification!
lumina truth export --format json --from 7d
```

---

## **Chapter 51**
## **The Provider Model**

_Breaking out of the core compiler._

Before v1.8, if you wanted to talk to a physical device, you wrote an adapter inside the Lumina compiler. Now, providers are Edge-deployable **WASM Components**. They are language-agnostic, sandboxed, and require zero daemons.

### **The WASM Interface**
Every Lumina Provider implements three functions:
```rust
trait LuminaProvider {
  // Used by the compiler to validate .lum files
  fn discover() -> ProviderSchema;
  
  // Called by the runtime every tick
  fn poll() -> Vec<EntityUpdate>;
  
  // Translates Lumina's intent to the real world
  fn write(target: EntityRef, field: &str, value: Value) -> WriteResult;
}
```

### **Syntax and CLI**
Providers are native syntax. First install your WASM file: `lumina provider install ./redfish.wasm`. Then map it directly in code:

```lumina
provider "redfish" {
  endpoint: "https://192.168.1.10/redfish/v1"
  credentials: env("REDFISH_TOKEN")
}

external entity Server from "redfish" {
  cpu_percent: Float
  temperature_c: Float
}
```

---

## **Chapter 52**
## **Historical Operators**

_Time travel natively in the DAG._

Using the ring-buffers provided by `lumina-store`, Lumina variables can now query history. Time units supported: `s` (seconds), `m` (minutes), `h` (hours), `d` (days), `w` (weeks).

```lumina
-- Averages
avgOver(server.temperature_c, 24h)

-- Trends (positive = rising, negative = falling)
trend(rack.temperature_c, 6h)

-- Percentiles
p95(server.latency_ms, 1h)

-- Bounds
maxOver(server.cpu_percent, 4h)
minOver(server.temperature_c, 24h)
```

**Rule Example:**
```lumina
rule thermal_trend_critical {
  when trend(server.temperature_c, 6h) > 0.3
    and avgOver(server.cpu_percent, 2h) > 75.0
    and p95(server.latency_ms, 1h) > 200.0
  cooldown 20m
  alert "Converging pressure on {server.id} — thermal, CPU, and latency trending badly."
}
```

---

## **Chapter 53**
## **WAL & Recovery**

_Surviving the blackout._

Lumina-store holds memory in RAM (Ring Buffers). To survive power loss or restarts, every tick appends to a fast Write-Ahead Log (WAL).

```rust
WalEntry {
  sequence:    u64,
  timestamp:   u64,
  entity_id:   String,
  field_name:  String,
  value:       Value,
  checksum:    u32, // CRC32 of contents
}
```

**The Startup Sequence:**
1. Compiler produces `RetentionManifest`.
2. Runtime reads the manifest and allocates the memory.
3. WAL replays sequentially.
4. Old entries outside retention window are discarded.
5. First tick evaluates precisely where the system left off.

```bash
lumina runtime --verify-wal   # Check integrity
lumina runtime --rebuild-wal  # Rebuild from clean checkpoint
```

---

## **Chapter 54**
## **Provider Architecture Deep Dive**

_The community ecosystem._

Providers define themselves with a `provider.toml` manifest:
```toml
[provider]
name = "redfish"
version = "1.0.0"
author = "community"
protocols = ["https"]
capabilities = ["poll", "write", "discover"]
```

Because `discover()` returns a strict schema, if a provider offers `cpu_percent` and your Lumina code queries `cpu_celsius`, the **compiler immediately fails build**. You cannot deploy broken reality.

---

**Appendix:**
## **New Error Codes**

| Code | Meaning |
|---|---|
| L035 | Invalid time window (zero or negative). Minimum is 1s. |
| L036 | Time window exceeds maximum retention limit set by the system. |
| L037 | Historical operator used on derived field — only stored fields supported. |
| L038 | Provider field not found in `discover()` schema. |
| L040 | WAL corruption detected. |
| L041 | Provider WASM interface version mismatch. |
