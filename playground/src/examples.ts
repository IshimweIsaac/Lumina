export const EXAMPLES = {
  fleet: {
    name: "Delivery Fleet",
    description: "Monitor a fleet of delivery motos",
    source: `
entity Moto {
  battery: Number
  isOnline: Boolean
  label: Text
  isLowBattery := battery < 20
}

aggregate FleetStatus over Moto {
  avgBattery := avg(battery)
  onlineCount := count(isOnline)
  anyLow := any(isLowBattery)
}

rule LowBattery for (m: Moto)
when m.isLowBattery becomes true cooldown 5m {
  alert severity: "warning", source: m.label,
  message: "low battery: {m.battery}%"
} on clear {
  alert severity: "resolved", source: m.label,
  message: "battery recovered"
}

let moto1 = Moto { battery: 80, isOnline: true, label: "moto-north-1" }
let moto2 = Moto { battery: 45, isOnline: true, label: "moto-north-2" }
let moto3 = Moto { battery: 12, isOnline: true, label: "moto-south-1" }
`
  },
  sensors: {
    name: "Temperature Sensor Network",
    description: "Multi-sensor aggregate monitoring with average temperature tracking",
    source: `
entity Sensor {
  temperature: Number
  location: Text
  isHighTemp := temperature > 80
}

aggregate NetworkStatus over Sensor {
  avgTemp := avg(temperature)
  maxTemp := max(temperature)
  criticalCount := count(isHighTemp)
}

rule CriticalTemp for (s: Sensor)
when s.isHighTemp becomes true {
  alert severity: "critical", source: s.location,
  message: "critical temperature detected: {s.temperature}C"
}

let lobby = Sensor { temperature: 22, location: "Main Lobby" }
let serverRoom = Sensor { temperature: 75, location: "Server Room" }
let warehouse = Sensor { temperature: 85, location: "Warehouse A" }
`
  },
  datacenter: {
    name: "Data Center Basic Monitoring",
    description: "Monitoring CPU and Power usage across server racks",
    source: `
entity Rack {
  powerUsage: Number
  cpuTotal: Number
  rackId: Text
  isOverloaded := cpuTotal > 90
}

aggregate DataCenter over Rack {
  totalPower := sum(powerUsage)
  avgCpu := avg(cpuTotal)
  anyOverload := any(isOverloaded)
}

rule OverloadAlert for (r: Rack)
when r.isOverloaded becomes true {
  alert severity: "error", source: r.rackId,
  message: "CPU utilization above 90%"
}

let rackA = Rack { powerUsage: 1200, cpuTotal: 45, rackId: "R-01" }
let rackB = Rack { powerUsage: 2500, cpuTotal: 96, rackId: "R-02" }
`
  },
  agriculture: {
    name: "Smart Agriculture",
    description: "Soil moisture and irrigation control simulation",
    source: `
entity Zone {
  moisture: Number
  label: Text
  needsWater := moisture < 30
}

aggregate FarmStatus over Zone {
  avgMoisture := avg(moisture)
  dryZones := count(needsWater)
}

rule AutoIrrigation for (z: Zone)
when z.needsWater becomes true {
  alert severity: "info", source: z.label,
  message: "starting irrigation: moisture at {z.moisture}%"
} on clear {
  alert severity: "resolved", source: z.label,
  message: "zone hydrated"
}

let zoneNorth = Zone { moisture: 80, label: "North Field" }
let zoneSouth = Zone { moisture: 25, label: "South Field" }
`
  }
};

export type ExampleKey = keyof typeof EXAMPLES;
