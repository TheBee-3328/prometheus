#!/usr/bin/env bash
# Patch INDIRECT_BITS / INDIRECT_MIX_WEIGHT (+ optional CROSS / ORDER0 / hit-gate),
# rebuild tune, compress slice25.bin. Restores tip params unless KEEP_PARAMS=1.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace/prometheus-p2-ind2/target-p2-ind2}"
OUT_DIR="${OUT_DIR:-/tmp/p2-ind2-out}"
CSV="${ROOT}/tune/p2_ind2_trials.csv"
PARAMS="${ROOT}/crates/prometheus-core/src/params.rs"
MATCH="${ROOT}/crates/prometheus-core/src/match_model.rs"
ORDER0="${ROOT}/crates/prometheus-core/src/order0.rs"
TIP_PARAMS="${OUT_DIR}/params_tip.rs"
TIP_MATCH="${OUT_DIR}/match_model_tip.rs"
TIP_ORDER0="${OUT_DIR}/order0_tip.rs"
SLICE="${ROOT}/tune/slice25.bin"
BASELINE="${BASELINE:-5610357}"

TAG="${1:?tag}"
IND_BITS="${2:?INDIRECT_BITS}"
IND_MIX="${3:?INDIRECT_MIX_WEIGHT}"
CROSS_MIX="${4:-}"
O0_MIX="${5:-}"
HIT_GATE="${6:-0}"  # 1 = skip IND mix on checksum miss
O0_MODE="${7:-}"    # ORDER0_MODE override

mkdir -p "$OUT_DIR"
if [[ ! -f "$TIP_PARAMS" ]]; then
  cp "$PARAMS" "$TIP_PARAMS"
fi
if [[ ! -f "$TIP_MATCH" ]]; then
  cp "$MATCH" "$TIP_MATCH"
fi
if [[ ! -f "$TIP_ORDER0" ]]; then
  cp "$ORDER0" "$TIP_ORDER0"
fi
if [[ ! -f "$SLICE" ]]; then
  echo "missing $SLICE" >&2
  exit 1
fi

cp "$TIP_PARAMS" "$PARAMS"
cp "$TIP_MATCH" "$MATCH"
cp "$TIP_ORDER0" "$ORDER0"

sed -i -E \
  -e "s/^pub const INDIRECT_BITS: usize = [0-9]+;/pub const INDIRECT_BITS: usize = ${IND_BITS};/" \
  -e "s/^pub const INDIRECT_MIX_WEIGHT: u32 = [0-9]+;/pub const INDIRECT_MIX_WEIGHT: u32 = ${IND_MIX};/" \
  "$PARAMS"

if [[ -n "$CROSS_MIX" ]]; then
  sed -i -E "s/^pub const MATCH_IND_CROSS_WEIGHT: u32 = [0-9]+;/pub const MATCH_IND_CROSS_WEIGHT: u32 = ${CROSS_MIX};/" "$PARAMS"
fi
if [[ -n "$O0_MIX" ]]; then
  sed -i -E "s/^pub const ORDER0_MIX_WEIGHT: u32 = [0-9]+;/pub const ORDER0_MIX_WEIGHT: u32 = ${O0_MIX};/" "$PARAMS"
fi
if [[ -n "$O0_MODE" ]]; then
  sed -i -E "s/^pub const ORDER0_MODE: u32 = [0-9]+;/pub const ORDER0_MODE: u32 = ${O0_MODE};/" "$PARAMS"
fi

if [[ "$HIT_GATE" == "1" ]]; then
  # Skip IND mix when predict returns flat 1/2 (checksum miss / cold).
  sed -i '/let pi = self.indirect.predict();/{
    n
    s/p0 = Self::mix_w(p0, pi, INDIRECT_MIX_WEIGHT);/if pi != 32768 {\n            p0 = Self::mix_w(p0, pi, INDIRECT_MIX_WEIGHT);\n        }/
  }' "$MATCH"
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
  echo "tag,ind_bits,ind_mix,cross_mix,o0_mix,hit_gate,o0_mode,compressed,delta,wall_s,rss_kb" > "$CSV"
fi
CROSS_LOG="${CROSS_MIX:-tip}"
O0_LOG="${O0_MIX:-tip}"
O0M_LOG="${O0_MODE:-tip}"
echo "${TAG},${IND_BITS},${IND_MIX},${CROSS_LOG},${O0_LOG},${HIT_GATE},${O0M_LOG},${BYTES},${DELTA},${WALL},${RSS_KB}" >> "$CSV"

if [[ "${KEEP_PARAMS:-0}" != "1" ]]; then
  cp "$TIP_PARAMS" "$PARAMS"
  cp "$TIP_MATCH" "$MATCH"
  cp "$TIP_ORDER0" "$ORDER0"
fi
