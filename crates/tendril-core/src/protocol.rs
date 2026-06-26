use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Messages exchanged between Tendril nodes and Pulse beacons.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    /// Sent by a Pulse beacon — "I exist, come find me."
    PulseAnnounce {
        node_name: String,
        addr: String,
        mac_addr: Option<String>,
    },
    /// Sent by Tendril daemon in response to a PulseAnnounce — "we see you, you're in."
    MeshInvite {
        mesh_id: Uuid,
        assigned_id: Uuid,
        peers: Vec<PeerInfo>,
    },
    /// Periodic heartbeat from a node — "still alive."
    Heartbeat {
        node_id: Uuid,
        node_name: String,
    },
    /// Sent when a node is being marked for recovery.
    RecoveryAttempt {
        target_id: Uuid,
        target_name: String,
    },
}

/// Minimal peer info shared during mesh invite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: Uuid,
    pub name: String,
    pub addr: String,
}
