use crate::adapter::LuminaAdapter;
use crate::value::Value;
use std::collections::VecDeque;

/// A simple adapter for testing that serves values from a pre-loaded queue.
pub struct StaticAdapter {
    entity: String,
    queue: VecDeque<(String, String, Value)>,
}

impl StaticAdapter {
    pub fn new(entity: impl Into<String>) -> Self {
        Self { entity: entity.into(), queue: VecDeque::new() }
    }

    /// Push a value to be delivered on the next poll.
    pub fn push(&mut self, instance: impl Into<String>, field: impl Into<String>, value: Value) {
        self.queue.push_back((instance.into(), field.into(), value));
    }
}

impl LuminaAdapter for StaticAdapter {
    fn entity_name(&self) -> &str { &self.entity }

    fn poll(&mut self) -> Option<(String, String, Value)> { self.queue.pop_front() }

    fn on_write(&mut self, _: &str, _: &Value) {}
}
