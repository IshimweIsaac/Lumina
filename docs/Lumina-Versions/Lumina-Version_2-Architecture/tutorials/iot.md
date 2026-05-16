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
