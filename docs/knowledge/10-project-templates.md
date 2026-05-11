# Lumina Project Templates

Full project architectures for common use cases. Use these as starting points for real projects.

---

## Template 1: IoT Sensor Fleet Monitor

**Use case**: Monitor a fleet of temperature/humidity sensors, alert on anomalies, track fleet health.

```lumina
-- ============================================================
-- IoT Sensor Fleet Monitor
-- Monitors N sensors, alerts on overheating, tracks fleet stats
-- ============================================================

-- 1. ENTITIES

entity Sensor {
    name: Text
    location: Text
    temperature: Number
    humidity: Number
    is_online: Boolean
    is_overheating := temperature > 80
    is_humid := humidity > 85
    status := if not is_online then "OFFLINE"
              else if is_overheating then "CRITICAL"
              else if is_humid then "WARNING"
              else "OK"
}

-- 2. AGGREGATES

aggregate FleetHealth over Sensor {
    avg_temp := avg(temperature)
    max_temp := max(temperature)
    min_temp := min(temperature)
    online_count := count(is_online)
    hot_count := count(is_overheating)
    any_critical := any(is_overheating)
    all_online := all(is_online)
}

-- 3. HELPER FUNCTIONS

fn temp_label(t: Number) -> Text {
    if t > 90 then "DANGER"
    else if t > 80 then "HIGH"
    else if t > 60 then "NORMAL"
    else "COOL"
}

-- 4. RULES

rule "Sensor Overheat" for (s: Sensor)
when Sensor.is_overheating becomes true {
    alert
        severity: "critical",
        message: "{Sensor.name} at {Sensor.location}: {Sensor.temperature}C",
        source: "ThermalMonitor"
}
on clear {
    show "RESOLVED: {Sensor.name} temperature normalized"
}

rule "Sensor Offline" for (s: Sensor)
when Sensor.is_online becomes false {
    alert
        severity: "warning",
        message: "{Sensor.name} went offline",
        source: "ConnectivityMonitor"
}
on clear {
    show "RECOVERED: {Sensor.name} back online"
}

rule "Fleet Emergency"
when FleetHealth.hot_count > 2 {
    alert
        severity: "critical",
        message: "Multiple sensors overheating! Count: {FleetHealth.hot_count}",
        source: "FleetMonitor"
}

-- 5. INSTANCES

let s1 = Sensor {
    name: "Sensor-A1", location: "Warehouse North",
    temperature: 45, humidity: 60, is_online: true
}
let s2 = Sensor {
    name: "Sensor-A2", location: "Warehouse South",
    temperature: 50, humidity: 55, is_online: true
}
let s3 = Sensor {
    name: "Sensor-B1", location: "Server Room",
    temperature: 55, humidity: 40, is_online: true
}
let s4 = Sensor {
    name: "Sensor-B2", location: "Server Room",
    temperature: 48, humidity: 42, is_online: true
}

-- 6. SIMULATION

show "=== FLEET STATUS ==="
show "Online: {FleetHealth.online_count}"
show "Avg Temp: {FleetHealth.avg_temp}C"
show "Max Temp: {FleetHealth.max_temp}C"

show "=== THERMAL EVENT ==="
update s3.temperature = 88
update s4.temperature = 85

show "Hot sensors: {FleetHealth.hot_count}"

show "=== SENSOR OFFLINE ==="
update s1.is_online = false

show "=== RECOVERY ==="
update s3.temperature = 55
update s4.temperature = 50
update s1.is_online = true
```

### How to extend:
- **Add new sensor**: Copy a `let` statement, change name/location/values
- **Add humidity alerts**: Create a `rule "Humidity Alert"` with `when Sensor.is_humid becomes true`
- **Add external data**: Replace `entity Sensor` with `external entity Sensor` and add sync config

---

## Template 2: Server Rack Health System

**Use case**: Monitor servers with cooling units, cross-entity references, power management.

```lumina
-- ============================================================
-- Server Rack Health System
-- Entities: CoolingUnit → Server → Rack (with refs)
-- ============================================================

-- 1. ENTITIES (in dependency order — referenced entities first)

entity CoolingUnit {
    name: Text
    power_watts: Number
    is_active: Boolean
    efficiency := if is_active then 100 - power_watts else 0
    is_failing := power_watts > 50 and not is_active
}

entity Server {
    name: Text
    cpu_temp: Number
    ram_usage: Number
    is_online: Boolean
    cooling: ref CoolingUnit
    is_overheating := cpu_temp > 80
    is_ram_full := ram_usage > 90
    is_critical := is_overheating and cooling.is_failing
    health := if is_critical then "CRITICAL"
              else if is_overheating then "WARNING"
              else if not is_online then "OFFLINE"
              else "HEALTHY"
}

-- 2. AGGREGATES

aggregate RackMetrics over Server {
    avg_temp := avg(cpu_temp)
    avg_ram := avg(ram_usage)
    online := count(is_online)
    critical_count := count(is_critical)
}

-- 3. RULES

rule "Thermal Warning" for (s: Server)
when Server.is_overheating becomes true {
    alert severity: "warning",
        message: "Server {Server.name}: {Server.cpu_temp}C (cooling: {Server.cooling.name})"
}

rule "CRITICAL ALERT" for (s: Server)
when Server.is_critical becomes true {
    alert severity: "critical",
        message: "EMERGENCY: {Server.name} overheating AND {Server.cooling.name} failing!"
}
on clear {
    show "RESOLVED: {Server.name} critical state cleared"
}

rule "Cooling Failure" for (c: CoolingUnit)
when CoolingUnit.is_failing becomes true {
    alert severity: "critical",
        message: "Cooling unit {CoolingUnit.name} is failing!"
}

rule "Rack Stress"
when RackMetrics.critical_count > 0 {
    show "RACK ALERT: {RackMetrics.critical_count} servers in critical state"
    show "Avg temp: {RackMetrics.avg_temp}C, Online: {RackMetrics.online}"
}

-- 4. INSTANCES

let cooler_a = CoolingUnit { name: "CRAC-A", power_watts: 30, is_active: true }
let cooler_b = CoolingUnit { name: "CRAC-B", power_watts: 25, is_active: true }

let web1 = Server {
    name: "web-01", cpu_temp: 45, ram_usage: 60,
    is_online: true, cooling: cooler_a
}
let web2 = Server {
    name: "web-02", cpu_temp: 50, ram_usage: 55,
    is_online: true, cooling: cooler_a
}
let db1 = Server {
    name: "db-01", cpu_temp: 55, ram_usage: 70,
    is_online: true, cooling: cooler_b
}

-- 5. SIMULATION

show "=== INITIAL STATE ==="
show "Rack avg temp: {RackMetrics.avg_temp}C"
show "Online servers: {RackMetrics.online}"

show "=== THERMAL SPIKE ON web-01 ==="
update web1.cpu_temp = 88

show "=== COOLING FAILURE ON CRAC-A ==="
update cooler_a.is_active = false

show "=== FULL RECOVERY ==="
update cooler_a.is_active = true
update web1.cpu_temp = 45
```

---

## Template 3: Security Monitoring System

**Use case**: Zero-trust network monitoring with quarantine and recovery.

```lumina
-- ============================================================
-- Zero-Trust Security Monitor
-- Auto-quarantine compromised nodes, auto-restore on recovery
-- ============================================================

entity NetworkNode {
    name: Text
    ip_address: Text
    security_score: Number
    vlan: Number
    failed_logins: Number
    is_quarantined := vlan == 999
    is_compromised := security_score < 40
    is_suspicious := security_score < 60 and security_score >= 40
    threat_level := if is_compromised then "CRITICAL"
                    else if is_suspicious then "ELEVATED"
                    else "NORMAL"
}

aggregate SecurityOverview over NetworkNode {
    avg_score := avg(security_score)
    quarantined_count := count(is_quarantined)
    compromised_count := count(is_compromised)
    any_compromised := any(is_compromised)
}

-- Auto-quarantine compromised nodes
rule "Auto-Quarantine" for (n: NetworkNode)
when NetworkNode.is_compromised becomes true {
    show "QUARANTINE: {NetworkNode.name} ({NetworkNode.ip_address}) score={NetworkNode.security_score}"
    update NetworkNode.vlan = 999
    alert severity: "critical",
        message: "Node {NetworkNode.name} quarantined: security score {NetworkNode.security_score}"
}
on clear {
    show "RESTORED: {NetworkNode.name} returned to production"
    update NetworkNode.vlan = 1
}

-- Alert on suspicious activity
rule "Suspicious Activity" for (n: NetworkNode)
when NetworkNode.is_suspicious becomes true {
    alert severity: "warning",
        message: "Elevated threat: {NetworkNode.name} score={NetworkNode.security_score}"
}

-- Fleet-wide breach alert
rule "Breach Alert"
when SecurityOverview.compromised_count > 1 {
    alert severity: "critical",
        message: "MULTI-NODE BREACH: {SecurityOverview.compromised_count} nodes compromised!"
}

-- Instances
let node1 = NetworkNode {
    name: "web-prod-01", ip_address: "10.0.1.10",
    security_score: 92, vlan: 1, failed_logins: 0
}
let node2 = NetworkNode {
    name: "db-prod-01", ip_address: "10.0.1.20",
    security_score: 88, vlan: 1, failed_logins: 0
}
let node3 = NetworkNode {
    name: "api-prod-01", ip_address: "10.0.1.30",
    security_score: 95, vlan: 1, failed_logins: 0
}

-- Simulation
show "=== INITIAL STATE ==="
show "Avg security: {SecurityOverview.avg_score}"
show "Quarantined: {SecurityOverview.quarantined_count}"

show "=== ATTACK SIMULATION ==="
update node1.security_score = 35
update node1.failed_logins = 50

show "Node1 VLAN: {node1.vlan} (should be 999)"
show "Node1 quarantined: {node1.is_quarantined}"

show "=== SECOND NODE COMPROMISED ==="
update node2.security_score = 25

show "=== RECOVERY ==="
update node1.security_score = 85
show "Node1 VLAN after recovery: {node1.vlan} (should be 1)"
```

---

## Template 4: Application Health Checker (Simple)

**Use case**: Quick health check for a web application stack.

```lumina
-- ============================================================
-- Application Health Checker
-- Simple stack: LB → App → DB
-- ============================================================

entity Service {
    name: Text
    response_time_ms: Number
    error_rate: Number
    is_healthy: Boolean
    is_slow := response_time_ms > 500
    is_degraded := error_rate > 5
}

aggregate StackHealth over Service {
    avg_response := avg(response_time_ms)
    avg_errors := avg(error_rate)
    all_healthy := all(is_healthy)
    any_degraded := any(is_degraded)
}

fn health_icon(healthy: Boolean) -> Text {
    if healthy then "✅" else "❌"
}

rule "Service Degraded" for (s: Service)
when Service.is_degraded becomes true {
    alert severity: "warning",
        message: "{Service.name} error rate: {Service.error_rate}%"
}

rule "Stack Unhealthy"
when StackHealth.all_healthy becomes false {
    alert severity: "critical", message: "Stack health compromised!"
}

let lb = Service { name: "Load Balancer", response_time_ms: 12, error_rate: 0.1, is_healthy: true }
let app = Service { name: "App Server", response_time_ms: 45, error_rate: 0.5, is_healthy: true }
let db = Service { name: "Database", response_time_ms: 8, error_rate: 0.0, is_healthy: true }

show "╔═══════════════════════════════╗"
show "║  Stack Health Dashboard       ║"
show "╠═══════════════════════════════╣"
show "║ {health_icon(lb.is_healthy)} LB:  {lb.response_time_ms}ms  err:{lb.error_rate}%"
show "║ {health_icon(app.is_healthy)} App: {app.response_time_ms}ms  err:{app.error_rate}%"
show "║ {health_icon(db.is_healthy)} DB:  {db.response_time_ms}ms  err:{db.error_rate}%"
show "╠═══════════════════════════════╣"
show "║ Avg Response: {StackHealth.avg_response}ms"
show "║ All Healthy: {StackHealth.all_healthy}"
show "╚═══════════════════════════════╝"

show "=== SIMULATING DB ISSUES ==="
update db.error_rate = 12
update db.is_healthy = false
```

---

## Project Architecture Guidelines

### File Organization

For larger projects, split across multiple `.lum` files:

```
my_project/
├── entities.lum        -- All entity definitions
├── aggregates.lum      -- All aggregate declarations
├── functions.lum       -- Helper functions
├── rules/
│   ├── thermal.lum     -- Temperature-related rules
│   ├── security.lum    -- Security rules
│   └── fleet.lum       -- Fleet-wide rules
├── instances.lum       -- Instance creation
└── main.lum            -- Imports + simulation
```

Use `import "path/to/file.lum"` to connect them.

### Naming Conventions

| Item       | Convention       | Example                    |
|-----------|-----------------|----------------------------|
| Entity    | PascalCase       | `Server`, `CoolingUnit`    |
| Field     | snake_case       | `cpu_temp`, `is_online`    |
| Instance  | camelCase or descriptive | `mainServer`, `web01` |
| Rule      | Quoted descriptive | `"Thermal Warning"`       |
| Function  | snake_case       | `clamp`, `severity_label`  |
| Aggregate | PascalCase       | `RackStats`, `FleetHealth` |

### Entity Definition Order

Define entities in dependency order — referenced entities FIRST:

```lumina
-- ✅ CORRECT ORDER:
entity CoolingUnit { ... }          -- referenced by Server
entity Server { cooling: ref CoolingUnit ... }  -- references CoolingUnit

-- ❌ WRONG ORDER:
entity Server { cooling: ref CoolingUnit ... }  -- CoolingUnit not defined yet!
entity CoolingUnit { ... }
```

### Rule Organization

1. **Alert rules** first (most important)
2. **Recovery rules** (`on clear`) attached to alert rules
3. **Periodic rules** (`every`) for maintenance
4. **Fleet rules** last (aggregate-dependent)

### Testing Pattern

Always include a simulation section at the bottom:

```lumina
-- ============ SIMULATION ============
show "=== INITIAL STATE ==="
-- show initial values

show "=== EVENT 1: Description ==="
update instance.field = value
-- show state after change

show "=== RECOVERY ==="
-- restore to normal
-- verify on clear fired
```
