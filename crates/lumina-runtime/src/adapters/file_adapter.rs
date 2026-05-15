use crate::adapter::LuminaAdapter;
use crate::value::Value;
use notify::{EventKind, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};

pub struct FileWatchAdapter {
    entity: String,
    rx: std::sync::Mutex<Receiver<(String, String, Value)>>,
    // Keep watcher alive as long as adapter is alive
    _watcher: notify::RecommendedWatcher,
}

impl FileWatchAdapter {
    pub fn new(entity: impl Into<String>, path: PathBuf) -> Result<Self, notify::Error> {
        let entity = entity.into();
        let (tx, rx) = channel();

        // Setup file watcher
        let watch_path = path.clone();
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    // If the file was modified
                    if matches!(event.kind, EventKind::Modify(_)) {
                        if let Ok(content) = fs::read_to_string(&watch_path) {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
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
                                            serde_json::Value::Number(n) => {
                                                n.as_f64().map(Value::Number)
                                            }
                                            serde_json::Value::String(s) => Some(Value::Text(s)),
                                            serde_json::Value::Bool(b) => Some(Value::Bool(b)),
                                            _ => None,
                                        };
                                        if let Some(v) = typed_val {
                                            let _ = tx.send((instance.clone(), key, v));
                                            // Ignore drop errors here
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            })?;

        watcher.watch(Path::new(&path), RecursiveMode::NonRecursive)?;

        Ok(Self {
            entity,
            rx: std::sync::Mutex::new(rx),
            _watcher: watcher,
        })
    }
}

impl LuminaAdapter for FileWatchAdapter {
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
    fn on_write(&mut self, _instance: &str, _field: &str, _value: &Value) {
        // Not currently propagating writes back to the file system.
    }
}
