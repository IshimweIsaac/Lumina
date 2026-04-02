use crate::adapter::LuminaAdapter;
use crate::value::Value;
use std::sync::mpsc::{Receiver, channel};
use std::path::{Path, PathBuf};
use notify::{Watcher, RecursiveMode, EventKind};
use std::fs;

pub struct FileWatchAdapter {
    entity: String,
    rx: std::sync::Mutex<Receiver<(String, Value)>>,
    // Keep watcher alive as long as adapter is alive
    _watcher: notify::RecommendedWatcher,
}

impl FileWatchAdapter {
    pub fn new(entity: impl Into<String>, path: PathBuf) -> Result<Self, notify::Error> {
        let entity = entity.into();
        let (tx, rx) = channel();
        
        // Setup file watcher
        let watch_path = path.clone();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                // If the file was modified
                if matches!(event.kind, EventKind::Modify(_)) {
                    if let Ok(content) = fs::read_to_string(&watch_path) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let serde_json::Value::Object(map) = json {
                                for (key, val) in map {
                                    let typed_val = match val {
                                        serde_json::Value::Number(n) => n.as_f64().map(Value::Number),
                                        serde_json::Value::String(s) => Some(Value::Text(s)),
                                        serde_json::Value::Bool(b)   => Some(Value::Bool(b)),
                                        _ => None,
                                    };
                                    if let Some(v) = typed_val {
                                        let _ = tx.send((key, v)); // Ignore drop errors here
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
            _watcher: watcher 
        })
    }
}

impl LuminaAdapter for FileWatchAdapter {
    fn entity_name(&self) -> &str { &self.entity }
    fn poll(&mut self) -> Option<(String, Value)> {
        self.rx.lock().ok()?.try_recv().ok()
    }
    fn on_write(&mut self, _field: &str, _value: &Value) {
        // Not currently propagating writes back to the file system.
    }
}
