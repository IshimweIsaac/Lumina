# Lumina v2.1.1 — "The Docker Adapter Release"

> **Release Type:** Minor · **Status:** Current Stable · **Codename:** Architect  
> **Minimum Rust Version:** 1.75+ · **Breaking Changes:** None from v2.1.0

---

## Overview

Lumina v2.1.1 marks the official launch of the **Infrastructure Provisioning Layer**. With this release, Lumina evolves from a reactive rules engine into a full **Infrastructure-as-Code** orchestrator — one that doesn't just *observe* your infrastructure, but *manages* it.

The flagship feature is the native **Docker Adapter**, which communicates directly with the Docker Daemon via Unix sockets. Combined with the new `resource entity` syntax and lifecycle actions (`provision`, `reconcile`, `destroy`), Lumina can now declare, create, monitor, and tear down containers without a single line of shell script or `docker-compose`.

---

## What's New

### 1. The `resource entity` Keyword

A new top-level declaration that binds a Lumina entity to a real-world infrastructure provider. Unlike standard `entity` (pure reactive state) or `external entity` (read-only sensor bridge), a `resource entity` declares **desired state** that the engine actively enforces.

```lumina
resource entity WebApp provider "docker" {
  image: Text
  port: Number
  target_port: Number
  status: Text
  env_vars: Text

  ensure {
    image: "nginx:alpine"
    port: 8080
    target_port: 80
    status: "running"
    env_vars: "NODE_ENV=production,LOG_LEVEL=info"
  }
}
```

**Key properties:**
- `provider` — selects which adapter handles the lifecycle (currently `"docker"`).
- `ensure` — a block of field-value pairs representing the target configuration. The engine compares desired vs. actual on every `reconcile` call.
- All standard entity features (derived fields, aggregates, rules) still apply.

### 2. Lifecycle Actions

Three new first-class actions are available in rule bodies and top-level statements:

| Action | Purpose | Adapter Method Called |
|---|---|---|
| `provision <instance>` | Create the resource if it doesn't exist, or start it if stopped | `adapter.provision(instance, desired_state)` |
| `reconcile <instance>` | Compare desired vs. actual state and converge | `adapter.reconcile(instance, desired_state)` |
| `destroy <instance>` | Force-remove the resource | `adapter.destroy(instance)` |

```lumina
let app = WebApp {
  image: "nginx:alpine"
  port: 8080
  target_port: 80
  status: "pending"
  env_vars: "NONE"
}

-- One-shot: bring the container to life
provision app

-- Periodic drift correction
rule "Self-Heal WebApp" every 30s {
  reconcile app
}

-- Teardown on critical failure
rule "Emergency Shutdown" when WebApp.status becomes "failed" {
  destroy app
  alert severity: "critical", message: "WebApp torn down after failure"
}
```

### 3. The Docker Adapter

A native Rust adapter (`docker_adapter.rs`) that communicates with the local Docker Daemon via the [`bollard`](https://crates.io/crates/bollard) crate over Unix sockets. It completely replaces the need for `docker-compose` or shell-based container management.

**Capabilities:**

| Feature | Details |
|---|---|
| **Image Pulling** | Automatically pulls required images via the Docker Registry API before creating containers. |
| **Port Binding** | Maps `port` (host) → `target_port` (container) with configurable host IP binding. |
| **Environment Variables** | Injects comma-separated `env_vars` into container configuration. |
| **State Polling** | Reads container state on every tick, mapping Docker statuses (`running`, `exited`, `dead`, `created`, `restarting`) to Lumina values (`running`, `stopped`, `starting`, `down`). |
| **Idempotent Provisioning** | If the container already exists, it synchronizes state instead of failing. |
| **Force Destroy** | Removes containers with `force: true` to handle stuck processes. |

**State mapping from Docker → Lumina:**
```
running       → "running"
exited, dead  → "stopped"
created, restarting → "starting"
(anything else)     → "down"
```

### 4. The `LuminaAdapter` Trait

The Docker Adapter is built on a generalized trait that any provider can implement. This means v2.1.1 establishes the **adapter pattern** for future providers (AWS EC2, Kubernetes, etc.):

```rust
pub trait LuminaAdapter: Send + Sync {
    fn entity_name(&self) -> &str;
    fn poll(&mut self) -> Vec<(String, String, Value)>;
    fn on_write(&mut self, instance: &str, field: &str, value: &Value) {}
    fn provision(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> { Ok(()) }
    fn destroy(&mut self, instance: &str) -> Result<(), String> { Ok(()) }
    fn reconcile(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> { Ok(()) }
}
```

### 5. The `trace` Debugger

A new `trace` action for real-time rule evaluation debugging. When trace mode is enabled (`lumina run --trace`), `trace` expressions output the evaluation context and value at that point in execution:

```lumina
rule "Debug Fleet" when RackStats.avg_temp > 70 {
  trace RackStats.avg_temp
  trace RackStats.critical_count
  alert severity: "warning", message: "Fleet warming: {RackStats.avg_temp}C"
}
```

Output:
```
[TRACE] global | value: 82.5
[TRACE] global | value: 3
[ALERT:warning] -- Fleet warming: 82.5C
```

### 6. The `global` Keyword for Broadcast Safety

Rules that operate on aggregates or non-instance state must now be explicitly marked with `global` to prevent accidental fleet-wide side effects:

```lumina
global rule "Fleet Alert"
when RackStats.critical_count > 2 {
  alert severity: "critical", message: "Fleet overheating"
}
```

Without `global`, a rule without a parameter binding is still allowed but will only fire in instance-specific contexts — preventing the "Broadcast Footgun" where a rule accidentally fires once per instance.

### 7. `for` Iterator in Action Blocks

A new `for` action allows iterating over all instances of an entity within a rule body:

```lumina
global rule "Shutdown All Hot Servers"
when RackStats.critical_count > 5 {
  for s in Server {
    update s.is_online = false
  }
  alert severity: "critical", message: "Emergency fleet shutdown"
}
```

---

## Native Adapters

v2.1.1 ships with a full suite of native Rust-based adapters (the **Standard Sensory Library**), eliminating the Python bridge overhead from earlier versions:

| Adapter | Module | Purpose |
|---|---|---|
| **Docker** | `docker_adapter.rs` | Container lifecycle management |
| **HTTP** | `http_adapter.rs` | REST API polling and webhooks |
| **MQTT** | `mqtt_adapter.rs` | IoT message broker integration |
| **File** | `file_adapter.rs` | Filesystem monitoring |
| **Ping** | `ping_adapter.rs` | Network reachability checks |
| **Process** | `process_adapter.rs` | OS process monitoring |
| **Static** | `static_adapter.rs` | Test fixtures and mock data |

> **Note:** `mqtt_adapter` is not available on Windows. `docker_adapter`, `http_adapter`, `ping_adapter`, `process_adapter`, and `file_adapter` are excluded from WASM builds.

---

## Cluster Foundation (inherited from v2.1.0)

v2.1.1 builds on the distributed clustering engine introduced in v2.1.0:

- **Gossip Protocol** — UDP-based peer discovery, health monitoring, and message routing (`gossip.rs`, `transport.rs`).
- **Leader Election** — Raft-inspired quorum-based promotion (`election.rs`).
- **State Mesh** — Distributed state resolution with version vectors and last-writer-wins conflict resolution (`state_mesh.rs`).
- **Write-Ahead Log** — File-backed persistence with replay and compaction (`wal.rs`).
- **Orchestration Expressions** — `migrate`, `evacuate`, and `deploy` for workload management.
- **Aggregate Scoping** — Aggregates can be scoped to `Local`, `Cluster`, or `Region`.

---

## Error Codes

New runtime error codes introduced in this release:

| Code | Description |
|---|---|
| `R020` | Provisioning failure — adapter returned an error during `provision` or resource creation. |
| `R021` | Reconciliation failure — adapter returned an error during `reconcile` or drift correction. |

---

## Migration from v2.1.0

No breaking changes. Existing `.lum` files continue to work without modification. To use the new features:

1. **Add `resource entity` declarations** for any infrastructure you want Lumina to manage.
2. **Replace shell-based container management** with `provision` / `reconcile` / `destroy` actions.
3. **Add `global` to aggregate-level rules** for explicit broadcast safety.
4. **Enable `--trace` mode** for debugging complex rule evaluation chains.

---

## What's Next

The v2.1.2 release will focus on expanding the provisioning layer with additional providers and introducing the `whenever` / `ensure` keywords for continuous state enforcement (Level-Triggered Rules). See the [Version Map](../../VERSION_MAP.md) for the full roadmap.

