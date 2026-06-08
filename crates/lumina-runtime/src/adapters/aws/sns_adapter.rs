use crate::value::Value;
use std::collections::HashMap;
use tokio::runtime::Handle;
use tokio::task::block_in_place;
use aws_sdk_sns::Client;
use crate::adapters::aws::credentials::AwsConfig;

pub struct SnsAdapter {
    entity_name: String,
    client: Option<Client>,
}

impl SnsAdapter {
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

impl crate::adapter::LuminaAdapter for SnsAdapter {
    fn entity_name(&self) -> &str {
        &self.entity_name
    }

    fn poll(&mut self) -> Vec<(String, String, Value)> {
        let mut updates = vec![];
        let client = self.get_client(None);

        let result = block_in_place(|| {
            Handle::current().block_on(client.list_topics().send())
        });

        if let Ok(response) = result {
            for topic in response.topics() {
                if let Some(arn) = topic.topic_arn() {
                    let parts: Vec<&str> = arn.split(':').collect();
                    if let Some(name) = parts.last() {
                        let name_str = name.to_string();
                        updates.push((name_str.clone(), "status".to_string(), Value::Text("active".to_string())));
                        updates.push((name_str.clone(), "arn".to_string(), Value::Text(arn.to_string())));
                        updates.push((name_str, "verified".to_string(), Value::Bool(true)));
                    }
                }
            }
        }
        updates
    }

    fn on_write(&mut self, _instance: &str, _field: &str, _value: &Value) {}

    fn provision(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> {
        let client = self.get_client(Some(desired));
        
        let topic_name = desired.get("topic_name").and_then(|v| v.as_text()).unwrap_or(instance);
        
        println!("\x1b[33m[PROVISIONING] SNS Topic '{}'...\x1b[0m", topic_name);
        
        let req = client.create_topic().name(topic_name);
        
        let result = block_in_place(|| Handle::current().block_on(req.send()));
        
        match result {
            Ok(resp) => {
                println!("\x1b[32m[CREATED] SNS Topic '{}' provisioned\x1b[0m", topic_name);
                
                if let Some(arn) = resp.topic_arn() {
                    if let (Some(protocol), Some(endpoint)) = (
                        desired.get("protocol").and_then(|v| v.as_text()),
                        desired.get("endpoint").and_then(|v| v.as_text())
                    ) {
                        let sub_result = block_in_place(|| {
                            Handle::current().block_on(
                                client.subscribe()
                                    .topic_arn(arn)
                                    .protocol(protocol)
                                    .endpoint(endpoint)
                                    .send()
                            )
                        });
                        
                        if let Err(e) = sub_result {
                            eprintln!("\x1b[31m[SNS ERROR] Failed to subscribe to {}: {}\x1b[0m", topic_name, e);
                        } else {
                            println!("\x1b[32m[CREATED] SNS Subscription added to '{}'\x1b[0m", topic_name);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("\x1b[31m[SNS ERROR] Failed to provision {}: {}\x1b[0m", topic_name, e);
                return Err(e.to_string());
            }
        }
        
        Ok(())
    }

    fn reconcile(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> {
        self.provision(instance, desired)
    }

    fn destroy(&mut self, instance: &str) -> Result<(), String> {
        let client = self.get_client(None);
        
        let list_result = block_in_place(|| {
            Handle::current().block_on(client.list_topics().send())
        });
        
        if let Ok(resp) = list_result {
            for topic in resp.topics() {
                if let Some(arn) = topic.topic_arn() {
                    let parts: Vec<&str> = arn.split(':').collect();
                    if let Some(name) = parts.last() {
                        if *name == instance {
                            let term_result = block_in_place(|| {
                                Handle::current().block_on(
                                    client.delete_topic().topic_arn(arn).send()
                                )
                            });
                            
                            if let Err(e) = term_result {
                                return Err(e.to_string());
                            }
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}
