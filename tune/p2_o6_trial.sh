#!/usr/bin/env bash
# Patch ORDER6_* (and optional mild O3/O4), rebuild tune, compress slice25.bin.
# Restores tip params after each trial unless KEEP_PARAMS=1.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace/prometheus-p2-o6/target-p2-o6}"
OUT_DIR="${OUT_DIR:-/tmp/p2-o6-out}"
CSV="${ROOT}/tune/p2_o6_trials.csv"
PARAMS="${ROOT}/crates/prometheus-core/src/params.rs"
TIP_PARAMS="${OUT_DIR}/params_tip.rs"
SLICE="${ROOT}/tune/slice25.bin"
BASELINE="${BASELINE:-0}"

TAG="${1:?tag}"
O6_BITS="${2:?ORDER6_BITS}"
O6_MIX="${3:?ORDER6_MIX_WEIGHT}"
O3_BITS="${4:-}"
O4_BITS="${5:-}"

mkdir -p "$OUT_DIR"
if [[ ! -f "$TIP_PARAMS" ]]; then
  cp "$PARAMS" "$TIP_PARAMS"
fi
if [[ ! -f "$SLICE" ]]; then
  echo "missing $SLICE" >&2
  exit 1
fi

# Restore tip first so each trial is absolute vs tip (unless intentional stack).
cp "$TIP_PARAMS" "$PARAMS"

sed -i -E \
  -e "s/^pub const ORDER6_BITS: usize = [0-9]+;/pub const ORDER6_BITS: usize = ${O6_BITS};/" \
  -e "s/^pub const ORDER6_MIX_WEIGHT: u32 = [0-9]+;/pub const ORDER6_MIX_WEIGHT: u32 = ${O6_MIX};/" \
  "$PARAMS"

if [[ -n "$O3_BITS" ]]; then
  sed -i -E "s/^pub const ORDER3_BITS: usize = [0-9]+;/pub const ORDER3_BITS: usize = ${O3_BITS};/" "$PARAMS"
fi
if [[ -n "$O4_BITS" ]]; then
  sed -i -E "s/^pub const ORDER4_BITS: usize = [0-9]+;/pub const ORDER4_BITS: usize = ${O4_BITS};/" "$PARAMS"
fi

BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/tune/prometheus-compress"
OUT="${OUT_DIR}/${TAG}.comp"
TIME_OUT="${OUT_DIR}/${TAG}.time"

nix develop -c bash -c "
  set -euo pipefail
  export CARGO_TARGET_DIR='$CARGO_TARGET_DIR'
  cargo build --profile tune -p prometheus-compress -q
  BIN='$BIN'
  if [[ ! -x \"\$BIN\" ]]; then
    BIN=\"\$CARGO_TARGET_DIR/tune/prometheus-compress\"
  fi
  TIME_BIN=\$(type -P time)
  \"\$TIME_BIN\" -f 'WALL=%e CPU_USER=%U CPU_SYS=%S RSS_KB=%M' -o '$TIME_OUT' \\
    \"\$BIN\" '$SLICE' '$OUT'
"

BYTES=$(wc -c < "$OUT" | tr -d ' ')
DELTA=$((BYTES - BASELINE))
# shellcheck disable=SC1090
source <(sed 's/^/export /' "$TIME_OUT")
echo "TAG=${TAG} COMPRESSED=${BYTES} DELTA=${DELTA} WALL=${WALL} RSS_KB=${RSS_KB}"

if [[ ! -f "$CSV" ]]; then
  echo "tag,o6_bits,o6_mix,o3_bits,o4_bits,compressed,delta,wall_s,rss_kb" > "$CSV"
fi
O3_LOG="${O3_BITS:-tip}"
O4_LOG="${O4_BITS:-tip}"
echo "${TAG},${O6_BITS},${O6_MIX},${O3_LOG},${O4_LOG},${BYTES},${DELTA},${WALL},${RSS_KB}" >> "$CSV"

if [[ "${KEEP_PARAMS:-0}" != "1" ]]; then
  cp "$TIP_PARAMS" "$PARAMS"
fi
