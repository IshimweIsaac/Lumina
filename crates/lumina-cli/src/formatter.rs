//! Lumina v2.0 — Opinionated Code Formatter
//!
//! Pretty-prints a parsed AST back to canonical Lumina source code.
//! Enforces consistent indentation (2 spaces), spacing, and ordering.

use lumina_parser::ast::*;

const INDENT: &str = "  ";

/// Format a complete Lumina program to a canonical string.
pub fn format_program(program: &Program) -> String {
    let mut out = String::new();
    let total = program.statements.len();
    for (i, stmt) in program.statements.iter().enumerate() {
        out.push_str(&format_statement(stmt));
        out.push('\n');
        // Add blank line between top-level declarations
        if i + 1 < total {
            out.push('\n');
        }
    }
    out
}

fn format_statement(stmt: &Statement) -> String {
    match stmt {
        Statement::Entity(e) => format_entity(e),
        Statement::ExternalEntity(e) => format_external_entity(e),
        Statement::Let(l) => format_let(l),
        Statement::Rule(r) => format_rule(r),
        Statement::Action(a) => format_action(a),
        Statement::Fn(f) => format_fn(f),
        Statement::Import(i) => {
            if let Some(ns) = &i.namespace {
                format!("import {}", ns.join("::"))
            } else {
                format!("import \"{}\"", i.path)
            }
        }
        Statement::Aggregate(a) => format_aggregate(a),
        Statement::PluginImport(p) => format!("import plugin \"{}\" as {}", p.path, p.alias),
        Statement::Provider(p) => format_provider(p),
        Statement::Cluster(c) => format_cluster(c),
        Statement::ResourceEntity(e) => format_resource_entity(e),
    }
}

fn format_resource_entity(e: &ResourceEntityDecl) -> String {
    let mut out = format!("resource entity {} provider \"{}\" {{\n", e.name, e.provider);
    for field in &e.fields {
        out.push_str(&format_field(field));
        out.push('\n');
    }
    if !e.desired_state.is_empty() {
        out.push_str(&format!("{}ensure {{\n", INDENT));
        for (name, expr) in &e.desired_state {
            out.push_str(&format!("{}{}{}: {},\n", INDENT, INDENT, name, format_expr(expr)));
        }
        out.push_str(&format!("{}}}\n", INDENT));
    }
    out.push('}');
    out
}

fn format_entity(e: &EntityDecl) -> String {
    let mut out = format!("entity {} {{\n", e.name);
    for field in &e.fields {
        out.push_str(&format_field(field));
        out.push('\n');
    }
    out.push('}');
    out
}

fn format_external_entity(e: &ExternalEntityDecl) -> String {
    let mut out = format!("external entity {} {{\n", e.name);
    for field in &e.fields {
        out.push_str(&format_field(field));
        out.push('\n');
    }
    if !e.sync_path.is_empty() {
        out.push_str(&format!("{}sync: \"{}\"\n", INDENT, e.sync_path));
    }
    match e.sync_strategy {
        SyncStrategy::Realtime => {}
        SyncStrategy::Poll => out.push_str(&format!("{}on: poll\n", INDENT)),
        SyncStrategy::Webhook => out.push_str(&format!("{}on: webhook\n", INDENT)),
    }
    if !e.sync_fields.is_empty() {
        out.push_str(&format!(
            "{}sync_on ({})\n",
            INDENT,
            e.sync_fields.join(", ")
        ));
    }
    if let Some(d) = &e.poll_interval {
        out.push_str(&format!(
            "{}poll_interval: {}\n",
            INDENT,
            format_duration(d)
        ));
    }
    if let Some(d) = &e.sync_timeout {
        out.push_str(&format!("{}timeout {}\n", INDENT, format_duration(d)));
    }
    if e.fallible {
        out.push_str(&format!("{}fallible\n", INDENT));
    }
    out.push('}');
    out
}

fn format_field(field: &Field) -> String {
    match field {
        Field::Stored(f) => {
            let meta = format_metadata(&f.metadata);
            format!("{}{}{}: {}", meta, INDENT, f.name, format_type(&f.ty))
        }
        Field::Derived(f) => {
            let meta = format_metadata(&f.metadata);
            format!("{}{}{} := {}", meta, INDENT, f.name, format_expr(&f.expr))
        }
        Field::Ref(r) => {
            format!("{}{}: ref {}", INDENT, r.name, r.target_entity)
        }
    }
}

fn format_metadata(meta: &FieldMetadata) -> String {
    let mut out = String::new();
    if let Some(doc) = &meta.doc {
        out.push_str(&format!("{}@doc \"{}\"\n", INDENT, doc));
    }
    if let Some((lo, hi)) = meta.range {
        out.push_str(&format!(
            "{}@range {} to {}\n",
            INDENT,
            format_number(lo),
            format_number(hi)
        ));
    }
    if !meta.affects.is_empty() {
        out.push_str(&format!("{}@affects {}\n", INDENT, meta.affects.join(", ")));
    }
    out
}

fn format_type(ty: &LuminaType) -> String {
    match ty {
        LuminaType::Text => "Text".into(),
        LuminaType::Number => "Number".into(),
        LuminaType::Boolean => "Boolean".into(),
        LuminaType::Timestamp => "Timestamp".into(),
        LuminaType::Duration => "Duration".into(),
        LuminaType::Secret => "Secret".into(),
        LuminaType::Entity(n) => n.clone(),
        LuminaType::List(inner) => format!("{}[]", format_type(inner)),
    }
}

fn format_let(l: &LetStmt) -> String {
    match &l.value {
        LetValue::Expr(e) => format!("let {} = {}", l.name, format_expr(e)),
        LetValue::EntityInit(init) => {
            let mut out = format!("let {} = {} {{\n", l.name, init.entity_name);
            for (name, expr) in &init.fields {
                out.push_str(&format!("{}{}: {},\n", INDENT, name, format_expr(expr)));
            }
            out.push('}');
            out
        }
    }
}

fn format_rule(r: &RuleDecl) -> String {
    let global_prefix = if r.is_global { "global " } else { "" };
    let mut out = format!("{}rule {}", global_prefix, r.name);
    if let Some(param) = &r.param {
        out.push_str(&format!(" for ({}: {})", param.name, param.entity));
    }
    out.push('\n');

    // Trigger
    match &r.trigger {
        RuleTrigger::When(conds) => {
            for (i, cond) in conds.iter().enumerate() {
                if i == 0 {
                    out.push_str(&format!("when {}", format_condition(cond)));
                } else {
                    out.push_str(&format!("\nand {}", format_condition(cond)));
                }
            }
        }
        RuleTrigger::Any(fc) => out.push_str(&format!("when any {}", format_fleet_condition(fc))),
        RuleTrigger::All(fc) => out.push_str(&format!("when all {}", format_fleet_condition(fc))),
        RuleTrigger::Whenever(conds) => {
            for (i, cond) in conds.iter().enumerate() {
                if i == 0 {
                    out.push_str(&format!("whenever {}", format_condition(cond)));
                } else {
                    out.push_str(&format!("\nand {}", format_condition(cond)));
                }
            }
        }
        RuleTrigger::Every(d) => out.push_str(&format!("every {}", format_duration(d))),
    }

    // Actions body
    out.push_str(" {\n");
    for action in &r.actions {
        out.push_str(&format!("{}{}\n", INDENT, format_action(action)));
    }
    out.push('}');

    // On clear
    if let Some(clear_actions) = &r.on_clear {
        out.push_str(" on clear {\n");
        for action in clear_actions {
            out.push_str(&format!("{}{}\n", INDENT, format_action(action)));
        }
        out.push('}');
    }

    // Cooldown
    if let Some(cd) = &r.cooldown {
        out.push_str(&format!("\ncooldown {}", format_duration(cd)));
    }

    out
}

fn format_condition(cond: &Condition) -> String {
    let mut out = format_expr(&cond.expr);
    if let Some(becomes) = &cond.becomes {
        out.push_str(&format!(" becomes {}", format_expr(becomes)));
    }
    if let Some(d) = &cond.for_duration {
        out.push_str(&format!(" for {}", format_duration(d)));
    }
    if let Some(freq) = &cond.frequency {
        out.push_str(&format!(
            " {} times within {}",
            freq.count,
            format_duration(&freq.within)
        ));
    }
    out
}

fn format_fleet_condition(fc: &FleetCondition) -> String {
    let mut out = format!(
        "{}.{} becomes {}",
        fc.entity,
        fc.field,
        format_expr(&fc.becomes)
    );
    if let Some(d) = &fc.for_duration {
        out.push_str(&format!(" for {}", format_duration(d)));
    }
    if let Some(freq) = &fc.frequency {
        out.push_str(&format!(
            " {} times within {}",
            freq.count,
            format_duration(&freq.within)
        ));
    }
    out
}

fn format_action(action: &Action) -> String {
    match action {
        Action::Show(expr) => format!("show {}", format_expr(expr)),
        Action::Update { target, value } => {
            format!(
                "update {}.{} to {}",
                target.instance,
                target.field,
                format_expr(value)
            )
        }
        Action::Write { target, value } => {
            format!(
                "write {}.{} = {}",
                target.instance,
                target.field,
                format_expr(value)
            )
        }
        Action::Create { entity, fields } => {
            let mut out = format!("create {} {{\n", entity);
            for (name, expr) in fields {
                out.push_str(&format!(
                    "{}{}{}: {},\n",
                    INDENT,
                    INDENT,
                    name,
                    format_expr(expr)
                ));
            }
            out.push_str(&format!("{}}}", INDENT));
            out
        }
        Action::Delete(name) => format!("delete {}", name),
        Action::Alert(a) => {
            let mut parts = vec![
                format!("severity: {}", format_expr(&a.severity)),
                format!("message: {}", format_expr(&a.message)),
            ];
            if let Some(src) = &a.source {
                parts.push(format!("source: {}", format_expr(src)));
            }
            if let Some(code) = &a.code {
                parts.push(format!("code: {}", format_expr(code)));
            }
            for (k, v) in &a.payload {
                parts.push(format!("{}: {}", k, format_expr(v)));
            }
            format!("alert {}", parts.join(", "))
        }
        Action::Provision { target, .. } => format!("provision {}", target),
        Action::Destroy { target, .. } => format!("destroy {}", target),
        Action::Reconcile { target, .. } => format!("reconcile {}", target),
        Action::Trace(expr) => format!("trace {}", format_expr(expr)),
        Action::For {
            param,
            entity,
            actions,
            ..
        } => {
            let mut out = format!("for ({}: {}) {{\n", param, entity);
            for action in actions {
                out.push_str(&format!("{}{}  \n", INDENT, format_action(action)));
            }
            out.push_str("}");
            out
        }
    }
}

fn format_fn(f: &FnDecl) -> String {
    let params: Vec<String> = f
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name, format_type(&p.type_)))
        .collect();
    let mut out = format!(
        "fn {}({}) -> {} {{\n",
        f.name,
        params.join(", "),
        format_type(&f.returns)
    );
    out.push_str(&format!("{}{}\n", INDENT, format_expr(&f.body)));
    out.push('}');
    out
}

fn format_aggregate(a: &AggregateDecl) -> String {
    let scope_str = match &a.scope {
        AggregateScope::Local => String::new(),
        AggregateScope::Cluster => " cluster".to_string(),
        AggregateScope::Region(r) => format!(" region[\"{}\"]", r),
    };
    let mut out = format!("aggregate {} over {}{} {{\n", a.name, a.over, scope_str);
    for field in &a.fields {
        let fn_str = match &field.expr {
            AggregateExpr::Avg(f) => format!("avg({})", f),
            AggregateExpr::Min(f) => format!("min({})", f),
            AggregateExpr::Max(f) => format!("max({})", f),
            AggregateExpr::Sum(f) => format!("sum({})", f),
            AggregateExpr::Count(Some(f)) => format!("count({})", f),
            AggregateExpr::Count(None) => "count()".into(),
            AggregateExpr::Any(f) => format!("any({})", f),
            AggregateExpr::All(f) => format!("all({})", f),
        };
        out.push_str(&format!("{}{} := {}\n", INDENT, field.name, fn_str));
    }
    out.push('}');
    out
}

fn format_duration(d: &Duration) -> String {
    let unit = match d.unit {
        TimeUnit::Seconds => "s",
        TimeUnit::Minutes => "m",
        TimeUnit::Hours => "h",
        TimeUnit::Days => "d",
    };
    format!("{} {}", format_number(d.value), unit)
}

fn format_expr(expr: &Expr) -> String {
    match expr {
        Expr::Number(n) => format_number(*n),
        Expr::Text(s) => format!("\"{}\"", s),
        Expr::Bool(true) => "true".into(),
        Expr::Bool(false) => "false".into(),
        Expr::Ident(name) => name.clone(),
        Expr::FieldAccess { obj, field, .. } => {
            format!("{}.{}", format_expr(obj), field)
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            format!("{} {} {}", format_expr(left), op, format_expr(right))
        }
        Expr::Unary { op, operand, .. } => {
            format!("{} {}", op, format_expr(operand))
        }
        Expr::If {
            cond, then_, else_, ..
        } => {
            format!(
                "if {} then {} else {}",
                format_expr(cond),
                format_expr(then_),
                format_expr(else_)
            )
        }
        Expr::InterpolatedString(segments) => {
            let mut out = String::from("\"");
            for seg in segments {
                match seg {
                    StringSegment::Literal(s) => out.push_str(s),
                    StringSegment::Expr(e) => {
                        out.push('{');
                        out.push_str(&format_expr(e));
                        out.push('}');
                    }
                }
            }
            out.push('"');
            out
        }
        Expr::Call { name, args, .. } => {
            let args_str: Vec<String> = args.iter().map(|a| format_expr(a)).collect();
            format!("{}({})", name, args_str.join(", "))
        }
        Expr::ListLiteral(elems) => {
            let elems_str: Vec<String> = elems.iter().map(|e| format_expr(e)).collect();
            format!("[{}]", elems_str.join(", "))
        }
        Expr::Index { list, index, .. } => {
            format!("{}[{}]", format_expr(list), format_expr(index))
        }
        Expr::Duration(d) => format_duration(d),
        Expr::Prev { field, .. } => format!("prev({})", field),
        // v2.0 expressions
        Expr::ClusterAccess { node_id, field, .. } => format!("cluster.{}.{}", node_id, field),
        Expr::Migrate {
            workloads, target, ..
        } => {
            format!(
                "migrate({}, to: {})",
                format_expr(workloads),
                format_expr(target)
            )
        }
        Expr::Evacuate { entities, .. } => format!("evacuate({})", format_expr(entities)),
        Expr::Deploy { spec, .. } => format!("deploy({})", format_expr(spec)),
    }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

fn format_provider(p: &ProviderDecl) -> String {
    let mut out = format!("provider \"{}\" {{\n", p.protocol);
    for entry in &p.config {
        out.push_str(&format!(
            "{}{}:{}\n",
            INDENT,
            entry.key,
            format!(" {}", format_expr(&entry.value))
        ));
    }
    out.push('}');
    out
}

fn format_cluster(c: &ClusterDecl) -> String {
    let mut out = String::from("cluster {\n");
    out.push_str(&format!("{}node_id: \"{}\"\n", INDENT, c.node_id));
    let peers_str: Vec<String> = c.peers.iter().map(|p| format!("\"{}\"", p)).collect();
    out.push_str(&format!("{}peers: [{}]\n", INDENT, peers_str.join(", ")));
    out.push_str(&format!("{}quorum: {}\n", INDENT, c.quorum));
    out.push_str(&format!(
        "{}election_timeout: {}\n",
        INDENT,
        format_duration(&c.election_timeout)
    ));
    out.push('}');
    out
}
