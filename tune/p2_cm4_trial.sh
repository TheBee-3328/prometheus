#!/usr/bin/env bash
# Patch free CM-scale knobs, rebuild tune, compress slice25.bin.
# Restores tip params after each trial unless KEEP_PARAMS=1.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace/prometheus-p2-cm4/target-p2-cm4}"
OUT_DIR="${OUT_DIR:-/tmp/p2-cm4-out}"
CSV="${ROOT}/tune/p2_cm4_trials.csv"
PARAMS="${ROOT}/crates/prometheus-core/src/params.rs"
TIP_PARAMS="${OUT_DIR}/params_tip.rs"
SLICE="${ROOT}/tune/slice25.bin"
BASELINE="${BASELINE:-0}"

TAG="${1:?tag}"
shift

mkdir -p "$OUT_DIR"
if [[ ! -f "$TIP_PARAMS" ]]; then
  cp "$PARAMS" "$TIP_PARAMS"
fi
if [[ ! -f "$SLICE" ]]; then
  echo "missing $SLICE" >&2
  exit 1
fi

cp "$TIP_PARAMS" "$PARAMS"

# Apply KEY=VAL overrides from remaining args
NOTE=""
for kv in "$@"; do
  if [[ "$kv" != *=* ]]; then
    echo "bad arg $kv (want KEY=VAL)" >&2
    exit 1
  fi
  key="${kv%%=*}"
  val="${kv#*=}"
  NOTE="${NOTE}${key}=${val};"
  case "$key" in
    ORDER3_BITS|ORDER4_BITS|ORDER5_BITS|ORDER6_BITS|ORDER7_BITS|HASH_BITS|BUF_BITS|HASH_ORDER|WORD_BITS|INDIRECT_BITS|PROSE_BITS)
      sed -i -E "s/^pub const ${key}: usize = [0-9]+;/pub const ${key}: usize = ${val};/" "$PARAMS"
      ;;
    MATCH_CONF|MIX_WEIGHT)
      # val is comma-separated 5 ints, e.g. 100,140,180,220,240
      sed -i -E "s/^pub const ${key}: \[u32; 5\] = \[[0-9, ]+\];/pub const ${key}: [u32; 5] = [${val}];/" "$PARAMS"
      ;;
    *)
      sed -i -E "s/^pub const ${key}: u32 = [0-9]+;/pub const ${key}: u32 = ${val};/" "$PARAMS"
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
echo "TAG=${TAG} COMPRESSED=${BYTES} DELTA=${DELTA} WALL=${WALL} RSS_KB=${RSS_KB} NOTE=${NOTE}"

if [[ ! -f "$CSV" ]]; then
  echo "tag,note,compressed,delta,wall_s,rss_kb" > "$CSV"
fi
echo "${TAG},${NOTE},${BYTES},${DELTA},${WALL},${RSS_KB}" >> "$CSV"

if [[ "${KEEP_PARAMS:-0}" != "1" ]]; then
  cp "$TIP_PARAMS" "$PARAMS"
fi
