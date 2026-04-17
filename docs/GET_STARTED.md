# Getting Started: The Lumina Mental Model 🛰️

Welcome to the official introduction to Lumina. This guide is designed to shift your perspective from **procedural** programming to **reactive** truth-telling. By the end of this page, you will understand not just how to write Lumina, but why it is the most robust way to monitor complex state.

---

## 1. The Core Philosophy: "Describe What is True"

Most programming languages are **Step-By-Step Instructions**. To check a temperature, you write:
1.  Read sensor data.
2.  Compare to threshold.
3.  If high, start a timer.
4.  If still high after 5 minutes, send an alert.

In Lumina, you skip the steps and **Describe the State**:
> "An alert exists if the sensor temperature is above the threshold for 5 minutes."

Lumina's engine then handles the reading, the comparison, the timing, and the state cleanup automatically. This is **Declarative Reactivity**.

---

## 2. The 3-Tier Layered Logic

Lumina programs are built using three distinct layers of data, each with different properties:

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

---

## 3. Safety Guarantees: Why Lumina is different.

Lumina isn't just a language; it's a **Correctness Engine**. It provides two mathematical guarantees that traditional languages ignore:

### **A. Directed Acyclic Graph (DAG) Stability**
When you define a derived field `A := B + C`, Lumina builds a dependency graph. If you try to define `B := A + 1`, the compiler will catch the **Circular Dependency (L004)** before a single line of code runs. This ensures that your system never enters an infinite calculation loop.

### **B. The Snapshot VM & Atomic Ticks**
Every state change in Lumina happens in an **Atomic Tick**. 
1.  **Snapshot**: Before the tick, the VM takes a bit-level copy of the entire system state.
2.  **Propagation**: Every field is recalculated in strict topological order.
3.  **Validation**: All `@range` and safety invariants are checked.
4.  **Commit/Rollback**: If any rule fails or an invariant is breached, the engine **Rolls Back** to the snapshot. Your system state is never "half-updated" or corrupt.

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

### **📊 Reliability Engineering**
Build alerting systems that auto-resolve. Use the `on clear` block to send recovery signals the millisecond a condition is no longer true.

---

## 5. Next Steps

*   **[Installation Guide](guides/distribution.md)**: Setup the `lumina` CLI.
*   **[Zero-to-Hero Curriculum](book/zero-to-hero.md)**: Your first 5 minutes with the language.
*   **[Interactive Playground](https://lumina-lang.web.app/playground)**: Test your mental model in a live simulation.

---

_Describing what is true. Automating what to do._
