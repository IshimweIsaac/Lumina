use std::sync::{Arc, Mutex};
use std::time::Duration;
use lumina_runtime::engine::Evaluator;
use lumina_analyzer::analyze;
use lumina_parser::parse;
use lumina_cluster::{ClusterNode, ClusterConfig};
use lumina_runtime::value::Value;

#[tokio::test]
async fn test_workload_migration() {
    let source = r#"
        cluster {
            node_id: "node1"
            bind_addr: "127.0.0.1:9101"
            peers: ["node2@127.0.0.1:9102"]
            quorum: 2
        }
        cluster {
            node_id: "node2"
            bind_addr: "127.0.0.1:9102"
            peers: ["node1@127.0.0.1:9101"]
            quorum: 2
        }
        
        entity Task {
            status: Text
        }
        
        let t1 = Task { status: "pending" }
    "#;

    let program = parse(source).unwrap();
    let analyzed = analyze(program.clone(), source, "test.lum", true).unwrap();

    // Node 1 Setup
    let mut ev1 = Evaluator::new(analyzed.schema.clone(), analyzed.graph.clone(), vec![]);
    let config1 = ClusterConfig {
        node_id: "node1".to_string(),
        bind_addr: "127.0.0.1:9101".to_string(),
        peers: vec!["node2@127.0.0.1:9102".to_string()],
        ..Default::default()
    };
    let mut node1 = ClusterNode::new(config1);
    node1.initialize();
    let arc_node1 = Arc::new(Mutex::new(node1));
    ev1.cluster_node = Some(Arc::clone(&arc_node1));
    
    // Create t1 on node1
    ev1.exec_statement(&program.statements.iter().find(|s| matches!(s, lumina_parser::ast::Statement::Let(_))).unwrap()).unwrap();

    // Node 2 Setup
    let mut ev2 = Evaluator::new(analyzed.schema.clone(), analyzed.graph.clone(), vec![]);
    let config2 = ClusterConfig {
        node_id: "node2".to_string(),
        bind_addr: "127.0.0.1:9102".to_string(),
        peers: vec!["node1@127.0.0.1:9101".to_string()],
        ..Default::default()
    };
    let mut node2 = ClusterNode::new(config2);
    node2.initialize();
    let arc_node2 = Arc::new(Mutex::new(node2));
    ev2.cluster_node = Some(Arc::clone(&arc_node2));

    // Wait for discovery
    for _ in 0..10 {
        ev1.tick().unwrap();
        ev2.tick().unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Verify t1 is on node1
    assert!(ev1.store.get("t1").is_some());
    assert!(ev2.store.get("t1").is_none());

    // Trigger migration from node1 to node2
    println!("Migrating t1 from node1 to node2...");
    let migrate_expr = lumina_parser::ast::Expr::Migrate {
        workloads: Box::new(lumina_parser::ast::Expr::Text("t1".into())),
        target: Box::new(lumina_parser::ast::Expr::Text("node2".into())),
        span: Default::default(),
    };
    ev1.eval_expr(&migrate_expr, None).unwrap();

    // Wait for migration gossip
    for _ in 0..20 {
        ev1.tick().unwrap();
        ev2.tick().unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Verify t1 moved to node2
    assert!(ev1.store.get("t1").is_none(), "t1 should be gone from node1");
    assert!(ev2.store.get("t1").is_some(), "t1 should be present on node2");
    
    let t1_node2 = ev2.store.get("t1").unwrap();
    assert_eq!(ev2.get_instance_field(t1_node2, "status").unwrap(), Value::Text("pending".into()));
    
    println!("SUCCESS: Workload migrated across nodes.");
}
