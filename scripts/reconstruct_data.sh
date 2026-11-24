#!/usr/bin/env bash
# Rebuild enwik8 / enwik9 from GitHub-safe 50_000_000-byte slices.
# Usage:
#   ./scripts/reconstruct_data.sh          # both
#   ./scripts/reconstruct_data.sh enwik8
#   ./scripts/reconstruct_data.sh enwik9
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET="${1:-both}"

reconstruct_parts() {
  local name="$1" expect_bytes="$2" nparts="$3" prefix="$4"
  local parts_dir="${ROOT}/data/${name}.parts"
  local out="${ROOT}/${name}"
  local got
  got=$(ls -1 "${parts_dir}/${prefix}"_[0-9][0-9] 2>/dev/null | wc -l)
  if [[ "${got}" -ne "${nparts}" ]]; then
    echo "reconstruct: ${name}: expected ${nparts} parts in ${parts_dir}, found ${got}" >&2
    return 1
  fi
  echo "Writing ${out} (${nparts} × 50MB parts)…"
  cat "${parts_dir}/${prefix}"_?? > "${out}"
  local bytes
  bytes=$(wc -c < "${out}")
  if [[ "${bytes}" -ne "${expect_bytes}" ]]; then
    echo "reconstruct: ${name} size ${bytes} != ${expect_bytes}" >&2
    return 1
  fi
  if [[ -f "${parts_dir}/SHA256SUMS" ]]; then
    (cd "${parts_dir}" && sha256sum -c SHA256SUMS)
  fi
  echo "OK ${name}=${bytes}"
}

case "${TARGET}" in
  enwik8) reconstruct_parts enwik8 100000000 2 enwik8.part ;;
  enwik9) reconstruct_parts enwik9 1000000000 20 enwik9.part ;;
  both)
    reconstruct_parts enwik8 100000000 2 enwik8.part
    reconstruct_parts enwik9 1000000000 20 enwik9.part
    ;;
  *)
    echo "usage: $0 [enwik8|enwik9|both]" >&2
    exit 2
    ;;
esac
