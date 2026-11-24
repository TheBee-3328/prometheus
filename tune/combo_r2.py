#!/usr/bin/env python3
"""Combo probe: MIX_WEIGHT[0] x MIX_WEIGHT[2] around slice best."""
from __future__ import annotations

import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
WRITE = ROOT / "tune" / "write_params.py"
SLICE = ROOT / "tune" / "slice_eval.sh"


def ev(p: dict) -> int:
    subprocess.check_call(
        [sys.executable, str(WRITE), "--json", json.dumps(p)], cwd=str(ROOT)
    )
    out = subprocess.check_output(["bash", str(SLICE)], cwd=str(ROOT), text=True)
    for line in out.splitlines():
        if line.startswith("COMPRESSED="):
            return int(line.split("=", 1)[1])
    raise RuntimeError(out)


def main() -> None:
    base = {
        "ORDER2_RESCALE": 100,
        "BUF_BITS": 25,
        "HASH_BITS": 26,
        "HASH_ORDER": 6,
        "MIN_MIX_LEN": 6,
        "MATCH_CONF": [32700] * 5,
        "MIX_WEIGHT": [120, 160, 190, 230, 245],
    }
    cands: list[tuple[str, dict]] = []
    for m0 in [115, 120, 122, 125]:
        for m2 in [180, 185, 188, 190, 192, 195]:
            p = json.loads(json.dumps(base))
            p["MIX_WEIGHT"][0] = m0
            p["MIX_WEIGHT"][2] = m2
            cands.append((f"m0={m0},m2={m2}", p))
    for m1 in [155, 160, 165]:
        p = json.loads(json.dumps(base))
        p["MIX_WEIGHT"][1] = m1
        cands.append((f"m1={m1},m2=190", p))

    results: list[tuple[int, str, dict]] = []
    for note, p in cands:
        size = ev(p)
        print(size, note, p["MIX_WEIGHT"], flush=True)
        results.append((size, note, p))

    results.sort(key=lambda x: x[0])
    print("COMBO_TOP", results[0][0], results[0][1], json.dumps(results[0][2]), flush=True)
    (ROOT / "tune" / "combo_r2.json").write_text(
        json.dumps(
            [{"size": s, "note": n, "params": p} for s, n, p in results[:6]],
            indent=2,
        )
    )
    (ROOT / "tune" / "candidates").mkdir(exist_ok=True)
    for i, (s, n, p) in enumerate(results[:3]):
        name = ["A", "B", "C"][i]
        safe = n.replace(",", "_").replace("=", "")
        path = ROOT / "tune" / "candidates" / f"R2_{name}_{safe}.json"
        path.write_text(json.dumps(p, indent=2) + "\n")
        print("WROTE", path, s, flush=True)

    # Update best_params to combo top.
    best_p = results[0][2]
    (ROOT / "tune" / "best_params.json").write_text(json.dumps(best_p, indent=2) + "\n")
    subprocess.check_call(
        [sys.executable, str(WRITE), "--json", json.dumps(best_p)], cwd=str(ROOT)
    )


if __name__ == "__main__":
    main()
