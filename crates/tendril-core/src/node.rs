use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The health state of a mesh node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeState {
    /// Node is reachable and healthy.
    Alive,
    /// Node missed heartbeats — recovery in progress.
    Recovering,
    /// Node is unreachable and recovery has failed.
    Dead,
}

/// A node registered in the Tendril mesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub addr: String,
    pub state: NodeState,
    pub last_seen: DateTime<Utc>,
    pub mac_addr: Option<String>,
}

impl Node {
    pub fn new(name: &str, addr: &str, mac_addr: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            addr: addr.to_string(),
            state: NodeState::Alive,
            last_seen: Utc::now(),
            mac_addr: mac_addr.map(|s| s.to_string()),
        }
    }

    pub fn touch(&mut self) {
        self.last_seen = Utc::now();
        self.state = NodeState::Alive;
    }
}

#[cfg(test)]
mod tests {
    use super::{Node, NodeState};
    use chrono::{Duration, Utc};

    #[test]
    fn new_node_starts_alive_with_identity() {
        let node = Node::new("demo", "127.0.0.1:7777", Some("aa:bb:cc:dd:ee:ff"));

        assert_eq!(node.name, "demo");
        assert_eq!(node.addr, "127.0.0.1:7777");
        assert_eq!(node.state, NodeState::Alive);
        assert_eq!(node.mac_addr.as_deref(), Some("aa:bb:cc:dd:ee:ff"));
    }

    #[test]
    fn touch_marks_node_alive_and_updates_timestamp() {
        let mut node = Node::new("demo", "127.0.0.1:7777", None);
        node.state = NodeState::Recovering;
        node.last_seen = Utc::now() - Duration::seconds(60);
        let before = node.last_seen;

        node.touch();

        assert_eq!(node.state, NodeState::Alive);
        assert!(node.last_seen > before);
    }
}
