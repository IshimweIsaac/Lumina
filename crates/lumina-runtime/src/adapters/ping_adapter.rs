use crate::value::Value;
use std::collections::HashMap;
use std::process::Command;

/// A native Lumina sensor for network reachability.
/// Wraps the system 'ping' command for broad compatibility.
pub struct PingAdapter {
    entity_name: String,
    targets: HashMap<String, String>, // instance_name -> ip/hostname
}

impl PingAdapter {
    pub fn new(entity_name: &str) -> Self {
        Self {
            entity_name: entity_name.to_string(),
            targets: HashMap::new(),
        }
    }

    pub fn add_target(&mut self, instance: &str, target: &str) {
        self.targets.insert(instance.to_string(), target.to_string());
    }
}

impl crate::adapter::LuminaAdapter for PingAdapter {
    fn entity_name(&self) -> &str {
        &self.entity_name
    }

    fn poll(&mut self) -> Vec<(String, String, Value)> {
        let mut updates = vec![];

        for (instance, target) in &self.targets {
            // Run a single ping with 1s timeout
            let output = if cfg!(windows) {
                Command::new("ping")
                    .args(&["-n", "1", "-w", "1000", target])
                    .output()
            } else {
                Command::new("ping")
                    .args(&["-c", "1", "-W", "1", target])
                    .output()
            };

            match output {
                Ok(out) if out.status.success() => {
                    updates.push((instance.clone(), "up".to_string(), Value::Bool(true)));
                    // Parse latency if possible (simplified for now)
                    updates.push((instance.clone(), "status".to_string(), Value::Text("online".to_string())));
                }
                _ => {
                    updates.push((instance.clone(), "up".to_string(), Value::Bool(false)));
                    updates.push((instance.clone(), "status".to_string(), Value::Text("offline".to_string())));
                }
            }
        }

        updates
    }

    fn on_write(&mut self, instance: &str, field: &str, value: &Value) {
        if field == "target" {
            if let Some(target) = value.as_text() {
                self.add_target(instance, target);
            }
        }
    }
}
