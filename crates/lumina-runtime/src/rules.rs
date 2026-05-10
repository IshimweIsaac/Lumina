use lumina_parser::ast::*;
use crate::engine::Evaluator;
use crate::value::Value;
use crate::RuntimeError;

pub fn condition_is_met(
    evaluator: &Evaluator,
    condition: &Condition,
    instance_name: &str,
    check_transition: bool,
) -> Result<bool, RuntimeError> {
    let current = evaluator.eval_expr(&condition.expr, Some(instance_name))?;

    match &condition.becomes {
        None => Ok(current == Value::Bool(true)),
        Some(target_expr) => {
            let target = evaluator.eval_expr(target_expr, Some(instance_name))?;
            let currently_matches = current == target;

            if !currently_matches {
                return Ok(false);
            }

            // When check_transition is true and we have a prev_store snapshot,
            // verify the value actually *changed* to the target (wasn't already there).
            if check_transition {
                if let Some(prev_store) = &evaluator.prev_store {
                    // Evaluate the same expression against the previous state
                    let prev_val = eval_expr_with_store(evaluator, &condition.expr, instance_name, prev_store);
                    if let Ok(prev) = prev_val {
                        // If the previous value already matched the target, this is NOT a transition
                        if prev == target {
                            return Ok(false);
                        }
                    }
                }
            }

            Ok(true)
        }
    }
}

/// Evaluate a field-access expression against a previous store snapshot.
/// This is used to check whether a `becomes` condition was already true before the update.
fn eval_expr_with_store(
    evaluator: &Evaluator,
    expr: &Expr,
    instance_name: &str,
    prev_store: &crate::store::EntityStore,
) -> Result<Value, RuntimeError> {
    // For field access expressions like `Entity.field`, look up in prev_store
    match expr {
        Expr::FieldAccess { obj, field, .. } => {
            // Resolve the instance: could be entity name or instance name
            let inst_name = match obj.as_ref() {
                Expr::Ident(name) => {
                    // Check if this is an entity name — resolve to the instance
                    if let Some(inst) = prev_store.get(instance_name) {
                        if inst.entity_name == *name {
                            instance_name.to_string()
                        } else {
                            name.clone()
                        }
                    } else {
                        name.clone()
                    }
                }
                _ => instance_name.to_string(),
            };
            if let Some(inst) = prev_store.get(&inst_name) {
                if let Some(val) = evaluator.get_instance_field(inst, field) {
                    return Ok(val);
                }
                // Check prev_fields for the previous-previous value
                if let Some(val) = evaluator.get_instance_prev_field(inst, field) {
                    return Ok(val);
                }
            }
            // Fallback: use the current evaluator
            evaluator.eval_expr(expr, Some(instance_name))
        }
        Expr::Binary { op, left, right, span: _ } => {
            let l = eval_expr_with_store(evaluator, left, instance_name, prev_store)?;
            let r = eval_expr_with_store(evaluator, right, instance_name, prev_store)?;
            // Re-evaluate the binary operation with previous values
            evaluator.eval_binary_values(op, &l, &r)
        }
        _ => evaluator.eval_expr(expr, Some(instance_name)),
    }
}
