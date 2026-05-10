use lumina_runtime::Evaluator;
use lumina_runtime::value::Value;
use lumina_runtime::store::Instance;
use lumina_runtime::adapters::static_adapter::StaticAdapter;
use lumina_analyzer::types::Schema;
use lumina_analyzer::graph::DependencyGraph;
use std::collections::HashMap;

#[test]
fn test_multi_instance_adapter_polling() {
    // 1. Setup a schema with an external entity
    let mut schema = Schema::new();
    schema.register_field("Sensor", "temp", &lumina_analyzer::types::LuminaType::Number);
    
    if let Some(sensor_ent) = schema.entities.get_mut("Sensor") {
        sensor_ent.is_external = true;
    }

    // 2. Create an evaluator
    let mut ev = Evaluator::new(schema, DependencyGraph::new(), vec![]);

    // 3. Insert two instances of the Sensor entity
    let mut inst1 = Instance::new("Sensor", 1);
    inst1.set(0, Value::Number(20.0));
    ev.store.insert("sensor-A", inst1);

    let mut inst2 = Instance::new("Sensor", 1);
    inst2.set(0, Value::Number(25.0));
    ev.store.insert("sensor-B", inst2);

    ev.instances.insert("sensor-A".to_string(), "Sensor".to_string());
    ev.instances.insert("sensor-B".to_string(), "Sensor".to_string());

    // 4. Register a StaticAdapter and push updates for BOTH instances
    let mut adapter = StaticAdapter::new("Sensor");
    adapter.push("sensor-A", "temp", Value::Number(30.0));
    adapter.push("sensor-B", "temp", Value::Number(35.0));
    ev.register_adapter(Box::new(adapter));

    // 5. Tick the engine
    let _ = ev.tick().expect("Tick failed");

    // 6. Verify BOTH instances were updated correctly
    let inst_a = ev.store.get("sensor-A").expect("sensor-A missing");
    assert_eq!(ev.get_instance_field(inst_a, "temp").unwrap(), Value::Number(30.0));

    let inst_b = ev.store.get("sensor-B").expect("sensor-B missing");
    assert_eq!(ev.get_instance_field(inst_b, "temp").unwrap(), Value::Number(35.0));
}
