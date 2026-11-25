#!/usr/bin/env bash
# Build (tune profile) and compress tune/b4_25mb.bin. Prints COMPRESSED=<n>.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target-b4}"

SLICE="${ROOT}/tune/b4_25mb.bin"
OUT="${ROOT}/tune/b4_25mb.compressed"
PROFILE="${TUNE_PROFILE:-tune}"

if [[ ! -f "$SLICE" ]]; then
  echo "b4_slice_eval: missing $SLICE" >&2
  exit 1
fi

if [[ "${B4_ALLOW_COMPRESS:-}" != "1" ]]; then
  if ps -eo args= | grep -E '(^|/)prometheus-compress( |$)' | grep -vqE 'bash|nix develop|grep'; then
    echo "b4_slice_eval: prometheus-compress already running; abort" >&2
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
