use crate::config::ClusterConfig;
use crate::election::{ElectionState, NodeRole};
use crate::gossip::{GossipLayer, GossipMessageKind};
use crate::state_mesh::ClusterStateMesh;
use crate::transport::UdpTransport;
use crate::wal::WriteAheadLog;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

/// Node lifecycle state
#[derive(Debug, Clone, PartialEq)]
pub enum NodeState {
    Starting,
    Joining,
    Active,
    Isolated,
}

/// Summary of the cluster node's current status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub node_id: String,
    pub role: String,
    pub state: String,
    pub term: u64,
    pub leader_id: Option<String>,
    pub peers_total: usize,
    pub peers_alive: usize,
    pub mesh_nodes: usize,
    pub wal_entries: usize,
}

pub struct ClusterNode {
    pub config: ClusterConfig,
    pub state: NodeState,
    pub gossip: Arc<GossipLayer>,
    pub election: ElectionState,
    pub state_mesh: ClusterStateMesh,
    pub wal: Option<WriteAheadLog>,
    pub orchestration_queue: Vec<GossipMessageKind>,
    tick_count: u64,
}

impl ClusterNode {
    pub fn new(config: ClusterConfig) -> Self {
        let election = ElectionState::new(config.election_timeout);
        Self {
            config,
            state: NodeState::Starting,
            gossip: Arc::new(GossipLayer::new()),
            election,
            state_mesh: ClusterStateMesh::new(),
            wal: None,
            orchestration_queue: Vec::new(),
            tick_count: 0,
        }
    }

    /// Initialize the node: register peers, set up election, transition to Active
    pub fn initialize(&mut self) {
        let total_nodes = (self.config.peers.len() + 1) as u32; // peers + self

        let mut peer_map = FxHashMap::default();
        // Register all peers in the gossip layer
        for peer_str in &self.config.peers {
            // Parse "node_id@addr" or just "addr" (in which case node_id is addr)
            let (id, addr_str) = if let Some((id, addr)) = peer_str.split_once('@') {
                (id.to_string(), addr)
            } else {
                (peer_str.clone(), peer_str.as_str())
            };

            if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                peer_map.insert(id.clone(), addr);
                self.gossip.add_peer(id);
            }
        }

        // Initialize election with our identity and cluster size
        self.election.init(&self.config.node_id, total_nodes);

        // Initialize networking transport
        if let Ok(bind_addr) = self.config.bind_addr.parse::<SocketAddr>() {
            let transport = UdpTransport::new(Arc::clone(&self.gossip), bind_addr, peer_map);
            tokio::spawn(async move {
                if let Err(e) = transport.start().await {
                    eprintln!("Failed to start cluster transport: {}", e);
                }
            });
        }

        // Initialize WAL (best-effort; ignore errors in simulation)
        let wal_path = format!("/tmp/lumina-wal-{}.log", self.config.node_id);
        self.wal = WriteAheadLog::new(&wal_path).ok();

        // Announce ourselves to the cluster
        self.gossip.broadcast(
            self.config.node_id.clone(),
            GossipMessageKind::Join {
                node_id: self.config.node_id.clone(),
            },
        );

        // Register ourselves in the state mesh
        self.state_mesh
            .update_node_state(&self.config.node_id, FxHashMap::default());

        self.state = NodeState::Active;
    }

    /// Main tick loop — called periodically by the runtime
    pub fn tick(&mut self, now: Instant) {
        self.tick_count += 1;

        // 1. Check peer health
        self.gossip.check_peer_health();

        // 2. Process incoming gossip messages
        let messages = self.gossip.drain_inbox();
        for msg in messages {
            match msg.kind {
                GossipMessageKind::Heartbeat { term, leader_id } => {
                    self.election.heartbeat_received(term, &leader_id, now);
                }
                GossipMessageKind::VoteRequest { candidate_id, term } => {
                    let granted = self.election.handle_vote_request(&candidate_id, term, now);
                    self.gossip.broadcast(
                        self.config.node_id.clone(),
                        GossipMessageKind::VoteResponse {
                            voter_id: self.config.node_id.clone(),
                            term: self.election.current_term,
                            granted,
                        },
                    );
                }
                GossipMessageKind::VoteResponse {
                    voter_id,
                    term,
                    granted,
                } => {
                    self.election.receive_vote(&voter_id, term, granted);
                }
                GossipMessageKind::StateSync { node_id, state } => {
                    self.state_mesh.update_node_state(&node_id, state);
                }
                GossipMessageKind::Join { node_id } => {
                    self.gossip.add_peer(node_id);
                }
                GossipMessageKind::WorkloadMove { .. }
                | GossipMessageKind::WorkloadHandoff { .. }
                | GossipMessageKind::WorkloadDeploy { .. } => {
                    self.orchestration_queue.push(msg.kind);
                }
            }
        }

        // 3. Run election tick
        self.election.tick(now);

        // 4. If leader, send heartbeats periodically
        if self.election.should_send_heartbeat() && self.tick_count % 10 == 0 {
            self.gossip.broadcast(
                self.config.node_id.clone(),
                GossipMessageKind::Heartbeat {
                    term: self.election.current_term,
                    leader_id: self.config.node_id.clone(),
                },
            );
        }

        // 5. Periodically sync local state to the mesh via gossip
        if self.tick_count % 5 == 0 {
            if let Some(local_state) = self.state_mesh.snapshot_node(&self.config.node_id) {
                let raw_state: FxHashMap<String, Vec<u8>> =
                    local_state.into_iter().map(|(k, v)| (k, v.value)).collect();
                self.gossip.broadcast(
                    self.config.node_id.clone(),
                    GossipMessageKind::StateSync {
                        node_id: self.config.node_id.clone(),
                        state: raw_state,
                    },
                );
            }
        }

        // 6. Update node state based on connectivity
        let alive = self.gossip.alive_peer_count();
        let total = self.gossip.total_peer_count();
        if total > 0 && alive == 0 {
            self.state = NodeState::Isolated;
        } else if self.state == NodeState::Isolated && alive > 0 {
            self.state = NodeState::Active;
        }
    }

    /// Push a local state update into the mesh (called by the Evaluator)
    pub fn apply_state_update(&mut self, field: &str, value: Vec<u8>) {
        self.state_mesh
            .update_local(&self.config.node_id, field, value.clone());

        // Write to WAL
        if let Some(ref mut wal) = self.wal {
            let _ = wal.append(self.election.current_term, value);
        }
    }

    /// Collect the full cluster state from the mesh for the Evaluator
    pub fn collect_cluster_state(&self) -> FxHashMap<String, FxHashMap<String, Vec<u8>>> {
        self.state_mesh.snapshot_raw()
    }

    /// Get current node role
    pub fn role(&self) -> NodeRole {
        self.election.role
    }

    /// Check if this node is the leader
    pub fn is_leader(&self) -> bool {
        self.role() == NodeRole::Leader
    }

    /// Get a full status summary
    pub fn status(&self) -> ClusterStatus {
        let role_str = match self.election.role {
            NodeRole::Leader => "Leader",
            NodeRole::Follower => "Follower",
            NodeRole::Candidate => "Candidate",
        };
        let state_str = match self.state {
            NodeState::Starting => "Starting",
            NodeState::Joining => "Joining",
            NodeState::Active => "Active",
            NodeState::Isolated => "Isolated",
        };

        ClusterStatus {
            node_id: self.config.node_id.clone(),
            role: role_str.to_string(),
            state: state_str.to_string(),
            term: self.election.current_term,
            leader_id: self.election.leader_id.clone(),
            peers_total: self.gossip.total_peer_count(),
            peers_alive: self.gossip.alive_peer_count(),
            mesh_nodes: self.state_mesh.node_count(),
            wal_entries: self.wal.as_ref().map(|w| w.len()).unwrap_or(0),
        }
    }

    pub fn drain_orchestration(&mut self) -> Vec<GossipMessageKind> {
        std::mem::take(&mut self.orchestration_queue)
    }
}
