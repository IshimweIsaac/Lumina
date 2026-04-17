# Chapter 2: The Zero-to-Hero Curriculum 🚀

Welcome to the definitive guide to becoming a Lumina expert. This curriculum builds from foundational concepts to advanced reactive systems through a series of hands-on tutorials.

---

## 2.1 The Foundations (v1.3 Concepts)

### **Lesson 1: Your First Entity**
In Lumina, everything starts with an `entity`. An entity is a template for state.
```lumina
entity SmartHome {
    name: String
    temp: Number
    targetTemp: Number
}
```
Open the **Playground**, paste this code, and click "Run". You can now create an instance of `SmartHome` and see its fields.

### **Lesson 2: Derived Fields (The Core)**
Derived fields use the `:=` operator. They are automatically calculated.
```lumina
entity SmartHome {
    temp: Number
    targetTemp: Number
    
    # This is a derived field!
    isCoolingRequired := temp > targetTemp
}
```
Try changing the `temp` in the Playground. Notice how `isCoolingRequired` updates instantly.

---

## 2.2 Adding Reactive Logic (v1.5/1.6)

### **Lesson 3: The `rule` of Law**
Rules allow you to take action when state changes. Rules use **triggers**.
```lumina
rule "Cooling On" when SmartHome.isCoolingRequired becomes true {
    alert severity: "info", message: "{SmartHome.name}: Turning on AC!"
}
```
The `becomes` keyword is crucial. It only fires when the condition *transitions* to true.

### **Lesson 4: Historical Context (`prev`)**
Lumina stores the previous state automatically. Use it to detect trends.
```lumina
entity SmartHome {
    temp: Number
    # Detect a rapid temperature spike!
    tempSpike := temp - prev(temp) > 5
}

rule "Emergency" when SmartHome.tempSpike becomes true {
    alert severity: "critical", message: "CRITICAL TEMP SPIKE DETECTED!"
}
```

---

## 2.3 Advanced Fleet Operations (v1.6+)

### **Lesson 5: Aggregates**
What if you have 1,000 smart homes? Use `aggregate` to summarize them.
```lumina
aggregate Neighborhood over SmartHome {
    avgTemp := avg(temp)
    homesInCooling := count(isCoolingRequired)
}
```
Lumina updates these aggregates incrementally (`O(1)` performance), so they are never out of sync.

### **Lesson 6: Fleet Triggers (`any` / `all`)**
Rules can monitor the entire fleet at once.
```lumina
rule "Grid Alert" when any SmartHome.isCoolingRequired becomes true {
    alert severity: "warning", message: "Grid load increasing!"
}
```

---

## 2.4 Mastering v1.8 Experience Features

### **Lesson 7: Durable Logic (Stabilization)**
In the real world, sensors are noisy. Use `for duration` to stabilize your rules.
```lumina
rule "Stable Cooling" when SmartHome.temp > 25 for 10m {
    alert severity: "info", message: "Temp HAS been high for 10 minutes. Activating."
}
```

### **Lesson 8: The Diagnostic System**
V1.7 introduces "Teaching" error messages. Try creating a circular dependency:
```lumina
entity Loop {
    a := b + 1
    b := a + 1
}
```
The compiler (Analyzer) will now give a detailed, human-readable explanation:
> **L004: Circular Dependency Detected**
> I noticed that 'a' depends on 'b', which in turn depends back on 'a'. Lumina requires a Directed Acyclic Graph (DAG) for evaluation. To fix this, break the circuit...

---

## 2.5 Summary: Your Journey Has Begun
You have mastered:
1.  Declaring state with **Entities** and **Stored Fields**.
2.  Describing truth with **Derived Fields**.
3.  Automating actions with **Rules** and **Triggers**.
4.  Reasoning about time with **`prev()`** and **`for duration`**.
5.  Summarizing fleets with **Aggregates**.

You are now ready to dive deep into Chapter 3: Engine Internals.
