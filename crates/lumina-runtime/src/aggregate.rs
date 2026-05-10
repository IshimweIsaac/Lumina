use rustc_hash::FxHashMap;
use crate::value::Value;
use crate::store::EntityStore;
use lumina_parser::ast::{AggregateDecl, AggregateExpr, AggregateScope};

pub struct AggregateStore {
    decls:  Vec<AggregateDecl>,
    values: FxHashMap<String, FxHashMap<String, Value>>,
}

impl AggregateStore {
    pub fn new() -> Self {
        Self { decls: Vec::new(), values: FxHashMap::default() }
    }

    pub fn register(&mut self, decl: AggregateDecl) {
        self.decls.push(decl);
    }

    pub fn get(&self, agg: &str, field: &str) -> Option<&Value> {
        self.values.get(agg)?.get(field)
    }

    pub fn recompute(
        &mut self,
        store: &EntityStore,
        schema: &lumina_analyzer::types::Schema,
        cluster_state: Option<&FxHashMap<String, FxHashMap<String, Value>>>,
    ) {
        for decl in &self.decls {
            let mut agg_vals = FxHashMap::default();

            for field in &decl.fields {
                let source_field = match &field.expr {
                    AggregateExpr::Avg(f) | AggregateExpr::Min(f) | 
                    AggregateExpr::Max(f) | AggregateExpr::Sum(f) |
                    AggregateExpr::Any(f) | AggregateExpr::All(f) |
                    AggregateExpr::Count(Some(f)) => Some(f),
                    AggregateExpr::Count(None) => None,
                };

                let values: Vec<Value> = match decl.scope {
                    AggregateScope::Cluster | AggregateScope::Region(_) => {
                        if let Some(cs) = cluster_state {
                            if let Some(f) = source_field {
                                cs.values().filter_map(|node_state| node_state.get(f).cloned()).collect()
                            } else {
                                // Count(*) across nodes
                                vec![Value::Unknown; cs.len()]
                            }
                        } else {
                            vec![]
                        }
                    }
                    AggregateScope::Local => {
                        if let Some(entity_schema) = schema.get_entity(&decl.over) {
                            if let Some(f) = source_field {
                                if let Some(idx) = entity_schema.field_indices.get(f) {
                                    store.all_of_entity(&decl.over).filter_map(|(_, inst)| inst.get(*idx).cloned()).collect()
                                } else {
                                    vec![]
                                }
                            } else {
                                // Count(*) local
                                vec![Value::Unknown; store.all_of_entity(&decl.over).count()]
                            }
                        } else {
                            vec![]
                        }
                    }
                };

                let val = compute_agg_from_values(&field.expr, values);
                agg_vals.insert(field.name.clone(), val);
            }
            self.values.insert(decl.name.clone(), agg_vals);
        }
    }
}

fn compute_agg_from_values(expr: &AggregateExpr, values: Vec<Value>) -> Value {
    match expr {
        AggregateExpr::Avg(_) => {
            let ns: Vec<f64> = values.into_iter().filter_map(|v| if let Value::Number(n) = v { Some(n) } else { None }).collect();
            if ns.is_empty() { return Value::Number(0.0); }
            Value::Number(ns.iter().sum::<f64>() / ns.len() as f64)
        }
        AggregateExpr::Min(_) => {
            let ns: Vec<f64> = values.into_iter().filter_map(|v| if let Value::Number(n) = v { Some(n) } else { None }).collect();
            Value::Number(ns.into_iter().fold(f64::INFINITY, f64::min))
        }
        AggregateExpr::Max(_) => {
            let ns: Vec<f64> = values.into_iter().filter_map(|v| if let Value::Number(n) = v { Some(n) } else { None }).collect();
            Value::Number(ns.into_iter().fold(f64::NEG_INFINITY, f64::max))
        }
        AggregateExpr::Sum(_) => {
            let ns: Vec<f64> = values.into_iter().filter_map(|v| if let Value::Number(n) = v { Some(n) } else { None }).collect();
            Value::Number(ns.iter().sum())
        }
        AggregateExpr::Count(None) => {
            Value::Number(values.len() as f64)
        }
        AggregateExpr::Count(Some(_)) => {
            let bs: Vec<bool> = values.into_iter().filter_map(|v| if let Value::Bool(b) = v { Some(b) } else { None }).collect();
            Value::Number(bs.iter().filter(|&&b| b).count() as f64)
        }
        AggregateExpr::Any(_) => {
            let bs: Vec<bool> = values.into_iter().filter_map(|v| if let Value::Bool(b) = v { Some(b) } else { None }).collect();
            Value::Bool(bs.iter().any(|&b| b))
        }
        AggregateExpr::All(_) => {
            if values.is_empty() { return Value::Bool(false); }
            let bs: Vec<bool> = values.into_iter().filter_map(|v| if let Value::Bool(b) = v { Some(b) } else { None }).collect();
            Value::Bool(!bs.is_empty() && bs.iter().all(|&b| b))
        }
    }
}

