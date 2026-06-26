# Tendril + Pulse

**A self-healing, self-discovering mesh network — built from scratch in Rust.**

> The mesh stays alive on its own. It grows on its own.

---

## The Problem

Most mesh systems are push — you configure a node, tell it where the mesh is, and manually onboard it. If a node goes silent, someone has to notice and act.

Tendril inverts both of those.

---

## Two Programs. One Idea.

### Tendril
The mesh daemon. Runs on every node in your network. It:
- Maintains a live registry of all connected nodes
- Sends and receives heartbeats
- Detects when a node goes silent
- Attempts recovery automatically (Wake-on-LAN, reconnect)
- Listens for Pulse beacons and pulls new nodes into the mesh

### Pulse
The beacon. A small, standalone program you drop on any machine. It:
- Knows nothing about the mesh
- Broadcasts a UDP announcement on the local network every 15 seconds: *"I exist"*
- Waits

When Tendril hears a Pulse, it reaches out, assigns the node an ID, and sends it the peer list. The node is in. No manual configuration.

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Local Network                     │
│                                                     │
│   [Node A]          [Node B]          [Node C]      │
│  tendril-daemon   tendril-daemon    tendril-daemon  │
│       │                │                 │          │
│       └────────────────┴─────────────────┘          │
│              heartbeat mesh (UDP)                   │
│                                                     │
│                  [New Machine]                      │
│                  pulse beacon                       │
│                       │                             │
│              broadcast: "I exist"                   │
│                       │                             │
│              Tendril hears it ──► MeshInvite        │
│              Node is now part of the mesh           │
└─────────────────────────────────────────────────────┘
```

### Message Protocol

All messages are JSON over UDP.

| Message | Direction | Purpose |
|---|---|---|
| `pulse_announce` | Pulse → Network | "I exist, come find me" |
| `mesh_invite` | Tendril → Pulse | "You're in. Here are your peers." |
| `heartbeat` | Node → Tendril | "Still alive." |
| `recovery_attempt` | Internal | Node went silent, attempting WoL |

### Recovery Flow

```
Node goes silent
      │
      ▼
heartbeat_watcher detects timeout
      │
      ▼
MAC address known? ──Yes──► Send WoL magic packet
      │                            │
      No                     Wait for heartbeat
      │                            │
      ▼                            ▼
Mark: Recovering          Alive again? ──Yes──► Mark: Alive
                                   │
                                   No
                                   ▼
                              Mark: Dead
```

---

## Project Structure

```
tendril/
├── crates/
│   ├── tendril-core/       # Shared types and protocol
│   ├── tendril-daemon/     # Mesh daemon binary
│   └── pulse/              # Beacon binary
├── docs/
│   └── architecture.md
└── Cargo.toml              # Workspace
```

---

## Getting Started

### Build

```bash
cargo build --release
```

### Run Tendril (mesh daemon)

```bash
# Optional: copy and edit the config
cp tendril.example.toml tendril.toml

RUST_LOG=info ./target/release/tendril
```

### Run Pulse (beacon — on a new machine)

```bash
PULSE_NODE_NAME=my-machine \
PULSE_ADDR=192.168.1.50 \
PULSE_MAC=aa:bb:cc:dd:ee:ff \
RUST_LOG=info ./target/release/pulse
```

The mesh will find it within 15 seconds.

---

## Configuration

`tendril.toml` (auto-created with defaults if missing):

```toml
mesh_name = "tendril"
node_name = "node-1"
listen_addr = "0.0.0.0:7777"
beacon_multicast = "224.0.0.251:7778"
heartbeat_timeout_secs = 30
heartbeat_interval_secs = 10
```

---

## Architecture Decisions

| Concern | Decision |
|---------|----------|
| Encryption | `boringtun` — pure Rust WireGuard, no kernel module, runs anywhere |
| NAT traversal | UDP hole-punch attempted first, `tendril-relay` as fallback |
| Relay | Any node with a public IP can act as relay — no single point of failure |
| Key storage | Encrypted file (`~/.tendril/keys`) — portable, protected at rest |
| Pulse identity | Stateful — remembers keypair and assigned node ID across restarts |
| Relay transport | WebSocket over TCP — punches through firewalls, works on port 443 |

---

## Roadmap

- [ ] WireGuard keypair generation via `boringtun` (per-node, encrypted at rest)
- [ ] `tendril-relay` binary — WebSocket broker, introduction only, blind to traffic
- [ ] UDP hole-punch NAT traversal with relay fallback
- [ ] Any tendril node with public IP can elect itself as relay
- [ ] Persistent node registry (survive daemon restarts)
- [ ] Stateful Pulse — remembers keypair and assigned ID
- [ ] Mesh key gate on both local and relay paths
- [ ] CLI — `tendril status`, `tendril nodes`, `tendril eject <node>`
- [ ] TUI dashboard

---

## Built by

[Primitive Origin LLC](https://github.com/primitive-0rigins)

> "Just like to build things."
