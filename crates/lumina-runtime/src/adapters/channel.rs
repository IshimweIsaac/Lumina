use crate::adapter::LuminaAdapter;
use crate::value::Value;
use std::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

/// An adapter backed by Rust `mpsc` channels.
/// Receives values from a `Receiver` and optionally sends write-backs
/// through a `Sender`.
pub struct ChannelAdapter {
    entity: String,
    rx: Mutex<Receiver<(String, String, Value)>>,
    tx: Option<Sender<(String, Value)>>,
}

impl ChannelAdapter {
    pub fn new(
        entity: impl Into<String>,
        rx: Receiver<(String, String, Value)>,
        tx: Option<Sender<(String, Value)>>,
    ) -> Self {
        Self {
            entity: entity.into(),
            rx: Mutex::new(rx),
            tx,
        }
    }
}

impl LuminaAdapter for ChannelAdapter {
    fn entity_name(&self) -> &str {
        &self.entity
    }

    fn poll(&mut self) -> Vec<(String, String, Value)> {
        let mut updates = vec![];
        if let Ok(rx) = self.rx.lock() {
            while let Ok(update) = rx.try_recv() {
                updates.push(update);
            }
        }
        updates
    }

    fn on_write(&mut self, _instance: &str, field: &str, value: &Value) {
        if let Some(tx) = &self.tx {
            let _ = tx.send((field.to_string(), value.clone()));
        }
    }
}
