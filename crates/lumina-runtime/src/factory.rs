//! Evaluator Factory — Single Source of Truth
//!
//! Centralizes the `build_evaluator` logic that was previously duplicated
//! across `lumina-cli`, `lumina_ffi`, and `lumina-wasm`.

use std::collections::HashMap;
use lumina_parser::ast::*;
use lumina_analyzer::AnalyzedProgram;
use crate::engine::Evaluator;
use crate::adapters::static_adapter::StaticAdapter;

/// Build an `Evaluator` from an analyzed program.
///
/// This is the canonical factory function. It handles:
/// - Rule extraction
/// - Derived field registration (from both `entity` and `external entity`)
/// - Function registration
/// - Aggregate registration and initial computation
/// - Static adapter stubs for external entities
///
/// Callers can attach additional adapters (MQTT, HTTP, File) after this returns.
pub fn build_evaluator(analyzed: &AnalyzedProgram) -> Evaluator {
    let mut rules = Vec::new();
    let mut derived = HashMap::new();

    // Pass 1: Extract rules and derived fields from entities
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

    let mut ev = Evaluator::new(analyzed.schema.clone(), analyzed.graph.clone(), rules);
    ev.derived_exprs = derived;

    // Pass 2: Process external entities, functions, and aggregates
    for stmt in &analyzed.program.statements {
        match stmt {
            Statement::ExternalEntity(e) => {
                // Register a static adapter stub so write-backs don't drop
                ev.register_adapter(Box::new(StaticAdapter::new(&e.name)));
                // Add derived fields from external entities
                for f in &e.fields {
                    if let Field::Derived(df) = f {
                        ev.derived_exprs.insert((e.name.clone(), df.name.clone()), df.expr.clone());
                    }
                }
            }
            Statement::Fn(f) => {
                ev.functions.insert(f.name.clone(), f.clone());
            }
            Statement::Aggregate(a) => {
                ev.agg_store.register(a.clone());
            }
            _ => {}
        }
    }

    // Compute initial aggregate values
    ev.agg_store.recompute(&ev.store, Some(&ev.cluster_state));

    ev
}
