#!/usr/bin/env bash
# One-family sweeps for P2-MATCH. Restores tip params after each family.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
PARAMS="${ROOT}/crates/prometheus-core/src/params.rs"
export BASELINE=5680631
export BASE_CPU_S="${BASE_CPU_S:-104.170}"
export BASE_RSS_MB="${BASE_RSS_MB:-3466}"

restore_tip() {
  sed -i -E \
    -e 's/^pub const BUF_BITS: usize = [0-9]+;/pub const BUF_BITS: usize = 25;/' \
    -e 's/^pub const HASH_BITS: usize = [0-9]+;/pub const HASH_BITS: usize = 26;/' \
    -e 's/^pub const MIN_MIX_LEN: u32 = [0-9]+;/pub const MIN_MIX_LEN: u32 = 6;/' \
    -e 's/^pub const MATCH_CONF: \[u32; 5\] = \[[0-9, ]+\];/pub const MATCH_CONF: [u32; 5] = [32700, 32700, 32700, 32700, 32700];/' \
    -e 's/^pub const MIX_WEIGHT: \[u32; 5\] = \[[0-9, ]+\];/pub const MIX_WEIGHT: [u32; 5] = [120, 160, 190, 230, 245];/' \
    -e 's/^pub const MATCH_IND_CROSS_WEIGHT: u32 = [0-9]+;/pub const MATCH_IND_CROSS_WEIGHT: u32 = 16;/' \
    "$PARAMS"
}

set_hash() {
  local h="$1"
  sed -i -E "s/^pub const HASH_BITS: usize = [0-9]+;/pub const HASH_BITS: usize = ${h};/" "$PARAMS"
}

set_buf() {
  local b="$1"
  sed -i -E "s/^pub const BUF_BITS: usize = [0-9]+;/pub const BUF_BITS: usize = ${b};/" "$PARAMS"
}

set_mix() {
  local m="$1"
  sed -i -E "s/^pub const MIX_WEIGHT: \[u32; 5\] = \[[0-9, ]+\];/pub const MIX_WEIGHT: [u32; 5] = [${m}];/" "$PARAMS"
}

set_min_mix() {
  local m="$1"
  sed -i -E "s/^pub const MIN_MIX_LEN: u32 = [0-9]+;/pub const MIN_MIX_LEN: u32 = ${m};/" "$PARAMS"
}

set_cross() {
  local c="$1"
  sed -i -E "s/^pub const MATCH_IND_CROSS_WEIGHT: u32 = [0-9]+;/pub const MATCH_IND_CROSS_WEIGHT: u32 = ${c};/" "$PARAMS"
}

set_conf() {
  local c="$1"
  sed -i -E "s/^pub const MATCH_CONF: \[u32; 5\] = \[[0-9, ]+\];/pub const MATCH_CONF: [u32; 5] = [${c}];/" "$PARAMS"
}

FAMILY="${1:?family: hash|buf|mix_weight|min_mix|match_conf|mix}"

case "$FAMILY" in
  hash)
    restore_tip
    for h in 24 25 27; do
      restore_tip
      set_hash "$h"
      FAMILY=hash bash "${ROOT}/tune/p2_match_trial.sh" "hash_${h}" "HASH_BITS=${h}"
    done
    restore_tip
    ;;
  buf)
    restore_tip
    # Skip tip 25; sweep neighbors including 23/27
    for b in 23 24 26 27; do
      restore_tip
      set_buf "$b"
      FAMILY=buf bash "${ROOT}/tune/p2_match_trial.sh" "buf_${b}" "BUF_BITS=${b}"
    done
    restore_tip
    ;;
  mix_weight)
    # One knob family: MIX_WEIGHT only
    restore_tip
    for tag_m in \
      "mix_steep:80,140,190,235,250" \
      "mix_flat:160,170,180,190,200" \
      "mix_hot:140,180,210,240,250" \
      "mix_cool:100,140,170,200,220" \
      "mix_late:60,100,160,220,250" \
      "mix_early:180,200,220,235,245"
    do
      tag="${tag_m%%:*}"
      m="${tag_m#*:}"
      restore_tip
      set_mix "$m"
      FAMILY=mix_weight bash "${ROOT}/tune/p2_match_trial.sh" "$tag" "MIX_WEIGHT=[${m}]"
    done
    restore_tip
    ;;
  min_mix)
    restore_tip
    for ml in 5 7 8; do
      restore_tip
      set_min_mix "$ml"
      FAMILY=min_mix bash "${ROOT}/tune/p2_match_trial.sh" "minmix_${ml}" "MIN_MIX_LEN=${ml}"
    done
    restore_tip
    ;;
  match_conf)
    restore_tip
    set_conf "20000,26000,30000,32000,32700"
    FAMILY=match_conf bash "${ROOT}/tune/p2_match_trial.sh" "conf_grad" "MATCH_CONF=graduated"
    restore_tip
    set_conf "28000,30000,31000,32000,32700"
    FAMILY=match_conf bash "${ROOT}/tune/p2_match_trial.sh" "conf_soft" "MATCH_CONF=soft"
    restore_tip
    set_conf "16000,22000,28000,31000,32700"
    FAMILY=match_conf bash "${ROOT}/tune/p2_match_trial.sh" "conf_ramp" "MATCH_CONF=ramp"
    restore_tip
    ;;
  mix)
    # Legacy all-in-one (prefer split families: mix_weight|min_mix|match_conf)
    bash "$0" mix_weight
    bash "$0" min_mix
    restore_tip
    for c in 0 8 24 32; do
      restore_tip
      set_cross "$c"
      FAMILY=cross bash "${ROOT}/tune/p2_match_trial.sh" "cross_${c}" "MATCH_IND_CROSS=${c}"
    done
    bash "$0" match_conf
    restore_tip
    ;;
  *)
    echo "unknown family $FAMILY" >&2
    exit 1
    ;;
esac
