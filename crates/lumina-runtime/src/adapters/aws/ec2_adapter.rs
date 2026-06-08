use crate::value::Value;
use std::collections::HashMap;
use tokio::runtime::Handle;
use tokio::task::block_in_place;
use aws_sdk_ec2::Client;
use aws_sdk_ec2::types::{ResourceType, TagSpecification, Tag, InstanceStateName};
use crate::adapters::aws::credentials::AwsConfig;

pub struct Ec2Adapter {
    entity_name: String,
    known_states: HashMap<String, String>,
    client: Option<Client>,
}

impl Ec2Adapter {
    pub fn new(entity_name: &str) -> Self {
        Self {
            entity_name: entity_name.to_string(),
            known_states: HashMap::new(),
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

impl crate::adapter::LuminaAdapter for Ec2Adapter {
    fn entity_name(&self) -> &str {
        &self.entity_name
    }

    fn poll(&mut self) -> Vec<(String, String, Value)> {
        let mut updates = vec![];
        let client = self.get_client(None);

        let result = block_in_place(|| {
            Handle::current().block_on(client.describe_instances().send())
        });

        if let Ok(response) = result {
            for reservation in response.reservations() {
                for instance in reservation.instances() {
                            let mut name = instance.instance_id().unwrap_or_default().to_string();
                            
                            for tag in instance.tags() {
                                if tag.key() == Some("Name") {
                                    if let Some(v) = tag.value() {
                                        name = v.to_string();
                                    }
                                }
                            }
                            
                            if name.is_empty() { continue; }

                            let status = instance.state().and_then(|s| s.name()).map(|n| n.as_str()).unwrap_or("unknown");
                            
                            self.known_states.insert(name.clone(), status.to_string());
                            updates.push((name.clone(), "status".to_string(), Value::Text(status.to_string())));
                            if let Some(id) = instance.instance_id() {
                                updates.push((name.clone(), "instance_id".to_string(), Value::Text(id.to_string())));
                            }
                            updates.push((name, "verified".to_string(), Value::Bool(true)));
                }
            }
        }
        updates
    }

    fn on_write(&mut self, _instance: &str, _field: &str, _value: &Value) {
        // Handled via provision/reconcile
    }

    fn provision(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> {
        let client = self.get_client(Some(desired));
        
        let instance_type = desired.get("instance_type").and_then(|v| v.as_text()).unwrap_or("t3.micro");
        let ami_id = desired.get("ami_id").and_then(|v| v.as_text()).ok_or("Missing ami_id")?;
        
        let exists = block_in_place(|| {
            Handle::current().block_on(
                client.describe_instances()
                    .filters(aws_sdk_ec2::types::Filter::builder().name("tag:Name").values(instance).build())
                    .send()
            )
        }).is_ok_and(|resp| {
            resp.reservations().iter().any(|r| {
                r.instances().iter().any(|i| {
                    !matches!(i.state().and_then(|s| s.name()), Some(InstanceStateName::Terminated) | Some(InstanceStateName::ShuttingDown))
                })
            })
        });

        if !exists {
            println!("\x1b[33m[PROVISIONING] EC2 Instance '{}' ({})...\x1b[0m", instance, instance_type);
            
            let tag_spec = TagSpecification::builder()
                .resource_type(ResourceType::Instance)
                .tags(Tag::builder().key("Name").value(instance).build())
                .build();
            
            let mut req = client.run_instances()
                .image_id(ami_id)
                .instance_type(aws_sdk_ec2::types::InstanceType::from(instance_type))
                .min_count(1)
                .max_count(1)
                .tag_specifications(tag_spec);
                
            if let Some(subnet_id) = desired.get("subnet_id").and_then(|v| v.as_text()) {
                req = req.subnet_id(subnet_id);
            }
            if let Some(key_name) = desired.get("key_name").and_then(|v| v.as_text()) {
                req = req.key_name(key_name);
            }
            if let Some(sg) = desired.get("security_group_ids").and_then(|v| v.as_list()) {
                for g in sg {
                    if let Some(g_str) = g.as_text() {
                        req = req.security_group_ids(g_str);
                    }
                }
            }

            let result = block_in_place(|| Handle::current().block_on(req.send()));
            
            match result {
                Ok(resp) => {
                    let id = resp.instances().first().and_then(|i| i.instance_id()).unwrap_or("unknown");
                    println!("\x1b[32m[CREATED] EC2 Instance '{}' launched with ID {}\x1b[0m", instance, id);
                }
                Err(e) => {
                    eprintln!("\x1b[31m[EC2 ERROR] Failed to provision {}: {}\x1b[0m", instance, e);
                    return Err(e.to_string());
                }
            }
        } else {
            println!("\x1b[34m[VERIFIED] EC2 Instance '{}' already exists.\x1b[0m", instance);
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
                client.describe_instances()
                    .filters(aws_sdk_ec2::types::Filter::builder().name("tag:Name").values(instance).build())
                    .send()
            )
        });
        
        if let Ok(resp) = result {
            let mut ids = vec![];
            for r in resp.reservations() {
                for i in r.instances() {
                    if !matches!(i.state().and_then(|s| s.name()), Some(InstanceStateName::Terminated)) {
                        if let Some(id) = i.instance_id() {
                            ids.push(id.to_string());
                        }
                    }
                }
            }
            
            if !ids.is_empty() {
                let term_result = block_in_place(|| {
                    Handle::current().block_on(
                        client.terminate_instances()
                            .set_instance_ids(Some(ids))
                            .send()
                    )
                });
                
                if let Err(e) = term_result {
                    return Err(e.to_string());
                }
            }
        }
        
        Ok(())
    }
}
