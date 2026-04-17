use std::collections::HashMap;
use parking_lot::RwLock;

/// The distributed DAG state mesh
#[derive(Default)]
pub struct ClusterStateMesh {
    // node_id -> field_name -> Value represented as JSON or similar byte payload
    pub mesh: RwLock<HashMap<String, HashMap<String, Vec<u8>>>>,
}

impl ClusterStateMesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_node_state(&self, node_id: &str, state: HashMap<String, Vec<u8>>) {
        let mut m = self.mesh.write();
        m.insert(node_id.to_string(), state);
    }

    pub fn get_field(&self, node_id: &str, field: &str) -> Option<Vec<u8>> {
        let m = self.mesh.read();
        m.get(node_id).and_then(|node_state| node_state.get(field).cloned())
    }
}
