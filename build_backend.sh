#!/usr/bin/env bash
# ============================================================================
# Polaris (fork) - Backend (Rust-Server) bauen.
# Erzeugt target/release/polaris, das spaeter ins Docker-Image kopiert wird.
#   ./build_backend.sh
# ============================================================================
set -euo pipefail
cd "$(dirname "$0")"

echo "==> Backend (Rust) bauen: cargo build --release ..."
cargo build --release

if [ ! -f target/release/polaris ]; then
  echo "FEHLER: target/release/polaris wurde nicht erzeugt." >&2
  exit 1
fi

echo "==> OK: Backend-Binary vorhanden:"
ls -la target/release/polaris