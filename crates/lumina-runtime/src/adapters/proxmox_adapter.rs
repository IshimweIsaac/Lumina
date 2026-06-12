use std::collections::HashMap;
use crate::adapter::LuminaAdapter;
use crate::value::Value;
use crate::state_store::StateStore;

pub struct ProxmoxAdapter {
    entity_name: String,
    client: reqwest::blocking::Client,
}

impl ProxmoxAdapter {
    pub fn new(entity_name: String) -> Self {
        let client = reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_default();
        Self { 
            entity_name, 
            client,
        }
    }

    fn api_request(&self, method: reqwest::Method, url: &str, token: &str, endpoint: &str) -> Result<reqwest::blocking::Response, String> {
        let full_url = format!("{}{}", url, endpoint);
        
        let mut req = self.client.request(method, &full_url);
        if !token.is_empty() {
            req = req.header("Authorization", format!("PVEAPIToken={}", token));
        }

        req.send().map_err(|e| format!("Proxmox API Error: {}", e))
    }
}

impl LuminaAdapter for ProxmoxAdapter {
    fn entity_name(&self) -> &str {
        &self.entity_name
    }

    fn poll(&mut self) -> Vec<(String, String, Value)> {
        vec![]
    }

    fn provision(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> {
        let node = desired.get("node").and_then(|v| v.as_text()).unwrap_or("pve");
        let source_vmid = desired.get("source_vmid").and_then(|v| v.as_number()).unwrap_or(100.0) as u32;
        let new_vmid = desired.get("vmid").and_then(|v| v.as_number()).unwrap_or(101.0) as u32;
        let url = desired.get("proxmox_url").and_then(|v| v.as_text()).unwrap_or("https://localhost:8006");
        let token = desired.get("api_token").and_then(|v| v.as_text()).unwrap_or("");
        
        let mut store = StateStore::load();
        let mut metadata = HashMap::new();
        metadata.insert("url".to_string(), Value::Text(url.to_string()));
        metadata.insert("token".to_string(), Value::Text(token.to_string()));
        metadata.insert("node".to_string(), Value::Text(node.to_string()));
        metadata.insert("vmid".to_string(), Value::Number(new_vmid as f64));
        let _ = store.set_metadata(instance, metadata);
        
        // Check if exists first
        let status_endpoint = format!("/api2/json/nodes/{}/qemu/{}/status/current", node, new_vmid);
        let resp = self.api_request(reqwest::Method::GET, url, token, &status_endpoint);
        
        if let Ok(r) = resp {
            if r.status().is_success() {
                println!("\x1b[34m[VERIFIED] Proxmox VM '{}' already exists.\x1b[0m", new_vmid);
                return Ok(());
            }
        }

        println!("\x1b[33m[PROVISIONING] Cloning Proxmox VM {} to {}...\x1b[0m", source_vmid, new_vmid);
        let clone_endpoint = format!("/api2/json/nodes/{}/qemu/{}/clone?newid={}", node, source_vmid, new_vmid);
        let resp = self.api_request(reqwest::Method::POST, url, token, &clone_endpoint)?;
        
        if resp.status().is_success() {
            println!("\x1b[32m[CREATED] Proxmox VM {} created.\x1b[0m", new_vmid);
            Ok(())
        } else {
            Err(format!("Failed to clone VM: HTTP {}", resp.status()))
        }
    }

    fn destroy(&mut self, instance: &str) -> Result<(), String> {
        let mut store = StateStore::load();
        if let Some(metadata) = store.get_metadata(instance).cloned() {
            let url = metadata.get("url").and_then(|v| v.as_text()).unwrap_or("").to_string();
            let token = metadata.get("token").and_then(|v| v.as_text()).unwrap_or("").to_string();
            let node = metadata.get("node").and_then(|v| v.as_text()).unwrap_or("").to_string();
            let vmid = metadata.get("vmid").and_then(|v| v.as_number()).unwrap_or(0.0) as u32;

            println!("\x1b[31m[DESTROYING] Removing Proxmox VM {}...\x1b[0m", vmid);
            
            // Stop it first
            let stop_endpoint = format!("/api2/json/nodes/{}/qemu/{}/status/stop", node, vmid);
            let _ = self.api_request(reqwest::Method::POST, &url, &token, &stop_endpoint);

            // Delete
            let delete_endpoint = format!("/api2/json/nodes/{}/qemu/{}", node, vmid);
            let resp = self.api_request(reqwest::Method::DELETE, &url, &token, &delete_endpoint)?;
            
            if resp.status().is_success() {
                let _ = store.remove_metadata(instance);
                Ok(())
            } else {
                Err(format!("Failed to destroy VM: HTTP {}", resp.status()))
            }
        } else {
            Err("Cannot destroy Proxmox VM: missing metadata in state store (was never provisioned/reconciled)".to_string())
        }
    }

    fn reconcile(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> {
        self.provision(instance, desired)
    }
}
