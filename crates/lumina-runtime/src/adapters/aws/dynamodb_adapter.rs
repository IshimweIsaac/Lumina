use crate::value::Value;
use std::collections::HashMap;
use tokio::runtime::Handle;
use tokio::task::block_in_place;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::{AttributeDefinition, KeySchemaElement, KeyType, ScalarAttributeType, ProvisionedThroughput, BillingMode};
use crate::adapters::aws::credentials::AwsConfig;

pub struct DynamodbAdapter {
    entity_name: String,
    client: Option<Client>,
}

impl DynamodbAdapter {
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

impl crate::adapter::LuminaAdapter for DynamodbAdapter {
    fn entity_name(&self) -> &str {
        &self.entity_name
    }

    fn poll(&mut self) -> Vec<(String, String, Value)> {
        let mut updates = vec![];
        let client = self.get_client(None);

        let result = block_in_place(|| {
            Handle::current().block_on(client.list_tables().send())
        });

        if let Ok(response) = result {
            for table in response.table_names() {
                let name_str = table.to_string();
                updates.push((name_str.clone(), "status".to_string(), Value::Text("active".to_string())));
                updates.push((name_str, "verified".to_string(), Value::Bool(true)));
            }
        }
        updates
    }

    fn on_write(&mut self, _instance: &str, _field: &str, _value: &Value) {}

    fn provision(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> {
        let client = self.get_client(Some(desired));
        
        let table_name = desired.get("table_name").and_then(|v| v.as_text()).unwrap_or(instance);
        
        let exists = block_in_place(|| {
            Handle::current().block_on(client.describe_table().table_name(table_name).send())
        }).is_ok();

        if !exists {
            println!("\x1b[33m[PROVISIONING] DynamoDB Table '{}'...\x1b[0m", table_name);
            
            let partition_key = desired.get("partition_key").and_then(|v| v.as_text()).unwrap_or("id");
            let billing_mode_str = desired.get("billing_mode").and_then(|v| v.as_text()).unwrap_or("PAY_PER_REQUEST");
            
            let mut req = client.create_table()
                .table_name(table_name)
                .attribute_definitions(
                    AttributeDefinition::builder().attribute_name(partition_key).attribute_type(ScalarAttributeType::S).build().unwrap()
                )
                .key_schema(
                    KeySchemaElement::builder().attribute_name(partition_key).key_type(KeyType::Hash).build().unwrap()
                );
                
            if let Some(sort_key) = desired.get("sort_key").and_then(|v| v.as_text()) {
                req = req.attribute_definitions(
                    AttributeDefinition::builder().attribute_name(sort_key).attribute_type(ScalarAttributeType::S).build().unwrap()
                ).key_schema(
                    KeySchemaElement::builder().attribute_name(sort_key).key_type(KeyType::Range).build().unwrap()
                );
            }
            
            if billing_mode_str == "PROVISIONED" {
                let rcu = desired.get("read_capacity").and_then(|v| v.as_number()).unwrap_or(5.0) as i64;
                let wcu = desired.get("write_capacity").and_then(|v| v.as_number()).unwrap_or(5.0) as i64;
                
                req = req.billing_mode(BillingMode::Provisioned)
                    .provisioned_throughput(
                        ProvisionedThroughput::builder().read_capacity_units(rcu).write_capacity_units(wcu).build().unwrap()
                    );
            } else {
                req = req.billing_mode(BillingMode::PayPerRequest);
            }
            
            let result = block_in_place(|| Handle::current().block_on(req.send()));
            
            match result {
                Ok(_) => {
                    println!("\x1b[32m[CREATED] DynamoDB Table '{}' provisioned\x1b[0m", table_name);
                }
                Err(e) => {
                    eprintln!("\x1b[31m[DYNAMODB ERROR] Failed to provision {}: {}\x1b[0m", table_name, e);
                    return Err(e.to_string());
                }
            }
        } else {
            println!("\x1b[34m[VERIFIED] DynamoDB Table '{}' already exists.\x1b[0m", table_name);
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
                client.delete_table().table_name(instance).send()
            )
        });
        
        if let Err(e) = result {
            return Err(e.to_string());
        }
        Ok(())
    }
}
