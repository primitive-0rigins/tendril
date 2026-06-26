# Tendril Project Notes

## Portfolio Role

Tendril demonstrates compact Rust infrastructure work: local discovery, UDP protocol design,
node liveness, recovery state, and inspectable registry persistence.

## What Works

- `tendril-core` shared node/protocol types
- `tendril` mesh daemon
- `pulse` UDP beacon
- JSON-over-UDP `pulse_announce` and `mesh_invite` path
- Heartbeat refresh for known nodes
- Stale-node recovery state transition
- Wake-on-LAN packet path for known MAC addresses
- Local JSON registry persistence
- `--status` and `--nodes` inspection commands
- Static demo report and one-command demo script

## Key Design Choices

- **Local-first MVP:** proves LAN-style discovery before relay/NAT traversal.
- **Small Rust workspace:** separate crates for shared core, daemon, and beacon.
- **Inspectable state:** registry JSON can be read by humans and CLI commands.
- **Deterministic demo:** `tendril --demo` avoids needing a real network setup.

## Roadmap Boundary

Tendril does not yet implement WireGuard tunnels, relay introduction, UDP hole punching,
stateful Pulse identity, mesh keys, or a dashboard. Those remain roadmap layers.

## Proof Path

```bash
./scripts/demo.sh
cargo test
cargo clippy --workspace --all-targets
```
