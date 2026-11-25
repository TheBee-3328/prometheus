#!/usr/bin/env bash
# PROMETHEUS judge harness — measures SCORE = compressed + decompressor.
# See JUDGE.md for the scoring contract and aux-file rules.
set -euo pipefail

# Never inherit a poisoned target dir from another worktree/agent.
unset CARGO_TARGET_DIR || true

# Re-exec under the project flake when not already in a nix develop shell
# (or when required tools are missing). Prefer: nix develop -c ./judge.sh …
ensure_nix() {
  local need=0
  command -v cargo >/dev/null 2>&1 || need=1
  command -v rustc >/dev/null 2>&1 || need=1
  command -v strip >/dev/null 2>&1 || need=1
  command -v cmp >/dev/null 2>&1 || need=1
  command -v sha256sum >/dev/null 2>&1 || need=1
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
  exec env -u CARGO_TARGET_DIR nix develop "${root}" -c "$0" "$@"
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
# Only clear this dataset's artifacts (do not wipe unrelated judge_work contents)
rm -f \
  "${WORK}/${DATASET}.compressed" \
  "${WORK}/${DATASET}.compressed2" \
  "${WORK}/${DATASET}.roundtrip" \
  "${WORK}/${DATASET}.roundtrip2" \
  "${WORK}/time_compress.txt" \
  "${WORK}/time_decompress.txt" \
  "${WORK}/resources.json" \
  "${WORK}/${DATASET}.hashes.txt" 2>/dev/null || true

COMPRESS_BIN="${WORKSPACE}/target/release/prometheus-compress"
DECOMPRESS_BIN="${WORKSPACE}/target/release/prometheus-decompress"
COMPRESSED="${WORK}/${DATASET}.compressed"
ROUNDTRIP="${WORK}/${DATASET}.roundtrip"
COMPRESSED2="${WORK}/${DATASET}.compressed2"
ROUNDTRIP2="${WORK}/${DATASET}.roundtrip2"
TIME_COMP="${WORK}/time_compress.txt"
TIME_DECOMP="${WORK}/time_decompress.txt"

# Working ceilings (Phase 2 provisional until host Geekbench T is measured).
# Official Hutter: each program < 70000/T hours, peak RAM ≤ 10 GB (hrules.htm).
WALL_LIMIT_SECONDS="${PROMETHEUS_WALL_LIMIT_SECONDS:-14400}"  # 4h per direction
RAM_LIMIT_MB="${PROMETHEUS_RAM_LIMIT_MB:-9216}"              # 9 GB working ceiling

fail_only() {
  # Hygiene: never emit SCORE=… on failure.
  echo "ROUNDTRIP=FAIL"
  exit 1
}

# --- release build (size opts live in workspace Cargo.toml [profile.release]) ---
(
  cd "${WORKSPACE}"
  unset CARGO_TARGET_DIR || true
  cargo build --release -p prometheus-compress -p prometheus-decompress
  # flake cargo-wrap mirrors to target/release/; ensure present for build-std
  if [[ ! -f target/release/prometheus-decompress ]] && \
     [[ -f target/x86_64-unknown-linux-gnu/release/prometheus-decompress ]]; then
    mkdir -p target/release
    cp -f target/x86_64-unknown-linux-gnu/release/prometheus-compress target/release/
    cp -f target/x86_64-unknown-linux-gnu/release/prometheus-decompress target/release/
  fi
)

if [[ ! -x "${COMPRESS_BIN}" || ! -x "${DECOMPRESS_BIN}" ]]; then
  echo "judge.sh: missing release binaries under ${WORKSPACE}/target/release/" >&2
  fail_only
fi

# Extra strip pass (Cargo strip=symbols may leave some symbols; be explicit)
strip -s "${COMPRESS_BIN}" "${DECOMPRESS_BIN}" 2>/dev/null || strip "${COMPRESS_BIN}" "${DECOMPRESS_BIN}"

# --- compress (timed + peak RSS + CPU) ---
# %e wall s, %U user s, %S sys s, %M max RSS KB (GNU time)
set +e
command time -f '%e %U %S %M' -o "${TIME_COMP}" \
  "${COMPRESS_BIN}" "${ROOT}/${DATASET}" "${COMPRESSED}"
COMP_RC=$?
set -e
if [[ "${COMP_RC}" -ne 0 ]]; then
  echo "judge.sh: compress failed rc=${COMP_RC}" >&2
  fail_only
fi

# --- decompress (timed + peak RSS + CPU); must not need original ---
set +e
command time -f '%e %U %S %M' -o "${TIME_DECOMP}" \
  "${DECOMPRESS_BIN}" "${COMPRESSED}" "${ROUNDTRIP}"
DECOMP_RC=$?
set -e

# --- round-trip: byte-identical + hashes ---
ORIG_HASH="$(sha256sum "${ROOT}/${DATASET}" | awk '{print $1}')"
RT_HASH="FAIL"
ROUNDTRIP_STATUS="FAIL"
if [[ "${DECOMP_RC}" -eq 0 && -f "${ROUNDTRIP}" ]]; then
  RT_HASH="$(sha256sum "${ROUNDTRIP}" | awk '{print $1}')"
  if [[ "${ORIG_HASH}" == "${RT_HASH}" ]] && cmp -s "${ROOT}/${DATASET}" "${ROUNDTRIP}"; then
    ROUNDTRIP_STATUS="OK"
  fi
fi
{
  echo "ORIG_SHA256=${ORIG_HASH}"
  echo "ROUNDTRIP_SHA256=${RT_HASH}"
} > "${WORK}/${DATASET}.hashes.txt"

# --- determinism: compress×2 identical (hash) ---
"${COMPRESS_BIN}" "${ROOT}/${DATASET}" "${COMPRESSED2}"
COMP_HASH="$(sha256sum "${COMPRESSED}" | awk '{print $1}')"
COMP2_HASH="$(sha256sum "${COMPRESSED2}" | awk '{print $1}')"
if [[ "${COMP_HASH}" != "${COMP2_HASH}" ]]; then
  ROUNDTRIP_STATUS="FAIL"
  echo "judge.sh: compress determinism failed (archive hash mismatch)" >&2
fi

# --- determinism: decompress×2 identical ---
env -i PATH="${PATH}" HOME="${HOME}" \
  "${DECOMPRESS_BIN}" "${COMPRESSED}" "${ROUNDTRIP2}"
if ! cmp -s "${ROUNDTRIP}" "${ROUNDTRIP2}"; then
  ROUNDTRIP_STATUS="FAIL"
  echo "judge.sh: decompress determinism failed (clean-env mismatch)" >&2
fi

if [[ "${ROUNDTRIP_STATUS}" != "OK" ]]; then
  echo "ORIG_SHA256=${ORIG_HASH} ROUNDTRIP_SHA256=${RT_HASH}" >&2
  fail_only
fi

# --- sizes (only after round-trip OK) ---
COMPRESSED_BYTES=$(wc -c < "${COMPRESSED}" | tr -d ' ')
DECOMPRESSOR_BYTES=$(wc -c < "${DECOMPRESS_BIN}" | tr -d ' ')
AUX_BYTES=0
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
      fail_only
    fi
  done < "${WORKSPACE}/judge_aux.txt"
fi
DECOMPRESSOR_BYTES=$((DECOMPRESSOR_BYTES + AUX_BYTES))
SCORE=$((COMPRESSED_BYTES + DECOMPRESSOR_BYTES))

# --- parse timings (wall + CPU + RSS) ---
read -r COMPRESS_SECONDS COMP_USER COMP_SYS COMP_RSS_KB < "${TIME_COMP}"
read -r DECOMPRESS_SECONDS DECOMP_USER DECOMP_SYS DECOMP_RSS_KB < "${TIME_DECOMP}"

PEAK_RSS_KB="${COMP_RSS_KB}"
if [[ "${DECOMP_RSS_KB}" -gt "${PEAK_RSS_KB}" ]]; then
  PEAK_RSS_KB="${DECOMP_RSS_KB}"
fi
PEAK_RSS_MB=$(( (PEAK_RSS_KB + 1023) / 1024 ))

CPU_COMPRESS_SECONDS="$(awk -v u="${COMP_USER}" -v s="${COMP_SYS}" 'BEGIN{printf "%.2f", u+s}')"
CPU_DECOMPRESS_SECONDS="$(awk -v u="${DECOMP_USER}" -v s="${DECOMP_SYS}" 'BEGIN{printf "%.2f", u+s}')"
RAM_TIME_GB_H="$(awk -v kb="${PEAK_RSS_KB}" -v c="${COMPRESS_SECONDS}" -v d="${DECOMPRESS_SECONDS}" \
  'BEGIN{printf "%.4f", (kb/1048576.0)*((c+d)/3600.0)}')"
WALL_TOTAL_SECONDS="$(awk -v c="${COMPRESS_SECONDS}" -v d="${DECOMPRESS_SECONDS}" 'BEGIN{printf "%.2f", c+d}')"
CPU_TOTAL_SECONDS="$(awk -v c="${CPU_COMPRESS_SECONDS}" -v d="${CPU_DECOMPRESS_SECONDS}" 'BEGIN{printf "%.2f", c+d}')"

# Limit verdicts (working ceilings; not a substitute for official 70000/T)
TIME_LIMIT_PASS=PASS
awk -v c="${COMPRESS_SECONDS}" -v d="${DECOMPRESS_SECONDS}" -v lim="${WALL_LIMIT_SECONDS}" \
  'BEGIN{ if (c>lim || d>lim) exit 1 }' || TIME_LIMIT_PASS=FAIL
RAM_LIMIT_PASS=PASS
if [[ "${PEAK_RSS_MB}" -gt "${RAM_LIMIT_MB}" ]]; then
  RAM_LIMIT_PASS=FAIL
fi

cat > "${WORK}/resources.json" <<EOF
{
  "dataset": "${DATASET}",
  "compress_wall_s": ${COMPRESS_SECONDS},
  "decompress_wall_s": ${DECOMPRESS_SECONDS},
  "wall_total_s": ${WALL_TOTAL_SECONDS},
  "compress_cpu_s": ${CPU_COMPRESS_SECONDS},
  "decompress_cpu_s": ${CPU_DECOMPRESS_SECONDS},
  "cpu_total_s": ${CPU_TOTAL_SECONDS},
  "peak_rss_mb": ${PEAK_RSS_MB},
  "peak_rss_kb": ${PEAK_RSS_KB},
  "ram_time_gb_h": ${RAM_TIME_GB_H},
  "wall_limit_s": ${WALL_LIMIT_SECONDS},
  "ram_limit_mb": ${RAM_LIMIT_MB},
  "time_limit_pass": "${TIME_LIMIT_PASS}",
  "ram_limit_pass": "${RAM_LIMIT_PASS}",
  "orig_sha256": "${ORIG_HASH}",
  "roundtrip_sha256": "${RT_HASH}",
  "archive_sha256": "${COMP_HASH}",
  "hutter_peak_ram_limit_gb": 10,
  "under_10gb_ram": $([ "${PEAK_RSS_MB}" -le 10240 ] && echo true || echo false)
}
EOF

echo "ROUNDTRIP=OK COMPRESSED=${COMPRESSED_BYTES} DECOMPRESSOR=${DECOMPRESSOR_BYTES} SCORE=${SCORE} COMPRESS_SECONDS=${COMPRESS_SECONDS} DECOMPRESS_SECONDS=${DECOMPRESS_SECONDS} PEAK_RSS_MB=${PEAK_RSS_MB} CPU_COMPRESS_SECONDS=${CPU_COMPRESS_SECONDS} CPU_DECOMPRESS_SECONDS=${CPU_DECOMPRESS_SECONDS} CPU_TOTAL_SECONDS=${CPU_TOTAL_SECONDS} WALL_TOTAL_SECONDS=${WALL_TOTAL_SECONDS} RAM_TIME_GB_H=${RAM_TIME_GB_H} TIME_LIMIT=${TIME_LIMIT_PASS} RAM_LIMIT=${RAM_LIMIT_PASS} ORIG_SHA256=${ORIG_HASH} ROUNDTRIP_SHA256=${RT_HASH} ARCHIVE_SHA256=${COMP_HASH}"

if [[ "${TIME_LIMIT_PASS}" != "PASS" || "${RAM_LIMIT_PASS}" != "PASS" ]]; then
  echo "judge.sh: working ceiling FAIL TIME_LIMIT=${TIME_LIMIT_PASS} RAM_LIMIT=${RAM_LIMIT_PASS}" >&2
  exit 1
fi
