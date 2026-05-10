use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::time::{self, Duration};
use crate::gossip::{GossipLayer, GossipMessage, GossipMessageKind};
use serde_json;

pub struct UdpTransport {
    gossip: Arc<GossipLayer>,
    bind_addr: SocketAddr,
    peer_map: rustc_hash::FxHashMap<String, SocketAddr>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NetMessage {
    pub sender: String,
    pub kind: GossipMessageKind,
}

impl UdpTransport {
    pub fn new(
        gossip: Arc<GossipLayer>,
        bind_addr: SocketAddr,
        peers: rustc_hash::FxHashMap<String, SocketAddr>,
    ) -> Self {
        Self {
            gossip,
            bind_addr,
            peer_map: peers,
        }
    }

    pub async fn start(self) -> Result<(), std::io::Error> {
        let socket = UdpSocket::bind(self.bind_addr).await?;
        let socket = Arc::new(socket);
        println!("  [UDP] Transport listening on {}", self.bind_addr);
        
        let gossip_recv = Arc::clone(&self.gossip);
        let socket_recv = Arc::clone(&socket);

        // Receiver Task
        tokio::spawn(async move {
            let mut buf = [0u8; 65535];
            loop {
                match socket_recv.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        if let Ok(net_msg) = serde_json::from_slice::<NetMessage>(&buf[..len]) {
                            gossip_recv.receive(GossipMessage {
                                sender: net_msg.sender,
                                kind: net_msg.kind,
                                timestamp: std::time::Instant::now(),
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("UDP Receive Error: {}", e);
                    }
                }
            }
        });

        // Sender Task
        let gossip_send = Arc::clone(&self.gossip);
        let socket_send = Arc::clone(&socket);
        let peer_map = self.peer_map.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(50));
            loop {
                interval.tick().await;
                let messages = gossip_send.drain_outbox();
                for msg in messages {
                    let net_msg = NetMessage {
                        sender: msg.sender,
                        kind: msg.kind,
                    };
                    if let Ok(data) = serde_json::to_vec(&net_msg) {
                        // Broadcast to all peers
                        for addr in peer_map.values() {
                            let _ = socket_send.send_to(&data, addr).await;
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
