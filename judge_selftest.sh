#!/usr/bin/env bash
# Self-test for the judge harness: corrupt compressed must FAIL round-trip.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${ROOT}"

# Re-exec under nix if tools missing (same policy as judge.sh)
if ! command -v cargo >/dev/null 2>&1 || ! command time -f '%e' true >/dev/null 2>&1; then
  if [[ "${PROMETHEUS_NIX_REEXEC:-}" == "1" ]]; then
    echo "judge_selftest.sh: tools missing inside nix develop" >&2
    exit 1
  fi
  export PROMETHEUS_NIX_REEXEC=1
  exec nix develop "${ROOT}" -c "$0" "$@"
fi

# Defense in depth: space-containing NIX_LDFLAGS from mkShell break ld
if [[ "${NIX_LDFLAGS:-}" == *" "* ]]; then
  export NIX_LDFLAGS=""
fi

WORK="${ROOT}/judge_work/selftest"
rm -rf "${WORK}"
mkdir -p "${WORK}"

# Tiny fixture (do not need full enwik8)
{
  printf 'PROMETHEUS-selftest-'
  # 256 payload bytes after header region for easy corruption
  dd if=/dev/zero bs=256 count=1 2>/dev/null | tr '\0' 'x'
} > "${WORK}/input.bin"

# Build
cargo build --release -p prometheus-compress -p prometheus-decompress
COMP="${ROOT}/target/release/prometheus-compress"
DECOMP="${ROOT}/target/release/prometheus-decompress"
strip -s "${COMP}" "${DECOMP}" 2>/dev/null || strip "${COMP}" "${DECOMP}"

# Happy path
"${COMP}" "${WORK}/input.bin" "${WORK}/ok.compressed"
"${DECOMP}" "${WORK}/ok.compressed" "${WORK}/ok.out"
if ! cmp -s "${WORK}/input.bin" "${WORK}/ok.out"; then
  echo "SELFTEST FAIL: happy-path round-trip mismatch" >&2
  exit 1
fi

# Corrupt compressed → decompress or cmp must fail
cp "${WORK}/ok.compressed" "${WORK}/bad.compressed"
# Overwrite magic so the archive is invalid (decomp must not silently succeed)
printf 'XXXX' | dd of="${WORK}/bad.compressed" bs=1 seek=0 conv=notrunc 2>/dev/null
# Also flip a payload byte in a second copy to cover silent-wrong-output case
cp "${WORK}/ok.compressed" "${WORK}/bad2.compressed"
printf '\xff' | dd of="${WORK}/bad2.compressed" bs=1 seek=20 conv=notrunc 2>/dev/null

set +e
"${DECOMP}" "${WORK}/bad.compressed" "${WORK}/bad.out"
DECOMP_RC=$?
set -e

if [[ "${DECOMP_RC}" -eq 0 ]]; then
  echo "SELFTEST FAIL: corrupt magic still decompressed OK" >&2
  exit 1
fi

set +e
"${DECOMP}" "${WORK}/bad2.compressed" "${WORK}/bad2.out"
DECOMP2_RC=$?
set -e
if [[ "${DECOMP2_RC}" -eq 0 ]] && cmp -s "${WORK}/input.bin" "${WORK}/bad2.out"; then
  echo "SELFTEST FAIL: flipped payload still matched original" >&2
  exit 1
fi
# Payload flip must at least fail cmp when decompress "succeeds"
if [[ "${DECOMP2_RC}" -eq 0 ]] && ! cmp -s "${WORK}/input.bin" "${WORK}/bad2.out"; then
  : # expected for store baseline
fi

# Determinism smoke
"${COMP}" "${WORK}/input.bin" "${WORK}/a.compressed"
"${COMP}" "${WORK}/input.bin" "${WORK}/b.compressed"
cmp -s "${WORK}/a.compressed" "${WORK}/b.compressed"
"${DECOMP}" "${WORK}/a.compressed" "${WORK}/a.out"
"${DECOMP}" "${WORK}/a.compressed" "${WORK}/b.out"
cmp -s "${WORK}/a.out" "${WORK}/b.out"

echo "SELFTEST PASS"
exit 0
