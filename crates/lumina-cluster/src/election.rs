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
    last_heartbeat: Instant,
    election_timeout: Duration,
}

impl ElectionState {
    pub fn new(election_timeout: Duration) -> Self {
        Self {
            role: NodeRole::Follower,
            current_term: 0,
            voted_for: None,
            leader_id: None,
            last_heartbeat: Instant::now(),
            election_timeout,
        }
    }

    pub fn tick(&mut self, now: Instant) {
        if self.role == NodeRole::Follower || self.role == NodeRole::Candidate {
            if now.duration_since(self.last_heartbeat) >= self.election_timeout {
                self.start_election(now);
            }
        }
    }

    fn start_election(&mut self, now: Instant) {
        self.role = NodeRole::Candidate;
        self.current_term += 1;
        self.last_heartbeat = now;
        // In real Raft, we'd vote for ourselves and send RequestVote RPCs
    }

    pub fn heartbeat_received(&mut self, term: u64, leader_id: &str, now: Instant) {
        if term >= self.current_term {
            self.current_term = term;
            self.role = NodeRole::Follower;
            self.leader_id = Some(leader_id.to_string());
            self.last_heartbeat = now;
        }
    }
}
