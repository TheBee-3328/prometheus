#!/usr/bin/env bash
# Patch PROSE_* params, rebuild, compress tune/b5_25mb.bin, append CSV row.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace/prometheus-b5-prose/target-b5}"

MODE="${1:?mode}"
MIX="${2:?mix}"
GATE="${3:?gate}"
BITS="${4:-22}"
WCTX="${5:-0}"
SM="${6:-0}"
WHIT="${7:-0}"
TAG="${8:-m${MODE}_w${MIX}_g${GATE}_b${BITS}_c${WCTX}_sm${SM}_wh${WHIT}}"
BASELINE="${BASELINE:-5699220}"
CSV="${ROOT}/tune/b5_trials.csv"
PARAMS="${ROOT}/crates/prometheus-core/src/params.rs"

sed -i -E \
  -e "s/^pub const PROSE_BITS: usize = [0-9]+;/pub const PROSE_BITS: usize = ${BITS};/" \
  -e "s/^pub const PROSE_MODE: u32 = [0-9]+;/pub const PROSE_MODE: u32 = ${MODE};/" \
  -e "s/^pub const PROSE_MIX_WEIGHT: u32 = [0-9]+;/pub const PROSE_MIX_WEIGHT: u32 = ${MIX};/" \
  -e "s/^pub const PROSE_MATCH_GATE: u32 = [0-9]+;/pub const PROSE_MATCH_GATE: u32 = ${GATE};/" \
  -e "s/^pub const WORD_CTX_MODE: u32 = [0-9]+;/pub const WORD_CTX_MODE: u32 = ${WCTX};/" \
  -e "s/^pub const PROSE_SM: u32 = [0-9]+;/pub const PROSE_SM: u32 = ${SM};/" \
  -e "s/^pub const WORD_HIT_GATE: u32 = [0-9]+;/pub const WORD_HIT_GATE: u32 = ${WHIT};/" \
  "$PARAMS"

nix develop -c bash -c "
  set -euo pipefail
  export CARGO_TARGET_DIR='$CARGO_TARGET_DIR'
  cargo build --profile tune -p prometheus-compress -q
"

BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/tune/prometheus-compress"
OUT="${ROOT}/tune/b5_${TAG}.comp"
START=$(date +%s)
"$BIN" tune/b5_25mb.bin "$OUT"
END=$(date +%s)
BYTES=$(wc -c < "$OUT" | tr -d ' ')
DELTA=$((BYTES - BASELINE))
echo "TAG=${TAG} COMPRESSED=${BYTES} DELTA=${DELTA} ELAPSED=$((END-START))s"
if [[ ! -f "$CSV" ]]; then
  echo "tag,mode,mix,gate,bits,wctx,sm,whit,compressed,delta,elapsed_s" > "$CSV"
fi
echo "${TAG},${MODE},${MIX},${GATE},${BITS},${WCTX},${SM},${WHIT},${BYTES},${DELTA},$((END-START))" >> "$CSV"
