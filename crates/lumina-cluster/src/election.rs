use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Follower,
    Candidate,
    Leader,
}

pub struct ElectionState {
    pub role: NodeRole,
    pub current_term: u64,
    pub voted_for: Option<String>,
    pub leader_id: Option<String>,
    pub vote_count: u32,
    pub total_nodes: u32,
    last_heartbeat: Instant,
    election_timeout: Duration,
    node_id: String,
}

impl ElectionState {
    pub fn new(election_timeout: Duration) -> Self {
        Self {
            role: NodeRole::Follower,
            current_term: 0,
            voted_for: None,
            leader_id: None,
            vote_count: 0,
            total_nodes: 1,
            last_heartbeat: Instant::now(),
            election_timeout,
            node_id: String::new(),
        }
    }

    /// Initialize with node identity and cluster size
    pub fn init(&mut self, node_id: &str, total_nodes: u32) {
        self.node_id = node_id.to_string();
        self.total_nodes = total_nodes;

        // Single-node cluster: immediately promote to leader
        if total_nodes <= 1 {
            self.role = NodeRole::Leader;
            self.current_term = 1;
            self.leader_id = Some(node_id.to_string());
            self.voted_for = Some(node_id.to_string());
            self.vote_count = 1;
        }
    }

    pub fn tick(&mut self, now: Instant) {
        match self.role {
            NodeRole::Leader => {
                // Leaders don't time out, they send heartbeats
            }
            NodeRole::Follower | NodeRole::Candidate => {
                if now.duration_since(self.last_heartbeat) >= self.election_timeout {
                    self.start_election(now);
                }
            }
        }
    }

    fn start_election(&mut self, now: Instant) {
        self.current_term += 1;
        self.role = NodeRole::Candidate;
        self.voted_for = Some(self.node_id.clone());
        self.vote_count = 1; // Vote for ourselves
        self.last_heartbeat = now;

        // If we're the only node, win immediately
        if self.quorum_reached() {
            self.promote_to_leader();
        }
    }

    /// Receive a vote from a peer. Returns true if quorum is now reached.
    pub fn receive_vote(&mut self, from_node: &str, term: u64, granted: bool) -> bool {
        if term != self.current_term || self.role != NodeRole::Candidate {
            return false;
        }
        if granted {
            self.vote_count += 1;
            if self.quorum_reached() {
                self.promote_to_leader();
                return true;
            }
        }
        false
    }

    /// Check if we have enough votes for quorum (majority)
    fn quorum_reached(&self) -> bool {
        self.vote_count > self.total_nodes / 2
    }

    /// Promote this node to leader
    fn promote_to_leader(&mut self) {
        self.role = NodeRole::Leader;
        self.leader_id = Some(self.node_id.clone());
    }

    /// Handle an incoming heartbeat from a leader
    pub fn heartbeat_received(&mut self, term: u64, leader_id: &str, now: Instant) {
        if term >= self.current_term {
            self.current_term = term;
            self.role = NodeRole::Follower;
            self.leader_id = Some(leader_id.to_string());
            self.last_heartbeat = now;
            self.vote_count = 0;
        }
    }

    /// Handle a vote request from a candidate
    pub fn handle_vote_request(&mut self, candidate_id: &str, term: u64, now: Instant) -> bool {
        if term > self.current_term {
            self.current_term = term;
            self.role = NodeRole::Follower;
            self.voted_for = None;
            self.last_heartbeat = now;
        }

        if term == self.current_term && self.voted_for.is_none() {
            self.voted_for = Some(candidate_id.to_string());
            self.last_heartbeat = now;
            true
        } else {
            false
        }
    }

    /// Generate a heartbeat payload (for leader to broadcast)
    pub fn should_send_heartbeat(&self) -> bool {
        self.role == NodeRole::Leader
    }
}
