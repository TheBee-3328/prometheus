#!/usr/bin/env bash
# Adversarial round-trip suite for PROMETHEUS (P3-REVIEW).
# Compress → decompress → cmp for edge inputs. Exit 0 only if all PASS.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

if ! command -v cargo >/dev/null 2>&1; then
  if [[ "${PROMETHEUS_NIX_REEXEC:-}" == "1" ]]; then
    echo "adversarial_roundtrip.sh: tools missing inside nix develop" >&2
    exit 1
  fi
  export PROMETHEUS_NIX_REEXEC=1
  exec nix develop "${ROOT}" -c "$0" "$@"
fi

if [[ "${NIX_LDFLAGS:-}" == *" "* ]]; then
  export NIX_LDFLAGS=""
fi

WORK="${ROOT}/judge_work/adversarial"
rm -rf "${WORK}"
mkdir -p "${WORK}"

cargo build --release -p prometheus-compress -p prometheus-decompress
COMP="${ROOT}/target/release/prometheus-compress"
DECOMP="${ROOT}/target/release/prometheus-decompress"
strip -s "${COMP}" "${DECOMP}" 2>/dev/null || strip "${COMP}" "${DECOMP}"

PASS=0
FAIL=0

run_case() {
  local name="$1"
  local infile="$2"
  local c="${WORK}/${name}.compressed"
  local o="${WORK}/${name}.out"
  echo -n "CASE ${name}: "
  if ! "${COMP}" "${infile}" "${c}"; then
    echo "FAIL (compress)"
    FAIL=$((FAIL + 1))
    return
  fi
  if ! "${DECOMP}" "${c}" "${o}"; then
    echo "FAIL (decompress)"
    FAIL=$((FAIL + 1))
    return
  fi
  if ! cmp -s "${infile}" "${o}"; then
    echo "FAIL (cmp mismatch)"
    FAIL=$((FAIL + 1))
    return
  fi
  # Determinism: second compress identical
  local c2="${WORK}/${name}.compressed2"
  "${COMP}" "${infile}" "${c2}"
  if ! cmp -s "${c}" "${c2}"; then
    echo "FAIL (compress nondeterminism)"
    FAIL=$((FAIL + 1))
    return
  fi
  echo "PASS"
  PASS=$((PASS + 1))
}

# 1. empty
: > "${WORK}/empty.bin"
run_case "empty" "${WORK}/empty.bin"

# 2. one byte
printf '\x42' > "${WORK}/onebyte.bin"
run_case "one_byte" "${WORK}/onebyte.bin"

# 3. all zeros (4 KiB)
dd if=/dev/zero of="${WORK}/zeros.bin" bs=4096 count=1 status=none
run_case "all_zeros_4k" "${WORK}/zeros.bin"

# 4. cap-marker control bytes (0x01–0x08) + escape (0x7F) — preprocessor collision guard
python3 - <<PY
from pathlib import Path
p = Path("${WORK}/ctrl_markers.bin")
p.write_bytes(bytes(range(0x01, 0x09)) + b"\x7f" + bytes(range(0x00, 0x20)) * 64)
PY
run_case "ctrl_markers" "${WORK}/ctrl_markers.bin"

# 5. random bytes (64 KiB)
dd if=/dev/urandom of="${WORK}/random.bin" bs=65536 count=1 status=none
run_case "random_64k" "${WORK}/random.bin"

# 6. invalid UTF-8 sequences
printf '\xff\xfe\x80\xc0\xc1\xf5\xff\x00\xed\xa0\x80' > "${WORK}/bad_utf8.bin"
run_case "invalid_utf8" "${WORK}/bad_utf8.bin"

# 7. 1MB repeated line
python3 - <<PY
line = b"The quick brown fox jumps over the lazy dog.\n"
open("${WORK}/repeat_1mb.bin", "wb").write(line * ((1 << 20) // len(line)))
PY
run_case "repeat_1mb" "${WORK}/repeat_1mb.bin"

# 8–10. first 1 / 10 / 100 MB of enwik8 (100MB = full enwik8)
if [[ ! -f "${ROOT}/enwik8" ]]; then
  echo "CASE enwik8_prefixes: SKIP (missing enwik8)"
else
  for mb in 1 10 100; do
    bytes=$((mb * 1000000))
    # enwik8 is exactly 100000000; take min
    if [[ "${bytes}" -gt 100000000 ]]; then
      bytes=100000000
    fi
    # Prefer exact MB of file; for 100 use full file
    if [[ "${mb}" -eq 100 ]]; then
      infile="${ROOT}/enwik8"
    else
      dd if="${ROOT}/enwik8" of="${WORK}/enwik8_${mb}mb.bin" bs=1000000 count="${mb}" status=none
      infile="${WORK}/enwik8_${mb}mb.bin"
    fi
    run_case "enwik8_first_${mb}mb" "${infile}"
  done
fi

echo "---"
echo "ADVERSARIAL PASS=${PASS} FAIL=${FAIL}"
if [[ "${FAIL}" -ne 0 ]]; then
  exit 1
fi
exit 0
