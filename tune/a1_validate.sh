#!/usr/bin/env bash
# Validate A1 candidates A/B/C with full enwik8 judge. One at a time.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target-a1}"
BASELINE_SCORE=20911620
BASELINE_SLICE=5819041

active_foreign() {
  ps -eo args= | grep -E '(^|/)prometheus-compress( |$)' | grep -vE "bash|nix develop|grep|${ROOT}/target-a1" || true
}

wait_clear() {
  while [[ -n "$(active_foreign)" ]]; do
    echo "waiting for foreign compress..." >&2
    sleep 20
  done
}

for name in A B C; do
  CAND="$ROOT/tune/candidates/A1_${name}.json"
  if [[ ! -f "$CAND" ]]; then
    echo "SKIP ${name}: no candidate file"
    continue
  fi
  echo "======== VALIDATE ${name} ========"
  python3 - "$CAND" <<'PY'
import json, sys, pathlib
sys.path.insert(0, str(pathlib.Path("tune").resolve()))
from write_params_full import write_params
d = json.loads(pathlib.Path(sys.argv[1]).read_text())
write_params(d["params"])
print("note=", d.get("note"), "slice=", d.get("slice"), "delta=", d.get("delta"))
PY
  wait_clear
  nix develop -c ./judge.sh enwik8 . | tee "tune/a1_judge_${name}.txt"
  # Parse SCORE
  line=$(grep -E '^ROUNDTRIP=' "tune/a1_judge_${name}.txt" | tail -1)
  echo "JUDGE_${name}: $line"
  score=$(echo "$line" | sed -n 's/.*SCORE=\([0-9]*\).*/\1/p')
  slice_delta=$(python3 -c "import json; print(json.load(open('$CAND'))['delta'])")
  full_delta=$((score - BASELINE_SCORE))
  echo "SLICE_DELTA=${slice_delta} FULL_DELTA=${full_delta} BASELINE_SCORE=${BASELINE_SCORE}"
  # Reject if slice gain > 2x full gain (when both improve)
  python3 - <<PY
slice_d = int("${slice_delta}")
full_d = int("${full_delta}")
score = int("${score}")
base = ${BASELINE_SCORE}
ok = "ROUNDTRIP=OK" in """${line}"""
keep = ok and score < base
# Overfit gate: if slice improved more than 2x the full improvement, reject
# (only when full also improved; if full worsened, reject anyway)
if keep and slice_d < 0 and full_d < 0:
    if abs(slice_d) > 2 * abs(full_d):
        keep = False
        print(f"REJECT ${name}: slice gain {slice_d} > 2x full gain {full_d}")
    else:
        print(f"KEEP_CANDIDATE ${name}: SCORE={score} full_d={full_d} slice_d={slice_d}")
elif keep:
    print(f"KEEP_CANDIDATE ${name}: SCORE={score} full_d={full_d} slice_d={slice_d}")
else:
    print(f"REJECT ${name}: ok={ok} SCORE={score} full_d={full_d}")
PY
done

# Restore baseline params
python3 tune/write_params_full.py --defaults >/dev/null
echo "VALIDATE_DONE"
