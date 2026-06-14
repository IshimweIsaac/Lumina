use crate::value::Value;
use std::time::Duration;
use rand::Rng;

/// v2.1.4: Configuration for adapter operation retries
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 500,
            max_delay_ms: 30_000,
            jitter: true,
        }
    }
}

/// Helper function to execute an operation with exponential backoff
pub fn with_retry<T, E, F>(config: &RetryConfig, mut operation: F) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut attempt = 0;
    let mut current_delay = config.base_delay_ms;

    loop {
        match operation() {
            Ok(val) => return Ok(val),
            Err(e) => {
                if attempt >= config.max_retries {
                    return Err(e);
                }
                
                let mut sleep_ms = current_delay;
                if config.jitter {
                    let mut rng = rand::thread_rng();
                    // Add up to 25% jitter
                    let jitter_amount = rng.gen_range(0..=(sleep_ms / 4).max(1));
                    sleep_ms += jitter_amount;
                }
                
                std::thread::sleep(Duration::from_millis(sleep_ms));
                
                current_delay = (current_delay * 2).min(config.max_delay_ms);
                attempt += 1;
            }
        }
    }
}

/// Trait for connecting Lumina external entities to real-world data sources.
///
/// Implementors supply data on each `poll()` call and receive write-backs
/// when a rule action updates an external entity field.
pub trait LuminaAdapter: Send + Sync {
    /// The external entity name this adapter serves.
    /// Must match: `external entity <Name> { ... }` or `resource entity <Name> { ... }`
    fn entity_name(&self) -> &str;

    /// v2.1.4: Returns the retry configuration for this specific adapter
    fn retry_config(&self) -> RetryConfig {
        RetryConfig::default()
    }

    /// Called on every tick(). Returns a list of updates: `(instance, field, value)`.
    fn poll(&mut self) -> Vec<(String, String, Value)>;

    /// Called when a rule action writes to an external entity field.
    fn on_write(&mut self, _instance: &str, _field: &str, _value: &Value) {}

    /// v2.1: Lifecycle hooks for infrastructure management
    fn provision(&mut self, _instance: &str, _desired: &std::collections::HashMap<String, Value>) -> Result<(), String> {
        Ok(())
    }
    
    fn destroy(&mut self, _instance: &str) -> Result<(), String> {
        Ok(())
    }

    fn reconcile(&mut self, _instance: &str, _desired: &std::collections::HashMap<String, Value>) -> Result<(), String> {
        Ok(())
    }
}
