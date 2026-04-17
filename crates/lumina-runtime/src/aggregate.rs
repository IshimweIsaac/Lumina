use std::collections::HashMap;
use crate::value::Value;
use crate::store::EntityStore;
use lumina_parser::ast::{AggregateDecl, AggregateExpr, AggregateScope};

pub struct AggregateStore {
    decls:  Vec<AggregateDecl>,
    values: HashMap<String, HashMap<String, Value>>,
}

impl AggregateStore {
    pub fn new() -> Self {
        Self { decls: Vec::new(), values: HashMap::new() }
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
        cluster_state: Option<&HashMap<String, HashMap<String, Value>>>,
    ) {
        for decl in &self.decls {
            let instances: Vec<String>;
            let provider: Box<dyn Fn(&str, &str) -> Option<Value>>;

            match decl.scope {
                AggregateScope::Cluster | AggregateScope::Region(_) => {
                    if let Some(cs) = cluster_state {
                        instances = cs.keys().cloned().collect();
                        let cs_clone = cs.clone();
                        provider = Box::new(move |node_id, field| {
                            cs_clone.get(node_id)?.get(field).cloned()
                        });
                    } else {
                        instances = vec![];
                        provider = Box::new(|_, _| None);
                    }
                }
                AggregateScope::Local => {
                    instances = store.all_of_entity(&decl.over).map(|(n, _)| n.clone()).collect();
                    let store_clone = store.clone();
                    provider = Box::new(move |inst_id, field| {
                        store_clone.get(inst_id)?.get(field).cloned()
                    });
                }
            }

            let mut agg_vals = HashMap::new();
            for field in &decl.fields {
                let val = compute_agg(&field.expr, &instances, &*provider);
                agg_vals.insert(field.name.clone(), val);
            }
            self.values.insert(decl.name.clone(), agg_vals);
        }
    }
}

fn nums(insts: &[String], field: &str, provider: &dyn Fn(&str, &str) -> Option<Value>) -> Vec<f64> {
    insts.iter().filter_map(|i| {
        provider(i, field).and_then(|v| {
            if let Value::Number(n) = v { Some(n) } else { None }
        })
    }).collect()
}

fn bools(insts: &[String], field: &str, provider: &dyn Fn(&str, &str) -> Option<Value>) -> Vec<bool> {
    insts.iter().filter_map(|i| {
        provider(i, field).and_then(|v| {
            if let Value::Bool(b) = v { Some(b) } else { None }
        })
    }).collect()
}

fn compute_agg(
    expr: &AggregateExpr,
    insts: &[String],
    provider: &dyn Fn(&str, &str) -> Option<Value>,
) -> Value {
    match expr {
        AggregateExpr::Avg(f) => {
            let ns = nums(insts, f, provider);
            if ns.is_empty() { return Value::Number(0.0); }
            Value::Number(ns.iter().sum::<f64>() / ns.len() as f64)
        }
        AggregateExpr::Min(f) => {
            let ns = nums(insts, f, provider);
            Value::Number(ns.into_iter().fold(f64::INFINITY, f64::min))
        }
        AggregateExpr::Max(f) => {
            let ns = nums(insts, f, provider);
            Value::Number(ns.into_iter().fold(f64::NEG_INFINITY, f64::max))
        }
        AggregateExpr::Sum(f) => {
            Value::Number(nums(insts, f, provider).iter().sum())
        }
        AggregateExpr::Count(None) => {
            Value::Number(insts.len() as f64)
        }
        AggregateExpr::Count(Some(f)) => {
            Value::Number(
                bools(insts, f, provider).iter().filter(|&&b| b).count() as f64
            )
        }
        AggregateExpr::Any(f) => {
            Value::Bool(bools(insts, f, provider).iter().any(|&b| b))
        }
        AggregateExpr::All(f) => {
            Value::Bool(
                !insts.is_empty() &&
                bools(insts, f, provider).iter().all(|&b| b)
            )
        }
    }
}
