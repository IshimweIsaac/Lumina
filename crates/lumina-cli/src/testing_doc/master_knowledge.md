# LUMINA: THE ULTIMATE KNOWLEDGE ATLAS (GRAND ENCYCLOPEDIA)
**Version: 2.0.0 | Status: DEFINITIVE SINGLE SOURCE OF TRUTH**

---

## **TABLE OF CONTENTS**

- [Philosophical Foundations & The Vibe Coding Manifesto](#philosophical-foundations--the-vibe-coding-manifesto)
- [Core Architecture: The Reactive Engine](#core-architecture-the-reactive-engine)
- [LSL Language Specification & AST](#lsl-language-specification--ast)
- [Diagnostic Encyclopedia (L-Codes & R-Codes)](#diagnostic-encyclopedia-l-codes--r-codes)
- [Cluster Networking, Consensus & The Truth Log](#cluster-networking-consensus--the-truth-log)
- [The Anthology of Vibe Coding (Patterns 1-150)](#the-anthology-of-vibe-coding-patterns)
- [Developer Integration: FFI & Providers](#developer-integration-ffi--providers)
- [Platform Support: WASM, HCL & IDE](#platform-support-wasm-hcl--ide)
- [Advanced Orchestration & Case Studies](#advanced-orchestration--case-studies)
- [Appendix: Test Specification & Roadmap](#appendix-test-specification--roadmap)

---

## **PHILOSOPHICAL FOUNDATIONS & THE VIBE CODING MANIFESTO**

### The Core Axiom: The One Rule
Lumina is a **Distributed Reactive Language** (DRL) designed for agentless infrastructure orchestration. It is governed by a single, unbreakable rule:
> *"Every feature must help an engineer describe what is TRUE about their system."*

### From Infrastructure-as-Code (IaC) to Infrastructure-as-Truth (IaT)
- **IaC (The Past)**: Procedural scripts (Ansible, Chef) or provisioning plans (Terraform) that describe *how to get* to a state. They are fragile and prone to configuration drift.
- **IaT (Lumina)**: Continuous models that describe *what is true*. Lumina doesn't "run" a script; it maintains a **Continuous State DAG**. If a node fails, the rest of the cluster still knows the "truth" and reacts atomically.

### The Vibe Coding Principles
1. **Truth is Sovereign**: The current state is the only authority. We do not trust cached intentions.
2. **Logic follows the DAG**: Complexity is managed through Directed Acyclic Graphs. Any cycle is a fatal instability (Error L004).
3. **Latency is the Signal**: High latency is a "Vibe" that signals stress. We move workloads away from friction and toward flow.
4. **Reactivity is Atomic**: A change in one part of the system results in an atomic transition for all dependents, or a total rollback.
5. **Decentralization is Resilience**: The cluster is a mesh of equals. If the majority survives, the Truth survives.
6. **Automation is Self-Defense**: Infrastructure should defend itself against thermal spikes and breaches without human permission.
7. **Immutable Memory**: Every decision is etched in the **Blake3 Truth Log**, creating a mathematically verifiable audit trail.
8. **Harmony over Orchestration**: The goal is to define rules that allow the system to find its own stable harmony.

---

## **CORE ARCHITECTURE: THE REACTIVE ENGINE**

The Lumina engine transforms raw text into a high-performance reactive state machine through a 5-crate pipeline.

### The 5-Crate Pipeline
1.  **`lumina-lexer` (O(1) Tokenizer)**: Uses `logos` for zero-copy tokenization. Comments are treated as trivia but preserved for LSP support.
2.  **`lumina-parser` (Pratt Parsing)**: A hand-written recursive descent parser that handles operator precedence via binding power. It features "Panic Mode" error recovery to show multiple syntax errors in one pass.
3.  **`lumina-analyzer` (The Semantic Heart)**: Performs scope mapping and DAG generation. It runs **Kahn's Algorithm** for topological sorting and cycle detection.
4.  **`lumina-diagnostics` (The UX)**: Provides Rust-style error snippets with help injection for "Vibe Coding" suggestions.
5.  **`lumina-runtime` (Vectorized Evaluator)**: The execution engine.

### Runtime Internals: Memory & Recomputation
- **Vectorized Slot Architecture**: State is stored in contiguous `Vec<LuminaValue>` arrays. Each `StateSlot` contains a `current` and `previous` value, allowing `prev()` lookups in O(1).
- **The `TOPOLOGICAL_SWEEP`**: When a `stored` field changes, the engine identifies "dirty" nodes and iterates through the topological sort.
- **FxHashSet Dirty Tracking**: To reach 1M events/sec, the engine uses `FxHashSet` (from `rustc-hash`) to track only affected instances, pruning unnecessary graph traversals.

### The Snapshot Stack & Atomic Transactions
Lumina wraps every event in a transaction to ensure "Truth is Atomic."
- **`push_snapshot()`**: Before an event, the engine takes a delta-snapshot (O(M) complexity, where M is modified fields).
- **Intent Buffer**: `write` actions (external side effects) are buffered. They are only flushed to providers if the transaction successfully commits.
- **Rollback**: If a rule raises an error (e.g., R003 recursion limit), `rollback_to_latest()` restores the pre-event state in O(1).

---

## **LSL LANGUAGE SPECIFICATION & AST**

### The Lumina AST
The Abstract Syntax Tree is the definitive representation of an LSL program.
- **Structural Nodes**: `EntityNode` (defines types), `FieldNode` (stored vs. derived), `RuleNode` (triggers and actions).
- **Expression Nodes**: `BinaryExpression` (arithmetic), `CallExpression` (built-ins like `trend()`), `MemberExpression` (field access).
- **Orchestration Nodes**: Native primitives like `migrate()`, `evacuate()`, and `deploy()`.

### Syntax Invariants (v2.0-GOLD)
- **Variable Declaration**: Use `let` for dynamic instances: `let n = LoopNode { iteration: 0 }`.
- **The Update Operator**: Always use `=` for mutations: `update n.iteration = n.iteration + 1`. (The `to` keyword is legacy).
- **Reactive Loops**: Loops are feedback cycles. Use a temporal trigger (`every 1s`) to push state and a conditional trigger (`when n.val > 100`) to reset.

### Security Architecture: The `Secret` Type
- **Taint Analysis**: Fields marked `@secret` are tracked. If a secret flows into a non-secure sink (like an `alert`), the compiler raises **L050**.
- **Redaction**: Secrets are encrypted at rest and displayed as `[REDACTED]` in the REPL.
- **KMS Integration**: The runtime requests DEKs from an external Key Management Service on startup.

### LSL Registry: Standard Namespaces
*   **`LSL::datacenter`**: `Server` (temp, power, status), `Rack` (total_kw, used_u), `PDU`, `CRAC`.
*   **`LSL::network`**: `Switch` (packet_loss), `Router` (bgp_peers), `Firewall`, `Link` (latency_ms, jitter).
*   **`LSL::k8s`**: `Pod` (cpu_mcore, restarts), `Node` (disk_pressure), `Deployment`.

---

## **DIAGNOSTIC ENCYCLOPEDIA (L-CODES & R-CODES)**

### Analyzer Errors (L-Codes: Static)
*   **L001: DUPLICATE_ENTITY**: Two entities share a name in the flat global namespace.
*   **L004: DAG_CYCLE**: A circular dependency exists in `derived` fields. Breaking this requires a `stored` field and a `rule`.
*   **L041: TIME_IN_DERIVED**: Prohibits `now()` in `:=` fields to prevent CPU-consuming recomputation storms. Use rule triggers instead.
*   **L050: SECRET_LEAK**: A `Secret` value is being exposed to a non-secure sink.
*   **L061: SAFE_MODE_VIOLATION**: A `write` action is found in a path that could execute during quorum loss.

### Runtime Errors (R-Codes: Dynamic)
*   **R001: INSTANCE_NOT_FOUND**: Accessing an instance that doesn't exist (often due to race conditions with provider deletions).
*   **R003: MAX_DEPTH**: Rule recursion limit (100) exceeded. Triggers an atomic rollback.
*   **R006: RANGE_VIOLATION**: A mutation attempted to set a value outside its `@range` metadata.
*   **R011: QUORUM_LOST_EXEC**: Connectivity to peers lost during a transaction.
*   **R019: STACK_CORRUPTION**: Internal Snapshot Stack misalignment (critical engine failure).

---

## **CLUSTER NETWORKING, CONSENSUS & THE TRUTH LOG**

### Gossip Protocol (UDP/7777)
Lumina nodes operate as a decentralized mesh using a custom SWIM-inspired protocol.
- **Health Monitoring**: Nodes cycle through `Alive`, `Suspect`, and `Dead` states based on heartbeats.
- **State Mesh Sync**: Uses **Merkle Tree Anti-Entropy**. Every 5s, nodes exchange roots. If they mismatch, a recursive delta-sync resolves the divergence.
- **Vectorized Delta-Pull**: Only *changed* slots are gossiped, reducing bandwidth by >90% compared to full-state sync.

### Raft-lite Leader Election
For sensitive actions like `deploy`, Lumina uses term-based consensus.
- **Quorum**: A majority (N/2 + 1) is required to commit any `STATE_SYNC`.
- **Safe Mode**: If a node loses quorum, it freezes all side effects (`write` actions) to prevent split-brain scenarios.

### The Blake3 Truth Log
Every state transition is cryptographically chained: `Hash_N = Blake3(Hash_{N-1} + Payload)`.
- **Auditability**: Auditors can verify the log remotely to ensure no history tampering has occurred.
- **WAL Catch-up**: Recovering nodes use the log to fast-forward to the current "Truth."

---

## **THE ANTHOLOGY OF VIBE CODING (PATTERNS 1-150)**

### Resilience & Self-Healing
*   **Pattern 1: Basic Health**: `when s.temp > 80 becomes true { alert severity: "warning", message: "Server {s.id} hot!" }`.
*   **Pattern 11: Zombie Reaper**: `when p.state == "zombie" for 1h { write p.signal = 9 }`.
*   **Pattern 63: Circuit Breaker**: Isolates slow services by monitoring `latency_ms` and automatically reintroducing them `on clear`.
*   **Pattern 109: DB Replica Promotion**: Promotes the lowest-lag replica if the Master heartbeats fail 3 times.

### Security & Zero-Trust
*   **Pattern 3: Zero-Trust Quarantine**: `when n.security_score < 50 { write n.vlan = 999 }`.
*   **Pattern 83: Ransomware Detector**: Monitors `trend(f.entropy, 1m)`. High entropy spikes trigger a `read-only` freeze and immediate snapshot.
*   **Pattern 120: Port Rotation**: Rotates SSH management ports every 24h, using `prev()` to block "stale" port probes.
*   **Pattern 143: Impossible Travel**: Reaps VPN sessions if geo-location changes by >500km in <1h.

### Power & Sustainability
*   **Pattern 7: Energy-Aware Jobs**: Starts queued jobs only when `SolarArray.output_kw > 500`. Uses `on clear` to pause when output drops.
*   **Pattern 105: Green Migration**: Moves low-priority workloads to datacenters with current peak renewable output.
*   **Pattern 130: Autonomous Power Shedding**: Shuts down dev clusters when UPS enters "On Battery" mode.

### Cloud Orchestration (v2.0-GOLD)
*   **Pattern 69: Flash Crowd Scaler**: Uses `trend(Site.traffic, 5m)` to deploy extra servers *before* the site overloads.
*   **Pattern 91: Ghost Instance Handoff**: `migrate(i, to: min(cluster.nodes.cpu))`—workloads "flow" to nodes with least resistance.
*   **Pattern 150: The Sovereign Overlord**: Coordinates full datacenter evacuation (GSLB shift + workload migration) during regional disasters.

---

## **DEVELOPER INTEGRATION: FFI & PROVIDERS**

### C FFI API (LUMINA_FFI)
The `lumina_ffi` crate allows embedding the runtime in C, C++, Python, Go, or Node.js.
- **Memory Rule**: Lumina strings are Rust-owned. **Always** call `lumina_free_string(ptr)`; never use C `free()`.
- **Core Functions**:
    - `lumina_create(source)`: Initializes the runtime.
    - `lumina_apply_event(rt, instance, field, json)`: Injects external data.
    - `lumina_tick(rt)`: Triggers temporal recomputations.

### Building a Custom Provider
Providers are external processes that connect Lumina to the world via JSON-RPC 2.0 over `stdin/stdout`.
1. **Handshake**: Provider responds to `lumina_hello` with its name and managed entities.
2. **Schema Sync**: Provider sends field definitions via `lumina_get_schema`.
3. **State Stream**: Provider pushes `state_update` notifications.
4. **Write/Rollback**: Provider handles `lumina_write` for side effects and `lumina_rollback` if the transaction fails downstream.

---

## **PLATFORM SUPPORT: WASM, HCL & IDE**

### Lumina WASM
Runs the engine natively in browsers with ~90% native performance.
- **Direct Mapping**: `EntityStore` maps to WASM linear memory for O(1) access from JS.
- **Visualization**: Enables real-time, client-side cluster dashboards that can run "visual-only" rules.

### Hardware Compatibility List (HCL)
The `LSL::metal` namespace maps Redfish, IPMI, and SNMP data into slots.
- **Redfish Adapter**: Uses adaptive polling to map chassis thermal/power resources.
- **Supported**: Dell (iDRAC 9), HPE (iLO 5), Cisco Nexus (NX-OS), Arista (EOS), NetApp (ONTAP).

### VS Code Extension & LSP
- **Incremental Parsing**: Uses a "Salsa-style" engine to only re-parse modified AST branches.
- **Semantic Highlighting**: Visually distinguishes `stored` vs `derived` fields using full AST tokens.

---

## **ADVANCED ORCHESTRATION & CASE STUDIES**

### Advanced Aggregate Logic
Aggregates can now scope over `Local`, `Cluster`, or `Region(name)`.
- **Gossip Reduction**: Every node computes a local pre-aggregate. The Global Leader performs the final reduction, injecting a `VirtualInstance` back into the mesh for O(1) local lookups.

### Case Study: Global Cloud Evacuation
Scenario: `us-east-1` backbone failure.
1. **Quorum Loss**: East coast nodes enter **Safe Mode** immediately.
2. **Leader Election**: Healthy regions (`us-west-2`, `eu-central-1`) form a new quorum.
3. **Evacuation Trigger**: A rule detects `< 0.1` availability and initiates `evacuate(LSL::k8s::Pod)`.
4. **Truth Handoff**: The leader identifies the latest `version` of East coast instances and deploys them to healthy nodes with full history.
5. **Reconciliation**: Migrated workloads activate, and the master database is promoted to the healthy region.

---

## **APPENDIX: TEST SPECIFICATION & ROADMAP**

### Lumina Test Specification (LTS)
Define tests using `test`, `setup`, `action`, and `assert` blocks. LTS supports fault injection by playing back recorded JSON-RPC provider traffic.

### Future Roadmap: Lumina v3.0
- **eBPF Integration**: Moving triggers into the Linux kernel for zero context-switch packet-level reactivity.
- **Edge Federation**: Global-scale backplane for sub-millisecond state sharing between clusters.
- **Natural Language "Vibe" Compiler**: Describe logic in natural language to generate LSL.

---
**[DOCUMENT END]**
Integrity: Verified | Revision: 2.0.0-GOLD-REF
**Stay Reactive. Stay Sovereign. Code the Truth.**
