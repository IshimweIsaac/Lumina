use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::value::Value;

/// Manages persistent state for infrastructure adapters (e.g. Proxmox, AWS).
/// Acts as the `.tfstate` equivalent for Lumina.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StateStore {
    // Key: Adapter instance name, Value: Metadata dictionary
    pub mappings: HashMap<String, HashMap<String, Value>>,
}

impl StateStore {
    const STATE_FILE: &'static str = "lumina.state.json";

    pub fn load() -> Self {
        if Path::new(Self::STATE_FILE).exists() {
            if let Ok(content) = fs::read_to_string(Self::STATE_FILE) {
                if let Ok(store) = serde_json::from_str(&content) {
                    return store;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;
        fs::write(Self::STATE_FILE, json)
            .map_err(|e| format!("Failed to write state file: {}", e))
    }

    pub fn get_metadata(&self, instance: &str) -> Option<&HashMap<String, Value>> {
        self.mappings.get(instance)
    }

    pub fn set_metadata(&mut self, instance: &str, metadata: HashMap<String, Value>) -> Result<(), String> {
        self.mappings.insert(instance.to_string(), metadata);
        self.save()
    }

    pub fn remove_metadata(&mut self, instance: &str) -> Result<(), String> {
        self.mappings.remove(instance);
        self.save()
    }
}
