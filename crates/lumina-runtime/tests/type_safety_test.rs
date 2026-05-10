use lumina_parser::ast::{BinOp, Expr, UnOp};
use lumina_runtime::engine::Evaluator;
use lumina_runtime::value::Value;
use lumina_runtime::RuntimeError;

#[test]
fn test_strict_math_error() {
    let ev = Evaluator::new_empty();

    // Test: 10 + "fail"
    let expr = Expr::Binary {
        op: BinOp::Add,
        left: Box::new(Expr::Number(10.0)),
        right: Box::new(Expr::Text("fail".into())),
        span: Default::default(),
    };

    let res = ev.eval_expr(&expr, None);

    match res {
        Err(RuntimeError::R018 { op, left, right }) => {
            assert_eq!(op, "+");
            assert_eq!(left, "Number");
            assert_eq!(right, "Text");
            println!("SUCCESS: Caught expected math type mismatch.");
        }
        other => panic!("Expected R018, got {:?}", other),
    }
}

#[test]
fn test_strict_unary_error() {
    let ev = Evaluator::new_empty();

    // Test: !10 (NOT on a Number)
    let expr = Expr::Unary {
        op: UnOp::Not,
        operand: Box::new(Expr::Number(10.0)),
        span: Default::default(),
    };

    let res = ev.eval_expr(&expr, None);

    match res {
        Err(RuntimeError::R018 { op, left, right }) => {
            assert_eq!(op, "not");
            assert_eq!(left, "Number");
            assert_eq!(right, "N/A");
            println!("SUCCESS: Caught expected unary type mismatch.");
        }
        other => panic!("Expected R018, got {:?}", other),
    }
}

#[test]
fn test_strict_list_error() {
    let ev = Evaluator::new_empty();

    // Test: len(42)
    let expr = Expr::Call {
        name: "len".into(),
        args: vec![Expr::Number(42.0)],
        span: Default::default(),
    };

    let res = ev.eval_expr(&expr, None);

    match res {
        Err(RuntimeError::R018 { op, left, right }) => {
            assert_eq!(op, "list expected");
            assert_eq!(left, "Number");
            assert_eq!(right, "List");
            println!("SUCCESS: Caught expected list helper type mismatch.");
        }
        other => panic!("Expected R018, got {:?}", other),
    }
}
