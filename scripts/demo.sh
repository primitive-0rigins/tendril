#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-"$ROOT/target/tendril-demo"}"
BIN="$ROOT/target/release/tendril"
REGISTRY="$OUT/registry.json"
REPORT="$OUT/report.html"
DEMO_JSON="$OUT/demo.json"

mkdir -p "$OUT"

echo "== build release binaries =="
cargo build --release --manifest-path "$ROOT/Cargo.toml"

echo
echo "== run in-memory mesh demo =="
"$BIN" --demo | tee "$DEMO_JSON"

echo
echo "== write static demo report =="
"$BIN" --demo-report "$REPORT"

echo
echo "== create inspectable registry =="
cat > "$REGISTRY" <<'JSON'
[
  {
    "id": "00000000-0000-4000-8000-000000000001",
    "name": "demo-alpha",
    "addr": "127.0.0.1:4101",
    "state": "Alive",
    "last_seen": "2026-06-26T00:00:00Z",
    "mac_addr": null
  },
  {
    "id": "00000000-0000-4000-8000-000000000002",
    "name": "demo-beta",
    "addr": "127.0.0.1:4102",
    "state": "Recovering",
    "last_seen": "2026-06-26T00:00:00Z",
    "mac_addr": "aa:bb:cc:dd:ee:ff"
  }
]
JSON

echo
echo "== inspect registry status =="
"$BIN" --status --registry "$REGISTRY"

echo
echo "== list registry nodes =="
"$BIN" --nodes --registry "$REGISTRY"

echo
echo "Demo complete."
echo "output: $OUT"
echo "json: $DEMO_JSON"
echo "report: $REPORT"
echo "registry: $REGISTRY"
