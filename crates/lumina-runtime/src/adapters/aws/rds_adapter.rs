use crate::value::Value;
use std::collections::HashMap;
use tokio::runtime::Handle;
use tokio::task::block_in_place;
use aws_sdk_rds::Client;
use crate::adapters::aws::credentials::AwsConfig;

pub struct RdsAdapter {
    entity_name: String,
    client: Option<Client>,
}

impl RdsAdapter {
    pub fn new(entity_name: &str) -> Self {
        Self {
            entity_name: entity_name.to_string(),
            client: None,
        }
    }

    fn get_client(&mut self, desired: Option<&HashMap<String, Value>>) -> Client {
        if let Some(c) = &self.client {
            return c.clone();
        }
        let config = AwsConfig::new(desired);
        let client = Client::new(&config.sdk_config);
        self.client = Some(client.clone());
        client
    }
}

impl crate::adapter::LuminaAdapter for RdsAdapter {
    fn entity_name(&self) -> &str {
        &self.entity_name
    }

    fn poll(&mut self) -> Vec<(String, String, Value)> {
        let mut updates = vec![];
        let client = self.get_client(None);

        let result = block_in_place(|| {
            Handle::current().block_on(client.describe_db_instances().send())
        });

        if let Ok(response) = result {
            for instance in response.db_instances() {
                if let Some(name) = instance.db_instance_identifier() {
                    let name_str = name.to_string();
                    let status = instance.db_instance_status().unwrap_or("unknown").to_string();
                    
                    updates.push((name_str.clone(), "status".to_string(), Value::Text(status)));
                    updates.push((name_str, "verified".to_string(), Value::Bool(true)));
                }
            }
        }
        updates
    }

    fn on_write(&mut self, _instance: &str, _field: &str, _value: &Value) {}

    fn provision(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> {
        let client = self.get_client(Some(desired));
        
        let exists = block_in_place(|| {
            Handle::current().block_on(client.describe_db_instances().db_instance_identifier(instance).send())
        }).is_ok();

        if !exists {
            println!("\x1b[33m[PROVISIONING] RDS Database '{}'...\x1b[0m", instance);
            
            let engine = desired.get("engine").and_then(|v| v.as_text()).unwrap_or("postgres");
            let instance_class = desired.get("instance_class").and_then(|v| v.as_text()).unwrap_or("db.t3.micro");
            let allocated_storage = desired.get("allocated_storage").and_then(|v| v.as_number()).unwrap_or(20.0) as i32;
            
            let mut req = client.create_db_instance()
                .db_instance_identifier(instance)
                .engine(engine)
                .db_instance_class(instance_class)
                .allocated_storage(allocated_storage)
                .master_username(desired.get("master_username").and_then(|v| v.as_text()).unwrap_or("postgres"))
                .master_user_password(desired.get("master_password").and_then(|v| match v {
                    Value::Text(s) => Some(s.as_str()),
                    Value::Secret(s) => Some(s.as_str()),
                    _ => None
                }).unwrap_or("lumina12345"));
                
            if let Some(db_name) = desired.get("db_name").and_then(|v| v.as_text()) {
                req = req.db_name(db_name);
            }
            
            let result = block_in_place(|| Handle::current().block_on(req.send()));
            
            match result {
                Ok(_) => {
                    println!("\x1b[32m[CREATED] RDS Database '{}' provisioned\x1b[0m", instance);
                }
                Err(e) => {
                    eprintln!("\x1b[31m[RDS ERROR] Failed to provision {}: {}\x1b[0m", instance, e);
                    return Err(e.to_string());
                }
            }
        } else {
            println!("\x1b[34m[VERIFIED] RDS Database '{}' already exists.\x1b[0m", instance);
        }
        
        Ok(())
    }

    fn reconcile(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> {
        self.provision(instance, desired)
    }

    fn destroy(&mut self, instance: &str) -> Result<(), String> {
        let client = self.get_client(None);
        
        let result = block_in_place(|| {
            Handle::current().block_on(
                client.delete_db_instance()
                    .db_instance_identifier(instance)
                    .skip_final_snapshot(true)
                    .send()
            )
        });
        
        if let Err(e) = result {
            return Err(e.to_string());
        }
        Ok(())
    }
}
