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
