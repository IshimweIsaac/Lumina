use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;
use lumina_runtime::engine::Evaluator;
use lumina_analyzer::analyze;
use lumina_parser::parse;
use lumina_cluster::{ClusterNode, ClusterConfig};

#[tokio::test]
async fn test_cross_node_state_sync() {
    // 1. Define two nodes in the cluster
    let source = r#"
        cluster {
            node_id: "node1"
            bind_addr: "127.0.0.1:9001"
            peers: ["node2@127.0.0.1:9002"]
            quorum: 2
        }
        cluster {
            node_id: "node2"
            bind_addr: "127.0.0.1:9002"
            peers: ["node1@127.0.0.1:9001"]
            quorum: 2
        }
        
        entity Sensor {
            temp: Number
        }
        
        let s1 = Sensor { temp: 20 }
        
        aggregate FleetStats over Sensor {
            avg_temp := avg(temp)
        }
    "#;

    let program = parse(source).unwrap();
    let analyzed = analyze(program.clone(), source, "test.lum", true).unwrap();

    // 2. Initialize Evaluator 1 (Node 1)
    let mut ev1 = Evaluator::new(analyzed.schema.clone(), analyzed.graph.clone(), vec![]);
    // Manually extract cluster decl for node1
    let decl1 = program.statements.iter().find_map(|s| {
        if let lumina_parser::ast::Statement::Cluster(c) = s {
            if c.node_id == "node1" { return Some(c); }
        }
        None
    }).unwrap();
    
    let config1 = ClusterConfig::from_decl(decl1);
    let mut node1 = ClusterNode::new(config1);
    node1.initialize();
    let arc_node1 = Arc::new(Mutex::new(node1));
    ev1.cluster_node = Some(Arc::clone(&arc_node1));
    
    // Initial instance setup for ev1
    ev1.exec_statement(&program.statements.iter().find(|s| matches!(s, lumina_parser::ast::Statement::Let(_))).unwrap()).unwrap();

    // 3. Initialize Evaluator 2 (Node 2)
    let mut ev2 = Evaluator::new(analyzed.schema.clone(), analyzed.graph.clone(), vec![]);
    let decl2 = program.statements.iter().find_map(|s| {
        if let lumina_parser::ast::Statement::Cluster(c) = s {
            if c.node_id == "node2" { return Some(c); }
        }
        None
    }).unwrap();
    
    let config2 = ClusterConfig::from_decl(decl2);
    let mut node2 = ClusterNode::new(config2);
    node2.initialize();
    let arc_node2 = Arc::new(Mutex::new(node2));
    ev2.cluster_node = Some(Arc::clone(&arc_node2));
    
    // Initial instance setup for ev2 (same instance s1)
    ev2.exec_statement(&program.statements.iter().find(|s| matches!(s, lumina_parser::ast::Statement::Let(_))).unwrap()).unwrap();

    // 4. Run a few ticks to allow discovery
    for _ in 0..10 {
        ev1.tick().unwrap();
        ev2.tick().unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // 5. Node 1 updates its local sensor temperature
    println!("Node 1 updating temp to 50...");
    ev1.apply_update("s1", "temp", lumina_runtime::value::Value::Number(50.0)).unwrap();
    
    // 6. Run ticks and wait for propagation
    // We need enough ticks for gossip to fire (every 5 ticks in ClusterNode)
    for _ in 0..20 {
        ev1.tick().unwrap();
        ev2.tick().unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // 7. Verify Node 2 has Node 1's state in its cluster_state
    let ev2_cluster_view = ev2.cluster_state.clone();
    println!("Node 2 Cluster View: {:?}", ev2_cluster_view);
    
    assert!(ev2_cluster_view.contains_key("node1"), "Node 2 should have state for node1");
    let node1_state = ev2_cluster_view.get("node1").unwrap();
    assert_eq!(node1_state.get("temp"), Some(&lumina_runtime::value::Value::Number(50.0)));
    
    println!("SUCCESS: State propagated across real UDP sockets.");
}
