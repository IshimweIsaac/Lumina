use std::collections::HashMap;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

/// Version vector entry for conflict resolution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionVector {
    /// node_id → monotonic version counter
    pub versions: HashMap<String, u64>,
}

impl VersionVector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the version for a specific node
    pub fn increment(&mut self, node_id: &str) {
        let counter = self.versions.entry(node_id.to_string()).or_insert(0);
        *counter += 1;
    }

    /// Check if self dominates (is causally after) other
    pub fn dominates(&self, other: &VersionVector) -> bool {
        // self dominates other if every entry in other is <= self
        // and at least one entry in self is strictly greater
        let mut dominated = false;
        for (node, &ver) in &other.versions {
            let self_ver = self.versions.get(node).copied().unwrap_or(0);
            if self_ver < ver {
                return false;
            }
            if self_ver > ver {
                dominated = true;
            }
        }
        // Check for entries in self not present in other
        for (node, &ver) in &self.versions {
            if !other.versions.contains_key(node) && ver > 0 {
                dominated = true;
            }
        }
        dominated
    }

    /// Merge two version vectors (point-wise max)
    pub fn merge(&mut self, other: &VersionVector) {
        for (node, &ver) in &other.versions {
            let entry = self.versions.entry(node.clone()).or_insert(0);
            if ver > *entry {
                *entry = ver;
            }
        }
    }
}

/// An entry in the state mesh with version tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshEntry {
    pub value: Vec<u8>,
    pub version: VersionVector,
    pub last_writer: String,
}

/// The distributed DAG state mesh — tracks entity state across cluster nodes
pub struct ClusterStateMesh {
    /// node_id → field_name → versioned entry
    pub mesh: RwLock<HashMap<String, HashMap<String, MeshEntry>>>,
}

impl Default for ClusterStateMesh {
    fn default() -> Self {
        Self {
            mesh: RwLock::new(HashMap::new()),
        }
    }
}

impl ClusterStateMesh {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update state for a local node, incrementing its version vector
    pub fn update_local(&self, node_id: &str, field: &str, value: Vec<u8>) {
        let mut mesh = self.mesh.write();
        let node_state = mesh.entry(node_id.to_string()).or_insert_with(HashMap::new);

        let entry = node_state.entry(field.to_string()).or_insert_with(|| MeshEntry {
            value: Vec::new(),
            version: VersionVector::new(),
            last_writer: node_id.to_string(),
        });

        entry.value = value;
        entry.version.increment(node_id);
        entry.last_writer = node_id.to_string();
    }

    /// Update state for a node with a full state map (bulk set)
    pub fn update_node_state(&self, node_id: &str, state: HashMap<String, Vec<u8>>) {
        let mut mesh = self.mesh.write();
        let node_state = mesh.entry(node_id.to_string()).or_insert_with(HashMap::new);

        for (field, value) in state {
            let entry = node_state.entry(field).or_insert_with(|| MeshEntry {
                value: Vec::new(),
                version: VersionVector::new(),
                last_writer: node_id.to_string(),
            });
            entry.value = value;
            entry.version.increment(node_id);
            entry.last_writer = node_id.to_string();
        }
    }

    /// Merge remote state using last-writer-wins conflict resolution
    pub fn merge_remote_state(&self, remote_node: &str, remote_state: HashMap<String, MeshEntry>) {
        let mut mesh = self.mesh.write();
        let node_state = mesh.entry(remote_node.to_string()).or_insert_with(HashMap::new);

        for (field, remote_entry) in remote_state {
            match node_state.get(&field) {
                Some(local_entry) => {
                    // Last-writer-wins: accept remote if its version dominates
                    if remote_entry.version.dominates(&local_entry.version) {
                        node_state.insert(field, remote_entry);
                    }
                    // If neither dominates (concurrent), also accept remote
                    // (LWW tiebreak by accepting the update)
                    else if !local_entry.version.dominates(&remote_entry.version) {
                        let mut merged = remote_entry;
                        merged.version.merge(&local_entry.version);
                        node_state.insert(field, merged);
                    }
                }
                None => {
                    node_state.insert(field, remote_entry);
                }
            }
        }
    }

    /// Get a specific field value for a node
    pub fn get_field(&self, node_id: &str, field: &str) -> Option<Vec<u8>> {
        let mesh = self.mesh.read();
        mesh.get(node_id)
            .and_then(|node_state| node_state.get(field))
            .map(|entry| entry.value.clone())
    }

    /// Get all values for a specific field across all nodes (for cluster-wide aggregates)
    pub fn get_all_field_values(&self, field: &str) -> Vec<(String, Vec<u8>)> {
        let mesh = self.mesh.read();
        mesh.iter()
            .filter_map(|(node_id, node_state)| {
                node_state.get(field).map(|entry| (node_id.clone(), entry.value.clone()))
            })
            .collect()
    }

    /// Export a snapshot of the mesh state for a given node (for gossip sync)
    pub fn snapshot_node(&self, node_id: &str) -> Option<HashMap<String, MeshEntry>> {
        self.mesh.read().get(node_id).cloned()
    }

    /// Export the full mesh as a serializable map (node → field → raw bytes)
    pub fn snapshot_raw(&self) -> HashMap<String, HashMap<String, Vec<u8>>> {
        let mesh = self.mesh.read();
        mesh.iter()
            .map(|(node_id, fields)| {
                let raw_fields: HashMap<String, Vec<u8>> = fields.iter()
                    .map(|(field, entry)| (field.clone(), entry.value.clone()))
                    .collect();
                (node_id.clone(), raw_fields)
            })
            .collect()
    }

    /// Get the total number of nodes in the mesh
    pub fn node_count(&self) -> usize {
        self.mesh.read().len()
    }

    /// Get all node IDs tracked in the mesh
    pub fn node_ids(&self) -> Vec<String> {
        self.mesh.read().keys().cloned().collect()
    }
}
