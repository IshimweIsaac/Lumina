use std::collections::{HashMap, HashSet};
use parking_lot::RwLock;

/// Simple simulated gossip layer for distributing events
#[derive(Default)]
pub struct GossipLayer {
    peers: RwLock<HashSet<String>>,
    messages: RwLock<Vec<GossipMessage>>,
}

#[derive(Clone, Debug)]
pub struct GossipMessage {
    pub sender: String,
    pub payload: Vec<u8>,
}

impl GossipLayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_peer(&self, peer_id: String) {
        self.peers.write().insert(peer_id);
    }

    pub fn broadcast(&self, sender: String, payload: Vec<u8>) {
        // In a real implementation, this would send via UDP/TCP to peers.
        // For testing/simulation, we just store it.
        self.messages.write().push(GossipMessage { sender, payload });
    }

    pub fn drain_messages(&self) -> Vec<GossipMessage> {
        let mut p = self.messages.write();
        std::mem::take(&mut *p)
    }
}
