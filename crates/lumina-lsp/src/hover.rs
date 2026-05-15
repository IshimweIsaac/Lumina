use lumina_parser::ast::{Field, Program, RuleTrigger, Statement};
use tower_lsp::lsp_types::*;

pub fn hover_at(prog: &Program, src: &str, pos: Position) -> Option<Hover> {
    for stmt in &prog.statements {
        // ── Entity / ExternalEntity fields ──────────────────────────
        let fields_info = match stmt {
            Statement::Entity(e) => Some((&e.name, &e.fields, e.span)),
            Statement::ExternalEntity(e) => Some((&e.name, &e.fields, e.span)),
            Statement::ResourceEntity(e) => Some((&e.name, &e.fields, e.span)),
            _ => None,
        };

        if let Some((entity_name, fields, entity_span)) = fields_info {
            // Hover on entity name itself
            let el = entity_span.line.saturating_sub(1);
            let ec = entity_span.col.saturating_sub(1);
            if pos.line == el
                && pos.character >= ec
                && pos.character <= ec + entity_name.len() as u32
            {
                let is_ext = matches!(stmt, Statement::ExternalEntity(_) | Statement::ResourceEntity(_));
                let field_count = fields.len();
                let label = if is_ext { "external entity" } else { "entity" };
                return Some(make_hover(format!(
                    "**{label}** `{entity_name}` — {field_count} field(s)"
                )));
            }

            for f in fields {
                let (name, span, doc, range) = match f {
                    Field::Stored(sf) => (&sf.name, &sf.span, &sf.metadata.doc, sf.metadata.range),
                    Field::Derived(df) => (&df.name, &df.span, &df.metadata.doc, df.metadata.range),
                    Field::Ref(rf) => (&rf.name, &rf.span, &None, None),
                };

                let type_label = match f {
                    Field::Stored(sf) => format!("{:?}", sf.ty),
                    Field::Derived(_) => "derived".to_string(),
                    Field::Ref(rf) => format!("ref {}", rf.target_entity),
                };

                let l = span.line.saturating_sub(1);
                let c = span.col.saturating_sub(1);
                let field_len = (span.end.saturating_sub(span.start)).max(1) as u32;

                if pos.line == l && pos.character >= c && pos.character <= c + field_len {
                    let mut lines = vec![format!("**{}**: {}", name, type_label)];
                    if let Some(d) = doc {
                        lines.push(d.clone());
                    }
                    if let Some((lo, hi)) = range {
                        lines.push(format!("Range: {} to {}", lo, hi));
                    }
                    return Some(make_hover(lines.join("\n\n")));
                }
            }
        }

        // ── Rule hover ─────────────────────────────────────────────
        if let Statement::Rule(r) = stmt {
            let rl = r.span.line.saturating_sub(1);
            let rc = r.span.col.saturating_sub(1);
            if pos.line == rl {
                let trigger_desc = match &r.trigger {
                    RuleTrigger::When(conds) => format!("when ({} condition(s))", conds.len()),
                    RuleTrigger::Whenever(conds) => format!("whenever ({} condition(s))", conds.len()),
                    RuleTrigger::Any(fc) => format!("when any {}.{}", fc.entity, fc.field),
                    RuleTrigger::All(fc) => format!("when all {}.{}", fc.entity, fc.field),
                    RuleTrigger::Every(dur) => format!("every {} {:?}", dur.value, dur.unit),
                };
                let param_desc = r
                    .param
                    .as_ref()
                    .map(|p| format!("for ({}: {})", p.name, p.entity))
                    .unwrap_or_default();
                let action_count = r.actions.len();
                let cooldown_desc = r
                    .cooldown
                    .as_ref()
                    .map(|cd| format!(" | cooldown: {} {:?}", cd.value, cd.unit))
                    .unwrap_or_default();
                return Some(make_hover(format!(
                    "**rule** `{}` {}\n\nTrigger: {}\n\n{} action(s){}",
                    r.name, param_desc, trigger_desc, action_count, cooldown_desc
                )));
            }
        }

        // ── Function hover ─────────────────────────────────────────
        if let Statement::Fn(f) = stmt {
            let fl = f.span.line.saturating_sub(1);
            if pos.line == fl {
                let params: Vec<String> = f
                    .params
                    .iter()
                    .map(|p| format!("{}: {:?}", p.name, p.type_))
                    .collect();
                return Some(make_hover(format!(
                    "**fn** `{}`({}) → {:?}",
                    f.name,
                    params.join(", "),
                    f.returns
                )));
            }
        }

        // ── Let binding hover ──────────────────────────────────────
        if let Statement::Let(l) = stmt {
            let ll = l.span.line.saturating_sub(1);
            if pos.line == ll {
                let val_desc = match &l.value {
                    lumina_parser::ast::LetValue::EntityInit(init) => {
                        format!("{} {{ {} field(s) }}", init.entity_name, init.fields.len())
                    }
                    lumina_parser::ast::LetValue::Expr(_) => "expression".to_string(),
                };
                return Some(make_hover(format!("**let** `{}` = {}", l.name, val_desc)));
            }
        }

        // ── Aggregate hover ────────────────────────────────────────
        if let Statement::Aggregate(a) = stmt {
            let al = a.span.line.saturating_sub(1);
            if pos.line == al {
                let field_list: Vec<String> = a
                    .fields
                    .iter()
                    .map(|f| format!("{}: {:?}", f.name, f.expr))
                    .collect();
                return Some(make_hover(format!(
                    "**aggregate** `{}` over `{}`\n\nFields:\n- {}",
                    a.name,
                    a.over,
                    field_list.join("\n- ")
                )));
            }
        }

        // ── Cluster hover ──────────────────────────────────────────
        if let Statement::Cluster(c) = stmt {
            let cl = c.span.line.saturating_sub(1);
            if pos.line == cl {
                return Some(make_hover(format!(
                    "**cluster node** `{}`\n\nPeers: {}\nQuorum: {}\nElection Timeout: {} {:?}",
                    c.node_id,
                    c.peers.len(),
                    c.quorum,
                    c.election_timeout.value,
                    c.election_timeout.unit
                )));
            }
        }
    }
    None
}

fn make_hover(content: String) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range: None,
    }
}
