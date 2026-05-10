use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Types of gossip messages exchanged between nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GossipMessageKind {
    /// Leader heartbeat with current term
    Heartbeat { term: u64, leader_id: String },
    /// State synchronization payload (node_id → field → value bytes)
    StateSync {
        node_id: String,
        state: FxHashMap<String, Vec<u8>>,
    },
    /// Election vote request
    VoteRequest { candidate_id: String, term: u64 },
    /// Election vote response
    VoteResponse {
        voter_id: String,
        term: u64,
        granted: bool,
    },
    /// Node announcing itself to the cluster
    Join { node_id: String },
    /// Orchestration: Request that a workload moves to another node
    WorkloadMove {
        target_node: String,
        workload: Vec<String>,
    },
    /// Orchestration: Actual transfer of instance data
    WorkloadHandoff {
        target_node: String,
        instances: Vec<(String, String, Vec<u8>)>,
    }, // (name, entity, data)
    /// Orchestration: Deploy a new workload
    WorkloadDeploy {
        target_node: String,
        spec_id: String,
        instances: Vec<(String, String, Vec<u8>)>,
    },
}

/// A gossip message with sender metadata
#[derive(Clone, Debug)]
pub struct GossipMessage {
    pub sender: String,
    pub kind: GossipMessageKind,
    pub timestamp: Instant,
}

/// Health status of a peer node
#[derive(Debug, Clone, PartialEq)]
pub enum PeerHealth {
    Alive,
    Suspect,
    Dead,
}

/// Tracked state for a known peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub health: PeerHealth,
    pub last_seen: Instant,
}

/// Gossip layer for distributing events and tracking peer health
pub struct GossipLayer {
    peers: RwLock<FxHashMap<String, PeerInfo>>,
    inbox: RwLock<Vec<GossipMessage>>,
    outbox: RwLock<Vec<GossipMessage>>,
    suspect_timeout_ms: u64,
    dead_timeout_ms: u64,
}

impl Default for GossipLayer {
    fn default() -> Self {
        Self {
            peers: RwLock::new(FxHashMap::default()),
            inbox: RwLock::new(Vec::new()),
            outbox: RwLock::new(Vec::new()),
            suspect_timeout_ms: 5000,
            dead_timeout_ms: 15000,
        }
    }
}

impl GossipLayer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a peer node in the gossip layer
    pub fn add_peer(&self, peer_id: String) {
        let now = Instant::now();
        self.peers.write().insert(
            peer_id.clone(),
            PeerInfo {
                peer_id,
                health: PeerHealth::Alive,
                last_seen: now,
            },
        );
    }

    /// Record that we heard from a peer (resets health timer)
    pub fn mark_alive(&self, peer_id: &str) {
        let now = Instant::now();
        if let Some(peer) = self.peers.write().get_mut(peer_id) {
            peer.health = PeerHealth::Alive;
            peer.last_seen = now;
        }
    }

    /// Queue an outgoing message to all peers
    pub fn broadcast(&self, sender: String, kind: GossipMessageKind) {
        self.outbox.write().push(GossipMessage {
            sender,
            kind,
            timestamp: Instant::now(),
        });
    }

    /// Push an incoming message into the inbox (simulates network receive)
    pub fn receive(&self, msg: GossipMessage) {
        // Mark the sender as alive
        self.mark_alive(&msg.sender);
        self.inbox.write().push(msg);
    }

    /// Drain all incoming messages for processing
    pub fn drain_inbox(&self) -> Vec<GossipMessage> {
        let mut inbox = self.inbox.write();
        std::mem::take(&mut *inbox)
    }

    /// Drain all outgoing messages for transmission
    pub fn drain_outbox(&self) -> Vec<GossipMessage> {
        let mut outbox = self.outbox.write();
        std::mem::take(&mut *outbox)
    }

    /// Check peer health based on elapsed time since last contact.
    /// Marks peers as Suspect or Dead based on configurable timeouts.
    pub fn check_peer_health(&self) {
        let now = Instant::now();
        let mut peers = self.peers.write();
        for peer in peers.values_mut() {
            let elapsed = now.duration_since(peer.last_seen).as_millis() as u64;
            if elapsed > self.dead_timeout_ms {
                peer.health = PeerHealth::Dead;
            } else if elapsed > self.suspect_timeout_ms {
                peer.health = PeerHealth::Suspect;
            }
            // Alive peers stay Alive (reset happens in mark_alive)
        }
    }

    /// Get the current health status of all peers
    pub fn peer_statuses(&self) -> Vec<PeerInfo> {
        self.peers.read().values().cloned().collect()
    }

    /// Count of alive peers
    pub fn alive_peer_count(&self) -> usize {
        self.peers
            .read()
            .values()
            .filter(|p| p.health == PeerHealth::Alive)
            .count()
    }

    /// Count of all registered peers
    pub fn total_peer_count(&self) -> usize {
        self.peers.read().len()
    }
}
