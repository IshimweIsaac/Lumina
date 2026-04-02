use crate::adapter::LuminaAdapter;
use crate::value::Value;
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::Duration;

pub struct HttpPollAdapter {
    entity: String,
    rx: std::sync::Mutex<Receiver<(String, Value)>>,
}

impl HttpPollAdapter {
    pub fn new(entity: impl Into<String>, url: String, interval: Duration) -> Self {
        let entity = entity.into();
        let (tx, rx) = channel();
        
        // Spawn background polling thread
        thread::spawn(move || {
            let client = reqwest::blocking::Client::new();
            loop {
                if let Ok(resp) = client.get(&url).send() {
                    if let Ok(json) = resp.json::<serde_json::Value>() {
                        if let serde_json::Value::Object(map) = json {
                            for (key, val) in map {
                                let typed_val = match val {
                                    serde_json::Value::Number(n) => n.as_f64().map(Value::Number),
                                    serde_json::Value::String(s) => Some(Value::Text(s)),
                                    serde_json::Value::Bool(b)   => Some(Value::Bool(b)),
                                    _ => None,
                                };
                                if let Some(v) = typed_val {
                                    if tx.send((key, v)).is_err() {
                                        return; // receiver dropped, exit thread
                                    }
                                }
                            }
                        }
                    }
                }
                thread::sleep(interval);
            }
        });

        Self { entity, rx: std::sync::Mutex::new(rx) }
    }
}

impl LuminaAdapter for HttpPollAdapter {
    fn entity_name(&self) -> &str { &self.entity }
    fn poll(&mut self) -> Option<(String, Value)> {
        self.rx.lock().ok()?.try_recv().ok()
    }
    fn on_write(&mut self, _field: &str, _value: &Value) {
        // Poll adapter typically doesn't handle write-backs currently.
    }
}
