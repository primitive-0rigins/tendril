# Tendril + Pulse

**A self-healing, self-discovering mesh network — built from scratch in Rust.**

> The mesh stays alive on its own. It grows on its own.

---

## Status

Tendril is an early Rust workspace for local mesh discovery, heartbeats, recovery flow,
and a separate Pulse beacon. The current codebase focuses on the local-network path first;
relay, WireGuard, NAT traversal, and dashboard work are tracked in the roadmap.

Working today:

1. Shared JSON protocol types for Pulse announcements, mesh invites, heartbeats, and recovery events
2. In-memory mesh registry with node health state
3. Heartbeat refresh path for known nodes
4. Stale-node recovery scan that marks silent nodes as `Recovering`
5. Wake-on-LAN packet construction for nodes with a MAC address
6. UDP listener path that accepts Pulse announcements and sends mesh invites
7. Standalone Pulse beacon binary
8. Self-contained `tendril --demo` mode with JSON output
9. Static HTML demo report
10. Optional local JSON registry persistence with `--registry`

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
- Broadcasts a UDP announcement every 15 seconds: *"I exist"*
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
git clone https://github.com/primitive-0rigins/tendril.git
cd tendril
cargo build --release
```

For development:

```bash
cargo test
cargo fmt --check
cargo clippy --workspace --all-targets
```

Run the self-contained local demo:

```bash
cargo run -p tendril-daemon --bin tendril -- --demo
cargo run -p tendril-daemon --bin tendril -- --demo-report target/tendril-demo-report.html
```

The demo simulates two Pulse beacons joining an in-memory mesh, refreshes one heartbeat,
marks the stale node as recovering, and prints a JSON report.
The report command writes the same proof path as a static HTML artifact.

### Run Tendril (mesh daemon)

```bash
# Optional: copy and edit the config
cp tendril.example.toml tendril.toml

RUST_LOG=info ./target/release/tendril
```

Persist the local registry across clean daemon restarts:

```bash
RUST_LOG=info ./target/release/tendril --registry .tendril/registry.json
```

### Run Pulse (beacon — on a new machine)

```bash
PULSE_NODE_NAME=my-machine \
PULSE_ADDR=192.168.1.50 \
PULSE_TARGET=255.255.255.255:7777 \
PULSE_MAC=aa:bb:cc:dd:ee:ff \
RUST_LOG=info ./target/release/pulse
```

The mesh will find it within 15 seconds.

For a one-shot local smoke test, set `PULSE_ONCE=1` and `PULSE_TARGET=127.0.0.1:7777`.

---

## Configuration

`tendril.toml` (auto-created with defaults if missing):

```toml
mesh_name = "tendril"
node_name = "node-1"
listen_addr = "0.0.0.0:7777"
heartbeat_timeout_secs = 30
heartbeat_interval_secs = 10
```

---

## Current Decisions

| Concern | Decision |
|---------|----------|
| Language | Rust workspace with small focused crates |
| Local transport | JSON messages over UDP |
| Discovery model | Pulse announces itself; Tendril responds with a mesh invite |
| Node health | `Alive`, `Recovering`, or `Dead` |
| Recovery MVP | Mark stale nodes as recovering; send Wake-on-LAN when MAC is known |
| Demo proof | `tendril --demo` runs without network setup |

---

## Roadmap

- [x] Workspace compiles with serializable protocol types
- [x] Unit tests for protocol, node state, mesh registry, and recovery helpers
- [x] Self-contained demo mode
- [x] Static HTML demo report
- [x] Local Pulse announcement and MeshInvite protocol path
- [x] Daemon-side Pulse handling test with real UDP sockets
- [x] Wake-on-LAN packet path for known MAC addresses
- [x] Local JSON registry persistence
- [ ] WireGuard keypair generation via `boringtun` (per-node, encrypted at rest)
- [ ] `tendril-relay` binary — WebSocket broker, introduction only, blind to traffic
- [ ] UDP hole-punch NAT traversal with relay fallback
- [ ] Any tendril node with public IP can elect itself as relay
- [ ] Stateful Pulse — remembers keypair and assigned ID
- [ ] Mesh key gate on both local and relay paths
- [ ] CLI — `tendril status`, `tendril nodes`, `tendril eject <node>`
- [ ] TUI dashboard

---

## Built by

[Primitive Origin LLC](https://github.com/primitive-0rigins)

> "Just like to build things."
