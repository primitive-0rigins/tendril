use crate::config::Config;
use crate::mesh::Mesh;
use anyhow::Result;
use chrono::{Duration, Utc};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tendril_core::node::{Node, NodeState};

#[derive(Debug, Serialize)]
pub struct DemoReport {
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
    let report = build_report().await;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

pub async fn write_html(path: impl AsRef<Path>) -> Result<PathBuf> {
    let report = build_report().await;
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, render_html(&report))?;
    Ok(path.to_path_buf())
}

async fn build_report() -> DemoReport {
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

    DemoReport {
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
    }
}

fn render_html(report: &DemoReport) -> String {
    let events = report
        .events
        .iter()
        .map(|event| format!("<li>{}</li>", escape(event)))
        .collect::<Vec<_>>()
        .join("\n");
    let nodes = report
        .nodes
        .iter()
        .map(|node| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{:?}</td><td>{}</td></tr>",
                escape(&node.name),
                escape(&node.addr),
                node.state,
                escape(node.mac_addr.as_deref().unwrap_or("none"))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let recovering = if report.recovering.is_empty() {
        "none".to_string()
    } else {
        escape(&report.recovering.join(", "))
    };

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Tendril Demo Report</title>
  <style>
    body {{
      margin: 0;
      font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background: #f5f7fb;
      color: #161a22;
    }}
    main {{
      max-width: 980px;
      margin: 0 auto;
      padding: 32px 20px 48px;
    }}
    h1, h2 {{
      margin: 0 0 14px;
    }}
    section {{
      margin-top: 28px;
    }}
    .stats {{
      display: flex;
      gap: 12px;
      flex-wrap: wrap;
    }}
    .stat {{
      min-width: 150px;
      padding: 12px 14px;
      border: 1px solid #d6dde8;
      border-radius: 8px;
      background: #ffffff;
    }}
    .label {{
      display: block;
      color: #5d6878;
      font-size: 12px;
      text-transform: uppercase;
    }}
    .value {{
      display: block;
      margin-top: 4px;
      font-size: 24px;
      font-weight: 700;
    }}
    table {{
      width: 100%;
      border-collapse: collapse;
      background: #ffffff;
      border: 1px solid #d6dde8;
      border-radius: 8px;
      overflow: hidden;
    }}
    th, td {{
      padding: 10px 12px;
      text-align: left;
      border-bottom: 1px solid #e7ecf3;
    }}
    li {{
      margin: 8px 0;
    }}
  </style>
</head>
<body>
  <main>
    <h1>Tendril Demo Report</h1>
    <div class="stats">
      <div class="stat"><span class="label">Mesh</span><span class="value">{mesh}</span></div>
      <div class="stat"><span class="label">Nodes</span><span class="value">{nodes_count}</span></div>
      <div class="stat"><span class="label">Recovering</span><span class="value">{recovering}</span></div>
    </div>
    <section>
      <h2>Events</h2>
      <ol>{events}</ol>
    </section>
    <section>
      <h2>Nodes</h2>
      <table>
        <thead><tr><th>Name</th><th>Address</th><th>State</th><th>MAC</th></tr></thead>
        <tbody>{nodes}</tbody>
      </table>
    </section>
  </main>
</body>
</html>
"#,
        mesh = escape(&report.mesh_name),
        nodes_count = report.nodes.len(),
        recovering = recovering,
        events = events,
        nodes = nodes
    )
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
