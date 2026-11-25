#!/usr/bin/env bash
# Build (tune profile) and compress data/enwik8_25mb.bin. Prints COMPRESSED=<n>.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target-a1}"

SLICE="${ROOT}/data/enwik8_25mb.bin"
OUT="${ROOT}/tune/a1_25mb.compressed"
PROFILE="${TUNE_PROFILE:-tune}"

if [[ ! -f "$SLICE" ]]; then
  echo "a1_slice_eval: missing $SLICE" >&2
  exit 1
fi

# Refuse to start if another prometheus-compress *binary* is running,
# unless A1_ALLOW_COMPRESS=1 (in-sweep after slot wait).
if [[ "${A1_ALLOW_COMPRESS:-}" != "1" ]]; then
  if ps -eo args= | grep -E '(^|/)prometheus-compress( |$)' | grep -vqE 'bash|nix develop|grep'; then
    echo "a1_slice_eval: prometheus-compress already running; abort" >&2
    ps -eo pid=,args= | grep -E '(^|/)prometheus-compress( |$)' | grep -vE 'bash|nix develop|grep' >&2 || true
    exit 2
  fi
fi

cargo build --profile "$PROFILE" -p prometheus-compress -q
TRIPLE="${CARGO_BUILD_TARGET:-x86_64-unknown-linux-gnu}"
BIN="${CARGO_TARGET_DIR}/${TRIPLE}/${PROFILE}/prometheus-compress"
if [[ ! -x "$BIN" ]]; then
  BIN="${CARGO_TARGET_DIR}/${PROFILE}/prometheus-compress"
fi
"$BIN" "$SLICE" "$OUT"
BYTES=$(wc -c < "$OUT" | tr -d ' ')
echo "COMPRESSED=${BYTES}"
