# Tendril Architecture

Tendril is a compact Rust workspace for experimenting with local mesh discovery and recovery.
The current implementation proves the local-network path first: Pulse announces itself,
Tendril registers the node, heartbeats keep nodes alive, and stale nodes move into recovery.

Relay, WireGuard, NAT traversal, persistent identity, and dashboard work are future layers.

---

## Current Pieces

| Crate | Role |
|---|---|
| `tendril-core` | Shared node state and JSON protocol messages |
| `tendril-daemon` | Mesh registry, UDP listener, heartbeat watcher, recovery flow |
| `pulse` | Standalone beacon that broadcasts Pulse announcements |

---

## Local Discovery Flow

```text
Pulse                          Tendril daemon
  |                                  |
  |-- pulse_announce --------------> |
  |   {node_name, addr, mac_addr}    |
  |                                  | register Node::Alive
  |                                  | build peer list
  |<---------------- mesh_invite --- |
  |   {mesh_id, assigned_id, peers}  |
```

Messages are JSON over UDP. The current protocol is intentionally small and serializable:

- `pulse_announce`
- `mesh_invite`
- `heartbeat`
- `recovery_attempt`

---

## Health And Recovery

```text
Node joins as Alive
      |
      v
Heartbeat refreshes last_seen
      |
      v
Watcher sees last_seen > timeout
      |
      v
MAC known? yes -> send Wake-on-LAN packet
      |
      v
Mark node Recovering
      |
      v
Future heartbeat marks it Alive again
```

The current recovery implementation sends a Wake-on-LAN magic packet when a MAC address is
known. If no MAC is known, it still marks the node as `Recovering`, which keeps the state
machine observable without pretending full recovery has happened.

---

## Demo Mode

`tendril --demo` runs without network setup. It creates an in-memory mesh, registers two
simulated Pulse nodes, refreshes one heartbeat, marks the stale node as recovering, and prints a
JSON report. `tendril --demo-report <path>` writes the same proof path as static HTML.

```bash
cargo run -p tendril-daemon --bin tendril -- --demo
cargo run -p tendril-daemon --bin tendril -- --demo-report target/tendril-demo-report.html
```

This is the portfolio proof path for the current MVP.

---

## Registry Persistence

The daemon can persist the local registry as JSON:

```bash
tendril --registry .tendril/registry.json
```

On startup, existing nodes are loaded into the in-memory mesh. On clean shutdown, the current
node list is saved back to the registry file. This is intentionally local and inspectable; a
distributed durable registry is not implemented yet.

The same file can be inspected without starting the daemon:

```bash
tendril --status --registry .tendril/registry.json
tendril --nodes --registry .tendril/registry.json
```

---

## Roadmap Layers

- Stateful Pulse identity
- Mesh key gate
- WireGuard key generation and tunnel setup
- NAT traversal and relay introduction
- CLI status commands
- TUI or static report for node state
