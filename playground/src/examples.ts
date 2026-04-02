export const EXAMPLES = {
  fleet: {
    name: "Delivery Fleet",
    description: "Monitor a fleet of delivery motos",
    source: `
-- A fleet of delivery motos. Each has a battery.
-- When battery drops below 20% -- alert the dispatcher.
-- When any moto goes offline -- alert immediately.
-- When the whole fleet is offline -- critical alert.

entity Moto {
  battery: Number
  isOnline: Boolean
  label: Text
  isLowBattery := battery < 20
  isCritical := battery < 5
}

aggregate FleetStatus over Moto {
  avgBattery := avg(battery)
  onlineCount := count(isOnline)
  anyLow := any(isLowBattery)
  allOffline := all(isOnline)
}

rule LowBattery for (m: Moto)
when m.isLowBattery becomes true {
  alert severity: "warning",
  source: m.label,
  message: "low battery: {m.battery}% on {m.label}"
} on clear {
  alert severity: "resolved",
  source: m.label,
  message: "battery recovered: {m.battery}%"
} cooldown 10m

rule MotoOffline for (m: Moto)
when m.isOnline becomes false {
  alert severity: "critical",
  source: m.label,
  message: "moto offline: {m.label}"
} on clear {
  alert severity: "resolved",
  source: m.label,
  message: "moto back online: {m.label}"
}

rule FleetOffline
when all Moto.isOnline becomes false {
  alert severity: "critical",
  source: "fleet",
  message: "entire fleet offline -- fleet avg battery: {FleetStatus.avgBattery}%"
}

let moto1 = Moto { battery: 80, isOnline: true, label: "moto-north-1" }
let moto2 = Moto { battery: 45, isOnline: true, label: "moto-north-2" }
let moto3 = Moto { battery: 12, isOnline: true, label: "moto-south-1" }
`
  },
  sensors: {
    name: "Temperature Sensor Network",
    description: "External entities, sync on, prev() for drift detection",
    source: `
entity Sensor {
  temperature: Number
  location: Text
  isHighTemp := temperature > 80
  lastTemp := prev(temperature)
  isDrifting := abs(temperature - lastTemp) > 10
}

aggregate NetworkStatus over Sensor {
  avgTemp := avg(temperature)
  maxTemp := max(temperature)
  criticalCount := count(isHighTemp)
}

rule TempDrift for (s: Sensor)
when s.isDrifting becomes true {
  alert severity: "warning", source: s.location,
  message: "temp drift: {s.temperature}C from {s.lastTemp}C"
}

let lobby = Sensor { temperature: 22, location: "Main Lobby" }
let warehouse = Sensor { temperature: 85, location: "Warehouse A" }
`
  },
  datacenter: {
    name: "Data Center Basic Monitoring",
    description: "ref relationships, multi-condition triggers, aggregate health",
    source: `
entity CoolingUnit {
  temp: Number
  isOnline: Boolean
}

entity Server {
  cpu: Number
  cooling: ref CoolingUnit
  isOverheating := cpu > 80 and cooling.temp > 25
}

aggregate DataCenter over Server {
  avgCpu := avg(cpu)
  anyOverheating := any(isOverheating)
}

rule OverheatAlert for (s: Server)
when s.isOverheating becomes true {
  alert severity: "critical", source: "DataCenter",
  message: "Server overheating: CPU={s.cpu}%, CoolingTemp={s.cooling.temp}C"
}

let cooling1 = CoolingUnit { temp: 22, isOnline: true }
let coreServer = Server { cpu: 85, cooling: ref cooling1 }
`
  },
  agriculture: {
    name: "Smart Agriculture",
    description: "Frequency conditions, Timestamp type, write action simulation",
    source: `
entity Zone {
  moisture: Number
  label: Text
  needsWater := moisture < 30
}

rule DryCondition for (z: Zone)
when z.needsWater == true 3 times within 1h {
  alert severity: "info", source: z.label,
  message: "persistent dry condition on {z.label}"
}

let zoneNorth = Zone { moisture: 25, label: "North Field" }
let zoneSouth = Zone { moisture: 80, label: "South Field" }
`
  }
};

export type ExampleKey = keyof typeof EXAMPLES;
