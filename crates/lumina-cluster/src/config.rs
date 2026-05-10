use std::time::Duration;
use lumina_parser::ast::ClusterDecl;

#[derive(Debug, Clone, Default)]
pub struct ClusterConfig {
    pub node_id: String,
    pub bind_addr: String,
    pub peers: Vec<String>,
    pub quorum: u32,
    pub election_timeout: Duration,
}

impl ClusterConfig {
    /// Convert AST ClusterDecl into runtime ClusterConfig
    pub fn from_decl(decl: &ClusterDecl) -> Self {
        Self {
            node_id: decl.node_id.clone(),
            bind_addr: decl.bind_addr.clone(),
            peers: decl.peers.clone(),
            quorum: decl.quorum,
            election_timeout: Duration::from_secs_f64(decl.election_timeout.to_seconds()),
        }
    }
}
