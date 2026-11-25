#!/usr/bin/env bash
# Build (tune) and compress tune/b3_25mb.bin. Prints COMPRESSED=<n>.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target-b3}"

SLICE="$ROOT/tune/b3_25mb.bin"
OUT="$ROOT/tune/b3_25mb.trial.out"
LABEL="${1:-trial}"

if [[ ! -f "$SLICE" ]]; then
  echo "missing $SLICE" >&2
  exit 1
fi

nix develop -c cargo build --profile tune -p prometheus-compress -q
TRIPLE="${CARGO_BUILD_TARGET:-x86_64-unknown-linux-gnu}"
BIN="$CARGO_TARGET_DIR/${TRIPLE}/tune/prometheus-compress"
if [[ ! -x "$BIN" ]]; then
  BIN="$CARGO_TARGET_DIR/tune/prometheus-compress"
fi
rm -f "$OUT"
"$BIN" "$SLICE" "$OUT"
BYTES=$(wc -c < "$OUT" | tr -d ' ')
echo "LABEL=${LABEL} COMPRESSED=${BYTES}"
