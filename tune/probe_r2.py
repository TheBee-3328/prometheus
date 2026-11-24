#!/usr/bin/env python3
"""Quick local probes around current slice best (MIX / RESCALE)."""
from __future__ import annotations

import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
WRITE = ROOT / "tune" / "write_params.py"
SLICE = ROOT / "tune" / "slice_eval.sh"

base = {
    "ORDER2_RESCALE": 100,
    "BUF_BITS": 25,
    "HASH_BITS": 26,
    "HASH_ORDER": 6,
    "MIN_MIX_LEN": 6,
    "MATCH_CONF": [32700] * 5,
    "MIX_WEIGHT": [120, 160, 200, 230, 245],
}


def clone(p: dict) -> dict:
    return json.loads(json.dumps(p))


def eval_one(p: dict) -> int:
    subprocess.check_call(
        [sys.executable, str(WRITE), "--json", json.dumps(p)], cwd=str(ROOT)
    )
    out = subprocess.check_output(["bash", str(SLICE)], cwd=str(ROOT), text=True)
    for line in out.splitlines():
        if line.startswith("COMPRESSED="):
            return int(line.split("=", 1)[1])
    raise RuntimeError(out)


def main() -> None:
    probes: list[tuple[str, dict]] = []
    for m0 in [118, 122, 125, 130, 140]:
        p = clone(base)
        p["MIX_WEIGHT"][0] = m0
        probes.append((f"mix0={m0}", p))
    for m1 in [150, 155, 165, 170]:
        p = clone(base)
        p["MIX_WEIGHT"][1] = m1
        probes.append((f"mix1={m1}", p))
    for m2 in [190, 195, 205, 210]:
        p = clone(base)
        p["MIX_WEIGHT"][2] = m2
        probes.append((f"mix2={m2}", p))
    for m3 in [220, 225, 235, 240]:
        p = clone(base)
        p["MIX_WEIGHT"][3] = m3
        probes.append((f"mix3={m3}", p))
    for m4 in [240, 250]:
        p = clone(base)
        p["MIX_WEIGHT"][4] = m4
        probes.append((f"mix4={m4}", p))
    for res in [90, 95, 105]:
        p = clone(base)
        p["ORDER2_RESCALE"] = res
        probes.append((f"res={res}", p))

    results: list[tuple[int, str, dict]] = []
    for note, p in probes:
        size = eval_one(p)
        mix = p["MIX_WEIGHT"]
        print(f"{size} {note} mix={mix} res={p['ORDER2_RESCALE']}", flush=True)
        results.append((size, note, p))

    results.sort(key=lambda x: x[0])
    top = results[0]
    print("PROBE_TOP", top[0], top[1], json.dumps(top[2]), flush=True)
    (ROOT / "tune" / "probe_r2.json").write_text(
        json.dumps(
            [{"size": s, "note": n, "params": p} for s, n, p in results[:10]],
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
