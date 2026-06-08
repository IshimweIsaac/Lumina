use crate::value::Value;
use std::collections::HashMap;
use tokio::runtime::Handle;
use tokio::task::block_in_place;
use aws_sdk_lambda::Client;
use aws_sdk_lambda::types::{Runtime, FunctionCode, Architecture};
use crate::adapters::aws::credentials::AwsConfig;

pub struct LambdaAdapter {
    entity_name: String,
    client: Option<Client>,
}

impl LambdaAdapter {
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

impl crate::adapter::LuminaAdapter for LambdaAdapter {
    fn entity_name(&self) -> &str {
        &self.entity_name
    }

    fn poll(&mut self) -> Vec<(String, String, Value)> {
        let mut updates = vec![];
        let client = self.get_client(None);

        let result = block_in_place(|| {
            Handle::current().block_on(client.list_functions().send())
        });

        if let Ok(response) = result {
            for function in response.functions() {
                if let Some(name) = function.function_name() {
                    let name_str = name.to_string();
                    let state = function.state().map(|s| s.as_str()).unwrap_or("unknown");
                    
                    updates.push((name_str.clone(), "status".to_string(), Value::Text(state.to_string())));
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
            Handle::current().block_on(client.get_function().function_name(instance).send())
        }).is_ok();

        if !exists {
            println!("\x1b[33m[PROVISIONING] Lambda Function '{}'...\x1b[0m", instance);
            
            let runtime_str = desired.get("runtime").and_then(|v| v.as_text()).unwrap_or("nodejs18.x");
            let handler = desired.get("handler").and_then(|v| v.as_text()).unwrap_or("index.handler");
            let role_arn = desired.get("role_arn").and_then(|v| v.as_text()).ok_or("Missing role_arn")?;
            
            let s3_bucket = desired.get("s3_bucket").and_then(|v| v.as_text());
            let s3_key = desired.get("s3_key").and_then(|v| v.as_text());
            
            let code = if let (Some(bucket), Some(key)) = (s3_bucket, s3_key) {
                FunctionCode::builder().s3_bucket(bucket).s3_key(key).build()
            } else {
                return Err("Missing s3_bucket or s3_key for Lambda function code".to_string());
            };

            let mut req = client.create_function()
                .function_name(instance)
                .runtime(Runtime::from(runtime_str))
                .handler(handler)
                .role(role_arn)
                .code(code)
                .architectures(Architecture::X8664);
                
            if let Some(memory) = desired.get("memory_size").and_then(|v| v.as_number()) {
                req = req.memory_size(memory as i32);
            }
            if let Some(timeout) = desired.get("timeout").and_then(|v| v.as_number()) {
                req = req.timeout(timeout as i32);
            }
            
            let result = block_in_place(|| Handle::current().block_on(req.send()));
            
            match result {
                Ok(_) => {
                    println!("\x1b[32m[CREATED] Lambda Function '{}' provisioned\x1b[0m", instance);
                }
                Err(e) => {
                    eprintln!("\x1b[31m[LAMBDA ERROR] Failed to provision {}: {}\x1b[0m", instance, e);
                    return Err(e.to_string());
                }
            }
        } else {
            println!("\x1b[34m[VERIFIED] Lambda Function '{}' already exists.\x1b[0m", instance);
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
                client.delete_function().function_name(instance).send()
            )
        });
        
        if let Err(e) = result {
            return Err(e.to_string());
        }
        Ok(())
    }
}
