use crate::config::Config;
use crate::recovery;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tendril_core::node::{Node, NodeState};
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct Mesh(Arc<RwLock<MeshInner>>);

struct MeshInner {
    pub id: Uuid,
    pub config: Config,
    pub nodes: HashMap<Uuid, Node>,
}

impl Mesh {
    pub fn new(config: Config) -> Self {
        Mesh(Arc::new(RwLock::new(MeshInner {
            id: Uuid::new_v4(),
            config,
            nodes: HashMap::new(),
        })))
    }

    pub async fn register(&self, node: Node) {
        let mut inner = self.0.write().await;
        info!("Node joined mesh: {} ({})", node.name, node.addr);
        inner.nodes.insert(node.id, node);
    }

    pub async fn heartbeat(&self, node_id: Uuid) {
        let mut inner = self.0.write().await;
        if let Some(node) = inner.nodes.get_mut(&node_id) {
            node.touch();
        }
    }

    pub async fn node_list(&self) -> Vec<Node> {
        self.0.read().await.nodes.values().cloned().collect()
    }

    pub async fn mesh_id(&self) -> Uuid {
        self.0.read().await.id
    }

    pub async fn config(&self) -> Config {
        self.0.read().await.config.clone()
    }
}

/// Background task — watches for silent nodes and triggers recovery.
pub async fn run_heartbeat_watcher(mesh: Mesh) {
    loop {
        let interval = mesh.config().await.heartbeat_interval_secs;
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;

        let timeout = mesh.config().await.heartbeat_timeout_secs;
        let now = Utc::now();

        let nodes = mesh.node_list().await;
        for node in nodes {
            let elapsed = (now - node.last_seen).num_seconds() as u64;
            if elapsed > timeout && node.state == NodeState::Alive {
                warn!("Node silent: {} — starting recovery", node.name);
                recovery::attempt(&node).await;

                let mut inner = mesh.0.write().await;
                if let Some(n) = inner.nodes.get_mut(&node.id) {
                    n.state = NodeState::Recovering;
                }
            }
        }
    }
}
