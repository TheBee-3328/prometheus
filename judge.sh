#!/usr/bin/env bash
# PROMETHEUS judge harness — measures SCORE = compressed + decompressor.
# See JUDGE.md for the scoring contract and aux-file rules.
set -euo pipefail

# Re-exec under the project flake when not already in a nix develop shell
# (or when required tools are missing). Prefer: nix develop -c ./judge.sh …
ensure_nix() {
  local need=0
  command -v cargo >/dev/null 2>&1 || need=1
  command -v rustc >/dev/null 2>&1 || need=1
  command -v strip >/dev/null 2>&1 || need=1
  command -v cmp >/dev/null 2>&1 || need=1
  # GNU time (not bash builtin)
  if ! command time -f '%e' true >/dev/null 2>&1; then
    need=1
  fi
  if [[ "${need}" -eq 0 ]]; then
    return 0
  fi
  if [[ "${PROMETHEUS_NIX_REEXEC:-}" == "1" ]]; then
    echo "judge.sh: required tools missing even inside nix develop" >&2
    exit 1
  fi
  local root
  root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  if [[ ! -f "${root}/flake.nix" ]]; then
    echo "judge.sh: flake.nix not found; run from repo root via nix develop" >&2
    exit 1
  fi
  export PROMETHEUS_NIX_REEXEC=1
  exec nix develop "${root}" -c "$0" "$@"
}

ensure_nix "$@"

# Defense in depth: space-containing NIX_LDFLAGS from mkShell break ld
if [[ "${NIX_LDFLAGS:-}" == *" "* ]]; then
  export NIX_LDFLAGS=""
fi

DATASET="${1:-enwik8}"
WORKSPACE="${2:-.}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${ROOT}"

case "${DATASET}" in
  enwik8|enwik9) ;;
  *)
    echo "usage: ./judge.sh [enwik8|enwik9] [workspace=.]" >&2
    exit 2
    ;;
esac

if [[ ! -f "${DATASET}" ]]; then
  echo "judge.sh: missing ${DATASET} (gitignored data; place at repo root)" >&2
  exit 1
fi

WORKSPACE="$(cd "${WORKSPACE}" && pwd)"
WORK="${ROOT}/judge_work"
mkdir -p "${WORK}"
rm -f "${WORK}"/* 2>/dev/null || true

COMPRESS_BIN="${WORKSPACE}/target/release/prometheus-compress"
DECOMPRESS_BIN="${WORKSPACE}/target/release/prometheus-decompress"
COMPRESSED="${WORK}/${DATASET}.compressed"
ROUNDTRIP="${WORK}/${DATASET}.roundtrip"
COMPRESSED2="${WORK}/${DATASET}.compressed2"
ROUNDTRIP2="${WORK}/${DATASET}.roundtrip2"
TIME_COMP="${WORK}/time_compress.txt"
TIME_DECOMP="${WORK}/time_decompress.txt"

# --- release build (size opts live in workspace Cargo.toml [profile.release]) ---
(
  cd "${WORKSPACE}"
  cargo build --release -p prometheus-compress -p prometheus-decompress
)

# Extra strip pass (Cargo strip=symbols may leave some symbols; be explicit)
strip -s "${COMPRESS_BIN}" "${DECOMPRESS_BIN}" 2>/dev/null || strip "${COMPRESS_BIN}" "${DECOMPRESS_BIN}"

# --- compress (timed + peak RSS) ---
# %e wall seconds, %M max RSS in KB (GNU time)
set +e
command time -f '%e %M' -o "${TIME_COMP}" \
  "${COMPRESS_BIN}" "${ROOT}/${DATASET}" "${COMPRESSED}"
COMP_RC=$?
set -e
if [[ "${COMP_RC}" -ne 0 ]]; then
  echo "ROUNDTRIP=FAIL COMPRESSED=0 DECOMPRESSOR=0 SCORE=0 COMPRESS_SECONDS=0 DECOMPRESS_SECONDS=0 PEAK_RSS_MB=0"
  exit 1
fi

# --- decompress (timed + peak RSS); must not need original ---
set +e
command time -f '%e %M' -o "${TIME_DECOMP}" \
  "${DECOMPRESS_BIN}" "${COMPRESSED}" "${ROUNDTRIP}"
DECOMP_RC=$?
set -e

# --- round-trip check ---
ROUNDTRIP_STATUS="FAIL"
if [[ "${DECOMP_RC}" -eq 0 ]] && cmp -s "${ROOT}/${DATASET}" "${ROUNDTRIP}"; then
  ROUNDTRIP_STATUS="OK"
fi

# --- determinism: compress×2 identical ---
"${COMPRESS_BIN}" "${ROOT}/${DATASET}" "${COMPRESSED2}"
if ! cmp -s "${COMPRESSED}" "${COMPRESSED2}"; then
  ROUNDTRIP_STATUS="FAIL"
  echo "judge.sh: compress determinism failed" >&2
fi

# --- determinism: decompress×2 identical ---
"${DECOMPRESS_BIN}" "${COMPRESSED}" "${ROUNDTRIP2}"
if ! cmp -s "${ROUNDTRIP}" "${ROUNDTRIP2}"; then
  ROUNDTRIP_STATUS="FAIL"
  echo "judge.sh: decompress determinism failed" >&2
fi

# --- sizes ---
COMPRESSED_BYTES=$(wc -c < "${COMPRESSED}" | tr -d ' ')

# DECOMPRESSOR = stripped exe + every aux file decomp reads at runtime.
# Phase 0 store baseline: no aux files. Future models list aux paths here
# or via a sidecar manifest next to the binary (see JUDGE.md).
DECOMPRESSOR_BYTES=$(wc -c < "${DECOMPRESS_BIN}" | tr -d ' ')
AUX_BYTES=0
# Optional: workspace-relative aux files declared in judge_aux.txt (one path per line)
if [[ -f "${WORKSPACE}/judge_aux.txt" ]]; then
  while IFS= read -r aux || [[ -n "${aux}" ]]; do
    [[ -z "${aux}" || "${aux}" =~ ^# ]] && continue
    if [[ -f "${WORKSPACE}/${aux}" ]]; then
      b=$(wc -c < "${WORKSPACE}/${aux}" | tr -d ' ')
      AUX_BYTES=$((AUX_BYTES + b))
    elif [[ -f "${aux}" ]]; then
      b=$(wc -c < "${aux}" | tr -d ' ')
      AUX_BYTES=$((AUX_BYTES + b))
    else
      echo "judge.sh: aux file missing: ${aux}" >&2
      ROUNDTRIP_STATUS="FAIL"
    fi
  done < "${WORKSPACE}/judge_aux.txt"
fi
DECOMPRESSOR_BYTES=$((DECOMPRESSOR_BYTES + AUX_BYTES))

SCORE=$((COMPRESSED_BYTES + DECOMPRESSOR_BYTES))

# --- parse timings ---
read -r COMPRESS_SECONDS COMP_RSS_KB < "${TIME_COMP}"
read -r DECOMPRESS_SECONDS DECOMP_RSS_KB < "${TIME_DECOMP}"

# Peak RSS across compress + decompress, in MB (ceil from KB)
PEAK_RSS_KB="${COMP_RSS_KB}"
if [[ "${DECOMP_RSS_KB}" -gt "${PEAK_RSS_KB}" ]]; then
  PEAK_RSS_KB="${DECOMP_RSS_KB}"
fi
PEAK_RSS_MB=$(( (PEAK_RSS_KB + 1023) / 1024 ))

# Integer-ish seconds for display (keep one decimal if present)
COMPRESS_SECONDS="${COMPRESS_SECONDS}"
DECOMPRESS_SECONDS="${DECOMPRESS_SECONDS}"

echo "ROUNDTRIP=${ROUNDTRIP_STATUS} COMPRESSED=${COMPRESSED_BYTES} DECOMPRESSOR=${DECOMPRESSOR_BYTES} SCORE=${SCORE} COMPRESS_SECONDS=${COMPRESS_SECONDS} DECOMPRESS_SECONDS=${DECOMPRESS_SECONDS} PEAK_RSS_MB=${PEAK_RSS_MB}"

if [[ "${ROUNDTRIP_STATUS}" != "OK" ]]; then
  exit 1
fi
