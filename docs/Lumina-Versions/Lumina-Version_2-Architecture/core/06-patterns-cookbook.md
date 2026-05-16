# Lumina Patterns Cookbook

Complete, runnable Lumina programs. Each program demonstrates a real pattern.

---

## Pattern 1: Hello World — Minimal Entity

```lumina
-- The simplest possible Lumina program
entity Greeter {
    message: Text
}

let g = Greeter { message: "Hello, Lumina!" }
show g.message
```

**Expected output:**
```
Hello, Lumina!
```

---

## Pattern 2: Temperature Monitor — Derived Fields + Rules

```lumina
entity Sensor {
    temperature: Number
    threshold: Number
    is_hot := temperature > threshold
    status := if is_hot then "OVERHEATING" else "NORMAL"
}

let s1 = Sensor { temperature: 45, threshold: 80 }

rule "Heat Alert" for (s: Sensor)
when Sensor.is_hot becomes true {
    show "ALERT: Temperature {Sensor.temperature}C exceeds {Sensor.threshold}C!"
}

show "--- Initial State ---"
show "Status: {s1.status}"

show "--- Heating Up ---"
update s1.temperature = 90
show "Status: {s1.status}"
```

**Expected output:**
```
--- Initial State ---
Status: NORMAL
--- Heating Up ---
ALERT: Temperature 90C exceeds 80C!
Status: OVERHEATING
```

---

## Pattern 3: Self-Resetting Loop — every + when

```lumina
entity LoopNode {
    iteration: Number
}

let n = LoopNode { iteration: 0 }

rule Heartbeat
every 2s {
    update n.iteration = n.iteration + 1
    show ">>> Current Iteration: {n.iteration}"
}

rule ResetLoop
when n.iteration >= 10 {
    show "--- Milestone: 10 reached. Resetting loop... ---"
    update n.iteration = 0
}
```

**Behavior**: Counter increments by 1 every 2 seconds. When it reaches 10, it resets to 0 and starts again.

---

## Pattern 4: Health Dashboard — Multiple Entities + Refs + Aggregates

```lumina
entity CoolingUnit {
    name: Text
    power_watts: Number
    is_active: Boolean
    is_failing := power_watts > 50 and not is_active
}

entity Server {
    name: Text
    cpu_temp: Number
    is_online: Boolean
    cooling: ref CoolingUnit
    is_overheating := cpu_temp > 80
    is_critical := is_overheating and cooling.is_failing
}

aggregate RackStats over Server {
    avg_temp := avg(cpu_temp)
    online_count := count(is_online)
}

rule "Cooling Recovery" for (c: CoolingUnit)
when CoolingUnit.is_active becomes true {
    show "Cooling {CoolingUnit.name}: ONLINE"
}

rule "Cooling Failure" for (c: CoolingUnit)
when CoolingUnit.is_active becomes false {
    show "Cooling {CoolingUnit.name}: OFFLINE"
}

rule "Thermal Warning" for (s: Server)
when Server.is_overheating becomes true {
    alert
        severity: "warning",
        message: "Server {Server.name} overheating: {Server.cpu_temp}C",
        source: "ThermalMonitor"
}

rule "CRITICAL: Cooling Failure" for (s: Server)
when Server.is_critical becomes true {
    alert
        severity: "critical",
        message: "EMERGENCY: {Server.name} overheating AND cooling failing!",
        source: "SovereignRuntime"
}
on clear {
    show "System stable: {Server.name} thermal state resolved."
}

-- Initialize
let MainCooler = CoolingUnit {
    name: "Main Cooler (Rack A)",
    power_watts: 100,
    is_active: true
}

let S1 = Server {
    name: "Web-01",
    cpu_temp: 45,
    is_online: true,
    cooling: MainCooler
}

let S2 = Server {
    name: "DB-01",
    cpu_temp: 50,
    is_online: true,
    cooling: MainCooler
}

-- Simulate
show "--- INITIAL STATE ---"
show "Avg Temp: {RackStats.avg_temp}C"

show "--- SIMULATING THERMAL SPIKE ---"
update S1.cpu_temp = 85

show "--- SIMULATING COOLING FAILURE ---"
update MainCooler.is_active = false

show "--- SIMULATING RECOVERY ---"
update MainCooler.is_active = true
update S1.cpu_temp = 45
```

---

## Pattern 5: Fleet Monitoring — Multiple Instances + Aggregate Rules

```lumina
entity Worker {
    name: Text
    cpu_percent: Number
    is_healthy: Boolean
    is_stressed := cpu_percent > 90
}

aggregate ClusterHealth over Worker {
    avg_cpu := avg(cpu_percent)
    stressed_count := count(is_stressed)
    all_healthy := all(is_healthy)
}

rule "Worker Stressed" for (w: Worker)
when Worker.is_stressed becomes true {
    show "WARNING: {Worker.name} CPU at {Worker.cpu_percent}%"
}

rule "Cluster Unhealthy"
when ClusterHealth.all_healthy becomes false {
    alert severity: "critical", message: "Not all workers are healthy!"
}

let w1 = Worker { name: "worker-1", cpu_percent: 45, is_healthy: true }
let w2 = Worker { name: "worker-2", cpu_percent: 55, is_healthy: true }
let w3 = Worker { name: "worker-3", cpu_percent: 30, is_healthy: true }

show "--- Cluster Status ---"
show "Avg CPU: {ClusterHealth.avg_cpu}%"
show "Stressed: {ClusterHealth.stressed_count}"

show "--- Simulating Load Spike ---"
update w1.cpu_percent = 95
update w2.cpu_percent = 92

show "Stressed count: {ClusterHealth.stressed_count}"
```

---

## Pattern 6: Security Quarantine — write + on clear

```lumina
entity NetworkNode {
    name: Text
    security_score: Number
    vlan: Number
    is_quarantined := vlan == 999
}

rule "Zero-Trust Quarantine" for (n: NetworkNode)
when NetworkNode.security_score < 50 {
    show "QUARANTINE: {NetworkNode.name} (score: {NetworkNode.security_score})"
    update NetworkNode.vlan = 999
}
on clear {
    show "RESTORED: {NetworkNode.name} returned to production VLAN"
    update NetworkNode.vlan = 1
}

let node1 = NetworkNode { name: "db-prod-01", security_score: 85, vlan: 1 }

show "--- Initial: score = 85 ---"
show "VLAN: {node1.vlan}, Quarantined: {node1.is_quarantined}"

show "--- Lowering security score ---"
update node1.security_score = 30

show "--- Restoring security score ---"
update node1.security_score = 90
```

---

## Pattern 7: Chaos Test — Rollback + Range Violations

```lumina
entity Node {
    @range 0 to 500 value: Number
}

entity Vault {
    @range 0 to 1000 balance: Number
}

aggregate GlobalStress over Node {
    total := sum(value)
}

-- This rule triggers infinite recursion (R003)
rule "Self-Trigger" for (n: Node)
when Node.value becomes 100 {
    update Node.value = 100
}

-- This rule violates @range (R006) — balance 2000 > max 1000
rule "Range Violation" for (v: Vault)
when Vault.balance becomes 500 {
    update Vault.balance = 2000
}

let N1 = Node { value: 0 }
let N2 = Node { value: 0 }
let MyVault = Vault { balance: 100 }

show "--- TEST 1: Recursion (MAX_DEPTH) ---"
update N1.value = 100

show "--- TEST 2: Rollback (Atomic Transaction) ---"
show "Balance before: {MyVault.balance}"
update MyVault.balance = 500
show "Balance after failure (should be 100): {MyVault.balance}"

show "--- TEST 3: Aggregate ---"
update N2.value = 300
show "Total: {GlobalStress.total}"
```

**Key behavior**: 
- Test 1: R003 — rollback, N1.value stays 0
- Test 2: R006 — rollback, balance stays 100
- Test 3: Aggregate computes correctly

---

## Pattern 8: User-Defined Functions

```lumina
fn clamp(val: Number, lo: Number, hi: Number) -> Number {
    if val < lo then lo else if val > hi then hi else val
}

fn severity_label(temp: Number) -> Text {
    if temp > 90 then "CRITICAL"
    else if temp > 70 then "WARNING"
    else "NORMAL"
}

entity Sensor {
    raw_reading: Number
    safe_reading := clamp(raw_reading, -40, 150)
    severity := severity_label(raw_reading)
}

let s = Sensor { raw_reading: 95 }
show "Raw: {s.raw_reading}"
show "Safe: {s.safe_reading}"
show "Severity: {s.severity}"

update s.raw_reading = -100
show "Raw: {s.raw_reading}"
show "Safe: {s.safe_reading}"
show "Severity: {s.severity}"
```

---

## Pattern 9: List Operations

```lumina
let temps = [72, 88, 45, 91, 33]

show "Count: {len(temps)}"
show "Min: {min(temps)}"
show "Max: {max(temps)}"
show "Sum: {sum(temps)}"
show "First: {head(temps)}"
show "Rest: {tail(temps)}"
show "Third: {at(temps, 2)}"

let extended = append(temps, 100)
show "Extended: {extended}"
show "New length: {len(extended)}"
```

---

## Pattern 10: Multi-Entity Datacenter Simulation

```lumina
entity PDU {
    name: Text
    total_kw: Number
    max_kw: Number
    is_overloaded := total_kw > max_kw * 0.9
}

entity Rack {
    name: Text
    server_count: Number
    pdu: ref PDU
    power_draw_kw: Number
    is_powered := not pdu.is_overloaded
}

entity Server {
    name: Text
    cpu_temp: Number
    rack_name: Text
    is_online: Boolean
    is_overheating := cpu_temp > 80
}

aggregate DatacenterStats over Server {
    avg_temp := avg(cpu_temp)
    hot_count := count(is_overheating)
    online_count := count(is_online)
}

-- Rules
rule "PDU Overload" for (p: PDU)
when PDU.is_overloaded becomes true {
    alert severity: "critical", message: "PDU {PDU.name} overloaded: {PDU.total_kw}kW / {PDU.max_kw}kW"
}

rule "Server Overheat" for (s: Server)
when Server.is_overheating becomes true {
    alert severity: "warning", message: "{Server.name} at {Server.cpu_temp}C"
}

rule "Fleet Status"
when DatacenterStats.hot_count > 0 {
    show "FLEET ALERT: {DatacenterStats.hot_count} servers overheating. Avg: {DatacenterStats.avg_temp}C"
}

-- Initialize
let pdu1 = PDU { name: "PDU-A1", total_kw: 15, max_kw: 20 }
let rack1 = Rack { name: "Rack-A", server_count: 3, pdu: pdu1, power_draw_kw: 5 }
let srv1 = Server { name: "web-01", cpu_temp: 45, rack_name: "Rack-A", is_online: true }
let srv2 = Server { name: "web-02", cpu_temp: 50, rack_name: "Rack-A", is_online: true }
let srv3 = Server { name: "db-01", cpu_temp: 55, rack_name: "Rack-A", is_online: true }

-- Simulate
show "=== INITIAL STATE ==="
show "Avg temp: {DatacenterStats.avg_temp}C"
show "Hot servers: {DatacenterStats.hot_count}"

show "=== THERMAL EVENT ==="
update srv1.cpu_temp = 85
update srv2.cpu_temp = 82

show "=== PDU OVERLOAD ==="
update pdu1.total_kw = 19

show "=== RECOVERY ==="
update srv1.cpu_temp = 45
update srv2.cpu_temp = 48
update pdu1.total_kw = 12
```

---

## Pattern 11: String Interpolation Showcase

```lumina
entity Report {
    server_name: Text
    cpu: Number
    memory: Number
    disk: Number
    health := if cpu < 80 and memory < 80 and disk < 90 then "HEALTHY" else "DEGRADED"
}

let r = Report {
    server_name: "prod-api-01",
    cpu: 65,
    memory: 72,
    disk: 45
}

show "╔══════════════════════════════╗"
show "║  Server: {r.server_name}"
show "║  CPU:    {r.cpu}%"
show "║  Memory: {r.memory}%"
show "║  Disk:   {r.disk}%"
show "║  Health: {r.health}"
show "╚══════════════════════════════╝"
```

---

## Pattern 12: Conditional Logic Chains

```lumina
fn risk_level(temp: Number, load: Number) -> Text {
    if temp > 90 and load > 90 then "CRITICAL"
    else if temp > 80 or load > 85 then "HIGH"
    else if temp > 60 or load > 70 then "MEDIUM"
    else "LOW"
}

entity Machine {
    temp: Number
    load: Number
    risk := risk_level(temp, load)
}

let m = Machine { temp: 50, load: 40 }
show "Risk: {m.risk}"

update m.temp = 85
show "Risk: {m.risk}"

update m.load = 95
show "Risk: {m.risk}"
```

---

## Pattern 13: Previous Value Detection

```lumina
entity Metric {
    value: Number
}

let m = Metric { value: 100 }

rule "Big Jump"
when m.value > prev(value) * 1.5 {
    show "SPIKE: value jumped from {prev(value)} to {m.value}"
}

rule "Big Drop"
when m.value < prev(value) * 0.5 {
    show "DROP: value fell from {prev(value)} to {m.value}"
}

update m.value = 200
update m.value = 50
```

---

## Pattern 14: IoT Sensor with External Entity

```lumina
external entity Sensor {
    temperature: Number
    sync: "mqtt://broker:1883"
    on: realtime
    sync on temperature
}

entity System {
    isOverheating := Sensor.temperature > 80
}

rule TemperatureAlert for (s: System)
when System.isOverheating becomes true {
    show "Critical: Temperature is {Sensor.temperature}"
    alert severity: "critical", message: "System is overheating!"
}

rule TemperatureResolved for (s: System)
when System.isOverheating becomes false {
    show "Status: Temperature stabilized at {Sensor.temperature}"
    alert severity: "info", message: "Temperature stabilized."
}
```
