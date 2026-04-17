use crate::config::ClusterConfig;
use crate::gossip::GossipLayer;
use crate::election::{ElectionState, NodeRole};
use crate::wal::WriteAheadLog;
use crate::state_mesh::ClusterStateMesh;
use std::time::Instant;

pub enum NodeState {
    Starting,
    Joining,
    Active,
    Isolated,
}

pub struct ClusterNode {
    pub config: ClusterConfig,
    pub state: NodeState,
    pub gossip: GossipLayer,
    pub election: ElectionState,
    pub state_mesh: ClusterStateMesh,
    // Add real WAL when file paths are injected
}

impl ClusterNode {
    pub fn new(config: ClusterConfig) -> Self {
        let election = ElectionState::new(config.election_timeout);
        Self {
            config,
            state: NodeState::Starting,
            gossip: GossipLayer::new(),
            election,
            state_mesh: ClusterStateMesh::new(),
        }
    }

    pub fn role(&self) -> NodeRole {
        self.election.role
    }

    pub fn is_leader(&self) -> bool {
        self.role() == NodeRole::Leader
    }

    pub fn tick(&mut self, now: Instant) {
        self.election.tick(now);
        // ... gossip sync, state mesh propagation, leader heartbeat ...
    }
}
