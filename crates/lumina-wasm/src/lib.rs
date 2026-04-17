use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use lumina_parser::parse;
use lumina_parser::ast::*;
use lumina_analyzer::analyze;
use lumina_runtime::engine::Evaluator;
use lumina_diagnostics::{Diagnostic, DiagnosticRenderer, SourceLocation};

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct LuminaRuntime {
    evaluator: Evaluator,
    pending_alerts: Vec<lumina_runtime::FiredEvent>,
}

#[wasm_bindgen]
impl LuminaRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new(source: &str) -> Result<LuminaRuntime, JsValue> {
        let now = js_sys::Date::now();
        let program = match parse(source) {
            Ok(p) => p,
            Err(e) => {
                let diag = Diagnostic::new(
                    "LEX/PARSE",
                    e.to_string(),
                    SourceLocation::new("<WASM>", 1, 1, 1),
                    "",
                    Some("Check your syntax — maybe you missed a brace or a semicolon?".to_string())
                );
                return Err(JsValue::from_str(&serde_json::to_string(&vec![diag]).unwrap_or_else(|e| format!("[\"Internal Error during parsing: {}\"]", e))));
            }
        };

        let analyzed = analyze(program, source, "<WASM>", false)
            .map_err(|errors| {
                JsValue::from_str(&serde_json::to_string(&errors).unwrap_or_else(|e| format!("[\"Internal Error during analysis: {}\"]", e)))
            })?;

        let mut rules = Vec::new();
        let mut derived = HashMap::new();
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

        let mut evaluator = Evaluator::new(analyzed.schema, analyzed.graph, rules);
        evaluator.now = now;
        evaluator.derived_exprs = derived;
        
        for stmt in &analyzed.program.statements {
            match stmt {
                Statement::ExternalEntity(e) => {
                    let adapter = lumina_runtime::adapters::static_adapter::StaticAdapter::new(&e.name);
                    evaluator.register_adapter(Box::new(adapter));
                    for f in &e.fields {
                        if let Field::Derived(df) = f {
                            evaluator.derived_exprs.insert((e.name.clone(), df.name.clone()), df.expr.clone());
                        }
                    }
                }
                Statement::Fn(f) => {
                    evaluator.functions.insert(f.name.clone(), f.clone());
                }
                Statement::Aggregate(a) => {
                    evaluator.agg_store.register(a.clone());
                }
                _ => {}
            }
        }
        evaluator.agg_store.recompute(&evaluator.store, Some(&evaluator.cluster_state));

        let mut pending_alerts = Vec::new();

        for stmt in &analyzed.program.statements {
            let evts = evaluator.exec_statement(stmt)
                .map_err(|e| JsValue::from_str(&format!(
                    "Runtime error [{}]: {}", e.code(), e.message()
                )))?;
            pending_alerts.extend(evts);
        }

        // Final recalculation to pick up initial steady-state alerts
        let initial_evts = evaluator.recalculate_all_rules()
            .map_err(|e| JsValue::from_str(&format!(
                "Runtime error [{}]: {}", e.code(), e.message()
            )))?;
        pending_alerts.extend(initial_evts);

        Ok(LuminaRuntime { evaluator, pending_alerts })
    }

    #[wasm_bindgen]
    pub fn apply_event(
        &mut self,
        instance_name: &str,
        field_name: &str,
        value_json: &str,
    ) -> Result<String, JsValue> {
        self.evaluator.now = js_sys::Date::now();
        let json_val: serde_json::Value = serde_json::from_str(value_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid JSON: {e}")))?;

        let value = match json_val {
            serde_json::Value::Number(n) => lumina_runtime::Value::Number(
                n.as_f64().ok_or_else(|| JsValue::from_str("Invalid number"))?
            ),
            serde_json::Value::String(s) => lumina_runtime::Value::Text(s),
            serde_json::Value::Bool(b)   => lumina_runtime::Value::Bool(b),
            serde_json::Value::Array(arr) => {
                let items: Result<Vec<_>, _> = arr.into_iter().map(|v| match v {
                    serde_json::Value::Number(n) => Ok(lumina_runtime::Value::Number(
                        n.as_f64().ok_or_else(|| JsValue::from_str("Invalid number in list"))?
                    )),
                    serde_json::Value::String(s) => Ok(lumina_runtime::Value::Text(s)),
                    serde_json::Value::Bool(b) => Ok(lumina_runtime::Value::Bool(b)),
                    _ => Err(JsValue::from_str("Unsupported value type in list")),
                }).collect();
                lumina_runtime::Value::List(items?)
            }
            _ => return Err(JsValue::from_str("Unsupported value type")),
        };

        match self.evaluator.apply_event(instance_name, field_name, value) {
            Ok(result) => Ok(serde_json::to_string(&result).unwrap_or_else(|e| format!("\"Serialization error: {}\"", e))),
            Err(rollback) => Err(JsValue::from_str(
                &serde_json::to_string(&rollback.diagnostic).unwrap_or_else(|e| format!("\"Rollback serialization error: {}\"", e))
            )),
        }
    }

    #[wasm_bindgen]
    pub fn export_state(&self) -> String {
        serde_json::to_string_pretty(&self.evaluator.export_state()).unwrap_or_else(|e| format!("\"State export error: {}\"", e))
    }

    #[wasm_bindgen]
    pub fn tick(&mut self) -> String {
        self.evaluator.now = js_sys::Date::now();
        let mut all_events = std::mem::take(&mut self.pending_alerts);
        
        match self.evaluator.tick() {
            Ok(events) => {
                all_events.extend(events);
                serde_json::to_string(&all_events).unwrap_or_else(|e| format!("\"Tick error: {}\"", e))
            }
            Err(rb) => format!("ERROR:{}", serde_json::to_string(&rb.diagnostic).unwrap_or_else(|e| format!("\"Rollback error: {}\"", e))),
        }
    }

    #[wasm_bindgen]
    pub fn get_output(&mut self) -> String {
        let out = self.evaluator.get_output().join("\n");
        self.evaluator.clear_output();
        out
    }

    #[wasm_bindgen]
    pub fn check(source: &str) -> String {
        match parse(source) {
            Err(e) => {
                let diag = Diagnostic::new(
                    "LEX/PARSE",
                    e.to_string(),
                    SourceLocation::new("<WASM>", 1, 1, 1),
                    "",
                    Some("Check your syntax rules.".to_string())
                );
                serde_json::to_string(&vec![diag]).unwrap_or_else(|e| format!("[\"Check parse error: {}\"]", e))
            },
            Ok(program) => match analyze(program, source, "<WASM>", false) {
                Err(errors) => serde_json::to_string(&errors).unwrap_or_else(|e| format!("[\"Check analysis error: {}\"]", e)),
                Ok(_) => "[]".to_string(),
            }
        }
    }
}
