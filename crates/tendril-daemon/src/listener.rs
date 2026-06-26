use crate::mesh::Mesh;
use anyhow::Result;
use std::net::SocketAddr;
use tendril_core::node::Node;
use tendril_core::protocol::{Message, PeerInfo};
use tokio::net::UdpSocket;
use tracing::{info, warn};

/// Listen for incoming UDP messages — heartbeats and Pulse announcements.
pub async fn run(mesh: Mesh, addr: String) -> Result<()> {
    let socket = UdpSocket::bind(&addr).await?;
    info!("Tendril listening on {}", addr);

    let mut buf = vec![0u8; 4096];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, src)) => {
                if let Err(e) = handle(&buf[..len], src, &mesh, &socket).await {
                    warn!("Error handling message from {}: {}", src, e);
                }
            }
            Err(e) => warn!("Socket error: {}", e),
        }
    }
}

async fn handle(data: &[u8], src: SocketAddr, mesh: &Mesh, socket: &UdpSocket) -> Result<()> {
    let msg: Message = serde_json::from_slice(data)?;

    match msg {
        Message::PulseAnnounce {
            node_name,
            addr,
            mac_addr,
        } => {
            info!("Pulse detected: {} at {}", node_name, addr);

            let node = Node::new(&node_name, &addr, mac_addr.as_deref());
            let assigned_id = node.id;
            mesh.register(node).await;

            // Build peer list to send back
            let peers: Vec<PeerInfo> = mesh
                .node_list()
                .await
                .into_iter()
                .filter(|n| n.id != assigned_id)
                .map(|n| PeerInfo {
                    id: n.id,
                    name: n.name,
                    addr: n.addr,
                })
                .collect();

            let invite = Message::MeshInvite {
                mesh_id: mesh.mesh_id().await,
                assigned_id,
                peers,
            };

            let reply = serde_json::to_vec(&invite)?;
            socket.send_to(&reply, src).await?;
            info!("MeshInvite sent to {}", src);
        }

        Message::Heartbeat { node_id, node_name } => {
            info!("Heartbeat: {} ({})", node_name, node_id);
            mesh.heartbeat(node_id).await;
        }

        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::handle;
    use crate::config::Config;
    use crate::mesh::Mesh;
    use tendril_core::protocol::Message;
    use tokio::net::UdpSocket;

    fn test_config() -> Config {
        Config {
            mesh_name: "test".to_string(),
            node_name: "node-a".to_string(),
            listen_addr: "127.0.0.1:0".to_string(),
            heartbeat_timeout_secs: 30,
            heartbeat_interval_secs: 10,
        }
    }

    #[tokio::test]
    async fn pulse_announce_registers_node_and_returns_mesh_invite() {
        let mesh = Mesh::new(test_config());
        let daemon_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let pulse_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let pulse_addr = pulse_socket.local_addr().unwrap();
        let message = Message::PulseAnnounce {
            node_name: "pulse-test".to_string(),
            addr: "127.0.0.1:4100".to_string(),
            mac_addr: Some("aa:bb:cc:dd:ee:ff".to_string()),
        };
        let payload = serde_json::to_vec(&message).unwrap();

        handle(&payload, pulse_addr, &mesh, &daemon_socket)
            .await
            .unwrap();

        let mut buf = [0_u8; 2048];
        let (len, _) = pulse_socket.recv_from(&mut buf).await.unwrap();
        let reply: Message = serde_json::from_slice(&buf[..len]).unwrap();
        let nodes = mesh.node_list().await;

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].name, "pulse-test");
        assert_eq!(nodes[0].mac_addr.as_deref(), Some("aa:bb:cc:dd:ee:ff"));
        match reply {
            Message::MeshInvite {
                assigned_id, peers, ..
            } => {
                assert_eq!(assigned_id, nodes[0].id);
                assert!(peers.is_empty());
            }
            _ => panic!("expected mesh invite"),
        }
    }
}
