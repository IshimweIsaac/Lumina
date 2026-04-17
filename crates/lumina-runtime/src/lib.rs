pub mod value;
pub mod store;
pub mod snapshot;
pub mod engine;
pub mod rules;
pub mod timers;
pub mod adapter;
pub mod adapters;
pub mod fleet;
pub mod aggregate;
pub mod lsl;

pub use value::Value;
pub use store::{Instance, EntityStore};
pub use snapshot::{Snapshot, SnapshotStack, PropResult, FiredEvent, RollbackResult, Diagnostic};
pub use adapter::LuminaAdapter;
pub use lsl::LslRegistry;

#[derive(Debug)]
pub enum RuntimeError {
    R001 { instance: String },
    R002,
    R003 { depth: usize },
    R004 { index: usize, len: usize },
    R005 { instance: String, field: String },
    R006 { field: String, value: f64, min: f64, max: f64 },
    R007 { entity: String, reason: String },
    R008 { rule: String },
    R009 { field: String },
    /// v1.9: Security violation — write blocked by auth context
    R010 { rule: String, reason: String },
    /// v2.0: Quorum lost — cluster cannot commit writes
    R011 { reason: String },
    /// v2.0: Node isolated — operating in read-only mode
    R012 { reason: String },
    /// v2.0: WAL replication lag exceeds threshold
    R013 { reason: String },
    /// v2.0: Cross-node entity reference unresolvable
    R014 { node: String, entity: String },
    /// v2.0: Orchestration write target unreachable
    R015 { target: String },
    /// v2.0: Cluster aggregate computation timeout
    R016 { reason: String },
    /// v2.0: Migration target has insufficient capacity
    R017 { target: String, reason: String },
}

impl RuntimeError {
    pub fn code(&self) -> &'static str {
        match self {
            RuntimeError::R001 { .. } => "R001",
            RuntimeError::R002        => "R002",
            RuntimeError::R003 { .. } => "R003",
            RuntimeError::R004 { .. } => "R004",
            RuntimeError::R005 { .. } => "R005",
            RuntimeError::R006 { .. } => "R006",
            RuntimeError::R007 { .. } => "R007",
            RuntimeError::R008 { .. } => "R008",
            RuntimeError::R009 { .. } => "R009",
            RuntimeError::R010 { .. } => "L039",
            RuntimeError::R011 { .. } => "L060",
            RuntimeError::R012 { .. } => "L061",
            RuntimeError::R013 { .. } => "L062",
            RuntimeError::R014 { .. } => "L063",
            RuntimeError::R015 { .. } => "L064",
            RuntimeError::R016 { .. } => "L065",
            RuntimeError::R017 { .. } => "L066",
        }
    }

    pub fn message(&self) -> String {
        match self {
            RuntimeError::R001 { instance }   => format!("Access to deleted instance: '{instance}'"),
            RuntimeError::R002                => "Division by zero".to_string(),
            RuntimeError::R003 { depth }      => format!("Rule re-entrancy limit exceeded ({depth})"),
            RuntimeError::R004 { index, len } => format!("List index out of bounds: {index} of {len}"),
            RuntimeError::R005 { instance, field } => format!("Null field access: '{instance}.{field}'"),
            RuntimeError::R006 { field, value, min, max } => format!("@range violation: {field} = {value}, expected {min}–{max}"),
            RuntimeError::R007 { entity, reason }  => format!("External entity sync failed: {entity} ({reason})"),
            RuntimeError::R008 { rule }            => format!("Timer conflict: rule '{rule}' already has a pending timer"),
            RuntimeError::R009 { field }             => format!("Cannot update derived field '{field}' — it is computed automatically"),
            RuntimeError::R010 { rule, reason }    => format!("Security violation in rule '{rule}': {reason}. Write action blocked by auth context."),
            RuntimeError::R011 { reason }          => format!("Quorum lost — cluster cannot commit writes: {reason}"),
            RuntimeError::R012 { reason }          => format!("Node isolated — operating in read-only mode: {reason}"),
            RuntimeError::R013 { reason }          => format!("WAL replication lag exceeds threshold: {reason}"),
            RuntimeError::R014 { node, entity }    => format!("Cross-node entity reference unresolvable: {node}.{entity}"),
            RuntimeError::R015 { target }          => format!("Orchestration write target unreachable: {target}"),
            RuntimeError::R016 { reason }          => format!("Cluster aggregate computation timeout: {reason}"),
            RuntimeError::R017 { target, reason }  => format!("Migration target '{target}' has insufficient capacity: {reason}"),
        }
    }
}
