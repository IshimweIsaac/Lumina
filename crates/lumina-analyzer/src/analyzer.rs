use crate::graph::DependencyGraph;
use crate::types::{EntitySchema, FieldSchema, Schema};
use lumina_lexer::token::Span;
use lumina_parser::ast::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct AnalyzerError {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

/// The output of a successful analysis pass
#[derive(Debug)]
pub struct AnalyzedProgram {
    pub program: Program,
    pub schema: Schema,
    pub graph: DependencyGraph,
    pub fn_defs: HashMap<String, FnDecl>,
    pub instances: HashMap<String, LuminaType>,
}

pub struct Analyzer {
    schema: Schema,
    graph: DependencyGraph,
    pub errors: Vec<AnalyzerError>,
    pub allow_imports: bool,
    locals: HashMap<String, LuminaType>,
    pub fn_defs: HashMap<String, FnDecl>,
    pub instances: HashMap<String, LuminaType>,
    in_prev_context: bool,
    in_derived_context: bool,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            schema: Schema::new(),
            graph: DependencyGraph::new(),
            errors: Vec::new(),
            allow_imports: true,
            locals: HashMap::new(),
            fn_defs: HashMap::new(),
            instances: HashMap::new(),
            in_prev_context: false,
            in_derived_context: false,
        }
    }

    pub fn analyze(mut self, program: Program) -> Result<AnalyzedProgram, Vec<AnalyzerError>> {
        self.pass1_register_entities(&program);
        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        self.pass2_typecheck(&program)?;

        Ok(AnalyzedProgram {
            program,
            schema: self.schema,
            graph: self.graph,
            fn_defs: self.fn_defs,
            instances: self.instances,
        })
    }

    fn pass1_register_entities(&mut self, program: &Program) {
        for stmt in &program.statements {
            match stmt {
                Statement::Entity(decl) => self.register_entity(decl, false),
                Statement::ExternalEntity(decl) => self.register_external_entity(decl),
                Statement::Fn(decl) => {
                    if self.fn_defs.contains_key(&decl.name) {
                        self.errors.push(AnalyzerError {
                            code: "L011",
                            message: format!(
                                "I've already seen a function named '{}()'. Every function in Lumina must have a unique name so the engine can build a distinct calculation node. Please use a different name for this definition.",
                                decl.name
                            ),
                            span: decl.span,
                        });
                    } else {
                        self.fn_defs.insert(decl.name.clone(), decl.clone());
                    }
                }
                Statement::Import(decl) => {
                    if let Some(ref ns) = decl.namespace {
                        // v1.9: LSL namespace import — validate the path statically
                        let known_namespaces = [
                            "LSL::datacenter::Server",
                            "LSL::datacenter::Rack",
                            "LSL::datacenter::PDU",
                            "LSL::datacenter::CRAC",
                            "LSL::network::Switch",
                            "LSL::network::Router",
                            "LSL::network::Firewall",
                            "LSL::k8s::Pod",
                            "LSL::k8s::Node",
                            "LSL::k8s::Deployment",
                            "LSL::power::UPS",
                            "LSL::power::Generator",
                        ];
                        if !known_namespaces.contains(&decl.path.as_str()) {
                            self.errors.push(AnalyzerError {
                                code: "L054",
                                message: format!(
                                    "Unknown LSL schema '{}'. Available: datacenter, network, k8s, power.",
                                    decl.path
                                ),
                                span: decl.span,
                            });
                        }
                    } else if !self.allow_imports {
                        self.errors.push(AnalyzerError {
                            code: "L018",
                            message: "import is not supported in single-file (WASM) mode"
                                .to_string(),
                            span: decl.span,
                        });
                    }
                }
                Statement::Let(decl) => match &decl.value {
                    LetValue::EntityInit(init) => {
                        self.instances.insert(
                            decl.name.clone(),
                            LuminaType::Entity(init.entity_name.clone()),
                        );
                    }
                    LetValue::Expr(expr) => {
                        if let Ok(ty) = self.infer_type(expr, None, None) {
                            self.instances.insert(decl.name.clone(), ty);
                        }
                    }
                },
                Statement::Aggregate(decl) => {
                    // E3 Fix: Register aggregate name in the global scope as a
                    // numeric provider so that references like `fleet_stats.avg_temp`
                    // resolve correctly during type checking.
                    self.instances
                        .insert(decl.name.clone(), LuminaType::Entity(decl.name.clone()));

                    // v2.0 Fix: Register aggregate fields into the schema as a
                    // virtual entity so that FieldAccess (e.g. ClusterAvg.avg_cpu)
                    // resolves correctly instead of panicking on unwrap().
                    for agg_field in &decl.fields {
                        let field_ty = match &agg_field.expr {
                            AggregateExpr::Avg(_)
                            | AggregateExpr::Min(_)
                            | AggregateExpr::Max(_)
                            | AggregateExpr::Sum(_)
                            | AggregateExpr::Count(_) => LuminaType::Number,
                            AggregateExpr::Any(_) | AggregateExpr::All(_) => LuminaType::Boolean,
                        };
                        self.schema
                            .register_field(&decl.name, &agg_field.name, &field_ty);
                    }
                }
                Statement::PluginImport(decl) => {
                    // v1.8: Register plugin alias as a known name
                    if self.instances.contains_key(&decl.alias) {
                        self.errors.push(AnalyzerError {
                            code: "L052",
                            message: format!(
                                "The plugin alias '{}' conflicts with an existing name. Each plugin alias must be unique.",
                                decl.alias
                            ),
                            span: decl.span,
                        });
                    }
                }
                Statement::Provider(decl) => {
                    // v1.9: Validate provider has required config
                    let has_endpoint = decl.config.iter().any(|e| e.key == "endpoint");
                    if !has_endpoint {
                        self.errors.push(AnalyzerError {
                            code: "L053",
                            message: format!(
                                "Provider '{}' is missing an 'endpoint' configuration. Every provider must specify where to connect.",
                                decl.protocol
                            ),
                            span: decl.span,
                        });
                    }
                }
                Statement::Cluster(decl) => {
                    // v2.0: Validate cluster configuration
                    if decl.node_id.is_empty() {
                        self.errors.push(AnalyzerError {
                            code: "L060",
                            message: "Cluster 'node_id' must be a non-empty string. Each node in the cluster needs a unique identifier.".to_string(),
                            span: decl.span,
                        });
                    }
                    if decl.peers.is_empty() {
                        self.errors.push(AnalyzerError {
                            code: "L061",
                            message: "Cluster 'peers' must contain at least one peer address for the cluster to form a quorum.".to_string(),
                            span: decl.span,
                        });
                    }
                    let total_nodes = decl.peers.len() as u32 + 1;
                    if decl.quorum > total_nodes {
                        self.errors.push(AnalyzerError {
                            code: "L062",
                            message: format!(
                                "Quorum size {} exceeds total nodes ({}). Quorum must be ≤ peers + 1.",
                                decl.quorum, total_nodes
                            ),
                            span: decl.span,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    fn register_entity(&mut self, decl: &EntityDecl, is_external: bool) {
        if self.schema.entities.contains_key(&decl.name) {
            self.errors.push(AnalyzerError {
                code: "L005",
                message: format!("Duplicate entity name: {}", decl.name),
                span: decl.span,
            });
            return;
        }

        let mut fields = HashMap::new();
        for field in &decl.fields {
            let (name, schema_field) = match field {
                Field::Stored(f) => (
                    f.name.clone(),
                    FieldSchema {
                        name: f.name.clone(),
                        ty: f.ty.clone(),
                        is_derived: false,
                        metadata: f.metadata.clone(),
                    },
                ),
                Field::Derived(f) => {
                    (
                        f.name.clone(),
                        FieldSchema {
                            name: f.name.clone(),
                            ty: LuminaType::Number, // Placeholder, resolved in pass 2
                            is_derived: true,
                            metadata: f.metadata.clone(),
                        },
                    )
                }
                Field::Ref(r) => {
                    // L036: Validate ref target entity exists
                    if !self.schema.entities.contains_key(&r.target_entity) {
                        // Defer check — target may not be registered yet in pass1
                        // We'll validate in pass2 instead
                    }
                    (
                        r.name.clone(),
                        FieldSchema {
                            name: r.name.clone(),
                            ty: LuminaType::Entity(r.target_entity.clone()),
                            is_derived: false,
                            metadata: FieldMetadata::default(),
                        },
                    )
                }
            };

            if fields.contains_key(&name) {
                self.errors.push(AnalyzerError {
                    code: "L006",
                    message: format!("Duplicate field name: {}", name),
                    span: decl.span, // Simplified span for field error
                });
            } else {
                fields.insert(name, schema_field);
            }
        }

        let mut field_names: Vec<String> = fields.keys().cloned().collect();
        field_names.sort();
        let field_indices: HashMap<String, usize> = field_names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i))
            .collect();

        self.schema.entities.insert(
            decl.name.clone(),
            EntitySchema {
                name: decl.name.clone(),
                fields,
                field_indices,
                field_names,
                is_external,
                sync_path: String::new(),
                sync_strategy: SyncStrategy::Realtime,
                sync_on: None,
                poll_interval: None,
            },
        );
    }

    fn register_external_entity(&mut self, decl: &ExternalEntityDecl) {
        // Reuse register_entity logic by converting ExternalEntityDecl to EntityDecl structure
        if self.schema.entities.contains_key(&decl.name) {
            self.errors.push(AnalyzerError {
                code: "L005",
                message: format!("Duplicate entity name: {}", decl.name),
                span: decl.span,
            });
            return;
        }

        let mut fields = HashMap::new();
        for field in &decl.fields {
            let (name, schema_field) = match field {
                Field::Stored(f) => (
                    f.name.clone(),
                    FieldSchema {
                        name: f.name.clone(),
                        ty: f.ty.clone(),
                        is_derived: false,
                        metadata: f.metadata.clone(),
                    },
                ),
                Field::Derived(f) => (
                    f.name.clone(),
                    FieldSchema {
                        name: f.name.clone(),
                        ty: LuminaType::Number,
                        is_derived: true,
                        metadata: f.metadata.clone(),
                    },
                ),
                Field::Ref(r) => (
                    r.name.clone(),
                    FieldSchema {
                        name: r.name.clone(),
                        ty: LuminaType::Entity(r.target_entity.clone()),
                        is_derived: false,
                        metadata: FieldMetadata::default(),
                    },
                ),
            };
            fields.insert(name, schema_field);
        }

        let mut field_names: Vec<String> = fields.keys().cloned().collect();
        field_names.sort();
        let field_indices: HashMap<String, usize> = field_names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i))
            .collect();

        self.schema.entities.insert(
            decl.name.clone(),
            EntitySchema {
                name: decl.name.clone(),
                fields,
                field_indices,
                field_names,
                is_external: true,
                sync_path: decl.sync_path.clone(),
                sync_strategy: decl.sync_strategy.clone(),
                sync_on: if decl.sync_fields.is_empty() {
                    None
                } else {
                    Some(decl.sync_fields.clone())
                },
                poll_interval: decl.poll_interval.clone(),
            },
        );
    }

    fn pass2_typecheck(&mut self, program: &Program) -> Result<(), Vec<AnalyzerError>> {
        // L036: Validate all ref targets exist now that all entities are registered
        for stmt in &program.statements {
            let fields = match stmt {
                Statement::Entity(e) => Some((&e.fields, e.span)),
                Statement::ExternalEntity(e) => Some((&e.fields, e.span)),
                _ => None,
            };
            if let Some((fields, _span)) = fields {
                for field in fields {
                    if let Field::Ref(r) = field {
                        if !self.schema.entities.contains_key(&r.target_entity) {
                            return Err(vec![AnalyzerError {
                                code: "L036",
                                message: format!(
                                    "I can't find the entity '{}' referenced in this 'ref'. Every 'ref' must point to a defined entity so Lumina can bridge the graph between them. Did you forget to define '{}'?",
                                    r.target_entity, r.target_entity
                                ),
                                span: r.span,
                            }]);
                        }
                    }
                }
            }
        }

        for stmt in &program.statements {
            match stmt {
                Statement::Entity(decl) => {
                    self.typecheck_entity_fields(&decl.name, &decl.fields)?;
                }
                Statement::ExternalEntity(decl) => {
                    self.typecheck_entity_fields(&decl.name, &decl.fields)?;
                }
                Statement::Rule(rule) => {
                    let mut rule_locals = HashMap::new();
                    if let Some(param) = &rule.param {
                        if self.schema.get_entity(&param.entity).is_some() {
                            rule_locals.insert(
                                param.name.clone(),
                                LuminaType::Entity(param.entity.clone()),
                            );
                        } else {
                            return Err(vec![AnalyzerError {
                                code: "L026",
                                message: format!(
                                    "I don't recognize the entity type '{}'. You're trying to use it as a parameter in this rule, but I only know about entities that were declared earlier. Did you forget to define '{}'?",
                                    param.entity, param.entity
                                ),
                                span: rule.span,
                            }]);
                        }
                    }
                    let locals_ref = if rule_locals.is_empty() {
                        None
                    } else {
                        Some(&rule_locals)
                    };

                    // Type check condition
                    match &rule.trigger {
                        RuleTrigger::When(conds) => {
                            // L035: Enforce maximum of 3 AND clauses
                            if conds.len() > 3 {
                                return Err(vec![AnalyzerError {
                                    code: "L035",
                                    message: format!(
                                        "multi-condition trigger has {} clauses, max is 3",
                                        conds.len()
                                    ),
                                    span: rule.span,
                                }]);
                            }
                            for cond in conds {
                                let ty = self
                                    .infer_type(&cond.expr, None, locals_ref)
                                    .map_err(|e| vec![e])?;

                                if let Some(becomes_expr) = &cond.becomes {
                                    let b_ty = self
                                        .infer_type(becomes_expr, None, locals_ref)
                                        .map_err(|e| vec![e])?;
                                    if ty != b_ty {
                                        return Err(vec![AnalyzerError {
                                            code: "L002",
                                            message: format!(
                                                "becomes target type mismatch: expected {:?}, got {:?}",
                                                ty, b_ty
                                            ),
                                            span: rule.span,
                                        }]);
                                    }
                                } else if ty != LuminaType::Boolean {
                                    return Err(vec![AnalyzerError {
                                        code: "L002",
                                        message: "when condition must be Boolean if 'becomes' is not used".to_string(),
                                        span: rule.span,
                                    }]);
                                }
                                // L039/L040: Validate frequency conditions
                                if let Some(freq) = &cond.frequency {
                                    if freq.count < 2 {
                                        return Err(vec![AnalyzerError {
                                            code: "L039",
                                            message: format!(
                                                "A frequency condition requires at least 2 occurrences, but you've specified {}. Lumina uses frequency to detect patterns over time (like 'at least twice in 5s'). Change this to 2 or more.",
                                                freq.count
                                            ),
                                            span: freq.span,
                                        }]);
                                    }
                                    if freq.within.to_seconds() <= 0.0 {
                                        return Err(vec![AnalyzerError {
                                            code: "L040",
                                            message: "A frequency 'within' duration must be greater than 0 seconds. Lumina cannot calculate a rate over a zero-length time window. Please provide a positive duration (e.g., 'within 5s').".to_string(),
                                            span: freq.span,
                                        }]);
                                    }
                                }
                            }
                        }
                        RuleTrigger::Any(fc) | RuleTrigger::All(fc) => {
                            // L026: entity must exist
                            if let Some(entity_schema) = self.schema.entities.get(&fc.entity) {
                                // L027: field must exist and be Boolean
                                if let Some(field_schema) = entity_schema.fields.get(&fc.field) {
                                    if field_schema.ty != LuminaType::Boolean {
                                        return Err(vec![AnalyzerError {
                                            code: "L027",
                                            message: format!(
                                                "fleet trigger field '{}.{}' must be Boolean, found {:?}",
                                                fc.entity, fc.field, field_schema.ty
                                            ),
                                            span: rule.span,
                                        }]);
                                    }
                                } else {
                                    return Err(vec![AnalyzerError {
                                        code: "L027",
                                        message: format!(
                                            "unknown field '{}' on entity '{}'",
                                            fc.field, fc.entity
                                        ),
                                        span: rule.span,
                                    }]);
                                }
                            } else {
                                return Err(vec![AnalyzerError {
                                    code: "L026",
                                    message: format!(
                                        "unknown entity '{}' in fleet trigger",
                                        fc.entity
                                    ),
                                    span: rule.span,
                                }]);
                            }
                            // Validate becomes value is Boolean
                            let b_ty = self
                                .infer_type(&fc.becomes, None, locals_ref)
                                .map_err(|e| vec![e])?;
                            if b_ty != LuminaType::Boolean {
                                return Err(vec![AnalyzerError {
                                    code: "L002",
                                    message: "fleet trigger becomes value must be Boolean"
                                        .to_string(),
                                    span: rule.span,
                                }]);
                            }
                            // L039/L040: Validate fleet frequency conditions
                            if let Some(freq) = &fc.frequency {
                                if freq.count < 2 {
                                    return Err(vec![AnalyzerError {
                                        code: "L039",
                                        message: format!(
                                            "frequency count must be >= 2, got {}",
                                            freq.count
                                        ),
                                        span: freq.span,
                                    }]);
                                }
                                if freq.within.to_seconds() <= 0.0 {
                                    return Err(vec![AnalyzerError {
                                        code: "L040",
                                        message: "frequency window duration must be > 0"
                                            .to_string(),
                                        span: freq.span,
                                    }]);
                                }
                            }
                        }
                        RuleTrigger::Every(_) => {}
                    }

                    // Type check actions
                    for action in &rule.actions {
                        self.check_action(action, rule.span, locals_ref)?;
                    }
                }
                Statement::Fn(decl) => {
                    let mut locals = HashMap::new();
                    let mut locals_set = std::collections::HashSet::new();
                    for param in &decl.params {
                        locals.insert(param.name.clone(), param.type_.clone());
                        locals_set.insert(param.name.clone());
                    }
                    self.check_fn_body(&decl.body, &locals_set, decl.span);

                    if let Ok(body_type) = self.infer_type(&decl.body, None, Some(&locals)) {
                        if body_type != decl.returns {
                            self.errors.push(AnalyzerError {
                                code: "L014",
                                message: format!(
                                    "The function's body returns a {:?}, but the signature says it should return a {:?}. Lumina requires functions to strictly follow their return type to ensure the reactive graph remains stable.",
                                    body_type, decl.returns
                                ),
                                span: decl.span,
                            });
                        }
                    }
                }
                Statement::Aggregate(_) => {}
                Statement::PluginImport(_) => {} // Validated in pass1
                Statement::Provider(_) => {}     // Validated in pass1
                Statement::Cluster(_) => {}      // Validated in pass1
                _ => {}
            }
        }

        // Check for cycles
        if let Err(err) = self.graph.compute_topo_order() {
            return Err(vec![AnalyzerError {
                code: "L004",
                message: format!(
                    "I've detected a circular dependency: {}. Lumina is a Directed Acyclic Graph (DAG); fields cannot depend on themselves (directly or indirectly) because the engine would loop forever trying to calculate the final truth.",
                    err.chain.join(" -> ")
                ),
                span: program.span,
            }]);
        }

        if !self.errors.is_empty() {
            Err(std::mem::take(&mut self.errors))
        } else {
            Ok(())
        }
    }

    fn typecheck_entity_fields(
        &mut self,
        entity_name: &str,
        fields: &[Field],
    ) -> Result<(), Vec<AnalyzerError>> {
        for field in fields {
            match field {
                Field::Derived(df) => {
                    let ty = self
                        .infer_type_ctx(&df.expr, Some(entity_name), None, true)
                        .map_err(|e| vec![e])?;
                    // v1.8: L051 — Secret values cannot be used in derived fields
                    if ty == LuminaType::Secret {
                        return Err(vec![AnalyzerError {
                            code: "L051",
                            message: format!(
                                "The derived field '{}' resolves to a Secret type. Derived fields are automatically computed and their values may be exposed through the reactive graph. Move secret handling into a 'write' action instead.",
                                df.name
                            ),
                            span: df.span,
                        }]);
                    }
                    if let Some(entity) = self.schema.entities.get_mut(entity_name) {
                        if let Some(f_schema) = entity.fields.get_mut(&df.name) {
                            f_schema.ty = ty;
                        }
                    }
                    // Build dependency graph for derived fields
                    let target_node = self.graph.intern(entity_name, &df.name);
                    self.collect_dependencies(&df.expr, entity_name, target_node)?;
                }
                Field::Ref(r) => {
                    // L037: Add ref edges to the dependency graph for cycle detection
                    let ref_node = self.graph.intern(entity_name, &r.name);
                    let target_node = self.graph.intern(&r.target_entity, "__entity__");
                    self.graph.add_edge(target_node, ref_node);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn check_fn_body(
        &mut self,
        expr: &Expr,
        locals: &std::collections::HashSet<String>,
        span: Span,
    ) {
        match expr {
            Expr::FieldAccess { obj, .. } => {
                if let Expr::Ident(ref name) = **obj {
                    if !locals.contains(name) {
                        self.errors.push(AnalyzerError {
                            code: "L015",
                            message: "fn body cannot access entity fields".to_string(),
                            span,
                        });
                    }
                } else {
                    self.errors.push(AnalyzerError {
                        code: "L015",
                        message: "Functions are 'pure' logic and cannot directly access entity fields. They can only use the parameters passed to them. Move the field value into an argument to use it here.".to_string(),
                        span,
                    });
                }
                self.check_fn_body(obj, locals, span);
            }
            Expr::Binary { left, right, .. } => {
                self.check_fn_body(left, locals, span);
                self.check_fn_body(right, locals, span);
            }
            Expr::Unary { operand, .. } => {
                self.check_fn_body(operand, locals, span);
            }
            Expr::If {
                cond, then_, else_, ..
            } => {
                self.check_fn_body(cond, locals, span);
                self.check_fn_body(then_, locals, span);
                self.check_fn_body(else_, locals, span);
            }
            Expr::Call { args, .. } => {
                for arg in args {
                    self.check_fn_body(arg, locals, span);
                }
            }
            Expr::InterpolatedString(segments) => {
                for seg in segments {
                    if let StringSegment::Expr(e) = seg {
                        self.check_fn_body(e, locals, span);
                    }
                }
            }
            Expr::ListLiteral(elems) => {
                for elem in elems {
                    self.check_fn_body(elem, locals, span);
                }
            }
            Expr::Index { list, index, .. } => {
                self.check_fn_body(list, locals, span);
                self.check_fn_body(index, locals, span);
            }
            Expr::Prev { .. } => {}
            _ => {}
        }
    }

    /// Wrapper that sets derived context before delegating to infer_type.
    /// Used for L041: now() is forbidden in derived field expressions.
    fn infer_type_ctx(
        &mut self,
        expr: &Expr,
        entity_ctx: Option<&str>,
        locals: Option<&HashMap<String, LuminaType>>,
        is_derived: bool,
    ) -> Result<LuminaType, AnalyzerError> {
        let prev = self.in_derived_context;
        self.in_derived_context = is_derived;
        let result = self.infer_type(expr, entity_ctx, locals);
        self.in_derived_context = prev;
        result
    }

    fn infer_type(
        &self,
        expr: &Expr,
        entity_ctx: Option<&str>,
        locals: Option<&HashMap<String, LuminaType>>,
    ) -> Result<LuminaType, AnalyzerError> {
        match expr {
            Expr::Number(_) => Ok(LuminaType::Number),
            Expr::Text(_) | Expr::InterpolatedString(_) => Ok(LuminaType::Text),
            Expr::Bool(_) => Ok(LuminaType::Boolean),
            Expr::Duration(_) => Ok(LuminaType::Duration),
            Expr::Ident(name) => {
                if let Some(locs) = locals {
                    if let Some(ty) = locs.get(name) {
                        return Ok(ty.clone());
                    }
                }
                if let Some(ty) = self.instances.get(name) {
                    return Ok(ty.clone());
                }
                // First check if it's a field in the current entity context
                if let Some(ent) = entity_ctx {
                    if let Some(f) = self.schema.get_field(ent, name) {
                        return Ok(f.ty.clone());
                    }
                }
                // Then check if it's an entity name
                if self.schema.entities.contains_key(name) {
                    if self.in_derived_context {
                        return Err(AnalyzerError {
                            code: "L001",
                            message: format!(
                                "Cannot use entity type '{}' as a variable in a derived field. Did you mean to reference a specific instance via a reference field?",
                                name
                            ),
                            span: Span::default(), // Will be bubbled up with correct span later if needed
                        });
                    }
                    Ok(LuminaType::Entity(name.clone()))
                } else {
                    Err(AnalyzerError {
                        code: "L001",
                        message: format!(
                            "I don't recognize the identifier '{}'. Lumina needs all names (entities, fields, or functions) to be declared before they are used to ensure the safety of the reactive graph. Have you defined this name elsewhere, or is there a typo?",
                            name
                        ),
                        span: Span::default(),
                    })
                }
            }
            Expr::FieldAccess { obj, field, span } => {
                let obj_ty = self.infer_type(obj, entity_ctx, locals)?;
                match obj_ty {
                    LuminaType::Entity(e_name) => {
                        if let Some(f) = self.schema.get_field(&e_name, field) {
                            Ok(f.ty.clone())
                        } else {
                            let known_fields = self.schema.get_entity(&e_name)
                                .map(|e| e.fields.keys().cloned().collect::<Vec<_>>())
                                .unwrap_or_default();
                            Err(AnalyzerError {
                                code: "L010",
                                message: format!(
                                    "I can't find a field named '.{}' on '{}'. Known fields: {:?}",
                                    field, e_name, known_fields
                                ),
                                span: *span,
                            })
                        }
                    }
                    // L042: .age on Timestamp returns Duration
                    LuminaType::Timestamp => {
                        if field == "age" {
                            Ok(LuminaType::Duration)
                        } else {
                            Err(AnalyzerError {
                                code: "L042",
                                message: format!("Timestamps only support the '.age' accessor (which returns a Duration). I don't recognize the field '.{}'.", field),
                                span: *span,
                            })
                        }
                    }
                    _ => Err(AnalyzerError {
                        code: "L002",
                        message: format!("I can only look up fields on Entities or Timestamps. You're trying to look up '.{}' on a {:?}.", field, obj_ty),
                        span: *span,
                    }),
                }
            }
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => {
                let l_ty = self.infer_type(left, entity_ctx, locals)?;
                let r_ty = self.infer_type(right, entity_ctx, locals)?;
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if l_ty == LuminaType::Number && r_ty == LuminaType::Number {
                            Ok(LuminaType::Number)
                        } else {
                            Err(AnalyzerError {
                                code: "L002",
                                message: format!(
                                    "I can only perform math ('{}') on two Numbers. You've provided a {:?} and a {:?}. Lumina's reactive engine needs consistent math types to guarantee that sensor values doesn't cause a runtime crash.",
                                    op, l_ty, r_ty
                                ),
                                span: *span,
                            })
                        }
                    }
                    BinOp::Eq | BinOp::Ne | BinOp::Gt | BinOp::Lt | BinOp::Ge | BinOp::Le => {
                        if l_ty == r_ty {
                            Ok(LuminaType::Boolean)
                        } else {
                            Err(AnalyzerError {
                                code: "L002",
                                message: format!(
                                    "I can't compare a {:?} with a {:?}. To check for equality or order, both sides must be the same type. This prevents 'comparing apples to oranges' which would break rule logic.",
                                    l_ty, r_ty
                                ),
                                span: *span,
                            })
                        }
                    }
                    BinOp::And | BinOp::Or => {
                        if l_ty == LuminaType::Boolean && r_ty == LuminaType::Boolean {
                            Ok(LuminaType::Boolean)
                        } else {
                            Err(AnalyzerError {
                                code: "L002",
                                message: "Logical operations require Booleans".to_string(),
                                span: *span,
                            })
                        }
                    }
                }
            }
            Expr::Unary { op, operand, span } => {
                let ty = self.infer_type(operand, entity_ctx, locals)?;
                match op {
                    UnOp::Neg => {
                        if ty == LuminaType::Number {
                            Ok(LuminaType::Number)
                        } else {
                            Err(AnalyzerError {
                                code: "L002",
                                message: "Negation requires Number".to_string(),
                                span: *span,
                            })
                        }
                    }
                    UnOp::Not => {
                        if ty == LuminaType::Boolean {
                            Ok(LuminaType::Boolean)
                        } else {
                            Err(AnalyzerError {
                                code: "L002",
                                message: "Logical NOT requires Boolean".to_string(),
                                span: *span,
                            })
                        }
                    }
                }
            }
            Expr::If {
                cond,
                then_,
                else_,
                span,
            } => {
                let c_ty = self.infer_type(cond, entity_ctx, locals)?;
                if c_ty != LuminaType::Boolean {
                    return Err(AnalyzerError {
                        code: "L002",
                        message: "If condition must be Boolean".to_string(),
                        span: *span,
                    });
                }
                let t_ty = self.infer_type(then_, entity_ctx, locals)?;
                let e_ty = self.infer_type(else_, entity_ctx, locals)?;
                if t_ty == e_ty {
                    Ok(t_ty)
                } else {
                    Err(AnalyzerError {
                        code: "L002",
                        message: "If branches must have same type".to_string(),
                        span: *span,
                    })
                }
            }
            Expr::Call { name, args, span } => {
                // Check built-in functions first
                match name.as_str() {
                    "now" => {
                        if !args.is_empty() {
                            return Err(AnalyzerError {
                                code: "L013",
                                message: "now() takes no arguments".to_string(),
                                span: *span,
                            });
                        }
                        // L041: now() cannot be used in derived field expressions
                        if self.in_derived_context {
                            return Err(AnalyzerError {
                                code: "L041",
                                message: "You can't use 'now()' inside a derived field (:=). Derived fields are calculated automatically and must be 'pure' (they shouldn't change depending on when they are read).".to_string(),
                                span: *span,
                            });
                        }
                        return Ok(LuminaType::Timestamp);
                    }
                    "len" => {
                        if args.len() != 1 {
                            return Err(AnalyzerError {
                                code: "L013",
                                message: format!("len expects 1 arg, got {}", args.len()),
                                span: *span,
                            });
                        }
                        let arg_ty = self.infer_type(&args[0], entity_ctx, locals)?;
                        if !matches!(arg_ty, LuminaType::List(_)) {
                            return Err(AnalyzerError {
                                code: "L002",
                                message: "len() requires a list argument".to_string(),
                                span: *span,
                            });
                        }
                        return Ok(LuminaType::Number);
                    }
                    "min" | "max" | "sum" => {
                        if args.len() != 1 {
                            return Err(AnalyzerError {
                                code: "L013",
                                message: format!("{} expects 1 arg, got {}", name, args.len()),
                                span: *span,
                            });
                        }
                        let arg_ty = self.infer_type(&args[0], entity_ctx, locals)?;
                        if arg_ty != LuminaType::List(Box::new(LuminaType::Number)) {
                            return Err(AnalyzerError {
                                code: "L002",
                                message: format!("{}() requires a Number[] argument", name),
                                span: *span,
                            });
                        }
                        return Ok(LuminaType::Number);
                    }
                    "append" => {
                        if args.len() != 2 {
                            return Err(AnalyzerError {
                                code: "L013",
                                message: format!("append expects 2 args, got {}", args.len()),
                                span: *span,
                            });
                        }
                        let list_ty = self.infer_type(&args[0], entity_ctx, locals)?;
                        let val_ty = self.infer_type(&args[1], entity_ctx, locals)?;
                        match &list_ty {
                            LuminaType::List(inner) if **inner == val_ty => return Ok(list_ty),
                            LuminaType::List(_) => {
                                return Err(AnalyzerError {
                                    code: "L002",
                                    message: "append value type must match list element type"
                                        .to_string(),
                                    span: *span,
                                })
                            }
                            _ => {
                                return Err(AnalyzerError {
                                    code: "L002",
                                    message: "append() first argument must be a list".to_string(),
                                    span: *span,
                                })
                            }
                        }
                    }
                    "head" => {
                        if args.len() != 1 {
                            return Err(AnalyzerError {
                                code: "L013",
                                message: format!("head expects 1 arg, got {}", args.len()),
                                span: *span,
                            });
                        }
                        let arg_ty = self.infer_type(&args[0], entity_ctx, locals)?;
                        match arg_ty {
                            LuminaType::List(inner) => return Ok(*inner),
                            _ => {
                                return Err(AnalyzerError {
                                    code: "L002",
                                    message: "head() requires a list argument".to_string(),
                                    span: *span,
                                })
                            }
                        }
                    }
                    "tail" => {
                        if args.len() != 1 {
                            return Err(AnalyzerError {
                                code: "L013",
                                message: format!("tail expects 1 arg, got {}", args.len()),
                                span: *span,
                            });
                        }
                        let arg_ty = self.infer_type(&args[0], entity_ctx, locals)?;
                        if !matches!(&arg_ty, LuminaType::List(_)) {
                            return Err(AnalyzerError {
                                code: "L002",
                                message: "tail() requires a list argument".to_string(),
                                span: *span,
                            });
                        }
                        return Ok(arg_ty);
                    }
                    "at" => {
                        if args.len() != 2 {
                            return Err(AnalyzerError {
                                code: "L013",
                                message: format!("at expects 2 args, got {}", args.len()),
                                span: *span,
                            });
                        }
                        let list_ty = self.infer_type(&args[0], entity_ctx, locals)?;
                        let idx_ty = self.infer_type(&args[1], entity_ctx, locals)?;
                        if idx_ty != LuminaType::Number {
                            return Err(AnalyzerError {
                                code: "L002",
                                message: "at() index must be a Number".to_string(),
                                span: *span,
                            });
                        }
                        match list_ty {
                            LuminaType::List(inner) => return Ok(*inner),
                            _ => {
                                return Err(AnalyzerError {
                                    code: "L002",
                                    message: "at() first argument must be a list".to_string(),
                                    span: *span,
                                })
                            }
                        }
                    }
                    _ => {} // Fall through to user-defined fn lookup
                }
                // User-defined function lookup
                let decl = match self.fn_defs.get(name) {
                    Some(d) => d.clone(),
                    None => {
                        return Err(AnalyzerError {
                            code: "L012",
                            message: format!(
                                "I don't recognize the function '{}()'. In Lumina, functions must be declared with 'fn' before they can be called. This ensures that every calculation in the DAG is accounted for. Did you forget to define it?",
                                name
                            ),
                            span: *span,
                        });
                    }
                };
                if args.len() != decl.params.len() {
                    return Err(AnalyzerError {
                        code: "L013",
                        message: format!(
                            "The function '{}()' expects {} arguments, but you've provided {}. Lumina uses strict function signatures to ensure that every input is accounted for in the reactive graph. Did you miss an argument or provide too many?",
                            name, decl.params.len(), args.len()
                        ),
                        span: *span,
                    });
                }
                for (arg, param) in args.iter().zip(decl.params.iter()) {
                    let arg_ty = self.infer_type(arg, entity_ctx, locals)?;
                    if arg_ty != param.type_ {
                        return Err(AnalyzerError {
                            code: "L013",
                            message: format!(
                                "Argument type mismatch for parameter '{}'. I expected a {:?}, but you provided a {:?}. Lumina's functions require exact type matches to guarantee safe calculations.",
                                param.name, param.type_, arg_ty
                            ),
                            span: *span,
                        });
                    }
                }
                Ok(decl.returns.clone())
            }
            Expr::ListLiteral(elems) => {
                if elems.is_empty() {
                    // Empty list — we can't infer element type, default to Number[]
                    return Ok(LuminaType::List(Box::new(LuminaType::Number)));
                }
                let first_ty = self.infer_type(&elems[0], entity_ctx, locals)?;
                for elem in &elems[1..] {
                    let ty = self.infer_type(elem, entity_ctx, locals)?;
                    if ty != first_ty {
                        return Err(AnalyzerError {
                            code: "L002",
                            message: "all list elements must have the same type".to_string(),
                            span: Span::default(),
                        });
                    }
                }
                Ok(LuminaType::List(Box::new(first_ty)))
            }
            Expr::Index { list, index, span } => {
                let list_ty = self.infer_type(list, entity_ctx, locals)?;
                let idx_ty = self.infer_type(index, entity_ctx, locals)?;
                if idx_ty != LuminaType::Number {
                    return Err(AnalyzerError {
                        code: "L002",
                        message: "list index must be a Number".to_string(),
                        span: *span,
                    });
                }
                match list_ty {
                    LuminaType::List(inner) => Ok(*inner),
                    _ => Err(AnalyzerError {
                        code: "L002",
                        message: format!(
                            "I can only use index access ([]) on Lists. You're trying to index into a {:?}. Lumina enforces strict typing to prevent runtime 'undefined' errors.",
                            list_ty
                        ),
                        span: *span,
                    }),
                }
            }
            Expr::Prev { field, span } => {
                if self.in_prev_context {
                    return Err(AnalyzerError {
                        code: "L025",
                        message: "Nested prev() expressions are not allowed".to_string(),
                        span: *span,
                    });
                }

                let entity_name = entity_ctx.ok_or_else(|| AnalyzerError {
                    code: "L001",
                    message: "prev() can only be used within an entity context".to_string(),
                    span: *span,
                })?;

                let field_schema = self.schema.get_field(entity_name, field).ok_or_else(|| AnalyzerError {
                    code: "L010",
                    message: format!(
                        "I can't look back at the history of '.{}' because that field doesn't exist on '{}'. 'prev()' requires a valid stored field to track its previous state.",
                        field, entity_name
                    ),
                    span: *span,
                })?;

                if field_schema.is_derived {
                    return Err(AnalyzerError {
                        code: "L024",
                        message: format!("I can't use 'prev()' on '{}' because it's a derived field. 'prev()' only works on stored data that has a history.", field),
                        span: *span,
                    });
                }

                Ok(field_schema.ty.clone())
            }
            Expr::ClusterAccess { .. } => {
                // Return Text by default for cluster fields unless we want to map known schema
                Ok(LuminaType::Text)
            }
            Expr::Migrate { .. } | Expr::Evacuate { .. } | Expr::Deploy { .. } => {
                // Orchestration actions evaluate to a Boolean indicating success
                Ok(LuminaType::Boolean)
            }
        }
    }

    fn collect_dependencies(
        &mut self,
        expr: &Expr,
        entity_name: &str,
        target_id: u32,
    ) -> Result<(), Vec<AnalyzerError>> {
        match expr {
            Expr::Ident(name) => {
                // If it's a field in the same entity
                if self.schema.get_field(entity_name, name).is_some() {
                    let dep_id = self.graph.intern(entity_name, name);
                    self.graph.add_edge(dep_id, target_id);
                }
            }
            Expr::FieldAccess { obj, field, .. } => {
                let obj_ty = self
                    .infer_type(obj, Some(entity_name), None)
                    .map_err(|e| vec![e])?;
                if let LuminaType::Entity(e_name) = obj_ty {
                    let dep_id = self.graph.intern(&e_name, field);
                    self.graph.add_edge(dep_id, target_id);
                }
                self.collect_dependencies(obj, entity_name, target_id)?;
            }
            Expr::Binary { left, right, .. } => {
                self.collect_dependencies(left, entity_name, target_id)?;
                self.collect_dependencies(right, entity_name, target_id)?;
            }
            Expr::Unary { operand, .. } => {
                self.collect_dependencies(operand, entity_name, target_id)?;
            }
            Expr::If {
                cond, then_, else_, ..
            } => {
                self.collect_dependencies(cond, entity_name, target_id)?;
                self.collect_dependencies(then_, entity_name, target_id)?;
                self.collect_dependencies(else_, entity_name, target_id)?;
            }
            Expr::InterpolatedString(segments) => {
                for seg in segments {
                    if let StringSegment::Expr(e) = seg {
                        self.collect_dependencies(e, entity_name, target_id)?;
                    }
                }
            }
            Expr::Call { args, .. } => {
                for arg in args {
                    self.collect_dependencies(arg, entity_name, target_id)?;
                }
            }
            Expr::ListLiteral(elems) => {
                for elem in elems {
                    self.collect_dependencies(elem, entity_name, target_id)?;
                }
            }
            Expr::Index { list, index, .. } => {
                self.collect_dependencies(list, entity_name, target_id)?;
                self.collect_dependencies(index, entity_name, target_id)?;
            }
            Expr::Prev { field, .. } => {
                if self.schema.get_field(entity_name, field).is_some() {
                    let dep_id = self.graph.intern(entity_name, field);
                    self.graph.add_edge(dep_id, target_id);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn check_action(
        &mut self,
        action: &Action,
        rule_span: Span,
        locals: Option<&HashMap<String, LuminaType>>,
    ) -> Result<(), Vec<AnalyzerError>> {
        match action {
            Action::Show(expr) => {
                // v1.8: L050 — Secret values cannot be displayed
                let ty = self.infer_type(expr, None, locals).map_err(|e| vec![e])?;
                if ty == LuminaType::Secret {
                    return Err(vec![AnalyzerError {
                        code: "L050",
                        message: "You can't use 'show' with a Secret value. Secret fields are designed to never appear in output to prevent credential leakage. Use 'write' to pass secrets to external adapters instead.".to_string(),
                        span: rule_span,
                    }]);
                }
                Ok(())
            }
            Action::Update { target, value } => {
                let entity_name = if let Some(Some(LuminaType::Entity(e))) =
                    locals.map(|l| l.get(&target.instance))
                {
                    e
                } else {
                    match self.instances.get(&target.instance) {
                        Some(LuminaType::Entity(e)) => e,
                        _ => &target.instance,
                    }
                };
                let field_schema = self
                    .schema
                    .get_field(entity_name, &target.field)
                    .ok_or_else(|| {
                        vec![AnalyzerError {
                            code: "L010",
                            message: format!(
                                "Unknown field '{}' on entity '{}'",
                                target.field, target.instance
                            ),
                            span: target.span,
                        }]
                    })?;

                if field_schema.is_derived {
                    return Err(vec![AnalyzerError {
                        code: "L003",
                        message: format!("I can't manually update '{}' because it's a derived field (it uses :=). Its value is automatically maintained by the engine.", target.field),
                        span: target.span,
                    }]);
                }

                let val_ty = self.infer_type(value, None, locals).map_err(|e| vec![e])?;
                if val_ty != field_schema.ty {
                    return Err(vec![AnalyzerError {
                        code: "L002",
                        message: "Type mismatch in update".to_string(),
                        span: target.span,
                    }]);
                }
                Ok(())
            }
            Action::Create { entity, fields } => {
                let schema_entity = self.schema.get_entity(entity).ok_or_else(|| vec![AnalyzerError {
                    code: "L008",
                    message: format!(
                        "I don't know what a '{}' is. You're trying to create one in a rule, but I only know about entities that have been defined earlier. Did you forget to add 'entity {} {{ ... }}' to your code?",
                        entity, entity
                    ),
                    span: rule_span,
                }])?;

                let mut provided_fields = HashMap::new();
                for (name, expr) in fields {
                    let field_schema = schema_entity.fields.get(name).ok_or_else(|| {
                        vec![AnalyzerError {
                            code: "L010",
                            message: format!("Unknown field '{}' on entity '{}'", name, entity),
                            span: rule_span,
                        }]
                    })?;

                    let ty = self.infer_type(expr, None, locals).map_err(|e| vec![e])?;
                    if ty != field_schema.ty {
                        return Err(vec![AnalyzerError {
                            code: "L002",
                            message: format!("Type mismatch for field '{}'", name),
                            span: rule_span,
                        }]);
                    }
                    provided_fields.insert(name.clone(), ());
                }

                for (name, field) in &schema_entity.fields {
                    if !field.is_derived && !provided_fields.contains_key(name) {
                        return Err(vec![AnalyzerError {
                            code: "L007",
                            message: format!(
                                "I can't create this entity because the field '.{}' is missing. Every stored field must be given a value during creation so the system starts with a valid 'truth'.",
                                name
                            ),
                            span: rule_span,
                        }]);
                    }
                }
                Ok(())
            }
            Action::Delete(instance) => {
                // Simplified: just check if an entity with this name exists in schema
                if !self.schema.entities.contains_key(instance) {
                    return Err(vec![AnalyzerError {
                        code: "L001",
                        message: format!("Unknown instance: {}", instance),
                        span: rule_span,
                    }]);
                }
                Ok(())
            }
            Action::Alert(_) => Ok(()),
            Action::Write { target, value } => {
                let entity_name = if let Some(Some(LuminaType::Entity(e))) =
                    locals.map(|l| l.get(&target.instance))
                {
                    e
                } else {
                    match self.instances.get(&target.instance) {
                        Some(LuminaType::Entity(e)) => e,
                        _ => &target.instance,
                    }
                };
                // L038: write actions can only target external entities
                if let Some(entity_schema) = self.schema.get_entity(entity_name) {
                    if !entity_schema.is_external {
                        return Err(vec![AnalyzerError {
                            code: "L038",
                            message: format!(
                                "I can't use a 'write' action on '{}' because it is a local entity. 'write' is reserved for sending data to external hardware or systems. For local data, use 'update' instead.",
                                entity_name
                            ),
                            span: target.span,
                        }]);
                    }
                    if let Some(field_schema) = entity_schema.fields.get(&target.field) {
                        let val_ty = self.infer_type(value, None, locals).map_err(|e| vec![e])?;
                        if val_ty != field_schema.ty {
                            return Err(vec![AnalyzerError {
                                code: "L002",
                                message: "Type mismatch in write".to_string(),
                                span: target.span,
                            }]);
                        }
                    } else {
                        return Err(vec![AnalyzerError {
                            code: "L010",
                            message: format!(
                                "Unknown field '{}' on entity '{}'",
                                target.field, entity_name
                            ),
                            span: target.span,
                        }]);
                    }
                } else {
                    return Err(vec![AnalyzerError {
                        code: "L001",
                        message: format!("Unknown entity: {}", entity_name),
                        span: target.span,
                    }]);
                }
                Ok(())
            }
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use lumina_parser::parse;

    fn analyze_source(source: &str) -> Result<AnalyzedProgram, Vec<AnalyzerError>> {
        let program = parse(source).map_err(|e| {
            vec![AnalyzerError {
                code: "LEX/PARSE",
                message: e.to_string(),
                span: Span::default(),
            }]
        })?;
        Analyzer::new().analyze(program)
    }

    #[test]
    fn test_prev_analyzer_errors() {
        // L024: cannot use prev on derived field
        let src1 = "entity E { val: Number d := val * 2  bad := prev(d) }";
        let errs1 = analyze_source(src1).unwrap_err();
        assert!(errs1.iter().any(|e| e.code == "L024"));

        // L025: no nested prev (this fails at the syntax level because prev is a keyword, not an identifier)
        let src2 = "entity E { val: Number bad := prev(prev(val)) }";
        let errs2 = analyze_source(src2).unwrap_err();
        assert!(errs2.iter().any(|e| e.code == "LEX/PARSE"));
    }

    #[test]
    fn test_valid_entity_with_derived_fields() {
        let source = "entity Person { age: Number isAdult := age >= 18 }";
        let res = analyze_source(source).expect("analysis should succeed");
        assert!(res.schema.get_entity("Person").is_some());
        let age_id = res.graph.get_node("Person", "age").unwrap();
        let adult_id = res.graph.get_node("Person", "isAdult").unwrap();
        assert!(res.graph.dependents[age_id as usize].contains(&adult_id));
    }

    #[test]
    fn test_circular_dependency() {
        let source = "entity A { a := b b := a }";
        let errs = analyze_source(source).err().unwrap();
        assert!(errs.iter().any(|e| e.code == "L004"));
    }

    #[test]
    fn test_type_mismatch_in_derived_field() {
        let source = "entity A { name: Text age := name + 1 }";
        let errs = analyze_source(source).err().unwrap();
        assert!(errs.iter().any(|e| e.code == "L002"));
    }

    #[test]
    fn test_update_derived_field() {
        let source = "entity A { x := 1 } rule test when true { update A.x to 2 }";
        let errs = analyze_source(source).err().unwrap();
        assert!(errs.iter().any(|e| e.code == "L003"));
    }

    #[test]
    fn test_unknown_field_access() {
        let source = "entity A { x: Number } rule test when true { update A.y to 2 }";
        let errs = analyze_source(source).err().unwrap();
        assert!(errs.iter().any(|e| e.code == "L010"));
    }

    #[test]
    fn test_valid_rule_with_becomes_condition() {
        let source = "entity A { x: Boolean } rule test when A.x becomes true { show \"changed\" }";
        let res = analyze_source(source).expect("analysis should succeed");
        assert_eq!(res.program.statements.len(), 2);
    }

    // ── Phase 2: L035–L042 tests ──────────────────────────────────────────

    #[test]
    fn test_l036_ref_to_nonexistent_entity() {
        let source = "entity A { link: ref NonExistent }";
        let errs = analyze_source(source).unwrap_err();
        assert!(
            errs.iter().any(|e| e.code == "L036"),
            "expected L036, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_valid_ref_declaration() {
        let source = "entity Cooling { ok: Boolean } entity Server { cooling: ref Cooling }";
        let res = analyze_source(source).expect("valid ref should pass");
        let server = res.schema.get_entity("Server").unwrap();
        assert!(server.fields.contains_key("cooling"));
    }

    #[test]
    fn test_l038_write_on_non_external_entity() {
        let source = "entity Local { x: Number } rule w when true { write Local.x = 5 }";
        let errs = analyze_source(source).unwrap_err();
        assert!(
            errs.iter().any(|e| e.code == "L038"),
            "expected L038, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_l042_invalid_timestamp_accessor() {
        let source = "entity S { ts: Timestamp bad := ts.foo }";
        let errs = analyze_source(source).unwrap_err();
        assert!(
            errs.iter().any(|e| e.code == "L042"),
            "expected L042, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_now_returns_timestamp() {
        // now() in a rule condition is valid and should infer as Timestamp
        let source = "entity S { ts: Timestamp } rule r when true { update S.ts to now() }";
        let res = analyze_source(source);
        // This should either succeed or fail with a type mismatch (Timestamp == Timestamp => ok)
        assert!(
            res.is_ok(),
            "now() should return Timestamp type, got: {:?}",
            res.err()
        );
    }

    #[test]
    fn test_age_returns_duration() {
        // ts.age should return Duration
        let source = "entity S { ts: Timestamp stale := ts.age > 60s }";
        let res = analyze_source(source);
        assert!(
            res.is_ok(),
            ".age should return Duration, got: {:?}",
            res.err()
        );
    }
}
