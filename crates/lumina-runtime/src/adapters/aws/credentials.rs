use std::collections::HashMap;
use crate::value::Value;
use tokio::task::block_in_place;
use tokio::runtime::Handle;
use aws_config::SdkConfig;

pub struct AwsConfig {
    pub sdk_config: SdkConfig,
}

impl AwsConfig {
    pub fn new(desired_state: Option<&HashMap<String, Value>>) -> Self {
        let mut region = None;
        if let Some(state) = desired_state {
            if let Some(r) = state.get("aws_region").or_else(|| state.get("region")).and_then(|v| v.as_text()) {
                region = Some(r.to_string());
            }
        }
        
        let sdk_config = block_in_place(|| {
            Handle::current().block_on(async {
                let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
                
                if let Some(r) = region {
                    config_loader = config_loader.region(aws_config::Region::new(r));
                }
                
                config_loader.load().await
            })
        });

        Self {
            sdk_config,
        }
    }
}
