use std::time::Duration;
use lumina_parser::ast::ClusterDecl;

#[derive(Debug, Clone)]
pub struct ClusterConfig {
    pub node_id: String,
    pub peers: Vec<String>,
    pub quorum: u32,
    pub election_timeout: Duration,
}

impl ClusterConfig {
    /// Convert AST ClusterDecl into runtime ClusterConfig
    pub fn from_decl(decl: &ClusterDecl) -> Self {
        Self {
            node_id: decl.node_id.clone(),
            peers: decl.peers.clone(),
            quorum: decl.quorum,
            election_timeout: Duration::from_secs_f64(decl.election_timeout.to_seconds()),
        }
    }
}
