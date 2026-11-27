#!/usr/bin/env bash
# MIX3 mixer-residual sweeps on tip SCORE 20353480 (slice baseline 5594393).
# Avoid IND/CROSS/SSE/O6. Wait for MemAvailable + no other compress.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace/prometheus-p2-mix3/target-p2-mix3}"
export OUT_DIR="${OUT_DIR:-/tmp/p2-mix3-out}"
export BASELINE="${BASELINE:-5594393}"
MIN_AVAIL_MB="${MIN_AVAIL_MB:-10000}"
TRIAL="./tune/p2_mix3_trial.sh"
LOG="${OUT_DIR}/progress.log"

wait_ram() {
  local i=0
  while true; do
    local avail=0 ncomp=0
    avail=$(awk '/MemAvailable/{print int($2/1024)}' /proc/meminfo)
    if ps -C prometheus-comp >/dev/null 2>&1 || ps -C prometheus-deco >/dev/null 2>&1; then
      ncomp=1
    fi
    if [[ "$ncomp" -eq 0 && "$avail" -ge "$MIN_AVAIL_MB" ]]; then
      echo "RAM_OK avail=${avail}MB" | tee -a "$LOG"
      return 0
    fi
    i=$((i + 1))
    if (( i % 6 == 1 )); then
      echo "$(date +%H:%M:%S) waiting avail=${avail}MB need=${MIN_AVAIL_MB} ncomp=${ncomp}" | tee -a "$LOG"
    fi
    sleep 10
  done
}

run_one() {
  local tag="$1"; shift
  wait_ram
  echo "=== RUN $tag $* ===" | tee -a "$LOG"
  "$TRIAL" "$tag" "$@" 2>&1 | tee -a "$LOG"
}

mkdir -p "$OUT_DIR"
# Tip snapshot (do not overwrite if already tip)
cp crates/prometheus-core/src/params.rs "$OUT_DIR/params_tip.rs"
cp crates/prometheus-core/src/match_model.rs "$OUT_DIR/match_model_tip.rs"

# Tip row already recorded; ensure CSV header
if [[ ! -f tune/p2_mix3_trials.csv ]]; then
  echo "tag,kv,compressed,delta,wall_s,rss_kb" > tune/p2_mix3_trials.csv
fi

echo "=== WAVE A APM2 (tip=160) ===" | tee -a "$LOG"
run_one apm2_128 APM2_MIX_WEIGHT=128
run_one apm2_144 APM2_MIX_WEIGHT=144
run_one apm2_176 APM2_MIX_WEIGHT=176
run_one apm2_192 APM2_MIX_WEIGHT=192
run_one apm2_112 APM2_MIX_WEIGHT=112
run_one apm2_208 APM2_MIX_WEIGHT=208

echo "=== WAVE B APM1 (tip=224) + rate ===" | tee -a "$LOG"
run_one apm_208 APM_MIX_WEIGHT=208
run_one apm_240 APM_MIX_WEIGHT=240
run_one apm_192 APM_MIX_WEIGHT=192
run_one apm_r6 APM_RATE_SHIFT=6
run_one apm_r8 APM_RATE_SHIFT=8

echo "=== WAVE C ORDER0 residual (tip=40 post-IND2) ===" | tee -a "$LOG"
run_one o0_32 ORDER0_MIX_WEIGHT=32
run_one o0_36 ORDER0_MIX_WEIGHT=36
run_one o0_44 ORDER0_MIX_WEIGHT=44
run_one o0_48 ORDER0_MIX_WEIGHT=48
run_one o0_56 ORDER0_MIX_WEIGHT=56
run_one o0_64 ORDER0_MIX_WEIGHT=64

echo "=== WAVE D WORD ===" | tee -a "$LOG"
run_one wmix_48 WORD_MIX_WEIGHT=48
run_one wmix_56 WORD_MIX_WEIGHT=56
run_one wmix_72 WORD_MIX_WEIGHT=72
run_one wmix_80 WORD_MIX_WEIGHT=80
run_one wmix_40 WORD_MIX_WEIGHT=40
run_one wbits_26 WORD_BITS=26

echo "=== WAVE E PROSE ===" | tee -a "$LOG"
run_one pmix_16 PROSE_MIX_WEIGHT=16
run_one pmix_20 PROSE_MIX_WEIGHT=20
run_one pmix_28 PROSE_MIX_WEIGHT=28
run_one pmix_32 PROSE_MIX_WEIGHT=32
run_one pmix_40 PROSE_MIX_WEIGHT=40
run_one pmix_12 PROSE_MIX_WEIGHT=12
run_one pgate1 PROSE_MATCH_GATE=1
run_one pbits_26 PROSE_BITS=26

echo "=== WAVE F STRUCT/ORDER1 ===" | tee -a "$LOG"
run_one o1_4 ORDER1_MIX_WEIGHT=4
run_one o1_12 ORDER1_MIX_WEIGHT=12
run_one struct_16 STRUCT_MIX_WEIGHT=16
run_one struct_32 STRUCT_MIX_WEIGHT=32

echo "=== BATCH1 DONE ===" | tee -a "$LOG"
awk -F, 'NR>1{printf "%8d %s %s\n",$4,$1,$2}' tune/p2_mix3_trials.csv | sort -n | tee -a "$LOG" | head -25
