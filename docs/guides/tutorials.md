<!-- Source: getting-started.md -->

# Getting Started: The Lumina Mental Model 🛰️

Welcome to the official introduction to Lumina. This guide is designed to shift your perspective from **procedural** programming to **reactive** truth-telling. By the end of this page, you will understand not just how to write Lumina, but why it is the most robust way to monitor complex state.

---

## 1. The Core Philosophy: "Describe What is True"

Most programming languages are **Step-By-Step Instructions**. To check a temperature, you write:
1. Read sensor data.
2. Compare to threshold.
3. If high, start a timer.
4. If still high after 5 minutes, send an alert.

In Lumina, you skip the steps and **Describe the State**:
> "An alert exists if the sensor temperature is above the threshold for 5 minutes."

Lumina's engine then handles the reading, the comparison, the timing, and the state cleanup automatically. This is **Declarative Reactivity**.

---

## 2. The 4-Tier Layered Logic

Lumina programs are built using four distinct layers of data, each with different properties:

### **Tier 1: Stored Fields (The Foundation)**
These are the raw facts of your system. They only change when an external event occurs or a rule explicitly updates them.
```lumina
entity Thermometer {
 current_temp: Number
 location: Text
}
-- Stored fields require manual updates
update t1.current_temp to 25.5
```

### **Tier 2: Derived Fields (The Living Logic)**
These are the "automatic" fields. They calculate themselves instantly whenever their dependencies change. They are the **Internal Truth** of your entity.
```lumina
entity Thermometer {
 current_temp: Number
 threshold: Number
 -- This field 'lives'. It is ALWAYS true if temp > threshold.
 is_overheating := current_temp > threshold
}
```

### **Tier 3: Rules & Alerts (The Action)**
Rules watch for "moments of transition" in your derived fields. This is where your system interacts with the outside world.
```lumina
rule "Safety Trip"
when Thermometer.is_overheating becomes true for 5m {
 alert severity: "critical", message: "Emergency Cooling Required!"
}
```

### **Tier 4: The Cluster Mesh (The Network)**
in Lumina, state isn't confined to a single node. You can define `cluster` topology, enabling workloads to seamlessly `migrate` and broadcast state changes across the network using a native UDP Gossip protocol.
```lumina
rule "Failover Orchestration"
when MainServer.is_unhealthy becomes true {
  migrate { workloads: "critical_db", target: "backup_node" }
}
```

---

## 3. Safety Guarantees: Why Lumina is different.

Lumina isn't just a language; it's a **Correctness Engine**. It provides two mathematical guarantees that traditional languages ignore:

### **A. Directed Acyclic Graph (DAG) Stability**
When you define a derived field `A := B + C`, Lumina builds a dependency graph. If you try to define `B := A + 1`, the compiler will catch the **Circular Dependency (L004)** before a single line of code runs. This ensures that your system never enters an infinite calculation loop.

### **B. The Snapshot VM & Atomic Ticks**
Every state change in Lumina happens in an **Atomic Tick**. 
1. **Snapshot**: Before the tick, the VM takes a bit-level copy of the entire system state.
2. **Propagation**: Every field is recalculated in strict topological order.
3. **Validation**: All `@range` and safety invariants are checked.
4. **Commit/Rollback**: If any rule fails or an invariant is breached, the engine **Rolls Back** to the snapshot. Your system state is never "half-updated" or corrupt.

---

## 4. Key Use Cases Revisited

### **📡 Industrial IoT**
Monitor 10,000 sensors. Use `O(1)` aggregates to summarize health across the entire fleet instantly.
```lumina
aggregate FactoryHealth over Sensor { 
 alerting_nodes := count(is_alerting) 
}
```

### **☁️ Cloud Compliance**
Ensure infrastructure remains secure. Detect "drift" from your desired state and remediate it without manual polling.

### **🌍 Multi-Datacenter Orchestration**
Using 's native clustering, detect when an entire datacenter region is degrading and automatically evaluate `migrate` expressions to evacuate workloads to healthy peers using quorum-based Raft elections.

### **📊 Reliability Engineering**
Build alerting systems that auto-resolve. Use the `on clear` block to send recovery signals the millisecond a condition is no longer true.

---

## 5. Next Steps

*  **[Installation Guide](guides/distribution.md)**: Setup the `lumina` CLI.
*  **[Zero-to-Hero Curriculum](book/zero-to-hero.md)**: Your first 5 minutes with the language.
*  **[Interactive Playground](https://lumina-lang.web.app/playground)**: Test your mental model in a live simulation.

---

_Describing what is true. Automating what to do._


---

<!-- Source: installation.md -->

# Installing Lumina

Lumina "Architect" offers multiple ways to get started on your machine.

## 1. Automated Installer (Recommended)

The easiest way to install Lumina is via the one-line installer script. It automatically detects your platform and downloads the correct binaries.

**Linux / macOS:**
```bash
curl -fsSL https://lumina-lang.web.app/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://lumina-lang.web.app/install.ps1 | iex
```
*(Or using curl: `curl.exe -fsSL https://lumina-lang.web.app/install.ps1 | powershell -noprofile -c -`)*

## 2. Homebrew (macOS)

If you use Homebrew, you can install Lumina using our official tap:

```bash
brew tap IshimweIsaac/lumina
brew install lumina
```

This installs both the `lumina` CLI and the `lumina-lsp` (Language Server Protocol) for IDE support.

## 3. Manual Binary Downloads

You can download the binaries directly from our [GitHub Releases](https://github.com/IshimweIsaac/Lumina/releases).

| Platform | Binary | Architecture |
| --- | --- | --- |
| **Linux** | [lumina-linux-x64](https://github.com/IshimweIsaac/Lumina/releases/latest/download/lumina-linux-x64) | x86_64 |
| **Linux** | [lumina-linux-arm64](https://github.com/IshimweIsaac/Lumina/releases/latest/download/lumina-linux-arm64) | ARM64 |
| **macOS** | [lumina-macos-x64](https://github.com/IshimweIsaac/Lumina/releases/latest/download/lumina-macos-x64) | x86_64 (Intel) |
| **macOS** | [lumina-macos-arm64](https://github.com/IshimweIsaac/Lumina/releases/latest/download/lumina-macos-arm64) | ARM64 (Apple Silicon) |
| **Windows** | [lumina-windows-x64.exe](https://github.com/IshimweIsaac/Lumina/releases/latest/download/lumina-windows-x64.exe) | x86_64 |

### Verification

After downloading, verify the checksum to ensure the file has not been tampered with:

```bash
# Example for Linux x64
curl -LO https://github.com/IshimweIsaac/Lumina/releases/latest/download/lumina-linux-x64.sha256
sha256sum -c lumina-linux-x64.sha256
```

## 4. Verification

Once installed, verify the installation by checking the version:

```bash
lumina --version
```

Expected output: `Lumina` (or similar).


---

<!-- Source: infrastructure.md -->

# [Flagship Guide] Target Infrastructure Monitoring

Lumina is a **Reactive Language for Target Infrastructure**. It is designed to model, monitor, and manage the desired state of complex digital ecosystems at scale. In this flagship guide, we'll build a production-ready system that monitors disk space usage across a global fleet of virtual nodes.

## 1. The Scenario
You have a fleet of 500 virtual nodes, each sending their health data via an HTTP endpoint. You want to:
1. Aggregate the average disk usage across the whole fleet.
2. Alert when the fleet-wide average disk usage exceeds 85%.
3. Alert when ANY individual node is critically low on space (e.g., > 95%).

## 2. Define the Fleet
First, we define what an individual "Node" looks like and where we get its data.

```lumina
external entity Node {
 sync: "https://api.your-infra.com/v1/nodes/{{id}}"
 on: poll
 poll_interval: 1m
 
 id: Text
 disk_usage: Number
}
```

## 3. Aggregate the Results
Lumina makes it easy to calculate statistics across all instances of an entity.

```lumina
aggregate FleetStats over Node {
 avg_disk := avg(disk_usage)
 max_disk := max(disk_usage)
}
```

## 4. Define the Rules
Now, we describe the rules for both the global fleet and the individual nodes.

```lumina
-- Rule 1: A fleet-wide global alert
rule "Fleet Disk Usage High"
when FleetStats.avg_disk > 85 {
 alert severity: "warning",
    message: "Fleet-wide average disk space is critical: ${FleetStats.avg_disk}%",
    source: "infra-monitor"
}

-- Rule 2: A per-node critical alert
rule "Node Disk Space Critical"
for (n: Node)
when n.disk_usage > 95 {
 alert severity: "critical",
    message: "Node ${n.id} is almost out of disk space: ${n.disk_usage}%",
    source: "infra-monitor"
}
```

## 5. Why this is better
- **Global Awareness**: Lumina simplifies fleet-wide aggregation (Rule 1) while still allowing you to monitor individual instances (Rule 2) in the same file.
- **Polling Integrated**: By defining `on: poll` and `poll_interval: 1m`, Lumina handles all the timing logic for you. 
- **Consistency**: The same language can be used for your IoT devices and your cloud servers.


---

<!-- Source: iot.md -->

# [Sector Guide] Industrial IoT with Lumina

Lumina's **Target Infrastructure** philosophy extends directly into the physical world. In this sector-specific guide, we'll build an Industrial IoT monitoring system that tracks temperature threshold violations across a fleet of physical hardware sensors.

## 1. The Scenario
You have a set of temperature sensors publishing to an MQTT broker. You want to:
1. Track the current temperature for each sensor.
2. Alert when any sensor exceeds 40°C for more than 30 seconds.
3. Automatically clear the alert when the temperature drops back down.

## 2. Define the Entity
First, we describe what a "Sensor" looks like and where its data comes from.

```lumina
external entity Sensor {
 -- The MQTT topic where data arrives
 sync: "sensors/temp/{{id}}"
 on: realtime
 
 -- The fields we care about
 id: Text
 temperature: Number
}
```

## 3. Define the Rule
Instead of writing an `if` statement that checks every incoming packet, we describe the **state** we want to avoid.

```lumina
rule "High Temperature Alert"
for (s: Sensor)
when s.temperature > 40 for 30s {
 alert severity: "critical",
    message: "Sensor ${s.id} is overheating!",
    source: "thermal-monitor"
}
on clear {
 alert severity: "info",
    message: "Sensor ${s.id} has cooled down.",
    source: "thermal-monitor"
}
```

## 4. Run the Monitor
Once you've written your `.lum` file, you can run it pointing to your MQTT broker:

```bash
lumina run monitor.lum --server mqtt://your-broker:1883
```

## 5. Why this is better
- **Declarative**: You didn't write a loop or a state machine. You just described the condition (`> 40 for 30s`).
- **Automatic State**: The `on clear` block is handled by Lumina's internal reactor. It knows exactly when the "High Temperature" condition is no longer true for that specific sensor.
- **Scale**: This single rule works whether you have 1 sensor or 10,000 sensors.


---

