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
