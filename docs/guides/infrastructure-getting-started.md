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
