#!/usr/bin/env bash
# Patch mixer residual knobs (ORDER0/APM/APM2/WORD/PROSE), rebuild tune, compress slice25.
# Restores tip params unless KEEP_PARAMS=1. Does NOT touch IND/CROSS/SSE.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace/prometheus-p2-mix3/target-p2-mix3}"
OUT_DIR="${OUT_DIR:-/tmp/p2-mix3-out}"
CSV="${ROOT}/tune/p2_mix3_trials.csv"
PARAMS="${ROOT}/crates/prometheus-core/src/params.rs"
TIP_PARAMS="${OUT_DIR}/params_tip.rs"
SLICE="${ROOT}/tune/slice25.bin"
BASELINE="${BASELINE:-5594393}"

TAG="${1:?tag}"
# Remaining args: KEY=VALUE pairs for params.rs
shift || true

mkdir -p "$OUT_DIR"
if [[ ! -f "$TIP_PARAMS" ]]; then
  cp "$PARAMS" "$TIP_PARAMS"
fi
if [[ ! -f "$SLICE" ]]; then
  echo "missing $SLICE" >&2
  exit 1
fi

cp "$TIP_PARAMS" "$PARAMS"

for kv in "$@"; do
  key="${kv%%=*}"
  val="${kv#*=}"
  case "$key" in
    ORDER0_MIX_WEIGHT|APM_MIX_WEIGHT|APM2_MIX_WEIGHT|APM_RATE_SHIFT|APM_MODE|\
    WORD_MIX_WEIGHT|WORD_BITS|WORD_CTX_MODE|WORD_HIT_GATE|\
    PROSE_MIX_WEIGHT|PROSE_BITS|PROSE_MODE|PROSE_MATCH_GATE|PROSE_SM|\
    ORDER1_MIX_WEIGHT|STRUCT_MIX_WEIGHT)
      if [[ "$key" == *BITS ]]; then
        sed -i -E "s/^pub const ${key}: usize = [0-9]+;/pub const ${key}: usize = ${val};/" "$PARAMS"
      else
        sed -i -E "s/^pub const ${key}: u32 = [0-9]+;/pub const ${key}: u32 = ${val};/" "$PARAMS"
      fi
      ;;
    *)
      echo "forbidden or unknown key: $key (lane avoids IND/CROSS/SSE)" >&2
      cp "$TIP_PARAMS" "$PARAMS"
      exit 2
      ;;
  esac
done

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
KV_LOG=$(printf '%s;' "$@" | sed 's/;$//')
echo "TAG=${TAG} COMPRESSED=${BYTES} DELTA=${DELTA} WALL=${WALL} RSS_KB=${RSS_KB} KV=${KV_LOG}"

if [[ ! -f "$CSV" ]]; then
  echo "tag,kv,compressed,delta,wall_s,rss_kb" > "$CSV"
fi
echo "${TAG},${KV_LOG},${BYTES},${DELTA},${WALL},${RSS_KB}" >> "$CSV"

if [[ "${KEEP_PARAMS:-0}" != "1" ]]; then
  cp "$TIP_PARAMS" "$PARAMS"
fi
