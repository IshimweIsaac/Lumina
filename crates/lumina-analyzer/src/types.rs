use std::collections::HashMap;
use lumina_parser::ast::{LuminaType, FieldMetadata};

/// The fully analyzed schema — all entity definitions after type checking
#[derive(Debug, Clone)]
pub struct Schema {
    pub entities: HashMap<String, EntitySchema>,
}

#[derive(Debug, Clone)]
pub struct EntitySchema {
    pub name:          String,
    pub fields:        HashMap<String, FieldSchema>,
    pub is_external:   bool,
    pub sync_path:     String,
    pub sync_strategy: SyncStrategy,
    pub sync_on:       Option<Vec<String>>,
    pub poll_interval: Option<Duration>,
}

pub use lumina_parser::ast::{SyncStrategy, Duration};

#[derive(Debug, Clone)]
pub struct FieldSchema {
    pub name:       String,
    pub ty:         LuminaType,
    pub is_derived: bool,
    pub metadata:   FieldMetadata,
}

impl Schema {
    pub fn new() -> Self {
        Self { entities: HashMap::new() }
    }

    pub fn get_entity(&self, name: &str) -> Option<&EntitySchema> {
        self.entities.get(name)
    }

    pub fn get_field(&self, entity: &str, field: &str) -> Option<&FieldSchema> {
        self.entities.get(entity)?.fields.get(field)
    }

    /// v1.9: Register a single field into an entity schema (used by LSL imports)
    pub fn register_field(&mut self, entity: &str, field_name: &str, ty: &LuminaType) {
        let entity_schema = self.entities.entry(entity.to_string()).or_insert_with(|| {
            EntitySchema {
                name: entity.to_string(),
                fields: HashMap::new(),
                is_external: false,
                sync_path: String::new(),
                sync_strategy: SyncStrategy::Realtime,
                sync_on: None,
                poll_interval: None,
            }
        });
        entity_schema.fields.entry(field_name.to_string()).or_insert_with(|| {
            FieldSchema {
                name: field_name.to_string(),
                ty: ty.clone(),
                is_derived: false,
                metadata: FieldMetadata { doc: None, range: None, affects: vec![] },
            }
        });
    }
}
