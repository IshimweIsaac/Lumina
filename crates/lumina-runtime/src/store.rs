use crate::value::Value;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSlot {
    pub current: Value,
    pub previous: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub entity_name: String,
    pub slots: Vec<StateSlot>,
    /// v2.0: Monotonically increasing version for state mesh conflict resolution
    pub version: u64,
    /// v2.0: The peer ID that last mutated this instance (None if local)
    pub source_node: Option<String>,
}

impl Instance {
    pub fn new(entity_name: impl Into<String>, field_count: usize) -> Self {
        Self {
            entity_name: entity_name.into(),
            slots: vec![
                StateSlot {
                    current: Value::Unknown,
                    previous: Value::Unknown
                };
                field_count
            ],
            version: 1,
            source_node: None,
        }
    }

    pub fn get(&self, idx: usize) -> Option<&Value> {
        self.slots.get(idx).map(|s| &s.current)
    }

    pub fn set(&mut self, idx: usize, value: Value) {
        if let Some(slot) = self.slots.get_mut(idx) {
            slot.previous = std::mem::replace(&mut slot.current, value);
            self.version += 1;
            self.source_node = None;
        }
    }

    pub fn prev(&self, idx: usize) -> Option<&Value> {
        self.slots.get(idx).map(|s| &s.previous)
    }

    pub fn iter_fields<'a>(
        &'a self,
        names: &'a [String],
    ) -> impl Iterator<Item = (&'a String, &'a Value)> {
        names
            .iter()
            .zip(self.slots.iter())
            .map(|(name, slot)| (name, &slot.current))
    }

    pub fn iter_prev_fields<'a>(
        &'a self,
        names: &'a [String],
    ) -> impl Iterator<Item = (&'a String, &'a Value)> {
        names
            .iter()
            .zip(self.slots.iter())
            .map(|(name, slot)| (name, &slot.previous))
    }

    pub fn commit(&mut self) {
        for slot in self.slots.iter_mut() {
            slot.previous = slot.current.clone();
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityStore {
    instances: FxHashMap<String, Instance>,
}

impl EntityStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, instance: Instance) {
        self.instances.insert(name.into(), instance);
    }

    pub fn get(&self, name: &str) -> Option<&Instance> {
        self.instances.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Instance> {
        self.instances.get_mut(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<Instance> {
        self.instances.remove(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.instances.contains_key(name)
    }

    pub fn all(&self) -> impl Iterator<Item = (&String, &Instance)> {
        self.instances.iter()
    }

    pub fn all_of_entity<'a>(
        &'a self,
        entity_name: &'a str,
    ) -> impl Iterator<Item = (&'a String, &'a Instance)> {
        self.instances
            .iter()
            .filter(move |(_, i)| i.entity_name == entity_name)
    }

    /// Find the first instance of a given entity type.
    /// Used by adapter polling to map entity names to instance names.
    pub fn find_instance_of(&self, entity_name: &str) -> Option<String> {
        self.instances
            .iter()
            .find(|(_, i)| i.entity_name == entity_name)
            .map(|(n, _)| n.clone())
    }

    /// Find all instances of a given entity type.
    pub fn all_instances_of(&self, entity_name: &str) -> Vec<String> {
        self.instances
            .iter()
            .filter(|(_, i)| i.entity_name == entity_name)
            .map(|(n, _)| n.clone())
            .collect()
    }

    /// Commit all instances — called after stable propagation
    pub fn commit_all(&mut self) {
        for instance in self.instances.values_mut() {
            instance.commit();
        }
    }

    /// Commit only specific instances — O(dirty) optimization
    pub fn commit_dirty(&mut self, dirty: &FxHashSet<String>) {
        for name in dirty {
            if let Some(instance) = self.instances.get_mut(name) {
                instance.commit();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::SnapshotStack;

    fn make_person(name: &str, age: f64) -> Instance {
        let mut inst = Instance::new("Person", 2);
        inst.set(0, Value::Text(name.to_string()));
        inst.set(1, Value::Number(age));
        inst
    }

    #[test]
    fn test_insert_and_get() {
        let mut store = EntityStore::new();
        store.insert("isaac", make_person("Isaac", 26.0));
        let inst = store.get("isaac").unwrap();
        assert_eq!(inst.entity_name, "Person");
        assert_eq!(inst.get(0), Some(&Value::Text("Isaac".to_string())));
        assert_eq!(inst.get(1), Some(&Value::Number(26.0)));
    }

    #[test]
    fn test_set_captures_prev() {
        let mut store = EntityStore::new();
        store.insert("isaac", make_person("Isaac", 26.0));
        let inst = store.get_mut("isaac").unwrap();

        inst.set(1, Value::Number(27.0));

        assert_eq!(inst.get(1), Some(&Value::Number(27.0)));
        assert_eq!(inst.prev(1), Some(&Value::Number(26.0)));
    }

    #[test]
    fn test_commit_syncs_prev() {
        let mut store = EntityStore::new();
        store.insert("isaac", make_person("Isaac", 26.0));
        let inst = store.get_mut("isaac").unwrap();

        inst.set(1, Value::Number(27.0));
        inst.commit();

        assert_eq!(inst.get(1), Some(&Value::Number(27.0)));
        assert_eq!(inst.prev(1), Some(&Value::Number(27.0)));
    }

    #[test]
    fn test_all_of_entity() {
        let mut store = EntityStore::new();
        store.insert("isaac", make_person("Isaac", 26.0));
        store.insert("alice", make_person("Alice", 30.0));

        let mut bike = Instance::new("Bike", 1);
        bike.set(0, Value::Text("Trek".to_string()));
        store.insert("bike1", bike);

        let people: Vec<_> = store.all_of_entity("Person").collect();
        assert_eq!(people.len(), 2);

        let bikes: Vec<_> = store.all_of_entity("Bike").collect();
        assert_eq!(bikes.len(), 1);
    }

    #[test]
    fn test_snapshot_take_and_restore() {
        let mut store = EntityStore::new();
        store.insert("isaac", make_person("Isaac", 26.0));

        let mut stack = SnapshotStack::new();
        let snap = stack.take(&store);
        stack.push(snap);

        // Modify the store
        store.get_mut("isaac").unwrap().set(1, Value::Number(99.0));
        assert_eq!(
            store.get("isaac").unwrap().get(1),
            Some(&Value::Number(99.0))
        );

        // Restore from snapshot
        let restored = stack.pop().unwrap();
        store = restored.store;
        assert_eq!(
            store.get("isaac").unwrap().get(1),
            Some(&Value::Number(26.0))
        );
    }
}
