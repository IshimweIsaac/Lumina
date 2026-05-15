use crate::value::Value;

/// Trait for connecting Lumina external entities to real-world data sources.
///
/// Implementors supply data on each `poll()` call and receive write-backs
/// when a rule action updates an external entity field.
pub trait LuminaAdapter: Send + Sync {
    /// The external entity name this adapter serves.
    /// Must match: `external entity <Name> { ... }` or `resource entity <Name> { ... }`
    fn entity_name(&self) -> &str;

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

    fn reconcile(&mut self, _instance: &str) -> Result<(), String> {
        Ok(())
    }
}
