#!/usr/bin/env bash
# Build hypersdk + hypecli against the checkout (uses root [workspace] + [patch.crates-io]).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
cargo build --release --workspace
echo "hypersdk: $ROOT/target/release/libhypersdk.rlib"
echo "hypecli:  $ROOT/target/release/hypecli"
