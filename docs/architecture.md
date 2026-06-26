# Tendril Architecture

## Design Principles

1. **Inversion of onboarding** — nodes don't find the mesh, the mesh finds them.
2. **Self-healing** — the mesh responds to failure without human intervention.
3. **Relay-brokered, not relay-dependent** — the relay introduces nodes but steps out of all traffic.
4. **End-to-end encrypted** — private keys never leave the node. The relay is blind to all traffic.
5. **Minimal footprint** — Pulse is a single binary, zero config required.

---

## Three Binaries

| Binary | Role |
|--------|------|
| `pulse` | Beacon — announces presence, holds its own keypair |
| `tendril` | Mesh daemon — manages peers, self-heals, holds its own keypair |
| `tendril-relay` | Broker only — introduces nodes, never touches traffic |

---

## Key Model

Each node (both `tendril` daemons and `pulse` beacons) generates a **WireGuard keypair on first run**.

- The **private key never leaves the node**.
- The relay only ever sees: public key, mesh key, node name, and address.
- After introduction, the relay steps out. All traffic is node-to-node over an encrypted WireGuard tunnel.

If the relay is compromised, an attacker sees public keys and node names — nothing else. They cannot read traffic or impersonate nodes.

---

## Message Flow

### Cross-Network Node Discovery (via Relay)

```
Node A (tendril)        Relay              Node B (pulse)
      │                   │                     │
      │── register ───────►│◄── pulse_announce ──│
      │   {pub_key,        │    {pub_key,         │
      │    mesh_key,       │     mesh_key,        │
      │    name, addr}     │     name, addr}      │
      │                   │                     │
      │         relay verifies mesh_key on both  │
      │                   │                     │
      │◄── introduction ──│                     │
      │    {name, addr,    │                     │
      │     pub_key}       │                     │
      │                   │                     │
      │── WireGuard handshake ─────────────────►│
      │   (direct P2P — relay steps out)        │
      │                   │                     │
      │◄══════════════════════════════════════►│
      │         encrypted tunnel (WireGuard)    │
```

### Local Network Discovery (no relay needed)

On the same subnet, `pulse` broadcasts via UDP multicast. `tendril` hears it and initiates directly — no relay involved. Same key verification applies.

```
Pulse                          Tendril Daemon
  │                                  │
  │── PulseAnnounce (multicast) ────►│
  │   {name, addr, pub_key,          │
  │    mesh_key, mac}                │
  │                             verify mesh_key
  │                             WireGuard handshake
  │◄────────────────── MeshInvite ───│
  │   {mesh_id, assigned_id, peers}  │
  │                                  │
  │── Heartbeat ───────────────────►│  (every N seconds)
```

### Self-Healing / Recovery

```
Tendril Daemon
  │
  ├── heartbeat_watcher (background loop)
  │       │
  │       ├── node.last_seen > timeout?
  │       │       │
  │       │       Yes
  │       │       │
  │       │       ├── node.mac_addr known?
  │       │       │       │
  │       │       │       Yes ──► send WoL magic packet (UDP broadcast :9)
  │       │       │       No  ──► log, mark Recovering
  │       │       │
  │       │       └── mark node state: Recovering
  │       │
  │       └── heartbeat received while Recovering ──► mark Alive
  │
  └── node remains Dead after N recovery attempts ──► remove from registry
```

---

## Ports

| Port | Protocol | Purpose |
|------|----------|---------|
| 7777 | UDP | Tendril daemon listener (unicast) |
| 7778 | UDP multicast | Pulse beacon announcements (local network only) |
| 7779 | TCP/UDP | Relay server (public-facing) |
| 51820 | UDP | WireGuard tunnels (standard port) |
| 9    | UDP broadcast | Wake-on-LAN magic packets |

---

## Mesh Key

The mesh key is a shared secret configured on every node and the relay. It is:
- Required to register with the relay
- Required to be accepted by a `tendril` daemon on local network
- **Not** a substitute for WireGuard encryption — it is only a membership gate

Think of it as the door. WireGuard is the vault inside.

---

## What Remains to Solve Before Building

See `docs/open-questions.md`.
