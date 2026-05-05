// ─── Lumina Documentation Content (The Book of Lumina: Zero-to-Hero) ───
// A 15-chapter curriculum from absolute basics (v1.3) to advanced (v1.8)

export const DOCS = {
  getting_started: {
    title: "Getting Started",
    intro: null,
    sections: [
      {
        heading: "Build reactive systems in minutes",
        tagline: "Describe what is true. Lumina figures out what to do.",
        text: "No loops. No events. No state management. Just describe what is true.",
        code: `entity Moto {
  battery: Number
  isLowBattery := battery < 20
}

let bike = Moto { battery: 80 }

rule "Low Battery"
when Moto.isLowBattery becomes true {
  show "Battery critically low!"
}

-- Battery drains...
update bike.battery to 12`,
        file: "hook.lum"
      },
      {
        heading: "Start in Seconds",
        installBlock: true,
        text: "Zero setup required. Choose your path."
      },
      {
        heading: "One Idea to Understand",
        text: `In Lumina, you don't write steps.
You define **facts** and **rules**.
The system reacts automatically when those facts change.`,
        concepts: [
          { icon: "box", term: "Entity", desc: "A thing in the world" },
          { icon: "database", term: "Field", desc: "Data about it" },
          { icon: "zap", term: "Derived :=", desc: "Automatic truth" },
          { icon: "bell", term: "Rule", desc: "React to change" }
        ]
      },
      {
        heading: "Build Something in 5 Minutes",
        text: "Let's build a temperature monitor from zero to reactive in three steps.",
        guidedSteps: [
          {
            step: 1,
            title: "Define your entity",
            desc: "An entity is a blueprint. A derived field (:=) calculates itself automatically.",
            code: `entity Sensor {
  temperature: Number
  isHot := temperature > 30
}`
          },
          {
            step: 2,
            title: "Add a rule",
            desc: "Rules watch for moments of transition this fires the instant isHot becomes true.",
            code: `rule "Overheat"
when Sensor.isHot becomes true {
  show "Warning: temperature is high!"
}`
          },
          {
            step: 3,
            title: "Run it watch it react",
            desc: "Create a sensor, change the temperature, and the rule fires automatically.",
            code: `entity Sensor {
  temperature: Number
  isHot := temperature > 30
}

rule "Overheat"
when Sensor.isHot becomes true {
  show "Warning: temperature is high!"
}

let s = Sensor { temperature: 25 }
show s.isHot

update s.temperature to 35
show s.isHot`,
            file: "tutorial.lum"
          }
        ]
      },
      {
        heading: "Why This Is Different",
        text: "Humans think in **relationships**. Traditional code forces **procedures**. Lumina fixes that gap.",
        comparison: {
          traditional: {
            title: "Traditional Code",
            code: `let isHot = false

onDataReceived(temp) {
  if (temp > 30 && !isHot) {
    isHot = true
    sendAlert("Overheating!")
  }
  if (temp <= 30) {
    isHot = false
  }
}

setInterval(check, 5000)`
          },
          lumina: {
            title: "Lumina",
            code: `entity Sensor {\n  temperature: Number\n  isHot := temperature > 30\n}\n\nrule "Overheat"\nwhen Sensor.isHot becomes true {\n  alert message: "Overheating!"\n}`
          }
        }
      },
      {
        heading: "Built for the Real World",
        usecases: [
          { icon: "trending-up", title: "Finances", desc: "Real-time fraud detection and risk scoring" },
          { icon: "cpu", title: "IoT", desc: "State management for massive sensor networks" },
          { icon: "server", title: "Data Center", desc: "Predictive maintenance and auto-recovery" },
          { icon: "shield", title: "Security", desc: "Instant threat response and access control" }
        ]
      },
    ]
  },
  chapter0: {
    title: "Introduction",
    intro: "In the previous section, you built your first reactive system. Now let's understand what's happening behind the scenes. Before Lumina can react, it needs something to describe: **state**.",
    sections: [
      {
        heading: "The Absolute Basic: Entities",
        text: `In Lumina, everything starts with an **entity**.

An entity is a blueprint for something in your system like a sensor, a light, or a user. You saw this in "Getting Started" when we wrote \`entity Sensor { ... }\`.

Let's break it down with the simplest possible example:`,
        code: `entity Thermometer {\n  current_temp: Number\n}\n\n-- This is a blueprint. Nothing exists yet.`,
        file: "entity.lum"
      },
      {
        heading: "Instances: Bringing the World to Life",
        text: `A blueprint alone does nothing. You need to create a real **instance** an actual thing in your system.

The \`let\` keyword gives a name to an instance:`,
        code: `entity Thermometer {\n  current_temp: Number\n}\n\n-- NOW something exists\nlet t1 = Thermometer { current_temp: 22 }\n\nshow t1.current_temp`,
        file: "instances.lum"
      },
      {
        heading: "Stored Fields",
        text: `\`current_temp: Number\` is a **stored field** data you control directly.

You set it when you create an instance. It stays exactly as you set it until you explicitly change it.

In Lumina's earliest version (v1.3), this was all there was entities that hold data. No reactions yet. Just the foundation.`
      },
      {
        heading: "Changing State: The 'update' Command",
        text: `In Lumina, you don't reassign variables like \`x = 5\`.

Instead, you **update the world**. The \`update\` keyword changes a field on a specific instance:`,
        code: `entity Counter {\n  count: Number\n}\n\nlet c = Counter { count: 0 }\n\nupdate c.count to 1\nupdate c.count to 10\n\nshow c.count`,
        file: "updates.lum"
      },
      {
        heading: "The Key Idea",
        text: `Lumina separates two things:

- **State** → what exists (\`count\`, \`temperature\`, \`isHot\`)
- **Logic** → what should happen (rules, derived fields coming next)

This separation is what makes Lumina powerful. You describe the world first, and then you describe the truth about that world. The engine handles the rest.

Remember the \`:=\` from Getting Started? That's where logic begins. Next, we'll learn how fields can **live** updating themselves automatically.`
      }
    ]
  },
  chapter1: {
    title: "The Living Blueprint",
    intro: "You've seen entities hold data. Now watch what happens when a field starts thinking for itself.",
    sections: [
      {
        heading: "See It First",
        text: `Run this. Watch what happens to \`is_full\` when the level changes:`,
        code: `entity WaterTank {\n  level: Number\n  is_full := level > 90\n}\n\nlet tank = WaterTank { level: 50 }\nshow tank.is_full\n\nupdate tank.level to 95\nshow tank.is_full`,
        file: "derived.lum"
      },
      {
        heading: "What Just Happened?",
        text: `The \`:=\` operator created a **derived field**. Unlike a stored field, you never set it manually it calculates itself from other fields.

When \`level\` was 50, \`is_full\` was automatically \`false\`.
When you updated \`level\` to 95, \`is_full\` instantly became \`true\`.

No if-statements. No recalculation code. The truth stays true, automatically.`
      },
      {
        heading: "Try It Yourself",
        text: `Modify the code above:
- Change the threshold from \`90\` to \`70\`
- Add a second derived field: \`is_empty := level < 10\`
- Update the level to 5 and check both fields

**Key Takeaway:** Derived fields are living truth. They always reflect reality. You describe what "full" means once, and Lumina keeps it accurate forever.`
      }
    ]
  },
  chapter2: {
    title: "Automatic Reactions",
    intro: "Derived fields know what's true. Rules know what to do about it.",
    sections: [
      {
        heading: "See It First",
        text: `Run this. Notice that the rule fires exactly once at the moment of change:`,
        code: `entity Light {\n  isOn: Boolean\n}\n\nlet lamp = Light { isOn: false }\n\nrule "Welcome Home"\nwhen lamp.isOn becomes true {\n  show "The light was turned on!"\n}\n\n-- This triggers the rule\nupdate lamp.isOn to true\n\n-- This does NOT trigger (already true)\nupdate lamp.isOn to true`,
        file: "rules.lum"
      },
      {
        heading: "What Just Happened?",
        text: `The keyword \`becomes true\` is an **edge trigger**. It only fires at the exact moment a value transitions from \`false\` to \`true\`.

Setting \`isOn\` to \`true\` again does nothing there's no transition. This is how Lumina prevents alert storms and duplicate reactions.

In traditional code, you'd need a flag variable to track "has this already fired?" Lumina handles that automatically.`
      },
      {
        heading: "Try It Yourself",
        text: `Modify the code:
- Add a second rule for \`becomes false\` that shows "Light turned off!"
- Update \`lamp.isOn\` to \`false\` after turning it on

**Key Takeaway:** Rules react to transitions, not states. \`becomes\` is your edge detector it sees the moment of change.`
      }
    ]
  },
  chapter3: {
    title: "Constants & Bindings",
    intro: "Every instance needs a name. Let's learn how Lumina organizes the world.",
    sections: [
      {
        heading: "See It First",
        text: `Run this to see how \`let\` creates named instances:`,
        code: `entity User {\n  name: Text\n  isAdmin: Boolean\n}\n\nlet admin = User { name: "Isaac", isAdmin: true }\nlet guest = User { name: "Visitor", isAdmin: false }\n\nshow admin.name\nshow guest.isAdmin`,
        file: "bindings.lum"
      },
      {
        heading: "What Just Happened?",
        text: `\`let\` binds a name to an instance. Once bound, the name always refers to that specific instance.

You can have multiple instances of the same entity each with its own data. This is how you model real systems: many sensors, many users, many devices.`
      },
      {
        heading: "Try It Yourself",
        text: `Add a third user. Give them a derived field \`is_privileged := isAdmin\` and check it.

**Key Takeaway:** \`let\` gives names to things in your world. Each instance lives independently with its own state.`
      }
    ]
  },
  chapter4: {
    title: "State Guardrails",
    intro: "Real systems have physical limits. A battery can't be 150%. Lumina enforces this for you.",
    sections: [
      {
        heading: "See It First",
        text: `Run this. The second update is physically impossible watch what Lumina does:`,
        code: `entity Battery {\n  @range 0 to 100\n  charge: Number\n}\n\nlet b = Battery { charge: 50 }\n\nupdate b.charge to 80\nshow b.charge\n\n-- This is impossible. Lumina rolls back.\nupdate b.charge to 150`,
        file: "safety.lum"
      },
      {
        heading: "What Just Happened?",
        text: `\`@range 0 to 100\` tells Lumina the physical limits of this field. When you tried to set charge to 150, the engine **rolled back the entire update** as if it never happened.

This isn't just validation it's a safety guarantee. Your system state is never corrupt, never "half-updated." The engine either commits everything or nothing.`
      },
      {
        heading: "Try It Yourself",
        text: `Try updating \`charge\` to \`-10\`. What happens?

Then add a derived field: \`is_critical := charge < 15\`. Now your battery has both safety limits AND automatic truth detection.

**Key Takeaway:** \`@range\` makes invalid states impossible. Lumina protects your data at the engine level.`
      }
    ]
  },
  chapter5: {
    title: "The Alerting Engine",
    intro: "Showing a message is basic. Real systems need alerts that live, persist, and auto-resolve.",
    sections: [
      {
        heading: "See It First",
        text: `Run this. Watch what happens when the problem appears and when it goes away:`,
        code: `entity Machine {\n  is_overheating: Boolean\n}\n\nlet m1 = Machine { is_overheating: false }\n\nrule "Heat Guard"\nwhen Machine.is_overheating becomes true {\n  alert severity: "critical", message: "Engine temp critical!"\n  \n  on clear {\n    show "Engine has cooled down. Resuming."\n  }\n}\n\nupdate m1.is_overheating to true\nupdate m1.is_overheating to false`,
        file: "alerts.lum"
      },
      {
        heading: "What Just Happened?",
        text: `An \`alert\` in Lumina isn't a one-time message it's a **living state**. It persists as long as the condition is true.

The \`on clear\` block runs automatically the moment the condition resolves. No timers, no polling, no "check if problem is gone" loops.

In traditional code, you'd need: a flag to track alert state, a background job to check resolution, and cleanup logic to dismiss old alerts. Lumina does all of this in 3 lines.`
      },
      {
        heading: "Try It Yourself",
        text: `Change the severity to \`"warning"\`. Add a second machine and trigger both alerts.

**Key Takeaway:** Alerts are managed lifecycle objects. They fire, persist, and auto-resolve no manual cleanup required.`
      }
    ]
  },
  chapter6: {
    title: "Fleet Intelligence",
    intro: "Monitoring one sensor is easy. Monitoring a thousand is where Lumina shines.",
    sections: [
      {
        heading: "See It First",
        text: `Run this. Three nodes, one aggregate that summarizes all of them instantly:`,
        code: `entity Node {\n  load: Number\n  is_high := load > 90\n}\n\nlet n1 = Node { load: 10 }\nlet n2 = Node { load: 95 }\nlet n3 = Node { load: 98 }\n\naggregate ClusterStats over Node {\n  avg_load := avg(load)\n  busy_nodes := count(is_high)\n}\n\nshow ClusterStats.avg_load\nshow ClusterStats.busy_nodes`,
        file: "aggregates.lum"
      },
      {
        heading: "What Just Happened?",
        text: `\`aggregate\` computes values across **every instance** of an entity in real-time.

\`avg_load\` automatically averages \`load\` across all 3 nodes. \`busy_nodes\` counts how many have \`is_high = true\`. If you add or remove nodes, the aggregate updates instantly.

In traditional code, you'd iterate over collections, manage subscriptions, and recalculate manually. Lumina does it in O(1) constant time.`
      },
      {
        heading: "Try It Yourself",
        text: `Add a fourth node with load 50. Check how \`avg_load\` changes. Then add \`max_load := max(load)\` to the aggregate.

**Key Takeaway:** Aggregates give you fleet-level intelligence with zero iteration code. Describe what you want to know, Lumina keeps it current.`
      }
    ]
  },
  chapter7: {
    title: "Reusable Logic",
    intro: "When you use the same calculation twice, extract it into a function.",
    sections: [
      {
        heading: "See It First",
        text: `Run this. The function converts Fahrenheit to Celsius and it's used in a derived field:`,
        code: `fn to_celsius(f: Number) -> Number {\n  (f - 32) * 5 / 9\n}\n\nentity WeatherStation {\n  temp_f: Number\n  temp_c := to_celsius(temp_f)\n}\n\nlet ws = WeatherStation { temp_f: 68 }\nshow ws.temp_c`,
        file: "functions.lum"
      },
      {
        heading: "What Just Happened?",
        text: `\`fn\` defines a **pure function**. It calculates a value but cannot change any state. This is intentional pure functions are predictable, testable, and safe to use inside derived fields.

Because \`temp_c\` uses \`to_celsius\` in a derived field, it automatically updates whenever \`temp_f\` changes. The function is part of the reactive graph.`
      },
      {
        heading: "Try It Yourself",
        text: `Update \`temp_f\` to 100 and check \`temp_c\`. Then write a \`to_fahrenheit\` function that reverses the conversion.

**Key Takeaway:** Functions are pure calculation blocks. Use them inside derived fields to build complex logic that still stays reactive.`
      }
    ]
  },
  chapter8: {
    title: "Managed Collections",
    intro: "Sometimes you need a list of values, not just a single field.",
    sections: [
      {
        heading: "See It First",
        text: `Run this to see lists in action:`,
        code: `let scores = [85, 92, 78, 95]\n\nshow scores[0]\nshow count(scores)`,
        file: "lists.lum"
      },
      {
        heading: "What Just Happened?",
        text: `Lists are strongly typed collections. You can access elements by index and use built-in functions like \`count\`, \`sum\`, \`min\`, \`max\` on them.

Lists work seamlessly with the rest of Lumina you can store them in entity fields and use them in derived calculations.`
      },
      {
        heading: "Try It Yourself",
        text: `Try adding more scores to the list. Use \`sum(scores)\` and \`max(scores)\` to explore the built-in functions.

**Key Takeaway:** Lists give you collections with built-in aggregate functions no loops needed.`
      }
    ]
  },
  chapter9: {
    title: "The Web of Data",
    intro: "Real systems aren't flat. Sensors belong to rooms, rooms belong to buildings. Let's connect them.",
    sections: [
      {
        heading: "See It First",
        text: `Run this. One entity references another creating a relationship:`,
        code: `entity Room {\n  name: Text\n}\n\nentity Sensor {\n  ref location: Room\n  value: Number\n}\n\nlet r1 = Room { name: "Lab A" }\nlet s1 = Sensor { location: r1, value: 25 }\n\nshow s1.location.name`,
        file: "refs.lum"
      },
      {
        heading: "What Just Happened?",
        text: `\`ref location: Room\` creates a **relationship** between Sensor and Room. You can traverse it with dot notation: \`s1.location.name\`.

This builds a living data graph. When the Room's name changes, any Sensor referencing it sees the update instantly no JOIN queries, no manual lookups.`
      },
      {
        heading: "Try It Yourself",
        text: `Create a second room and a second sensor. Add a derived field to Sensor: \`room_name := location.name\` and check it.

**Key Takeaway:** \`ref\` connects entities into a graph. Traverse relationships with dot notation everything stays reactive.`
      }
    ]
  },
  chapter10: {
    title: "Filtering the Noise",
    intro: "Real-world sensors spike constantly. You don't want an alert for every 1-second glitch.",
    sections: [
      {
        heading: "See It First",
        text: `Run this. The rule waits 5 seconds before firing filtering out brief spikes:`,
        code: `entity Link {\n  is_down: Boolean\n}\n\nlet l = Link { is_down: false }\n\nrule "Stable Outage"\nwhen Link.is_down becomes true for 5s {\n  alert severity: "high", message: "Link is officially DOWN"\n}\n\nupdate l.is_down to true\n-- The rule waits 5 seconds before firing.`,
        file: "temporal.lum"
      },
      {
        heading: "What Just Happened?",
        text: `\`for 5s\` is a **duration qualifier**. The condition must remain true for the entire duration before the rule fires.

If \`is_down\` flips back to \`false\` within 5 seconds, the rule cancels no alert. This filters transient noise automatically.

In traditional code, you'd need timers, cancellation tokens, and careful state management. Lumina does it with two words.`
      },
      {
        heading: "Try It Yourself",
        text: `Change the duration to \`10s\`. Then try adding \`update l.is_down to false\` right after setting it to true the rule should cancel.

**Key Takeaway:** \`for\` filters noise. Only sustained conditions trigger reactions brief spikes are ignored.`
      }
    ]
  },
  chapter11: {
    title: "Frequency & Flapping",
    intro: "Sometimes the problem isn't one failure it's a pattern of failures.",
    sections: [
      {
        heading: "See It First",
        text: `Run this. The rule detects a power source that keeps flapping on and off:`,
        code: `entity PowerSource {\n  is_active: Boolean\n}\n\nrule "Power Flap Detection"\nwhen PowerSource.is_active becomes false\n  3 times within 60s {\n  alert severity: "warning", message: "Power source is flapping!"\n}\n\nlet p = PowerSource { is_active: true }\nupdate p.is_active to false\nupdate p.is_active to true\nupdate p.is_active to false\nupdate p.is_active to true\nupdate p.is_active to false`,
        file: "frequency.lum"
      },
      {
        heading: "What Just Happened?",
        text: `\`3 times within 60s\` is a **frequency trigger**. It counts how many times a transition happens within a time window.

Three false-transitions in 60 seconds means the power source is unstable not just failing once, but oscillating. Lumina tracks this pattern internally without any counters or timers in your code.`
      },
      {
        heading: "Try It Yourself",
        text: `Change the threshold to \`2 times within 30s\`. Add a second PowerSource and see if it triggers independently.

**Key Takeaway:** \`times within\` detects patterns, not just events. It catches erratic behavior that single-event rules would miss.`
      }
    ]
  },
  chapter12: {
    title: "The Time Dimension",
    intro: "In monitoring, how long ago something happened matters as much as what happened.",
    sections: [
      {
        heading: "See It First",
        text: `Run this. The heartbeat becomes "stale" if it hasn't been updated recently:`,
        code: `entity Heartbeat {\n  last_seen: Timestamp\n  is_stale := last_seen.age > 10s\n}\n\nlet h = Heartbeat { last_seen: now() }\nshow h.is_stale`,
        file: "timestamp.lum"
      },
      {
        heading: "What Just Happened?",
        text: `\`Timestamp\` is a built-in type. \`now()\` captures the current time, and \`.age\` tells you how long ago it was.

\`is_stale := last_seen.age > 10s\` is a derived field that becomes \`true\` automatically when 10 seconds pass without an update. Time is a first-class citizen in Lumina no cron jobs, no polling intervals.`
      },
      {
        heading: "Try It Yourself",
        text: `Add a rule that fires when a heartbeat becomes stale. Then update \`last_seen\` to \`now()\` to reset the timer.

**Key Takeaway:** \`Timestamp\` and \`.age\` let you build time-aware logic declaratively. Staleness detection, timeouts, and SLA monitoring all without loops.`
      }
    ]
  },
  chapter13: {
    title: "The External World",
    intro: "Lumina doesn't live in isolation. It connects to APIs, MQTT brokers, and external data sources.",
    sections: [
      {
        heading: "See It First",
        text: `This declares a connection to a weather API data flows in automatically:`,
        code: `external WeatherAPI sync on "https://api.weather.com/v1" {\n  temperature: Number\n  humidity: Number\n}\n\nrule "Rain Check"\nwhen WeatherAPI.humidity becomes > 90 {\n  show "Humidity spike in external data!"\n}`,
        file: "external.lum"
      },
      {
        heading: "What Just Happened?",
        text: `\`external\` declares an entity whose data comes from outside Lumina. The \`sync on\` clause tells the engine where to pull data from.

Once declared, external entities work exactly like regular entities you can write derived fields, rules, and aggregates against them. The engine handles polling, parsing, and synchronization.`
      },
      {
        heading: "Real-World Connection",
        text: `This is how Lumina connects to:
- **MQTT brokers** for IoT sensor data
- **REST APIs** for cloud services
- **Webhooks** for event-driven integrations

The reactive graph extends beyond your code into the real world.

**Key Takeaway:** \`external\` brings the outside world into Lumina's reactive graph. Describe the data source once, react to it forever.`
      }
    ]
  },
  chapter14: {
    title: "Automated Write-Back",
    intro: "Lumina doesn't just observe. It can send commands back to external systems.",
    sections: [
      {
        heading: "See It First",
        text: `This rule locks an airlock when any node goes critical writing back to an external device:`,
        code: `external AirLock {\n  is_locked: Boolean\n}\n\nentity Node {\n  temp: Number\n  is_critical := temp > 100\n}\n\nrule "Emergency Lockdown"\nwhen any Node.is_critical becomes true {\n  write AirLock.is_locked to true\n}`,
        file: "writeback.lum"
      },
      {
        heading: "What Just Happened?",
        text: `\`write\` sends a command to an external system. When any Node's temperature exceeds 100, Lumina automatically locks the AirLock.

This closes the loop: data comes **in** via \`external\`, reactions go **out** via \`write\`. Your monitoring system doesn't just watch it acts.`
      },
      {
        heading: "The Full Picture",
        text: `You've now seen the complete Lumina workflow:

- **Entities** describe the world
- **Derived fields** compute truth automatically
- **Rules** react to transitions
- **Alerts** persist and auto-resolve
- **Aggregates** summarize fleets
- **External** connects to the real world
- **Write** sends actions back

All of this with no loops, no event listeners, no state management. Just truth.

**Key Takeaway:** \`write\` completes the reactive loop. Lumina observes, decides, and acts declaratively.`
      }
    ]
  },
  error_reference: {
    title: "Error Reference",
    intro: "When something goes wrong, Lumina doesn't just crash it teaches you what happened and how to fix it.",
    sections: [
      {
        heading: "L-Series (Compile Time)",
        text: `These errors are caught **before** your code runs. The compiler sees the problem and stops you from making it worse.`,
        table: [
          ["L001", "Type Mismatch", "Adding text to a number."],
          ["L003", "Invalid Update", "Trying to update a derived field."],
          ["L004", "Circular Logic", "A depends on B, B depends on A."],
          ["L042", "Frequency Error", "Invalid window duration."]
        ]
      },
      {
        heading: "R-Series (Runtime)",
        text: `These errors occur during execution the data was valid at compile time, but something went wrong with the actual values.`,
        table: [
          ["R002", "Div by Zero", "Logic error in expression."],
          ["R006", "Range Violation", "@range check failed."],
          ["R007", "Sync Failure", "External source unreachable."]
        ]
      }
    ]
  },
  changelog: {
    title: "The Lumina Journey",
    intro: "From a minimal reactive runtime to a full developer ecosystem.",
    sections: [
      {
        heading: "v1.3: Foundations",
        text: "Entities, stored fields, derived fields, edge-triggered rules, and the Snapshot VM."
      },
      {
        heading: "v1.4: Developer Experience",
        text: "Pure functions, lists, string interpolation, and improved error messages."
      },
      {
        heading: "v1.5: Production Ready",
        text: "The alerting lifecycle, aggregates, and O(1) fleet-level intelligence."
      },
      {
        heading: "v1.6: Enterprise",
        text: "Entity references, temporal filters, frequency detection, and external sync."
      },
      {
        heading: "v1.8: Experience",
        text: "Teaching-standard diagnostics, interactive documentation, one-line installer, and VS Code extension."
      }
    ]
  }
};

export const EXAMPLES = [
  {
    title: "Phase 1: Genesis",
    desc: "A pure v1.3 greeting script.",
    file: "hello.lum",
    tags: ["v1.3", "Basics"],
    code: `entity Welcome { msg: Text }
let w = Welcome { msg: "Hello Lumina" }
show w.msg`
  },
  {
    title: "Phase 2: Fleet",
    desc: "v1.5 Aggregate power.",
    file: "fleet.lum",
    tags: ["v1.5", "Fleet"],
    code: `entity Node { load: Number }
let n1 = Node { load: 50 }
let n2 = Node { load: 90 }
aggregate Fleet over Node { total := sum(load) }
show Fleet.total`
  }
];
