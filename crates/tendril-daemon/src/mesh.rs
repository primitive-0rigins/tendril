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

    pub async fn recover_silent_nodes_once(&self) -> Vec<Node> {
        let timeout = self.config().await.heartbeat_timeout_secs;
        let now = Utc::now();
        let nodes = self.node_list().await;
        let mut recovering = Vec::new();

        for node in nodes {
            let elapsed = (now - node.last_seen).num_seconds() as u64;
            if elapsed > timeout && node.state == NodeState::Alive {
                warn!("Node silent: {} — starting recovery", node.name);
                recovery::attempt(&node).await;

                let mut inner = self.0.write().await;
                if let Some(n) = inner.nodes.get_mut(&node.id) {
                    n.state = NodeState::Recovering;
                    recovering.push(n.clone());
                }
            }
        }

        recovering
    }
}

/// Background task — watches for silent nodes and triggers recovery.
pub async fn run_heartbeat_watcher(mesh: Mesh) {
    loop {
        let interval = mesh.config().await.heartbeat_interval_secs;
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;

        mesh.recover_silent_nodes_once().await;
    }
}

#[cfg(test)]
mod tests {
    use super::Mesh;
    use crate::config::Config;
    use chrono::{Duration, Utc};
    use tendril_core::node::{Node, NodeState};

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
    async fn register_adds_node_to_mesh() {
        let mesh = Mesh::new(test_config());
        let node = Node::new("peer-a", "127.0.0.1:7777", None);
        let node_id = node.id;

        mesh.register(node).await;

        let nodes = mesh.node_list().await;
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, node_id);
        assert_eq!(nodes[0].name, "peer-a");
    }

    #[tokio::test]
    async fn heartbeat_marks_recovering_node_alive() {
        let mesh = Mesh::new(test_config());
        let mut node = Node::new("peer-a", "127.0.0.1:7777", None);
        node.state = NodeState::Recovering;
        node.last_seen = Utc::now() - Duration::seconds(60);
        let node_id = node.id;
        mesh.register(node).await;

        mesh.heartbeat(node_id).await;

        let nodes = mesh.node_list().await;
        assert_eq!(nodes[0].state, NodeState::Alive);
        assert!(nodes[0].last_seen > Utc::now() - Duration::seconds(5));
    }

    #[tokio::test]
    async fn recover_silent_nodes_once_marks_stale_node_recovering() {
        let mesh = Mesh::new(test_config());
        let mut node = Node::new("peer-a", "127.0.0.1:7777", None);
        node.last_seen = Utc::now() - Duration::seconds(60);
        let node_id = node.id;
        mesh.register(node).await;

        let recovering = mesh.recover_silent_nodes_once().await;

        let nodes = mesh.node_list().await;
        assert_eq!(recovering.len(), 1);
        assert_eq!(recovering[0].id, node_id);
        assert_eq!(nodes[0].state, NodeState::Recovering);
    }
}
