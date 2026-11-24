#!/usr/bin/env bash
# Apply params JSON and run full enwik8 judge. Args: name params.json
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
NAME="${1:?name}"
JSON="${2:?params.json}"
python3 tune/write_params.py --file "$JSON" >/dev/null
echo "=== VALIDATE $NAME ===" | tee -a tune/validate.log
nix develop -c ./judge.sh enwik8 . 2>&1 | tee -a tune/validate.log | tee "tune/validate_${NAME}.out"
# Extract SCORE line
grep -E '^(ROUNDTRIP|SCORE)=' "tune/validate_${NAME}.out" | tee -a tune/validate.log || \
  grep -E 'ROUNDTRIP=' "tune/validate_${NAME}.out" | tee -a tune/validate.log
