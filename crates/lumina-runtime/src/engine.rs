use crate::adapter::LuminaAdapter;
use crate::aggregate::AggregateStore;
use crate::fleet::FleetState;
use crate::rules;
use crate::snapshot::{Diagnostic, FiredEvent, PropResult, RollbackResult, SnapshotStack};
use crate::store::{EntityStore, Instance};
use crate::timers::TimerHeap;
use crate::value::Value;
use crate::RuntimeError;
use lumina_analyzer::graph::DependencyGraph;
use lumina_analyzer::types::Schema;
use lumina_cluster::ClusterNode;
use lumina_parser::ast::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;


pub const MAX_DEPTH: usize = 100;

pub struct Evaluator {
    pub schema: Schema,
    pub graph: DependencyGraph,
    pub rules: Vec<RuleDecl>,
    pub store: EntityStore,
    pub snapshots: SnapshotStack,
    pub env: FxHashMap<String, Value>,
    pub instances: FxHashMap<String, String>,
    pub derived_exprs: FxHashMap<(String, String), Expr>,
    pub functions: FxHashMap<String, FnDecl>,
    pub timers: TimerHeap,
    pub adapters: Vec<Box<dyn LuminaAdapter>>,
    pub prev_store: Option<EntityStore>,
    pub fleet_state: FleetState,
    prev_fleet_any: FxHashMap<(String, String), bool>,
    prev_fleet_all: FxHashMap<(String, String), bool>,
    depth: usize,
    fired_this_cycle: FxHashSet<String>,
    output: Vec<String>,
    prev_rule_conditions: FxHashMap<(String, String), bool>,
    pub cooldown_map: FxHashMap<(String, String), f64>,
    pub rule_active: FxHashMap<(String, String), bool>,
    pub frequency_events: FxHashMap<(String, String), Vec<f64>>,
    pub agg_store: AggregateStore,
    pub cluster_state: FxHashMap<String, FxHashMap<String, Value>>,
    pub cluster_config: Option<lumina_cluster::ClusterConfig>,
    pub now: f64,
    /// Issue 003: Current rule parameter alias (param_name, entity_name)
    pub rule_param_alias: Option<(String, String)>,
    pub is_initializing: bool,
    pub reverse_refs: FxHashMap<String, rustc_hash::FxHashSet<(String, String)>>,
    pub dirty_instances: FxHashSet<String>,
    pub cluster_node: Option<Arc<Mutex<ClusterNode>>>,
}
impl Evaluator {
    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    pub fn new(schema: Schema, graph: DependencyGraph, rules: Vec<RuleDecl>) -> Self {
        let mut timers = TimerHeap::new();
        timers.register_every_rules(&rules);
        Self {
            schema,
            graph,
            rules,
            store: EntityStore::new(),
            snapshots: SnapshotStack::new(),
            env: FxHashMap::default(),
            instances: FxHashMap::default(),
            derived_exprs: FxHashMap::default(),
            functions: FxHashMap::default(),
            timers,
            adapters: Vec::new(),
            prev_store: None,
            fleet_state: FleetState::new(),
            prev_fleet_any: FxHashMap::default(),
            prev_fleet_all: FxHashMap::default(),
            depth: 0,
            fired_this_cycle: FxHashSet::default(),
            output: Vec::new(),
            prev_rule_conditions: FxHashMap::default(),
            cooldown_map: FxHashMap::default(),
            rule_active: FxHashMap::default(),
            frequency_events: FxHashMap::default(),
            agg_store: AggregateStore::new(),
            cluster_state: FxHashMap::default(),
            cluster_config: None,
            now: 0.0,
            rule_param_alias: None,
            is_initializing: false,
            reverse_refs: FxHashMap::default(),
            dirty_instances: FxHashSet::default(),
            cluster_node: None,
        }
    }

    /// Creates an empty evaluator with no entities, rules, or instances.
    /// Used by the REPL - statements are added one at a time via exec_statement().
    pub fn new_empty() -> Self {
        Self::default()
    }

    /// describe all declared entities as a human-readable string.
    pub fn sync_cluster_state(&mut self) {
        if let Some(ref node_lock) = self.cluster_node {
            if let Ok(mut node) = node_lock.lock() {
                node.tick(std::time::Instant::now());
                let raw_mesh = node.collect_cluster_state();
                for (node_id, fields) in raw_mesh {
                    // Don't overwrite local state with our own gossiped state in the cluster view
                    if node_id == node.config.node_id {
                        continue;
                    }

                    let entry = self.cluster_state.entry(node_id).or_default();
                    for (field, bytes) in fields {
                        if let Ok(val) = serde_json::from_slice::<Value>(&bytes) {
                            entry.insert(field, val);
                        }
                    }
                }
            }
        }
    }


    /// Describe all declared entities as a human-readable string.
    /// Used by :schema REPL command.
    pub fn describe_schema(&self) -> String {
        if self.schema.entities.is_empty() {
            return "(no entities declared)".into();
        }
        self.schema
            .entities
            .iter()
            .map(|(name, ent)| {
                let fields = ent
                    .fields
                    .iter()
                    .map(|(n, f)| format!("{}: {:?}", n, f.ty))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("entity {} {{ {} }}", name, fields)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn should_fire(&self, rule_name: &str, instance_name: &str, cooldown: &Duration) -> bool {
        if let Some(last_fired) = self
            .cooldown_map
            .get(&(rule_name.to_string(), instance_name.to_string()))
        {
            (self.now - *last_fired) >= (cooldown.to_seconds() * 1000.0)
        } else {
            true
        }
    }

    pub fn record_firing(&mut self, rule_name: &str, instance_name: &str) {
        self.cooldown_map
            .insert((rule_name.to_string(), instance_name.to_string()), self.now);
    }

    pub fn register_derived(&mut self, entity: &str, field: &str, expr: Expr) {
        self.derived_exprs
            .insert((entity.to_string(), field.to_string()), expr);
    }

    /// Register an external entity adapter.
    pub fn register_adapter(&mut self, a: Box<dyn LuminaAdapter>) {
        self.adapters.push(a);
    }

    pub fn drain_output(&mut self) -> Vec<String> {
        std::mem::take(&mut self.output)
    }

    pub fn get_field_idx(&self, entity_name: &str, field_name: &str) -> Option<usize> {
        self.schema
            .get_entity(entity_name)?
            .field_indices
            .get(field_name)
            .copied()
    }

    pub fn get_instance_field(&self, instance: &Instance, field: &str) -> Option<Value> {
        let idx = self.get_field_idx(&instance.entity_name, field)?;
        instance.get(idx).cloned()
    }

    pub fn get_instance_prev_field(&self, instance: &Instance, field: &str) -> Option<Value> {
        let idx = self.get_field_idx(&instance.entity_name, field)?;
        instance.prev(idx).cloned()
    }

    pub fn set_instance_field(
        &mut self,
        instance_name: &str,
        field: &str,
        val: Value,
    ) -> Result<(), RuntimeError> {
        let entity_name = self
            .store
            .get(instance_name)
            .ok_or(RuntimeError::R001 {
                instance: instance_name.to_string(),
            })?
            .entity_name
            .clone();
        let idx = self
            .get_field_idx(&entity_name, field)
            .ok_or(RuntimeError::R005 {
                instance: instance_name.to_string(),
                field: field.to_string(),
            })?;

        if let Some(inst) = self.store.get_mut(instance_name) {
            inst.set(idx, val);
        }
        Ok(())
    }

    // ── Expression evaluator ──────────────────────────────

    pub fn eval_expr(&self, expr: &Expr, ctx: Option<&str>) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::Text(s) => Ok(Value::Text(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Duration(d) => Ok(Value::Duration(d.to_seconds())),

            Expr::Ident(name) => {
                if let Some(inst) = ctx {
                    if let Some(instance) = self.store.get(inst) {
                        if let Some(val) = self.get_instance_field(instance, name) {
                            return Ok(val);
                        }
                    }
                }
                if let Some(val) = self.env.get(name) {
                    return Ok(val.clone());
                }
                if let Some(val) = self.agg_store.get(name, "") {
                    return Ok(val.clone());
                }
                // E1 Fix: Check if the identifier is a known instance name.
                // This resolves the R001 bug where instance names (e.g. 'unit1')
                // could not be resolved during rule execution.
                if self.instances.contains_key(name) {
                    return Ok(Value::Text(name.clone()));
                }
                Err(RuntimeError::R001 {
                    instance: name.clone(),
                })
            }

            Expr::FieldAccess { obj, field, .. } => {
                let obj_val = self.eval_expr(obj, ctx);

                if let Ok(Value::Timestamp(ts)) = obj_val {
                    if field == "age" {
                        return Ok(Value::Duration(self.now - ts));
                    }
                    return Err(RuntimeError::R005 {
                        instance: "Timestamp".into(),
                        field: field.clone(),
                    });
                }

                let mut inst_name = match obj_val {
                    Ok(Value::Text(s)) => s,
                    Err(_) => {
                        if let Expr::Ident(n) = obj.as_ref() {
                            n.clone()
                        } else {
                            return Err(RuntimeError::R001 {
                                instance: format!("{:?}", obj),
                            });
                        }
                    }
                    _ => {
                        return Err(RuntimeError::R001 {
                            instance: format!("{:?}", obj),
                        })
                    }
                };

                // Bug Fix: If inst_name is an entity name, and it matches the current context's entity,
                // then resolve it to the context instance.
                if let Some(ctx_inst) = ctx {
                    if let Some(ctx_ent) = self.instances.get(ctx_inst) {
                        if &inst_name == ctx_ent {
                            inst_name = ctx_inst.to_string();
                        }
                    }
                }

                // Issue 003: If inst_name matches a rule parameter alias, resolve to context instance
                if let Some((ref param_name, ref _entity_name)) = self.rule_param_alias {
                    if &inst_name == param_name {
                        if let Some(ctx_inst) = ctx {
                            inst_name = ctx_inst.to_string();
                        }
                    }
                }

                if let Some(val) = self.agg_store.get(&inst_name, field) {
                    return Ok(val.clone());
                }

                // Issue 004: Support built-in '.id' property
                if field == "id" {
                    return Ok(Value::Text(inst_name.clone()));
                }

                let instance = self.store.get(&inst_name).ok_or(RuntimeError::R001 {
                    instance: inst_name.clone(),
                })?;

                let val = self
                    .get_instance_field(instance, field)
                    .ok_or(RuntimeError::R005 {
                        instance: inst_name.clone(),
                        field: field.clone(),
                    })?;

                match &val {
                    Value::Timestamp(ts) if field == "age" => Ok(Value::Duration(self.now - ts)),
                    _ => Ok(val),
                }
            }

            Expr::Binary {
                op, left, right, ..
            } => {
                // Short-circuit
                if *op == BinOp::And {
                    let l = self.eval_expr(left, ctx)?;
                    if l == Value::Bool(false) {
                        return Ok(Value::Bool(false));
                    }
                    let r = self.eval_expr(right, ctx)?;
                    return Ok(Value::Bool(
                        l == Value::Bool(true) && r == Value::Bool(true),
                    ));
                }
                if *op == BinOp::Or {
                    let l = self.eval_expr(left, ctx)?;
                    if l == Value::Bool(true) {
                        return Ok(Value::Bool(true));
                    }
                    let r = self.eval_expr(right, ctx)?;
                    return Ok(Value::Bool(r == Value::Bool(true)));
                }

                let l = self.eval_expr(left, ctx)?;
                let r = self.eval_expr(right, ctx)?;
                self.apply_binop(op, l, r)
            }

            Expr::Unary { op, operand, .. } => {
                let v = self.eval_expr(operand, ctx)?;
                match op {
                    UnOp::Neg => match v {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        other => Err(RuntimeError::R018 { op: "negate".into(), left: other.type_name().into(), right: "N/A".into() }),
                    },
                    UnOp::Not => match v {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        other => Err(RuntimeError::R018 { op: "not".into(), left: other.type_name().into(), right: "N/A".into() }),
                    },
                }
            }

            Expr::If {
                cond, then_, else_, ..
            } => {
                if self.eval_expr(cond, ctx)? == Value::Bool(true) {
                    self.eval_expr(then_, ctx)
                } else {
                    self.eval_expr(else_, ctx)
                }
            }

            Expr::InterpolatedString(segments) => {
                let mut out = String::new();
                for seg in segments {
                    match seg {
                        StringSegment::Literal(s) => out.push_str(s),
                        StringSegment::Expr(e) => {
                            let v = self.eval_expr(e, ctx)?;
                            out.push_str(&v.to_string());
                        }
                    }
                }
                Ok(Value::Text(out))
            }

            Expr::Call { name, args, .. } => {
                // Built-in list functions (checked before user fn_defs)
                match name.as_str() {
                    "len" => {
                        let list = self.eval_to_list(&args[0], ctx)?;
                        return Ok(Value::Number(list.len() as f64));
                    }
                    "min" => {
                        let list = self.eval_to_num_list(&args[0], ctx)?;
                        if list.is_empty() {
                            return Err(RuntimeError::R004 { index: 0, len: 0 });
                        }
                        return Ok(Value::Number(
                            list.iter().cloned().fold(f64::INFINITY, f64::min),
                        ));
                    }
                    "max" => {
                        let list = self.eval_to_num_list(&args[0], ctx)?;
                        if list.is_empty() {
                            return Err(RuntimeError::R004 { index: 0, len: 0 });
                        }
                        return Ok(Value::Number(
                            list.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                        ));
                    }
                    "sum" => {
                        let list = self.eval_to_num_list(&args[0], ctx)?;
                        return Ok(Value::Number(list.iter().sum()));
                    }
                    "append" => {
                        let mut list = self.eval_to_list(&args[0], ctx)?;
                        let val = self.eval_expr(&args[1], ctx)?;
                        list.push(val);
                        return Ok(Value::List(list));
                    }
                    "head" => {
                        let list = self.eval_to_list(&args[0], ctx)?;
                        if list.is_empty() {
                            return Err(RuntimeError::R004 { index: 0, len: 0 });
                        }
                        return Ok(list[0].clone());
                    }
                    "tail" => {
                        let list = self.eval_to_list(&args[0], ctx)?;
                        if list.is_empty() {
                            return Err(RuntimeError::R004 { index: 0, len: 0 });
                        }
                        return Ok(Value::List(list[1..].to_vec()));
                    }
                    "at" => {
                        let list = self.eval_to_list(&args[0], ctx)?;
                        let idx = self
                            .eval_expr(&args[1], ctx)?
                            .as_number()
                            .ok_or(RuntimeError::R002)? as usize;
                        if idx >= list.len() {
                            return Err(RuntimeError::R004 {
                                index: idx,
                                len: list.len(),
                            });
                        }
                        return Ok(list[idx].clone());
                    }
                    "now" => {
                        return Ok(Value::Timestamp(self.now));
                    }
                    "env" => {
                        // v1.9: Read environment variable, return as Secret
                        if let Some(arg) = args.first() {
                            let var_name = match self.eval_expr(arg, ctx)? {
                                Value::Text(s) => s,
                                _ => return Err(RuntimeError::R002),
                            };
                            let val = std::env::var(&var_name).unwrap_or_default();
                            return Ok(Value::Secret(val));
                        }
                        return Err(RuntimeError::R002);
                    }
                    _ => {} // Fall through to user-defined fn lookup
                }
                let decl = self.functions.get(name).ok_or(RuntimeError::R002)?.clone();
                let arg_vals: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_expr(a, ctx))
                    .collect::<Result<_, _>>()?;
                let mut local: FxHashMap<String, Value> = FxHashMap::default();
                for (param, val) in decl.params.iter().zip(arg_vals) {
                    local.insert(param.name.clone(), val);
                }
                self.eval_expr_local(&decl.body, &local)
            }
            Expr::ListLiteral(elems) => {
                let vals: Vec<Value> = elems
                    .iter()
                    .map(|e| self.eval_expr(e, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(Value::List(vals))
            }
            Expr::Index { list, index, .. } => {
                let list_val = self.eval_to_list(list, ctx)?;
                let idx = self
                    .eval_expr(index, ctx)?
                    .as_number()
                    .ok_or(RuntimeError::R002)? as usize;
                if idx >= list_val.len() {
                    return Err(RuntimeError::R004 {
                        index: idx,
                        len: list_val.len(),
                    });
                }
                Ok(list_val[idx].clone())
            }
            Expr::Prev { field, .. } => {
                let inst_name = ctx.ok_or(RuntimeError::R001 {
                    instance: "global".into(),
                })?;

                // First check prev_store
                if let Some(prev) = &self.prev_store {
                    if let Some(instance) = prev.get(inst_name) {
                        return self.get_instance_field(instance, field).ok_or(
                            RuntimeError::R005 {
                                instance: inst_name.to_string(),
                                field: field.clone(),
                            },
                        );
                    }
                }

                // Fallback to current store if prev_store is not set (e.g. initialization)
                let instance = self.store.get(inst_name).ok_or(RuntimeError::R001 {
                    instance: inst_name.to_string(),
                })?;
                self.get_instance_field(instance, field)
                    .ok_or(RuntimeError::R005 {
                        instance: inst_name.to_string(),
                        field: field.clone(),
                    })
            }
            Expr::ClusterAccess { node_id, field, .. } => {
                // v2.0: Access cluster state.
                if let Some(node_state) = self.cluster_state.get(node_id) {
                    if let Some(val) = node_state.get(field) {
                        return Ok(val.clone());
                    }
                    return Err(RuntimeError::R014 {
                        node: node_id.clone(),
                        entity: field.clone(),
                    }); // Unresolvable ref
                }
                Err(RuntimeError::R012 {
                    reason: format!("Node {} not found in cluster state", node_id),
                }) // Node isolated / missing
            }
            Expr::Migrate {
                workloads, target, ..
            } => {
                let w_val = self.eval_expr(workloads, ctx)?;
                let t_val = self.eval_expr(target, ctx)?;

                let target_node = match t_val {
                    Value::Text(s) => s,
                    _ => return Err(RuntimeError::R002),
                };

                let inst_names: Vec<String> = match w_val {
                    Value::Text(s) => vec![s],
                    Value::List(l) => l
                        .into_iter()
                        .filter_map(|v| {
                            if let Value::Text(s) = v {
                                Some(s)
                            } else {
                                None
                            }
                        })
                        .collect(),
                    _ => return Err(RuntimeError::R002),
                };

                if let Some(ref node_lock) = self.cluster_node {
                    let mut node = node_lock.lock().unwrap();
                    let msg = lumina_cluster::GossipMessageKind::WorkloadMove {
                        target_node,
                        workload: inst_names,
                    };
                    node.gossip
                        .broadcast(node.config.node_id.clone(), msg.clone());
                    // Local loopback: handle migration of our own instances
                    node.orchestration_queue.push(msg);
                }
                Ok(Value::Bool(true))
            }
            Expr::Evacuate { entities, .. } => {
                let e_val = self.eval_expr(entities, ctx)?;
                let entity_names: Vec<String> = match e_val {
                    Value::Text(s) => vec![s],
                    Value::List(l) => l
                        .into_iter()
                        .filter_map(|v| {
                            if let Value::Text(s) = v {
                                Some(s)
                            } else {
                                None
                            }
                        })
                        .collect(),
                    _ => return Err(RuntimeError::R002),
                };

                // Evacuate: find all local instances of these entities and migrate them to others
                let mut instances_to_move = Vec::new();
                for (name, entity) in &self.instances {
                    if entity_names.contains(entity) {
                        instances_to_move.push(name.clone());
                    }
                }

                if !instances_to_move.is_empty() {
                    if let Some(ref node_lock) = self.cluster_node {
                        let mut node = node_lock.lock().unwrap();
                        let peers = node.gossip.peer_statuses();
                        let alive_peers: Vec<_> = peers
                            .iter()
                            .filter(|p| p.health == lumina_cluster::PeerHealth::Alive)
                            .collect();

                        if !alive_peers.is_empty() {
                            // Simple round-robin or just pick first alive peer for now
                            let target_node = alive_peers[0].peer_id.clone();
                            let msg = lumina_cluster::GossipMessageKind::WorkloadMove {
                                target_node,
                                workload: instances_to_move,
                            };
                            node.gossip
                                .broadcast(node.config.node_id.clone(), msg.clone());
                            // Local loopback
                            node.orchestration_queue.push(msg);
                        }
                    }
                }
                Ok(Value::Bool(true))
            }
            Expr::Deploy { spec, .. } => {
                // Deploy: evaluation of spec (simplified for v2.0)
                let _s = self.eval_expr(spec, ctx)?;
                // In a real system, the leader would broadcast WorkloadDeploy
                Ok(Value::Bool(true))
            }
        }
    }

    // ── Function evaluation ───────────────────────────────

    fn apply_binop(&self, op: &BinOp, l: Value, r: Value) -> Result<Value, RuntimeError> {
        match op {
            BinOp::Add => match (&l, &r) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::Text(a), Value::Text(b)) => Ok(Value::Text(format!("{}{}", a, b))),
                _ => Err(RuntimeError::R018 { op: "+".into(), left: l.type_name().into(), right: r.type_name().into() }),
            },
            BinOp::Sub => match (&l, &r) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                _ => Err(RuntimeError::R018 { op: "-".into(), left: l.type_name().into(), right: r.type_name().into() }),
            },
            BinOp::Mul => match (&l, &r) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                _ => Err(RuntimeError::R018 { op: "*".into(), left: l.type_name().into(), right: r.type_name().into() }),
            },
            BinOp::Div => match (&l, &r) {
                (Value::Number(a), Value::Number(b)) => {
                    if *b == 0.0 { Err(RuntimeError::R002) } else { Ok(Value::Number(a / b)) }
                }
                _ => Err(RuntimeError::R018 { op: "/".into(), left: l.type_name().into(), right: r.type_name().into() }),
            },
            BinOp::Mod => match (&l, &r) {
                (Value::Number(a), Value::Number(b)) => {
                    if *b == 0.0 { Err(RuntimeError::R002) } else { Ok(Value::Number(a % b)) }
                }
                _ => Err(RuntimeError::R018 { op: "%".into(), left: l.type_name().into(), right: r.type_name().into() }),
            },
            BinOp::Eq => Ok(Value::Bool(l == r)),
            BinOp::Ne => Ok(Value::Bool(l != r)),
            BinOp::Gt  => match (&l, &r) { 
                (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
                (Value::Duration(a), Value::Duration(b)) => Ok(Value::Bool(a > b)),
                _ => Err(RuntimeError::R018 { op: ">".into(), left: l.type_name().into(), right: r.type_name().into() }),
            },
            BinOp::Lt  => match (&l, &r) { 
                (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
                (Value::Duration(a), Value::Duration(b)) => Ok(Value::Bool(a < b)),
                _ => Err(RuntimeError::R018 { op: "<".into(), left: l.type_name().into(), right: r.type_name().into() }),
            },
            BinOp::Ge  => match (&l, &r) { 
                (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
                (Value::Duration(a), Value::Duration(b)) => Ok(Value::Bool(a >= b)),
                _ => Err(RuntimeError::R018 { op: ">=".into(), left: l.type_name().into(), right: r.type_name().into() }),
            },
            BinOp::Le  => match (&l, &r) { 
                (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
                (Value::Duration(a), Value::Duration(b)) => Ok(Value::Bool(a <= b)),
                _ => Err(RuntimeError::R018 { op: "<=".into(), left: l.type_name().into(), right: r.type_name().into() }),
            },
            BinOp::And | BinOp::Or => unreachable!(),
        }
    }

    /// Public wrapper around apply_binop for use by the rules module
    /// when evaluating expressions against historical state.
    pub fn eval_binary_values(
        &self,
        op: &BinOp,
        l: &Value,
        r: &Value,
    ) -> Result<Value, RuntimeError> {
        self.apply_binop(op, l.clone(), r.clone())
    }

    fn eval_expr_local(
        &self,
        expr: &Expr,
        locals: &FxHashMap<String, Value>,
    ) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Ident(name) => locals.get(name).cloned().ok_or(RuntimeError::R005 {
                instance: name.clone(),
                field: name.clone(),
            }),
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::Text(s) => Ok(Value::Text(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Binary {
                op, left, right, ..
            } => {
                if *op == BinOp::And {
                    let l = self.eval_expr_local(left, locals)?;
                    if l == Value::Bool(false) {
                        return Ok(Value::Bool(false));
                    }
                    let r = self.eval_expr_local(right, locals)?;
                    return Ok(Value::Bool(r == Value::Bool(true)));
                }
                if *op == BinOp::Or {
                    let l = self.eval_expr_local(left, locals)?;
                    if l == Value::Bool(true) {
                        return Ok(Value::Bool(true));
                    }
                    let r = self.eval_expr_local(right, locals)?;
                    return Ok(Value::Bool(r == Value::Bool(true)));
                }
                let l = self.eval_expr_local(left, locals)?;
                let r = self.eval_expr_local(right, locals)?;
                self.apply_binop(op, l, r)
            }
            Expr::If {
                cond, then_, else_, ..
            } => {
                let c = self.eval_expr_local(cond, locals)?;
                if c == Value::Bool(true) {
                    self.eval_expr_local(then_, locals)
                } else {
                    self.eval_expr_local(else_, locals)
                }
            }
            Expr::InterpolatedString(segments) => {
                let mut out = String::new();
                for seg in segments {
                    match seg {
                        StringSegment::Literal(s) => out.push_str(s),
                        StringSegment::Expr(e) => {
                            let v = self.eval_expr_local(e, locals)?;
                            out.push_str(&v.to_string());
                        }
                    }
                }
                Ok(Value::Text(out))
            }
            Expr::ListLiteral(elems) => {
                let vals: Vec<Value> = elems
                    .iter()
                    .map(|e| self.eval_expr_local(e, locals))
                    .collect::<Result<_, _>>()?;
                Ok(Value::List(vals))
            }
            Expr::Index { list, index, .. } => {
                let list_val = self.eval_expr_local(list, locals)?;
                let items = match list_val {
                    Value::List(l) => l,
                    _ => return Err(RuntimeError::R002),
                };
                let idx = self
                    .eval_expr_local(index, locals)?
                    .as_number()
                    .ok_or(RuntimeError::R002)? as usize;
                if idx >= items.len() {
                    return Err(RuntimeError::R004 {
                        index: idx,
                        len: items.len(),
                    });
                }
                Ok(items[idx].clone())
            }
            _ => Err(RuntimeError::R002), // unsupported expr in fn body
        }
    }

    // ── Statement executor ────────────────────────────────

    pub fn exec_statement(&mut self, stmt: &Statement) -> Result<Vec<FiredEvent>, RuntimeError> {
        match stmt {
            Statement::Entity(_) | Statement::Rule(_) => Ok(vec![]),
            // Issue 001: External entities must create a default instance in the store
            Statement::ExternalEntity(decl) => {
                let mut fields = FxHashMap::default();
                for field in &decl.fields {
                    match field {
                        lumina_parser::ast::Field::Stored(f) => {
                            let default_val = match &f.ty {
                                lumina_parser::ast::LuminaType::Number => Value::Number(0.0),
                                lumina_parser::ast::LuminaType::Text => Value::Text(String::new()),
                                lumina_parser::ast::LuminaType::Boolean => Value::Bool(false),
                                lumina_parser::ast::LuminaType::Timestamp => Value::Timestamp(0.0),
                                lumina_parser::ast::LuminaType::Secret => {
                                    Value::Secret(String::new())
                                }
                                lumina_parser::ast::LuminaType::Duration => Value::Duration(0.0),
                                lumina_parser::ast::LuminaType::List(_) => Value::List(vec![]),
                                lumina_parser::ast::LuminaType::Entity(_) => {
                                    Value::Text(String::new())
                                }
                            };
                            fields.insert(f.name.clone(), default_val);
                        }
                        lumina_parser::ast::Field::Derived(df) => {
                            self.derived_exprs
                                .insert((decl.name.clone(), df.name.clone()), df.expr.clone());
                            fields.insert(df.name.clone(), Value::Unknown);
                        }
                        lumina_parser::ast::Field::Ref(r) => {
                            fields.insert(r.name.clone(), Value::Text(String::new()));
                        }
                    }
                }
                let entity_schema = self.schema.get_entity(&decl.name).unwrap();
                let mut instance = Instance::new(&decl.name, entity_schema.field_names.len());
                for (name, val) in fields {
                    if let Some(idx) = entity_schema.field_indices.get(&name) {
                        if let Value::Text(ref target) = val {
                            if self.schema.is_ref_field(&decl.name, &name) {
                                self.reverse_refs
                                    .entry(target.clone())
                                    .or_default()
                                    .insert((decl.name.clone(), decl.name.clone()));
                            }
                        }
                        instance.set(*idx, val);
                    }
                }
                self.store.insert(decl.name.clone(), instance);
                self.instances.insert(decl.name.clone(), decl.name.clone());
                // Propagate derived fields for the new instance
                let _ = self.propagate_derived(&decl.name, &decl.name);
                Ok(vec![])
            }
            Statement::Aggregate(decl) => {
                self.agg_store.register(decl.clone());
                self.agg_store
                    .recompute(&self.store, &self.schema, Some(&self.cluster_state));
                Ok(vec![])
            }
            Statement::Fn(decl) => {
                self.functions.insert(decl.name.clone(), decl.clone());
                Ok(vec![])
            }
            Statement::Let(ls) => {
                match &ls.value {
                    LetValue::Expr(expr) => {
                        let val = self.eval_expr(expr, None)?;
                        self.env.insert(ls.name.clone(), val);
                        Ok(vec![])
                    }
                    LetValue::EntityInit(init) => {
                        let mut fields = FxHashMap::default();
                        let inst_name = ls.name.clone();
                        let entity_name = init.entity_name.clone();
                        for (name, expr) in &init.fields {
                            let val = self.eval_expr(expr, None)?;
                            if let Value::Text(ref target) = val {
                                if self.schema.is_ref_field(&entity_name, name) {
                                    self.reverse_refs
                                        .entry(target.clone())
                                        .or_default()
                                        .insert((inst_name.clone(), entity_name.clone()));
                                }
                            }
                            fields.insert(name.clone(), val);
                        }
                        self.instances
                            .insert(inst_name.clone(), entity_name.clone());
                        let entity_schema = self.schema.get_entity(&entity_name).unwrap();
                        let mut instance =
                            Instance::new(&entity_name, entity_schema.field_names.len());
                        for (name, val) in fields {
                            if let Some(idx) = entity_schema.field_indices.get(&name) {
                                instance.set(*idx, val);
                            }
                        }
                        self.store.insert(inst_name.clone(), instance);
                        // Compute derived fields for the new instance
                        self.propagate_derived(&inst_name, &entity_name)?;
                        if !self.is_initializing {
                            self.agg_store.recompute(
                                &self.store,
                                &self.schema,
                                Some(&self.cluster_state),
                            );
                        }
                        self.store.commit_dirty(&self.dirty_instances);
                        self.dirty_instances.clear();

                        // Initial rule evaluation for this new instance
                        if !self.is_initializing {
                            self.evaluate_rules(&inst_name)
                        } else {
                            Ok(vec![])
                        }
                    }
                }
            }
            Statement::Action(a) => self.exec_action(a, None),
            Statement::Import(import_decl) => {
                // v1.9: If this is an LSL import, register the schema fields
                if let Some(ref ns) = import_decl.namespace {
                    let lsl = crate::lsl::LslRegistry::new();
                    if let Some(entity_decl) = lsl.resolve(&import_decl.path) {
                        let entity_name = ns.last().cloned().unwrap_or_default();
                        // Register entity fields into the schema
                        for field in &entity_decl.fields {
                            match field {
                                lumina_parser::ast::Field::Stored(sf) => {
                                    self.schema.register_field(&entity_name, &sf.name, &sf.ty);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Ok(vec![])
            }
            Statement::PluginImport(_) => Ok(vec![]), // v1.8: Plugin registration handled at build time
            Statement::Provider(decl) => {
                // v1.9: Log provider registration for the orchestrator
                self.output.push(format!(
                    "Provider '{}' registered (endpoint configured)",
                    decl.protocol
                ));
                Ok(vec![])
            }
            Statement::Cluster(decl) => {
                // v2.0: Store cluster configuration for use by the CLI cluster commands
                let config = lumina_cluster::ClusterConfig::from_decl(decl);
                self.output.push(format!(
                    "Cluster configured: node='{}' peers={} quorum={}",
                    config.node_id,
                    config.peers.len(),
                    config.quorum
                ));
                self.cluster_config = Some(config);
                Ok(vec![])
            }
        }
    }

    // ── Action executor ───────────────────────────────────

    pub fn exec_action(
        &mut self,
        action: &Action,
        ctx: Option<&str>,
    ) -> Result<Vec<FiredEvent>, RuntimeError> {
        match action {
            Action::Show(expr) => {
                let val = self.eval_expr(expr, ctx)?;
                let s = val.to_string();
                println!("{}", s);
                self.output.push(s);
                Ok(vec![])
            }
            Action::Update { target, value } => {
                let val = self.eval_expr(value, ctx)?;
                let mut inst_name = target.instance.clone();
                if let Some(ctx_inst) = ctx {
                    if let Some(ctx_ent) = self.instances.get(ctx_inst) {
                        if ctx_ent == &inst_name {
                            inst_name = ctx_inst.to_string();
                        }
                    }
                }
                // Issue 003: Resolve rule parameter alias for update targets
                if let Some((ref param_name, _)) = self.rule_param_alias {
                    if &inst_name == param_name {
                        if let Some(ctx_inst) = ctx {
                            inst_name = ctx_inst.to_string();
                        }
                    }
                }
                // Issue 006: Handle nested field paths (e.g., server.cooling.power)
                if let Some(ref sub_field) = target.sub_field {
                    if let Some(inst) = self.store.get(&inst_name) {
                        if let Some(Value::Text(ref_target)) =
                            self.get_instance_field(inst, &target.field)
                        {
                            let ref_inst = ref_target.clone();
                            return self.apply_update(&ref_inst, sub_field, val);
                        }
                    }
                    return Err(RuntimeError::R005 {
                        instance: inst_name,
                        field: target.field.clone(),
                    });
                }
                self.apply_update(&inst_name, &target.field, val)
            }
            Action::Create { entity, fields } => {
                let mut fv = FxHashMap::default();
                let count = self.store.all_of_entity(entity).count();
                let inst_name = format!("{}_{}", entity.to_lowercase(), count + 1);

                for (name, expr) in fields {
                    let val = self.eval_expr(expr, ctx)?;
                    if let Value::Text(ref target) = val {
                        if self.schema.is_ref_field(entity, name) {
                            self.reverse_refs
                                .entry(target.clone())
                                .or_default()
                                .insert((inst_name.clone(), entity.clone()));
                        }
                    }
                    fv.insert(name.clone(), val);
                }
                self.instances.insert(inst_name.clone(), entity.clone());

                // Update fleet state for any Boolean fields on the new instance
                for (fname, val) in &fv {
                    if let Value::Bool(b) = val {
                        let total = self.store.all_of_entity(entity).count() + 1;
                        self.fleet_state.update(entity, fname, false, *b, total);
                    }
                }

                let entity_schema = self.schema.get_entity(entity).unwrap();
                let mut instance = Instance::new(entity, entity_schema.field_names.len());
                for (name, val) in fv {
                    if let Some(idx) = entity_schema.field_indices.get(&name) {
                        instance.set(*idx, val);
                    }
                }
                self.store.insert(inst_name, instance);

                // Recompute aggregates to include the new instance
                self.agg_store
                    .recompute(&self.store, &self.schema, Some(&self.cluster_state));

                Ok(vec![])
            }
            Action::Delete(name) => {
                let mut inst_name = name.clone();
                if let Some(ctx_inst) = ctx {
                    if let Some(ctx_ent) = self.instances.get(ctx_inst) {
                        if ctx_ent == &inst_name {
                            inst_name = ctx_inst.to_string();
                        }
                    }
                }
                if let Some(inst) = self.store.remove(&inst_name) {
                    let entity_schema = self.schema.get_entity(&inst.entity_name).unwrap();
                    for (name, val) in inst.iter_fields(&entity_schema.field_names) {
                        if let Value::Text(ref target) = val {
                            if self.schema.is_ref_field(&inst.entity_name, name) {
                                if let Some(refs) = self.reverse_refs.get_mut(target) {
                                    refs.remove(&(inst_name.clone(), inst.entity_name.clone()));
                                }
                            }
                        }
                    }
                    self.reverse_refs.remove(&inst_name);
                } else {
                    return Err(RuntimeError::R001 {
                        instance: inst_name.clone(),
                    });
                }
                Ok(vec![])
            }
            Action::Alert(alert_action) => {
                let severity = self.eval_expr(&alert_action.severity, ctx)?.to_string();
                let message = self.eval_expr(&alert_action.message, ctx)?.to_string();
                let source = alert_action
                    .source
                    .as_ref()
                    .and_then(|e| self.eval_expr(e, ctx).ok())
                    .map(|v| v.to_string())
                    .unwrap_or_default();

                // Validate severity
                match severity.as_str() {
                    "info" | "warning" | "critical" | "resolved" => {}
                    _ => return Err(RuntimeError::R002),
                }

                // Output for development visibility
                let line = format!("[ALERT:{}] {} -- {}", severity, source, message);
                println!("{}", line);
                self.output.push(line);

                Ok(vec![FiredEvent {
                    rule: ctx.unwrap_or("").to_string(),
                    instance: source,
                    severity,
                    message,
                    ts: self.now,
                }])
            }
            Action::Write { target, value } => {
                let val = self.eval_expr(value, ctx)?;
                let mut inst_name = target.instance.clone();
                if let Some(ctx_inst) = ctx {
                    if let Some(ctx_ent) = self.instances.get(ctx_inst) {
                        if ctx_ent == &inst_name {
                            inst_name = ctx_inst.to_string();
                        }
                    }
                }
                // Issue 003: Resolve rule parameter alias for write targets
                if let Some((ref param_name, _)) = self.rule_param_alias {
                    if &inst_name == param_name {
                        if let Some(ctx_inst) = ctx {
                            inst_name = ctx_inst.to_string();
                        }
                    }
                }
                // Issue 006: Handle nested field paths
                let actual_field = if let Some(ref sub_field) = target.sub_field {
                    if let Some(inst) = self.store.get(&inst_name) {
                        if let Some(Value::Text(ref_target)) =
                            self.get_instance_field(inst, &target.field)
                        {
                            inst_name = ref_target.clone();
                            sub_field.clone()
                        } else {
                            return Err(RuntimeError::R005 {
                                instance: inst_name,
                                field: target.field.clone(),
                            });
                        }
                    } else {
                        return Err(RuntimeError::R001 {
                            instance: inst_name,
                        });
                    }
                } else {
                    target.field.clone()
                };
                // Dispatch to adapter if one is registered for this entity
                let entity_name = self
                    .instances
                    .get(&inst_name)
                    .cloned()
                    .unwrap_or_else(|| inst_name.clone());
                let mut dispatched = false;
                for adapter in &mut self.adapters {
                    if adapter.entity_name() == entity_name {
                        adapter.on_write(&actual_field, &val);
                        dispatched = true;
                        break;
                    }
                }
                // Also update the local store so the state is consistent
                if self.store.get(&inst_name).is_some() {
                    self.apply_update(&inst_name, &actual_field, val)
                } else if dispatched {
                    Ok(vec![])
                } else {
                    self.apply_update(&inst_name, &actual_field, val)
                }
            }
        }
    }

    // ── Core update + propagation ─────────────────────────

    pub fn apply_update(
        &mut self,
        instance_name: &str,
        field_name: &str,
        new_value: Value,
    ) -> Result<Vec<FiredEvent>, RuntimeError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(RuntimeError::R003 { depth: self.depth });
        }

        // Capture pre-update state for `prev()` expressions
        if self.depth == 1 {
            self.prev_store = Some(self.store.clone());
        }

        let snap = self.snapshots.take(&self.store);
        self.snapshots.push(snap);

        let entity_name = self
            .store
            .get(instance_name)
            .ok_or(RuntimeError::R001 {
                instance: instance_name.to_string(),
            })?
            .entity_name
            .clone();
        // Check if field is derived (cannot be manually updated)
        if self
            .derived_exprs
            .contains_key(&(entity_name.clone(), field_name.to_string()))
        {
            self.snapshots.pop();
            self.depth -= 1;
            return Err(RuntimeError::R009 {
                field: field_name.to_string(),
            });
        }

        // Check @range
        if let Value::Number(n) = &new_value {
            if let Some(fs) = self.schema.get_field(&entity_name, field_name) {
                if let Some((min, max)) = fs.metadata.range {
                    if *n < min || *n > max {
                        self.snapshots.pop();
                        self.depth -= 1;
                        return Err(RuntimeError::R006 {
                            field: field_name.into(),
                            value: *n,
                            min,
                            max,
                        });
                    }
                }
            }
        }

        // Capture old Boolean value for fleet tracking
        let old_bool = self
            .store
            .get(instance_name)
            .and_then(|inst| self.get_instance_field(inst, field_name))
            .and_then(|v| {
                if let Value::Bool(b) = v {
                    Some(b)
                } else {
                    None
                }
            });

        // Capture old string value for RefField tracking
        let old_text = self
            .store
            .get(instance_name)
            .and_then(|inst| self.get_instance_field(inst, field_name))
            .and_then(|v| {
                if let Value::Text(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            });

        // Apply
        self.set_instance_field(instance_name, field_name, new_value.clone())?;
        self.dirty_instances.insert(instance_name.to_string());

        // Push update to cluster if connected
        if let Some(ref node_lock) = self.cluster_node {
            if let Ok(node) = node_lock.lock() {
                if let Ok(bytes) = serde_json::to_vec(&new_value) {
                    node.state_mesh
                        .update_local(&node.config.node_id, field_name, bytes);
                }
            }
        }

        // Update reverse_refs if this is a RefField
        if self.schema.is_ref_field(&entity_name, field_name) {
            if let Some(old_target) = old_text {
                if let Some(refs) = self.reverse_refs.get_mut(&old_target) {
                    refs.remove(&(instance_name.to_string(), entity_name.clone()));
                }
            }
            if let Value::Text(new_target) = &new_value {
                self.reverse_refs
                    .entry(new_target.clone())
                    .or_default()
                    .insert((instance_name.to_string(), entity_name.clone()));
            }
        }

        // Update fleet state for Boolean fields
        if let Value::Bool(new_b) = &new_value {
            let total = self.store.all_of_entity(&entity_name).count();
            self.fleet_state.update(
                &entity_name,
                field_name,
                old_bool.unwrap_or(false),
                *new_b,
                total,
            );
        }

        // Write-back to external entity adapters
        for a in &mut self.adapters {
            if a.entity_name() == entity_name {
                a.on_write(field_name, &new_value);
            }
        }

        // Propagate derived fields
        if let Err(e) = self.propagate_derived(instance_name, &entity_name) {
            if let Some(snap) = self.snapshots.pop() {
                self.store = snap.store;
            }
            self.depth -= 1;
            return Err(e);
        }

        // Cross-instance ref propagation: find all instances that reference
        // the updated instance via a `ref` field, and re-propagate their deriveds.
        let referencing_instances: Vec<(String, String)> = self
            .reverse_refs
            .get(instance_name)
            .map(|refs| refs.iter().cloned().collect())
            .unwrap_or_default();

        for (ref_inst_name, ref_entity_name) in &referencing_instances {

            if let Err(e) = self.propagate_derived(&ref_inst_name, &ref_entity_name) {
                if let Some(snap) = self.snapshots.pop() {
                    self.store = snap.store;
                }
                self.depth -= 1;
                return Err(e);
            }
        }


        self.agg_store
            .recompute(&self.store, &self.schema, Some(&self.cluster_state));

        // Evaluate rules for the directly updated instance
        let mut all_events = self.evaluate_rules(instance_name)?;

        // Issue 005: Also evaluate rules for referencing instances whose
        // derived fields may have changed due to the cross-instance propagation
        for (ref_inst_name, _) in &referencing_instances {
            if self.store.get(ref_inst_name).is_some() {
                let ref_events = self.evaluate_rules(ref_inst_name)?;
                all_events.extend(ref_events);
            }
        }

        // Only commit at outermost level to prevent re-triggering becomes
        if self.depth == 1 {
            self.store.commit_dirty(&self.dirty_instances);
            self.dirty_instances.clear();
            self.fired_this_cycle.clear();
        }
        self.snapshots.pop();
        self.depth -= 1;
        Ok(all_events)
    }

    fn evaluate_rules(&mut self, instance_name: &str) -> Result<Vec<FiredEvent>, RuntimeError> {
        let mut all_events = Vec::new();
        let rules_clone = self.rules.clone();
        for rule in &rules_clone {
            // FIX Issue 6: Check if instance was deleted by a previous rule in this cycle
            if self.store.get(instance_name).is_none() {
                break; // Instance is gone, stop evaluating rules for it
            }

            // Issue 003: Set parameter alias for this rule so eval_expr can resolve param names
            self.rule_param_alias = rule
                .param
                .as_ref()
                .map(|p| (p.name.clone(), p.entity.clone()));

            match &rule.trigger {
                RuleTrigger::When(conditions) => {
                    let active_key = (rule.name.clone(), instance_name.to_string());
                    // All conditions in the compound trigger must be met.
                    // For compound triggers, only enforce the transition check
                    // on the condition(s) that reference the field being updated.
                    // Other conditions just need to be currently true.
                    let is_compound = conditions.len() > 1;
                    let has_becomes = conditions.iter().any(|c| c.becomes.is_some());
                    let all_met = if is_compound {
                        // For compound triggers: each condition must currently match,
                        // but we don't require ALL of them to have just transitioned.
                        // The rising edge detection on prev_rule_conditions handles
                        // whether the compound state as a whole just became true.
                        conditions.iter().all(|c| {
                            rules::condition_is_met(self, c, instance_name, false).unwrap_or(false)
                        })
                    } else {
                        conditions.iter().all(|c| {
                            rules::condition_is_met(self, c, instance_name, has_becomes)
                                .unwrap_or(false)
                        })
                    };

                    // E2 Fix: Rising Edge Detection for triggers.
                    // Only fire when condition transitions false→true.
                    let edge_key = (rule.name.clone(), instance_name.to_string());
                    let prev_met = self
                        .prev_rule_conditions
                        .get(&edge_key)
                        .copied()
                        .unwrap_or(false);
                    let is_rising_edge = all_met && !prev_met;
                    self.prev_rule_conditions.insert(edge_key, all_met);

                    match all_met {
                        true if is_rising_edge => {
                            let fire_key = format!("{}::{}", rule.name, instance_name);
                            if self.fired_this_cycle.contains(&fire_key) {
                                // Mark as active even if we skip firing
                                self.rule_active.insert(active_key.clone(), true);
                                continue;
                            }

                            // Evaluate sliding window for frequency triggers
                            let freq = conditions.first().and_then(|c| c.frequency.as_ref());
                            if let Some(f) = freq {
                                let history =
                                    self.frequency_events.entry(active_key.clone()).or_default();
                                history.push(self.now);

                                // Retain timestamps within the window
                                let cutoff = self.now - f.within.to_seconds();
                                history.retain(|&ts| ts >= cutoff);

                                if history.len() < f.count as usize {
                                    self.rule_active.insert(active_key.clone(), true);
                                    continue;
                                }

                                // Reset the sliding window after firing
                                history.clear();
                            }

                            // Use the for_duration from the first condition if present
                            let for_duration =
                                conditions.first().and_then(|c| c.for_duration.as_ref());
                            if let Some(dur) = for_duration {
                                let _ = self.timers.start_for_timer(
                                    &rule.name,
                                    instance_name,
                                    dur.to_seconds(),
                                );
                            } else {
                                if let Some(cd) = &rule.cooldown {
                                    if !self.should_fire(&rule.name, instance_name, cd) {
                                        self.rule_active.insert(active_key, true);
                                        continue;
                                    }
                                }
                                self.record_firing(&rule.name, instance_name);

                                self.fired_this_cycle.insert(fire_key);
                                for action in &rule.actions {
                                    let evts = self.exec_action(action, Some(instance_name))?;
                                    all_events.extend(evts);
                                }
                                all_events.push(FiredEvent {
                                    rule: rule.name.clone(),
                                    instance: instance_name.to_string(),
                                    severity: "info".to_string(),
                                    message: format!("Rule '{}' fired", rule.name),
                                    ts: self.now,
                                });
                            }
                            self.rule_active.insert(active_key, true);
                        }
                        true => {
                            // Condition is true but not a rising edge — don't fire
                            self.rule_active.insert(active_key, true);
                        }
                        false => {
                            self.timers.cancel_for_timer(&rule.name, instance_name);
                            // on_clear: if rule was previously active, fire on_clear actions
                            let was_active =
                                self.rule_active.get(&active_key).copied().unwrap_or(false);
                            if was_active {
                                self.rule_active.insert(active_key, false);
                                if let Some(clear_actions) = &rule.on_clear {
                                    let clear_actions = clear_actions.clone();
                                    for action in &clear_actions {
                                        let evts = self.exec_action(action, Some(instance_name))?;
                                        all_events.extend(evts);
                                    }
                                    all_events.push(FiredEvent {
                                        rule: format!("{}_clear", rule.name),
                                        instance: instance_name.to_string(),
                                        severity: "resolved".to_string(),
                                        message: format!("Rule '{}' cleared", rule.name),
                                        ts: self.now,
                                    });
                                }
                            }
                        }
                    }
                }
                RuleTrigger::Any(fc) => {
                    let key = (fc.entity.clone(), fc.field.clone());
                    let target = matches!(&fc.becomes, Expr::Bool(true));
                    let now_met = if target {
                        self.fleet_state.any_true(&fc.entity, &fc.field)
                    } else {
                        !self.fleet_state.all_true(&fc.entity, &fc.field)
                    };
                    let prev = self.prev_fleet_any.get(&key).copied().unwrap_or(false);
                    let active_key = (rule.name.clone(), "fleet".to_string());
                    let fire_key = format!("{}::fleet_any", rule.name);

                    // Issue 002: Resolve the triggering instance for fleet context
                    let fleet_ctx: Option<&str> = if self
                        .instances
                        .get(instance_name)
                        .map(|e| e == &fc.entity)
                        .unwrap_or(false)
                    {
                        Some(instance_name)
                    } else {
                        None
                    };

                    // Edge detection: fire only on rising edge (or start timer)
                    if now_met {
                        if !prev {
                            if let Some(dur) = &fc.for_duration {
                                let _ = self.timers.start_for_timer(
                                    &rule.name,
                                    "fleet",
                                    dur.to_seconds(),
                                );
                            } else {
                                if !self.fired_this_cycle.contains(&fire_key) {
                                    self.fired_this_cycle.insert(fire_key);
                                    for action in &rule.actions {
                                        let evts = self.exec_action(action, fleet_ctx)?;
                                        all_events.extend(evts);
                                    }
                                    all_events.push(FiredEvent {
                                        rule: rule.name.clone(),
                                        instance: "fleet".to_string(),
                                        severity: "info".to_string(),
                                        message: format!(
                                            "Fleet any trigger fired for '{}'",
                                            rule.name
                                        ),
                                        ts: self.now,
                                    });
                                }
                            }
                        }
                        self.rule_active.insert(active_key, true);
                    } else {
                        self.timers.cancel_for_timer(&rule.name, "fleet");
                        let was_active =
                            self.rule_active.get(&active_key).copied().unwrap_or(false);
                        if was_active {
                            self.rule_active.insert(active_key, false);
                            if let Some(clear_actions) = &rule.on_clear {
                                for action in clear_actions {
                                    let evts = self.exec_action(action, None)?;
                                    all_events.extend(evts);
                                }
                                all_events.push(FiredEvent {
                                    rule: format!("{}_clear", rule.name),
                                    instance: "fleet".to_string(),
                                    severity: "resolved".to_string(),
                                    message: format!(
                                        "Fleet any trigger cleared for '{}'",
                                        rule.name
                                    ),
                                    ts: self.now,
                                });
                            }
                        }
                    }
                    self.prev_fleet_any.insert(key, now_met);
                }
                RuleTrigger::All(fc) => {
                    let key = (fc.entity.clone(), fc.field.clone());
                    let target = matches!(&fc.becomes, Expr::Bool(true));
                    let now_met = if target {
                        self.fleet_state.all_true(&fc.entity, &fc.field)
                    } else {
                        !self.fleet_state.any_true(&fc.entity, &fc.field)
                    };
                    let prev = self.prev_fleet_all.get(&key).copied().unwrap_or(false);
                    let active_key = (rule.name.clone(), "fleet".to_string());
                    let fire_key = format!("{}::fleet_all", rule.name);

                    // Issue 002: Resolve the triggering instance for fleet context
                    let fleet_ctx: Option<&str> = if self
                        .instances
                        .get(instance_name)
                        .map(|e| e == &fc.entity)
                        .unwrap_or(false)
                    {
                        Some(instance_name)
                    } else {
                        None
                    };

                    // Edge detection: fire only on rising edge (or start timer)
                    if now_met {
                        if !prev {
                            if let Some(dur) = &fc.for_duration {
                                let _ = self.timers.start_for_timer(
                                    &rule.name,
                                    "fleet",
                                    dur.to_seconds(),
                                );
                            } else {
                                if !self.fired_this_cycle.contains(&fire_key) {
                                    self.fired_this_cycle.insert(fire_key);
                                    for action in &rule.actions {
                                        let evts = self.exec_action(action, fleet_ctx)?;
                                        all_events.extend(evts);
                                    }
                                    all_events.push(FiredEvent {
                                        rule: rule.name.clone(),
                                        instance: "fleet".to_string(),
                                        severity: "info".to_string(),
                                        message: format!(
                                            "Fleet all trigger fired for '{}'",
                                            rule.name
                                        ),
                                        ts: self.now,
                                    });
                                }
                            }
                        }
                        self.rule_active.insert(active_key, true);
                    } else {
                        self.timers.cancel_for_timer(&rule.name, "fleet");
                        let was_active =
                            self.rule_active.get(&active_key).copied().unwrap_or(false);
                        if was_active {
                            self.rule_active.insert(active_key, false);
                            if let Some(clear_actions) = &rule.on_clear {
                                for action in clear_actions {
                                    let evts = self.exec_action(action, None)?;
                                    all_events.extend(evts);
                                }
                                all_events.push(FiredEvent {
                                    rule: format!("{}_clear", rule.name),
                                    instance: "fleet".to_string(),
                                    severity: "resolved".to_string(),
                                    message: format!(
                                        "Fleet all trigger cleared for '{}'",
                                        rule.name
                                    ),
                                    ts: self.now,
                                });
                            }
                        }
                    }
                    self.prev_fleet_all.insert(key, now_met);
                }
                RuleTrigger::Every(_) => {} // handled in tick()
            }
        }
        // Issue 003: Clear parameter alias after processing all rules
        self.rule_param_alias = None;
        Ok(all_events)
    }

    /// Run a full sweep of all rules across all instances.
    /// Typically used after initialization to establish first stable state.
    pub fn recalculate_all_rules(&mut self) -> Result<Vec<FiredEvent>, RuntimeError> {
        let mut all_events = Vec::new();

        // Single batch recomputation of aggregates after initialization
        self.agg_store
            .recompute(&self.store, &self.schema, Some(&self.cluster_state));

        let instance_names: Vec<String> = self.store.all().map(|(n, _)| n.clone()).collect();
        for name in instance_names {
            let evts = self.evaluate_rules(&name)?;
            all_events.extend(evts);
        }
        self.store.commit_all();
        Ok(all_events)
    }

    fn propagate_derived(
        &mut self,
        instance_name: &str,
        entity_name: &str,
    ) -> Result<(), RuntimeError> {
        let mut derived: Vec<(String, String)> = self
            .derived_exprs
            .keys()
            .filter(|(ent, _)| ent == entity_name)
            .cloned()
            .collect();
        derived.sort_by_key(|(e, f)| self.graph.get_node(e, f).unwrap_or(u32::MAX));

        for (ent, field) in derived {
            if let Some(expr) = self
                .derived_exprs
                .get(&(ent.clone(), field.clone()))
                .cloned()
            {
                // Capture old value for fleet tracking
                let old_val = self
                    .store
                    .get(instance_name)
                    .and_then(|inst| self.get_instance_field(inst, &field));

                let val = self.eval_expr(&expr, Some(instance_name))?;

                // Update fleet state if field is Boolean
                if let Value::Bool(new_b) = &val {
                    let old_b = if let Some(Value::Bool(b)) = old_val {
                        b
                    } else {
                        false
                    };
                    let total = self.store.all_of_entity(&ent).count();
                    self.fleet_state.update(&ent, &field, old_b, *new_b, total);
                }

                self.set_instance_field(instance_name, &field, val)?;
                self.dirty_instances.insert(instance_name.to_string());
            }
        }
        Ok(())
    }

    /// Execute all actions of a rule — helper for both immediate and timer-delayed firing
    fn exec_rule_actions(
        &mut self,
        rule: &RuleDecl,
        instance_name: &str,
    ) -> Result<Vec<FiredEvent>, RuntimeError> {
        let ctx = if instance_name.is_empty() {
            None
        } else {
            Some(instance_name)
        };
        let mut events = vec![];
        for action in &rule.actions {
            let evts = self.exec_action(action, ctx)?;
            events.extend(evts);
        }
        events.push(FiredEvent {
            rule: rule.name.clone(),
            instance: instance_name.to_string(),
            severity: "info".to_string(),
            message: format!("Timer callback for '{}'", rule.name),
            ts: self.now,
        });
        Ok(events)
    }

    /// Called periodically by the host — fires any elapsed for/every timers
    pub fn tick(&mut self) -> Result<Vec<FiredEvent>, RollbackResult> {
        self.sync_cluster_state();
        if let Err(e) = self.process_orchestration() {
            return Err(RollbackResult {
                diagnostic: Diagnostic::from_runtime_error(
                    e.code(),
                    &format!("Orchestration failure: {}", e.message()),
                    self.snapshots.current_version(),
                    vec![],
                ),
            });
        }
        let mut all_events = vec![];

        // ── Poll external entity adapters ──────────────────────────
        let updates: Vec<(String, String, String, Value)> = self
            .adapters
            .iter_mut()
            .flat_map(|a| {
                let ent_name = a.entity_name().to_string();
                std::iter::from_fn(move || {
                    a.poll().map(|(inst, f, v)| (ent_name.clone(), inst, f, v))
                })
                .collect::<Vec<_>>()
            })
            .collect();

        for (entity, instance, field, value) in updates {
            let inst_name = if instance == "default" {
                self.store.find_instance_of(&entity)
            } else if self.store.contains(&instance) {
                Some(instance)
            } else {
                None
            };

            if let Some(inst_name) = inst_name {
                // sync_on filtering: only propagate if the field matches sync_on
                // (or if no sync_on is set). Non-sync fields are still stored.
                if let Some(entity_schema) = self.schema.get_entity(&entity) {
                    if let Some(ref sync_fields) = entity_schema.sync_on {
                        if !sync_fields.contains(&field) {
                            // Store the value without triggering propagation
                            let idx = self.get_field_idx(&entity, &field);
                            if let Some(inst) = self.store.get_mut(&inst_name) {
                                if let Some(idx) = idx {
                                    inst.set(idx, value);
                                }
                            }
                            continue;
                        }
                    }
                }
                let _ = self.apply_update(&inst_name, &field, value);
            }
        }

        // ── Fire elapsed `for` timers ──────────────────────────────────
        let elapsed = self.timers.drain_elapsed_for_timers();
        for timer in elapsed {
            let rule = self
                .rules
                .iter()
                .find(|r| r.name == timer.rule_name)
                .cloned();
            if let Some(rule) = rule {
                let snap = self.snapshots.take(&self.store);
                let still_true = match &rule.trigger {
                    RuleTrigger::When(conditions) => conditions.iter().all(|c| {
                        rules::condition_is_met(self, c, &timer.instance_name, false)
                            .unwrap_or(false)
                    }),
                    RuleTrigger::Any(fc) => {
                        let target = matches!(&fc.becomes, Expr::Bool(true));
                        if target {
                            self.fleet_state.any_true(&fc.entity, &fc.field)
                        } else {
                            !self.fleet_state.all_true(&fc.entity, &fc.field)
                        }
                    }
                    RuleTrigger::All(fc) => {
                        let target = matches!(&fc.becomes, Expr::Bool(true));
                        if target {
                            self.fleet_state.all_true(&fc.entity, &fc.field)
                        } else {
                            !self.fleet_state.any_true(&fc.entity, &fc.field)
                        }
                    }
                    RuleTrigger::Every(_) => false,
                };

                if still_true {
                    match self.exec_rule_actions(&rule, &timer.instance_name) {
                        Ok(events) => {
                            self.store.commit_dirty(&self.dirty_instances);
                            self.dirty_instances.clear();
                            all_events.extend(events);
                        }
                        Err(e) => {
                            self.store = snap.store;
                            return Err(RollbackResult {
                                diagnostic: Diagnostic::from_runtime_error(
                                    e.code(),
                                    &e.message(),
                                    self.snapshots.current_version(),
                                    vec![rule.name.clone()],
                                ),
                            });
                        }
                    }
                }
            }
        }

        // ── Fire due `every` timers ────────────────────────────────────
        let due_rules = self.timers.drain_due_every_timers();
        for rule_name in due_rules {
            let rule = self.rules.iter().find(|r| r.name == rule_name).cloned();
            if let Some(rule) = rule {
                let snap = self.snapshots.take(&self.store);
                match self.exec_rule_actions(&rule, "") {
                    Ok(events) => {
                        self.store.commit_dirty(&self.dirty_instances);
                        self.dirty_instances.clear();
                        all_events.extend(events);
                    }
                    Err(e) => {
                        self.store = snap.store;
                        return Err(RollbackResult {
                            diagnostic: Diagnostic::from_runtime_error(
                                e.code(),
                                &e.message(),
                                self.snapshots.current_version(),
                                vec![rule.name.clone()],
                            ),
                        });
                    }
                }
            }
        }

        Ok(all_events)
    }

    // ── Public API ────────────────────────────────────────

    pub fn apply_event(
        &mut self,
        instance_name: &str,
        field_name: &str,
        new_value: Value,
    ) -> Result<PropResult, RollbackResult> {
        match self.apply_update(instance_name, field_name, new_value) {
            Ok(events) => Ok(PropResult {
                success: true,
                events_fired: events,
                version: self.snapshots.current_version(),
            }),
            Err(e) => Err(RollbackResult {
                diagnostic: Diagnostic::from_runtime_error(
                    e.code(),
                    &e.message(),
                    self.snapshots.current_version(),
                    vec![],
                ),
            }),
        }
    }

    pub fn export_state(&self) -> serde_json::Value {
        let mut instances = serde_json::Map::new();
        for (name, instance) in self.store.all() {
            let entity_schema = self.schema.get_entity(&instance.entity_name).unwrap();
            let mut fields = serde_json::Map::new();
            for (fname, val) in instance.iter_fields(&entity_schema.field_names) {
                fields.insert(fname.clone(), self.value_to_json(val));
            }
            instances.insert(name.clone(), serde_json::json!({
                "entity": instance.entity_name,
                "fields": fields,
                "active_alert": self.rule_active.iter().any(|((_, inst), active)| inst == name && *active),
            }));
        }
        serde_json::json!({
            "instances": instances,
            "stable": true,
            "version": self.snapshots.current_version()
        })
    }

    // ── List helpers ──────────────────────────────────────

    fn eval_to_list(&self, expr: &Expr, ctx: Option<&str>) -> Result<Vec<Value>, RuntimeError> {
        match self.eval_expr(expr, ctx)? {
            Value::List(l) => Ok(l),
            v => Err(RuntimeError::R018 {
                op: "list expected".into(),
                left: v.type_name().into(),
                right: "List".into(),
            }),

        }
    }

    fn eval_to_num_list(&self, expr: &Expr, ctx: Option<&str>) -> Result<Vec<f64>, RuntimeError> {
        let list = self.eval_to_list(expr, ctx)?;
        list.into_iter()
            .map(|v| {
                v.as_number().ok_or_else(|| RuntimeError::R018 {
                    op: "number expected".into(),
                    left: v.type_name().into(),
                    right: "Number".into(),
                })

            })
            .collect()
    }

    fn value_to_json(&self, val: &Value) -> serde_json::Value {
        match val {
            Value::Number(n) if n.fract() == 0.0 => serde_json::json!(*n as i64),
            Value::Number(n) => serde_json::json!(*n),
            Value::Text(s) => serde_json::json!(s),
            Value::Bool(b) => serde_json::json!(b),
            Value::List(items) => {
                let arr: Vec<serde_json::Value> =
                    items.iter().map(|v| self.value_to_json(v)).collect();
                serde_json::json!(arr)
            }
            Value::Timestamp(t) => serde_json::json!(*t),
            Value::Duration(d) => serde_json::json!(*d),
            Value::Secret(_) => serde_json::json!("***SECRET***"),
            Value::Unknown => serde_json::Value::Null,
        }
    }

    /// Check if all declared external entities have a corresponding adapter.
    /// Returns a list of warnings for missing adapters.
    pub fn validate_adapters(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        let registered_entities: FxHashSet<&str> =
            self.adapters.iter().map(|a| a.entity_name()).collect();
        for (name, entity) in &self.schema.entities {
            if entity.is_external && !registered_entities.contains(name.as_str()) {
                warnings.push(format!(
                    "External entity '{}' has no registered adapter and will be ignored.",
                    name
                ));
            }
        }
        warnings
    }

    fn process_orchestration(&mut self) -> Result<(), RuntimeError> {
        let (orchestration_events, local_node_id) = if let Some(ref node_lock) = self.cluster_node {
            let mut node = node_lock.lock().unwrap();
            (node.drain_orchestration(), node.config.node_id.clone())
        } else {
            return Ok(());
        };

        for event in orchestration_events {
            match event {
                lumina_cluster::GossipMessageKind::WorkloadMove {
                    target_node,
                    workload,
                } => {
                    if target_node != local_node_id {
                        let mut to_handoff = Vec::new();
                        for name in &workload {
                            if let Some(inst) = self.store.get(name) {
                                let data = serde_json::to_vec(inst).unwrap();
                                to_handoff.push((name.clone(), inst.entity_name.clone(), data));
                            }
                        }

                        if !to_handoff.is_empty() {
                            if let Some(ref node_lock) = self.cluster_node {
                                let node = node_lock.lock().unwrap();
                                node.gossip.broadcast(
                                    local_node_id.clone(),
                                    lumina_cluster::GossipMessageKind::WorkloadHandoff {
                                        target_node: target_node.clone(),
                                        instances: to_handoff.clone(),
                                    },
                                );
                            }
                            for (name, _, _) in to_handoff {
                                self.store.remove(&name);
                                self.instances.remove(&name);
                            }
                        }
                    }
                }
                lumina_cluster::GossipMessageKind::WorkloadHandoff {
                    target_node,
                    instances,
                } => {
                    if target_node == local_node_id {
                        for (name, entity, data) in instances {
                            let inst: Instance =
                                serde_json::from_slice(&data).map_err(|_| RuntimeError::R007 {
                                    entity: entity.clone(),
                                    reason: "Orchestration handoff deserialization failed".into(),
                                })?;
                            self.store.insert(name.clone(), inst);
                            self.instances.insert(name, entity);
                        }
                    }
                }
                lumina_cluster::GossipMessageKind::WorkloadDeploy {
                    target_node,
                    instances,
                    ..
                } => {
                    if target_node == local_node_id {
                        for (name, entity, data) in instances {
                            let inst: Instance =
                                serde_json::from_slice(&data).map_err(|_| RuntimeError::R007 {
                                    entity: entity.clone(),
                                    reason: "Orchestration deploy deserialization failed".into(),
                                })?;
                            self.store.insert(name.clone(), inst);
                            self.instances.insert(name, entity);
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

// ── Default ────────────────────────────────────────────────────────────────

impl Default for Evaluator {
    fn default() -> Self {
        Self {
            schema: Schema::new(),
            graph: DependencyGraph::new(),
            rules: Vec::new(),
            store: EntityStore::new(),
            snapshots: SnapshotStack::new(),
            env: FxHashMap::default(),
            instances: FxHashMap::default(),
            derived_exprs: FxHashMap::default(),
            functions: FxHashMap::default(),
            timers: TimerHeap::new(),
            adapters: Vec::new(),
            prev_store: None,
            fleet_state: FleetState::new(),
            prev_fleet_any: FxHashMap::default(),
            prev_fleet_all: FxHashMap::default(),
            depth: 0,
            fired_this_cycle: FxHashSet::default(),
            output: Vec::new(),
            prev_rule_conditions: FxHashMap::default(),
            cooldown_map: FxHashMap::default(),
            rule_active: FxHashMap::default(),
            frequency_events: FxHashMap::default(),
            agg_store: AggregateStore::new(),
            cluster_state: FxHashMap::default(),
            cluster_config: None,
            now: 0.0,
            rule_param_alias: None,
            is_initializing: false,
            reverse_refs: FxHashMap::default(),
            dirty_instances: FxHashSet::default(),
            cluster_node: None,
        }
    }
}


// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use lumina_analyzer::graph::DependencyGraph;
    use lumina_analyzer::types::Schema;
    use lumina_lexer::token::Span;

    fn empty_eval() -> Evaluator {
        Evaluator::new(Schema::new(), DependencyGraph::new(), vec![])
    }

    fn build_eval(source: &str) -> Evaluator {
        let program = lumina_parser::parse(source).expect("parse failed");
        let analyzed = lumina_analyzer::analyze(program, source, "<runtime-test>", true)
            .expect("analysis failed");
        let mut rules = Vec::new();
        let mut derived = FxHashMap::default();
        for stmt in &analyzed.program.statements {
            match stmt {
                Statement::Rule(r) => rules.push(r.clone()),
                Statement::Entity(e) => {
                    for f in &e.fields {
                        if let Field::Derived(df) = f {
                            derived.insert((e.name.clone(), df.name.clone()), df.expr.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        let mut ev = Evaluator::new(analyzed.schema, analyzed.graph, rules);
        ev.derived_exprs = derived;
        ev.functions = analyzed.fn_defs.into_iter().collect();
        ev
    }

    fn insert_instance(ev: &mut Evaluator, name: &str, entity: &str, fields: Vec<(&str, Value)>) {
        let schema = ev.schema.get_entity(entity).expect("entity not in schema");
        let mut inst = Instance::new(entity, schema.field_names.len());
        for (f, v) in fields {
            let idx = schema.field_indices.get(f).expect("field not in schema");
            inst.set(*idx, v);
        }
        ev.store.insert(name.to_string(), inst);
        ev.instances.insert(name.to_string(), entity.to_string());
    }

    #[test]
    fn test_function_evaluation() {
        let source = "
            fn double(x: Number) -> Number { x * 2 }
            entity Math { val: Number res := double(val) }
        ";
        let mut ev = build_eval(source);
        insert_instance(&mut ev, "m1", "Math", vec![("val", Value::Number(10.0))]);
        ev.propagate_derived("m1", "Math").unwrap();
        let inst = ev.store.get("m1").unwrap();
        assert_eq!(
            ev.get_instance_field(inst, "res").unwrap(),
            Value::Number(20.0)
        );
    }

    #[test]
    fn test_arithmetic() {
        let ev = empty_eval();
        let expr = Expr::Binary {
            op: BinOp::Mul,
            left: Box::new(Expr::Binary {
                op: BinOp::Add,
                left: Box::new(Expr::Number(2.0)),
                right: Box::new(Expr::Number(3.0)),
                span: Span::default(),
            }),
            right: Box::new(Expr::Number(4.0)),
            span: Span::default(),
        };
        assert_eq!(ev.eval_expr(&expr, None).unwrap(), Value::Number(20.0));
    }

    #[test]
    fn test_if_then_else() {
        let ev = empty_eval();
        let expr = Expr::If {
            cond: Box::new(Expr::Bool(true)),
            then_: Box::new(Expr::Number(1.0)),
            else_: Box::new(Expr::Number(2.0)),
            span: Span::default(),
        };
        assert_eq!(ev.eval_expr(&expr, None).unwrap(), Value::Number(1.0));
    }

    #[test]
    fn test_interpolation() {
        let mut ev = empty_eval();
        ev.env.insert("name".into(), Value::Text("Isaac".into()));
        ev.env.insert("age".into(), Value::Number(26.0));
        let expr = Expr::InterpolatedString(vec![
            StringSegment::Literal("Hello ".into()),
            StringSegment::Expr(Box::new(Expr::Ident("name".into()))),
            StringSegment::Literal(", you are ".into()),
            StringSegment::Expr(Box::new(Expr::Ident("age".into()))),
            StringSegment::Literal(" years old".into()),
        ]);
        assert_eq!(
            ev.eval_expr(&expr, None).unwrap(),
            Value::Text("Hello Isaac, you are 26 years old".into())
        );
    }

    #[test]
    fn test_derived_recomputes() {
        let mut ev = build_eval("entity Person {\n  age: Number\n  isAdult := age >= 18\n}");
        insert_instance(
            &mut ev,
            "p1",
            "Person",
            vec![
                ("age", Value::Number(17.0)),
                ("isAdult", Value::Bool(false)),
            ],
        );

        ev.apply_update("p1", "age", Value::Number(18.0)).unwrap();
        let inst = ev.store.get("p1").unwrap();
        assert_eq!(
            ev.get_instance_field(inst, "isAdult").unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_rule_fires_on_becomes() {
        let src = "entity S {\n  active: Boolean\n}\nrule activate when S.active becomes true {\n  show \"fired\"\n}";
        let mut ev = build_eval(src);
        insert_instance(&mut ev, "S", "S", vec![("active", Value::Bool(false))]);

        let events = ev.apply_update("S", "active", Value::Bool(true)).unwrap();
        assert!(events.iter().any(|e| e.rule == "activate"));
    }

    #[test]
    fn test_rollback_on_div_zero() {
        let mut ev = build_eval("entity A {\n  x: Number\n  y: Number\n  ratio := x / y\n}");
        insert_instance(
            &mut ev,
            "a1",
            "A",
            vec![
                ("x", Value::Number(10.0)),
                ("y", Value::Number(2.0)),
                ("ratio", Value::Number(5.0)),
            ],
        );

        let result = ev.apply_update("a1", "y", Value::Number(0.0));
        assert!(result.is_err());
        // Store should be rolled back
        let inst = ev.store.get("a1").unwrap();
        assert_eq!(
            ev.get_instance_field(inst, "y").unwrap(),
            Value::Number(2.0)
        );
    }

    #[test]
    fn test_export_state() {
        let source = "entity Person { name: Text age: Number }";
        let mut ev = build_eval(source);
        insert_instance(
            &mut ev,
            "isaac",
            "Person",
            vec![
                ("name", Value::Text("Isaac".into())),
                ("age", Value::Number(26.0)),
            ],
        );

        let state = ev.export_state();
        assert!(state["instances"]["isaac"]["entity"] == "Person");
        assert!(state["instances"]["isaac"]["fields"]["name"] == "Isaac");
        assert!(state["instances"]["isaac"]["fields"]["age"] == 26);
        assert!(state["stable"] == true);
    }

    #[test]
    fn test_rule_does_not_fire_without_transition() {
        let src = "entity S {\n  active: Boolean\n}\nrule activate when S.active becomes true {\n  show \"fired\"\n}";
        let mut ev = build_eval(src);
        insert_instance(&mut ev, "S", "S", vec![("active", Value::Bool(true))]);
        // Commit so prev_fields = fields (active=true already)
        ev.store.commit_all();

        let events = ev.apply_update("S", "active", Value::Bool(true)).unwrap();
        assert!(events.iter().all(|e| e.rule != "activate"));
    }

    #[test]
    fn test_adapter_poll_triggers_rule() {
        let src = "entity Sensor {\n  reading: Number\n  isCritical := reading > 90\n}\nrule overheat when Sensor.isCritical becomes true {\n  show \"overheating\"\n}";
        let mut ev = build_eval(src);
        insert_instance(
            &mut ev,
            "Sensor",
            "Sensor",
            vec![
                ("reading", Value::Number(50.0)),
                ("isCritical", Value::Bool(false)),
            ],
        );

        let mut adapter = crate::adapters::static_adapter::StaticAdapter::new("Sensor");
        adapter.push("Sensor", "reading", Value::Number(95.0));
        ev.register_adapter(Box::new(adapter));

        let result = ev.tick();
        assert!(result.is_ok());
        let inst = ev.store.get("Sensor").unwrap();
        assert_eq!(
            ev.get_instance_field(inst, "reading").unwrap(),
            Value::Number(95.0)
        );
        assert_eq!(
            ev.get_instance_field(inst, "isCritical").unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_unregistered_entity_ignored() {
        // Guide §28.5 Step 9: entities without a registered adapter are silently ignored
        let src = "entity Sensor {\n  reading: Number\n}";
        let mut ev = build_eval(src);

        // Register adapter for an entity that has no instance in the store
        let mut adapter = crate::adapters::static_adapter::StaticAdapter::new("UnknownEntity");
        adapter.push("default", "value", Value::Number(42.0));
        ev.register_adapter(Box::new(adapter));

        // tick() should not panic or error
        let result = ev.tick();
        assert!(result.is_ok());
    }

    #[test]
    fn test_prev_value_access() {
        let src = r#"
entity Battery {
  level: Number
  drop := prev(level) - level
}
        "#;
        let mut ev = build_eval(src);
        insert_instance(
            &mut ev,
            "batt1",
            "Battery",
            vec![
                ("level", Value::Number(100.0)),
                ("drop", Value::Number(0.0)),
            ],
        );
        ev.store.commit_all();

        ev.apply_update("batt1", "level", Value::Number(90.0))
            .unwrap();
        let inst = ev.store.get("batt1").unwrap();
        assert_eq!(
            ev.get_instance_field(inst, "drop").unwrap(),
            Value::Number(10.0)
        );
    }

    #[test]
    fn test_cascading_cleanup_issue_6() {
        let source = r#"
            entity Resource { status: Text }
            rule cleanup when Resource.status == "deleted" becomes true {
                delete Resource
            }
            rule log_deleted when Resource.status == "deleted" becomes true {
                update Resource.status to "forgotten"
            }
        "#;
        let mut ev = build_eval(source);
        insert_instance(
            &mut ev,
            "res1",
            "Resource",
            vec![("status", Value::Text("active".to_string()))],
        );

        let res = ev.apply_update("res1", "status", Value::Text("deleted".to_string()));
        res.unwrap();
        assert!(ev.store.get("res1").is_none());
    }
}
