use crate::value::Value;
use std::collections::HashMap;
use sysinfo::System;

/// A native Lumina sensor for local system processes and health.
pub struct ProcessAdapter {
    entity_name: String,
    system: System,
    watch_list: HashMap<String, String>, // instance_name -> process_name
}

impl ProcessAdapter {
    pub fn new(entity_name: &str) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self {
            entity_name: entity_name.to_string(),
            system,
            watch_list: HashMap::new(),
        }
    }

    pub fn watch(&mut self, instance: &str, process_name: &str) {
        self.watch_list.insert(instance.to_string(), process_name.to_string());
    }
}

impl crate::adapter::LuminaAdapter for ProcessAdapter {
    fn entity_name(&self) -> &str {
        &self.entity_name
    }

    fn poll(&mut self) -> Vec<(String, String, Value)> {
        let mut updates = vec![];
        self.system.refresh_all();

        for (instance, proc_name) in &self.watch_list {
            let mut found = false;
            let mut cpu_usage = 0.0;
            let mut mem_usage = 0.0;

            for process in self.system.processes_by_exact_name(proc_name) {
                found = true;
                cpu_usage += process.cpu_usage();
                mem_usage += process.memory() as f64 / 1024.0 / 1024.0; // MB
            }

            updates.push((instance.clone(), "running".to_string(), Value::Bool(found)));
            if found {
                updates.push((instance.clone(), "cpu_percent".to_string(), Value::Number(cpu_usage as f64)));
                updates.push((instance.clone(), "memory_mb".to_string(), Value::Number(mem_usage)));
                updates.push((instance.clone(), "status".to_string(), Value::Text("active".to_string())));
            } else {
                updates.push((instance.clone(), "status".to_string(), Value::Text("missing".to_string())));
            }
        }

        // Global system stats if the instance is "local_system"
        if self.watch_list.contains_key("local_system") {
            updates.push(("local_system".to_string(), "total_cpu_percent".to_string(), Value::Number(self.system.global_cpu_info().cpu_usage() as f64)));
            updates.push(("local_system".to_string(), "total_memory_mb".to_string(), Value::Number(self.system.total_memory() as f64 / 1024.0 / 1024.0)));
            updates.push(("local_system".to_string(), "used_memory_mb".to_string(), Value::Number(self.system.used_memory() as f64 / 1024.0 / 1024.0)));
        }

        updates
    }

    fn on_write(&mut self, instance: &str, field: &str, value: &Value) {
        if field == "proc_name" {
            if let Some(name) = value.as_text() {
                self.watch(instance, name);
            }
        }
    }
}
