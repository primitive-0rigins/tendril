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
    Heartbeat { node_id: Uuid, node_name: String },
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

#[cfg(test)]
mod tests {
    use super::{Message, PeerInfo};
    use uuid::Uuid;

    #[test]
    fn pulse_announce_round_trips_as_tagged_json() {
        let message = Message::PulseAnnounce {
            node_name: "demo-node".to_string(),
            addr: "192.168.1.20:7777".to_string(),
            mac_addr: Some("aa:bb:cc:dd:ee:ff".to_string()),
        };

        let json = serde_json::to_string(&message).unwrap();
        let decoded: Message = serde_json::from_str(&json).unwrap();

        assert!(json.contains("\"type\":\"pulse_announce\""));
        match decoded {
            Message::PulseAnnounce {
                node_name,
                addr,
                mac_addr,
            } => {
                assert_eq!(node_name, "demo-node");
                assert_eq!(addr, "192.168.1.20:7777");
                assert_eq!(mac_addr.as_deref(), Some("aa:bb:cc:dd:ee:ff"));
            }
            _ => panic!("expected pulse announcement"),
        }
    }

    #[test]
    fn mesh_invite_round_trips_with_uuid_peers() {
        let peer_id = Uuid::new_v4();
        let message = Message::MeshInvite {
            mesh_id: Uuid::new_v4(),
            assigned_id: Uuid::new_v4(),
            peers: vec![PeerInfo {
                id: peer_id,
                name: "peer-a".to_string(),
                addr: "10.0.0.2:7777".to_string(),
            }],
        };

        let json = serde_json::to_string(&message).unwrap();
        let decoded: Message = serde_json::from_str(&json).unwrap();

        match decoded {
            Message::MeshInvite { peers, .. } => {
                assert_eq!(peers.len(), 1);
                assert_eq!(peers[0].id, peer_id);
                assert_eq!(peers[0].name, "peer-a");
            }
            _ => panic!("expected mesh invite"),
        }
    }
}
