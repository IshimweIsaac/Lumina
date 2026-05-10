use crate::adapter::LuminaAdapter;
use crate::value::Value;
use paho_mqtt as mqtt;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

pub struct MqttAdapter {
    entity: String,
    cli: mqtt::Client,
    parsed_rx: std::sync::Mutex<Receiver<(String, String, Value)>>,
    pub_topic: String,
}

impl MqttAdapter {
    pub fn new(
        entity: impl Into<String>,
        uri: &str,
        client_id: &str,
        sub_topic: &str,
        pub_topic: &str,
    ) -> Result<Self, mqtt::Error> {
        let entity = entity.into();

        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(uri)
            .client_id(client_id)
            .finalize();

        let cli = mqtt::Client::new(create_opts)?;

        // Start the background consumption channel
        let rx = cli.start_consuming();

        let conn_opts = mqtt::ConnectOptionsBuilder::new().finalize();
        cli.connect(conn_opts)?;
        cli.subscribe(sub_topic, 1)?; // QoS 1

        let (tx, parsed_rx) = channel();

        // Spawn a background thread to interpret messages and queue them
        // to the synchronous adapter poll method.
        thread::spawn(move || {
            for msg_opt in rx.iter() {
                if let Some(msg) = msg_opt {
                    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(msg.payload()) {
                        if let serde_json::Value::Object(map) = json {
                            let instance = map
                                .get("id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| "default".to_string());

                            for (key, val) in map {
                                if key == "id" {
                                    continue;
                                }
                                let typed_val = match val {
                                    serde_json::Value::Number(n) => n.as_f64().map(Value::Number),
                                    serde_json::Value::String(s) => Some(Value::Text(s)),
                                    serde_json::Value::Bool(b) => Some(Value::Bool(b)),
                                    _ => None,
                                };
                                if let Some(v) = typed_val {
                                    if tx.send((instance.clone(), key, v)).is_err() {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            entity,
            cli,
            parsed_rx: std::sync::Mutex::new(parsed_rx),
            pub_topic: pub_topic.to_string(),
        })
    }
}

impl LuminaAdapter for MqttAdapter {
    fn entity_name(&self) -> &str {
        &self.entity
    }
    fn poll(&mut self) -> Option<(String, String, Value)> {
        self.parsed_rx.lock().ok()?.try_recv().ok()
    }

    // Broadcast updates natively through the network buffer
    fn on_write(&mut self, field: &str, value: &Value) {
        let v_json = match value {
            Value::Number(n) => serde_json::json!(*n),
            Value::Text(s) => serde_json::json!(s),
            Value::Bool(b) => serde_json::json!(*b),
            _ => return, // Ignore unsupported
        };

        // Output format: { "field": "value" }
        let payload = serde_json::json!({
            field: v_json
        });

        let msg = mqtt::Message::new(&self.pub_topic, payload.to_string(), 1);
        let _ = self.cli.publish(msg); // Ignored error in this sync execution paradigm
    }
}
