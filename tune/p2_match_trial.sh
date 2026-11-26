#!/usr/bin/env bash
# Patch one match/hash knob family, rebuild, compress slice25, append CSV.
# Usage: p2_match_trial.sh <tag> <note>
# Env overrides: HASH_BITS BUF_BITS MIN_MIX_LEN MIX_WEIGHT MATCH_CONF MATCH_IND_CROSS_WEIGHT
#   MIX_WEIGHT / MATCH_CONF as comma lists e.g. "120,160,190,230,245"
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

unset CARGO_TARGET_DIR
export CARGO_TARGET_DIR="${ROOT}/target-p2-match"

TAG="${1:?tag}"
NOTE="${2:-}"
BASELINE="${BASELINE:-5680631}"
CSV="${ROOT}/tune/p2_match_trials.csv"
PARAMS="${ROOT}/crates/prometheus-core/src/params.rs"
SLICE="${ROOT}/tune/slice25.bin"
OUT="${ROOT}/tune/p2_${TAG}.comp"
TIME_BIN="${TIME_BIN:-}"

# Capture current params for CSV (after optional patch by caller — this script
# only measures; patching is done by sweep driver before invoke).
HASH_BITS=$(grep -E '^pub const HASH_BITS:' "$PARAMS" | sed -E 's/.*= ([0-9]+);/\1/')
BUF_BITS=$(grep -E '^pub const BUF_BITS:' "$PARAMS" | sed -E 's/.*= ([0-9]+);/\1/')
MIN_MIX_LEN=$(grep -E '^pub const MIN_MIX_LEN:' "$PARAMS" | sed -E 's/.*= ([0-9]+);/\1/')
MIX_WEIGHT=$(grep -E '^pub const MIX_WEIGHT:' "$PARAMS" | sed -E 's/.*= \[([0-9, ]+)\];/\1/' | tr -d ' ')
MATCH_CONF=$(grep -E '^pub const MATCH_CONF:' "$PARAMS" | sed -E 's/.*= \[([0-9, ]+)\];/\1/' | tr -d ' ')
CROSS=$(grep -E '^pub const MATCH_IND_CROSS_WEIGHT:' "$PARAMS" | sed -E 's/.*= ([0-9]+);/\1/')

nix develop -c bash -c "
  set -euo pipefail
  unset CARGO_TARGET_DIR
  export CARGO_TARGET_DIR='$CARGO_TARGET_DIR'
  cargo build --profile tune -p prometheus-compress -q
"

BIN="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/tune/prometheus-compress"
if [[ ! -x "$BIN" ]]; then
  BIN="$CARGO_TARGET_DIR/tune/prometheus-compress"
fi

# Prefer absolute GNU time from nix develop PATH
if [[ -z "$TIME_BIN" ]]; then
  TIME_BIN=$(nix develop -c bash -c 'command -v time' | tail -1)
fi

TIME_OUT=$(mktemp)
set +e
"$TIME_BIN" -f '%e %U %S %M' -o "$TIME_OUT" "$BIN" "$SLICE" "$OUT"
RC=$?
set -e
if [[ $RC -ne 0 ]]; then
  echo "compress failed rc=$RC" >&2
  cat "$TIME_OUT" >&2 || true
  rm -f "$TIME_OUT"
  exit $RC
fi
read -r WALL_S USER_S SYS_S RSS_KB < "$TIME_OUT"
rm -f "$TIME_OUT"
CPU_S=$(awk -v u="$USER_S" -v s="$SYS_S" 'BEGIN{printf "%.3f", u+s}')
RSS_MB=$(( (RSS_KB + 1023) / 1024 ))
BYTES=$(wc -c < "$OUT" | tr -d ' ')
DELTA=$((BYTES - BASELINE))

# Budget ratios vs tip baseline resources (set BASE_CPU_S / BASE_RSS_MB after tip confirm)
BASE_CPU_S="${BASE_CPU_S:-0}"
BASE_RSS_MB="${BASE_RSS_MB:-0}"
BYTES_PER_CPU="NA"
BYTES_PER_RSS="NA"
if [[ "$BASE_CPU_S" != "0" ]]; then
  # negative DELTA (saved) / positive added CPU → positive ratio when saving bytes costs CPU
  BYTES_PER_CPU=$(awk -v d="$DELTA" -v c="$CPU_S" -v b="$BASE_CPU_S" \
    'BEGIN{dc=c-b; if(dc==0) print "inf"; else printf "%.2f", (-d)/dc}')
fi
if [[ "$BASE_RSS_MB" != "0" ]]; then
  BYTES_PER_RSS=$(awk -v d="$DELTA" -v r="$RSS_MB" -v b="$BASE_RSS_MB" \
    'BEGIN{dr=r-b; if(dr==0) print "inf"; else printf "%.2f", (-d)/dr}')
fi

if [[ ! -f "$CSV" ]]; then
  echo "tag,family,hash_bits,buf_bits,min_mix_len,mix_weight,match_conf,match_ind_cross,compressed,delta,wall_s,cpu_s,rss_mb,bytes_per_cpu_s,bytes_per_mb_rss,note" > "$CSV"
fi
FAMILY="${FAMILY:-unknown}"
echo "${TAG},${FAMILY},${HASH_BITS},${BUF_BITS},${MIN_MIX_LEN},${MIX_WEIGHT},${MATCH_CONF},${CROSS},${BYTES},${DELTA},${WALL_S},${CPU_S},${RSS_MB},${BYTES_PER_CPU},${BYTES_PER_RSS},${NOTE}" >> "$CSV"
echo "TAG=${TAG} COMPRESSED=${BYTES} DELTA=${DELTA} WALL_S=${WALL_S} CPU_S=${CPU_S} RSS_MB=${RSS_MB} B/CPU=${BYTES_PER_CPU} B/RSS=${BYTES_PER_RSS}"
