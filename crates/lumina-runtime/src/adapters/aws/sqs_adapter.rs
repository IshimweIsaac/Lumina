use crate::value::Value;
use std::collections::HashMap;
use tokio::runtime::Handle;
use tokio::task::block_in_place;
use aws_sdk_sqs::Client;
use crate::adapters::aws::credentials::AwsConfig;

pub struct SqsAdapter {
    entity_name: String,
    client: Option<Client>,
}

impl SqsAdapter {
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

impl crate::adapter::LuminaAdapter for SqsAdapter {
    fn entity_name(&self) -> &str {
        &self.entity_name
    }

    fn poll(&mut self) -> Vec<(String, String, Value)> {
        let mut updates = vec![];
        let client = self.get_client(None);

        let result = block_in_place(|| {
            Handle::current().block_on(client.list_queues().send())
        });

        if let Ok(response) = result {
            for url in response.queue_urls() {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(name) = parts.last() {
                    let name_str = name.to_string();
                    updates.push((name_str.clone(), "status".to_string(), Value::Text("active".to_string())));
                    updates.push((name_str.clone(), "url".to_string(), Value::Text(url.to_string())));
                    updates.push((name_str, "verified".to_string(), Value::Bool(true)));
                }
            }
        }
        updates
    }

    fn on_write(&mut self, _instance: &str, _field: &str, _value: &Value) {}

    fn provision(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> {
        let client = self.get_client(Some(desired));
        
        let queue_name = desired.get("queue_name").and_then(|v| v.as_text()).unwrap_or(instance);
        
        let mut attributes = HashMap::new();
        
        if let Some(Value::Bool(fifo)) = desired.get("fifo") {
            if *fifo {
                attributes.insert(aws_sdk_sqs::types::QueueAttributeName::FifoQueue, "true".to_string());
                if !queue_name.ends_with(".fifo") {
                    return Err("FIFO queue names must end with '.fifo'".to_string());
                }
            }
        }
        
        if let Some(delay) = desired.get("delay_seconds").and_then(|v| v.as_number()) {
            attributes.insert(aws_sdk_sqs::types::QueueAttributeName::DelaySeconds, (delay as i32).to_string());
        }
        
        if let Some(vis) = desired.get("visibility_timeout").and_then(|v| v.as_number()) {
            attributes.insert(aws_sdk_sqs::types::QueueAttributeName::VisibilityTimeout, (vis as i32).to_string());
        }
        
        if let Some(retention) = desired.get("message_retention").and_then(|v| v.as_number()) {
            attributes.insert(aws_sdk_sqs::types::QueueAttributeName::MessageRetentionPeriod, (retention as i32).to_string());
        }

        let mut req = client.create_queue().queue_name(queue_name);
        
        for (k, v) in attributes {
            req = req.attributes(k, v);
        }

        println!("\x1b[33m[PROVISIONING] SQS Queue '{}'...\x1b[0m", queue_name);
        
        let result = block_in_place(|| Handle::current().block_on(req.send()));
        
        match result {
            Ok(_) => {
                println!("\x1b[32m[CREATED] SQS Queue '{}' provisioned\x1b[0m", queue_name);
            }
            Err(e) => {
                eprintln!("\x1b[31m[SQS ERROR] Failed to provision {}: {}\x1b[0m", queue_name, e);
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
        
        let result = block_in_place(|| {
            Handle::current().block_on(
                client.get_queue_url().queue_name(instance).send()
            )
        });
        
        if let Ok(resp) = result {
            if let Some(url) = resp.queue_url() {
                let term_result = block_in_place(|| {
                    Handle::current().block_on(
                        client.delete_queue().queue_url(url).send()
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
