use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub mesh_name: String,
    pub node_name: String,
    pub listen_addr: String,
    /// Multicast address Pulse beacons broadcast on.
    pub beacon_multicast: String,
    /// Seconds before a silent node is marked Recovering.
    pub heartbeat_timeout_secs: u64,
    /// Seconds between heartbeat checks.
    pub heartbeat_interval_secs: u64,
}

pub fn load(path: &str) -> Result<Config> {
    let raw = fs::read_to_string(path).unwrap_or_else(|_| DEFAULT_CONFIG.to_string());
    Ok(toml::from_str(&raw)?)
}

const DEFAULT_CONFIG: &str = r#"
mesh_name = "tendril"
node_name = "node-1"
listen_addr = "0.0.0.0:7777"
beacon_multicast = "224.0.0.251:7778"
heartbeat_timeout_secs = 30
heartbeat_interval_secs = 10
"#;
