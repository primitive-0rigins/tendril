# CLAUDE.md

This is a public GitHub repository under Primitive Origin LLC.

## Privacy Rules

- Never commit real IP addresses — use `192.168.x.x` or `0.0.0.0` as placeholders in examples
- Never commit real MAC addresses — use `aa:bb:cc:dd:ee:ff` as placeholder
- Never commit real hostnames, node names, or network topology from Meda's actual cluster
- Never commit mesh keys, WireGuard private keys, or any secrets
- `tendril.toml` is gitignored — keep it that way, it holds real config
- Keys stored in `~/.tendril/keys` never touch the repo

## What This Repo Is

A showcase of systems-level thinking and Rust engineering from Primitive Origin LLC. The code is real and buildable — but all examples, configs, and documentation use placeholder values only.

## Coding Guidelines

- This is a Rust workspace — all crates share deps via `[workspace.dependencies]` in the root `Cargo.toml`. Do not add duplicate version pins in individual crates.
- Async runtime is `tokio` — stay consistent, do not introduce other runtimes.
- Keep binaries minimal — heavy logic lives in `tendril-core` and gets imported.
- No `unwrap()` in production paths — use `anyhow::Result` and propagate errors.
- All network messages are JSON over the wire — defined in `tendril-core::protocol`.

## Architecture

See `docs/architecture.md` for the full design. Key decisions are settled — do not reopen them without good reason:
- Encryption: `boringtun` (pure Rust WireGuard)
- NAT traversal: hole-punch first, relay fallback
- Relay transport: WebSocket over TCP
- Key storage: encrypted file at `~/.tendril/keys`
