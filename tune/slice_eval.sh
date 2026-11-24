#!/usr/bin/env bash
# Build (tune profile) and compress the fixed enwik8 slice. Prints COMPRESSED=<n>.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SLICE_OFFSET="${SLICE_OFFSET:-8000000}"
SLICE_LEN="${SLICE_LEN:-25000000}"
SLICE="${ROOT}/tune/enwik8.slice"
OUT="${ROOT}/tune/enwik8.slice.compressed"
PROFILE="${TUNE_PROFILE:-tune}"
TRIPLE="${CARGO_BUILD_TARGET:-x86_64-unknown-linux-gnu}"

if [[ ! -f "$SLICE" ]]; then
  echo "slice_eval: extracting offset=${SLICE_OFFSET} len=${SLICE_LEN}" >&2
  dd if=enwik8 of="$SLICE" bs=1000000 skip=$((SLICE_OFFSET / 1000000)) count=$((SLICE_LEN / 1000000)) status=none
fi

cargo build --profile "$PROFILE" -p prometheus-compress -q

# build-std + explicit [build].target places bins under target/<triple>/<profile>/.
BIN=""
for cand in \
  "target/${TRIPLE}/${PROFILE}/prometheus-compress" \
  "target/${PROFILE}/prometheus-compress"
do
  if [[ -x "$cand" ]]; then
    BIN="$cand"
    break
  fi
done
if [[ -z "$BIN" ]]; then
  echo "slice_eval: prometheus-compress not found after build" >&2
  exit 1
fi

# Keep a convenience copy for older tooling (judge-like paths).
mkdir -p "target/${PROFILE}"
cp -f "$BIN" "target/${PROFILE}/prometheus-compress" 2>/dev/null || true

"$BIN" "$SLICE" "$OUT"
BYTES=$(wc -c < "$OUT" | tr -d ' ')
echo "COMPRESSED=${BYTES}"
