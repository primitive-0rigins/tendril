use anyhow::Result;
use tendril_core::protocol::Message;
use tokio::net::UdpSocket;
use tracing::info;

const DEFAULT_TARGET_ADDR: &str = "255.255.255.255:7777";
const INTERVAL_SECS: u64 = 15;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let node_name = std::env::var("PULSE_NODE_NAME").unwrap_or_else(|_| hostname());
    let addr = std::env::var("PULSE_ADDR").unwrap_or_else(|_| local_ip());
    let mac_addr = std::env::var("PULSE_MAC").ok();
    let target_addr =
        std::env::var("PULSE_TARGET").unwrap_or_else(|_| DEFAULT_TARGET_ADDR.to_string());
    let once = std::env::var("PULSE_ONCE").ok().as_deref() == Some("1");

    info!(
        "Pulse beacon starting — name: {}, addr: {}",
        node_name, addr
    );

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;

    loop {
        let msg = Message::PulseAnnounce {
            node_name: node_name.clone(),
            addr: addr.clone(),
            mac_addr: mac_addr.clone(),
        };

        let payload = serde_json::to_vec(&msg)?;
        socket.send_to(&payload, &target_addr).await?;
        info!("Pulse sent to {}", target_addr);
        if once {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(INTERVAL_SECS)).await;
    }

    Ok(())
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
