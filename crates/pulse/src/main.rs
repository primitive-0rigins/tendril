use anyhow::Result;
use tendril_core::protocol::Message;
use tokio::net::UdpSocket;
use tracing::info;

const MULTICAST_ADDR: &str = "224.0.0.251:7778";
const INTERVAL_SECS: u64 = 15;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let node_name = std::env::var("PULSE_NODE_NAME")
        .unwrap_or_else(|_| hostname());
    let addr = std::env::var("PULSE_ADDR")
        .unwrap_or_else(|_| local_ip());
    let mac_addr = std::env::var("PULSE_MAC").ok();

    info!("Pulse beacon starting — name: {}, addr: {}", node_name, addr);

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;

    loop {
        let msg = Message::PulseAnnounce {
            node_name: node_name.clone(),
            addr: addr.clone(),
            mac_addr: mac_addr.clone(),
        };

        let payload = serde_json::to_vec(&msg)?;
        socket.send_to(&payload, MULTICAST_ADDR).await?;
        info!("Pulse sent to {}", MULTICAST_ADDR);

        tokio::time::sleep(tokio::time::Duration::from_secs(INTERVAL_SECS)).await;
    }
}

fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "unknown-node".to_string())
        .trim()
        .to_string()
}

fn local_ip() -> String {
    // Naive fallback — real impl would enumerate interfaces
    "0.0.0.0".to_string()
}
