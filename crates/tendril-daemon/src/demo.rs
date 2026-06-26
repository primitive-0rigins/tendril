use crate::config::Config;
use crate::mesh::Mesh;
use anyhow::Result;
use chrono::{Duration, Utc};
use serde::Serialize;
use tendril_core::node::{Node, NodeState};

#[derive(Debug, Serialize)]
struct DemoReport {
    mesh_name: String,
    mesh_id: String,
    events: Vec<String>,
    nodes: Vec<DemoNode>,
    recovering: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DemoNode {
    name: String,
    addr: String,
    state: NodeState,
    mac_addr: Option<String>,
}

pub async fn run() -> Result<()> {
    let cfg = Config {
        mesh_name: "tendril-demo".to_string(),
        node_name: "demo-daemon".to_string(),
        listen_addr: "127.0.0.1:7777".to_string(),
        heartbeat_timeout_secs: 5,
        heartbeat_interval_secs: 1,
    };
    let mesh = Mesh::new(cfg.clone());
    let mut events = Vec::new();

    let alpha = Node::new("pulse-alpha", "127.0.0.1:4101", None);
    let alpha_id = alpha.id;
    mesh.register(alpha).await;
    events.push("pulse-alpha announced and joined the mesh".to_string());

    let mut beta = Node::new("pulse-beta", "127.0.0.1:4102", None);
    beta.last_seen = Utc::now() - Duration::seconds(30);
    mesh.register(beta).await;
    events.push("pulse-beta joined, then missed the heartbeat window".to_string());

    mesh.heartbeat(alpha_id).await;
    events.push("pulse-alpha heartbeat refreshed its alive state".to_string());

    let recovering = mesh.recover_silent_nodes_once().await;
    events.push("recovery scan marked stale nodes as recovering".to_string());

    let report = DemoReport {
        mesh_name: cfg.mesh_name,
        mesh_id: mesh.mesh_id().await.to_string(),
        events,
        nodes: mesh
            .node_list()
            .await
            .into_iter()
            .map(|node| DemoNode {
                name: node.name,
                addr: node.addr,
                state: node.state,
                mac_addr: node.mac_addr,
            })
            .collect(),
        recovering: recovering.into_iter().map(|node| node.name).collect(),
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
