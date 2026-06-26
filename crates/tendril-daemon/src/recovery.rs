use tendril_core::node::Node;
use tracing::{info, warn};

/// Attempt to recover a silent node.
/// Strategy: Wake-on-LAN if MAC is known, then wait and recheck.
pub async fn attempt(node: &Node) {
    info!("Recovery attempt for: {} ({})", node.name, node.addr);

    if let Some(mac) = &node.mac_addr {
        info!("Sending Wake-on-LAN to {}", mac);
        if let Err(e) = send_wol(mac).await {
            warn!("WoL failed for {}: {}", node.name, e);
        }
    } else {
        info!("No MAC address for {} — skipping WoL", node.name);
    }
}

/// Send a Wake-on-LAN magic packet to the given MAC address.
async fn send_wol(mac: &str) -> anyhow::Result<()> {
    use tokio::net::UdpSocket;

    let mac_bytes = parse_mac(mac)?;

    // Magic packet: 6x 0xFF followed by 16x MAC address
    let mut packet = vec![0xFFu8; 6];
    for _ in 0..16 {
        packet.extend_from_slice(&mac_bytes);
    }

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;
    socket.send_to(&packet, "255.255.255.255:9").await?;

    Ok(())
}

fn parse_mac(mac: &str) -> anyhow::Result<[u8; 6]> {
    let parts: Vec<u8> = mac
        .split([':', '-'])
        .map(|s| u8::from_str_radix(s, 16))
        .collect::<Result<_, _>>()
        .map_err(|e| anyhow::anyhow!("Invalid MAC: {}", e))?;

    if parts.len() != 6 {
        anyhow::bail!("MAC must have 6 octets");
    }

    Ok([parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]])
}
