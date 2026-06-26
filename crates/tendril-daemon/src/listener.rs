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

async fn handle(
    data: &[u8],
    src: SocketAddr,
    mesh: &Mesh,
    socket: &UdpSocket,
) -> Result<()> {
    let msg: Message = serde_json::from_slice(data)?;

    match msg {
        Message::PulseAnnounce { node_name, addr, mac_addr } => {
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
