use crate::value::Value;
use std::collections::HashMap;
use tokio::runtime::Handle;
use tokio::task::block_in_place;
use aws_sdk_s3::Client;
use aws_sdk_s3::types::{BucketVersioningStatus, VersioningConfiguration};
use crate::adapters::aws::credentials::AwsConfig;

pub struct S3Adapter {
    entity_name: String,
    client: Option<Client>,
}

impl S3Adapter {
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

impl crate::adapter::LuminaAdapter for S3Adapter {
    fn entity_name(&self) -> &str {
        &self.entity_name
    }

    fn poll(&mut self) -> Vec<(String, String, Value)> {
        let mut updates = vec![];
        let client = self.get_client(None);

        let result = block_in_place(|| {
            Handle::current().block_on(client.list_buckets().send())
        });

        if let Ok(response) = result {
            for bucket in response.buckets() {
                if let Some(name) = bucket.name() {
                    let name_str = name.to_string();
                    updates.push((name_str.clone(), "status".to_string(), Value::Text("active".to_string())));
                    updates.push((name_str, "verified".to_string(), Value::Bool(true)));
                }
            }
        }
        updates
    }

    fn on_write(&mut self, _instance: &str, _field: &str, _value: &Value) {}

    fn provision(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> {
        let client = self.get_client(Some(desired));
        
        let bucket_name = desired.get("bucket_name").and_then(|v| v.as_text()).unwrap_or(instance);
        
        let exists = block_in_place(|| {
            Handle::current().block_on(client.head_bucket().bucket(bucket_name).send())
        }).is_ok();

        if !exists {
            println!("\x1b[33m[PROVISIONING] S3 Bucket '{}'...\x1b[0m", bucket_name);
            
            let req = client.create_bucket().bucket(bucket_name);
            
            let result = block_in_place(|| Handle::current().block_on(req.send()));
            
            match result {
                Ok(_) => {
                    println!("\x1b[32m[CREATED] S3 Bucket '{}' provisioned\x1b[0m", bucket_name);
                    
                    if let Some(Value::Bool(v)) = desired.get("versioning") {
                        if *v {
                            let _ = block_in_place(|| Handle::current().block_on(
                                client.put_bucket_versioning()
                                    .bucket(bucket_name)
                                    .versioning_configuration(
                                        VersioningConfiguration::builder().status(BucketVersioningStatus::Enabled).build()
                                    )
                                    .send()
                            ));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\x1b[31m[S3 ERROR] Failed to provision {}: {}\x1b[0m", bucket_name, e);
                    return Err(e.to_string());
                }
            }
        } else {
            println!("\x1b[34m[VERIFIED] S3 Bucket '{}' already exists.\x1b[0m", bucket_name);
        }
        
        Ok(())
    }

    fn reconcile(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> {
        self.provision(instance, desired)
    }

    fn destroy(&mut self, instance: &str) -> Result<(), String> {
        let client = self.get_client(None);
        
        let result = block_in_place(|| {
            Handle::current().block_on(client.delete_bucket().bucket(instance).send())
        });
        
        if let Err(e) = result {
            return Err(e.to_string());
        }
        Ok(())
    }
}
