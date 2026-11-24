#!/usr/bin/env python3
"""TUNER-R3 fine grid: MIX[0]/MIX[2]/STRUCT_MIX/ORDER2 around post-P5 best.

Phase A: 1D sweeps (slice).
Phase B: 2D combo of top MIX[0] x MIX[2] at best STRUCT/RESCALE.
Phase C: write top candidates for full validate.
"""
from __future__ import annotations

import csv
import json
import pathlib
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
WRITE = ROOT / "tune" / "write_params.py"
SLICE = ROOT / "tune" / "slice_eval.sh"
TRIALS = ROOT / "tune" / "trials_r3.csv"
BEST_JSON = ROOT / "tune" / "best_params.json"
CAND_DIR = ROOT / "tune" / "candidates"
RESULTS = ROOT / "tune" / "grid_r3.json"

BASE = {
    "ORDER2_RESCALE": 100,
    "BUF_BITS": 25,
    "HASH_BITS": 26,
    "HASH_ORDER": 6,
    "MIN_MIX_LEN": 6,
    "MATCH_CONF": [32700, 32700, 32700, 32700, 32700],
    "MIX_WEIGHT": [120, 160, 190, 230, 245],
    "STRUCT_MIX_WEIGHT": 14,
}


def clone(p: dict) -> dict:
    return json.loads(json.dumps(p))


def apply(p: dict) -> None:
    subprocess.check_call(
        [sys.executable, str(WRITE), "--json", json.dumps(p)], cwd=str(ROOT)
    )


def eval_slice(p: dict) -> int:
    apply(p)
    out = subprocess.check_output(["bash", str(SLICE)], cwd=str(ROOT), text=True)
    for line in out.splitlines():
        if line.startswith("COMPRESSED="):
            return int(line.split("=", 1)[1])
    raise RuntimeError(out)


def log_row(writer, trial: int, t0: float, size: int, best: int, p: dict, note: str) -> None:
    writer.writerow(
        [
            trial,
            f"{time.time() - t0:.1f}",
            size,
            size - best,
            json.dumps(p, separators=(",", ":")),
            note,
        ]
    )


def main() -> None:
    t0 = time.time()
    deadline = t0 + 42 * 60  # leave ~8+ min for full validates within soft 50
    CAND_DIR.mkdir(exist_ok=True)

    seed = clone(BASE)
    if BEST_JSON.exists():
        loaded = json.loads(BEST_JSON.read_text())
        seed.update(loaded)
        seed["MATCH_CONF"] = [int(x) for x in seed["MATCH_CONF"]]
        seed["MIX_WEIGHT"] = [int(x) for x in seed["MIX_WEIGHT"]]
        seed.setdefault("STRUCT_MIX_WEIGHT", 14)

    results: list[tuple[int, str, dict]] = []
    trial = 0

    with TRIALS.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(
            ["trial", "elapsed_s", "slice_compressed", "delta_vs_best", "params_json", "note"]
        )

        print("=== baseline ===", flush=True)
        best_p = clone(seed)
        best_size = eval_slice(best_p)
        trial += 1
        log_row(w, trial, t0, best_size, best_size, best_p, "baseline")
        f.flush()
        print(f"baseline slice={best_size} {json.dumps(best_p)}", flush=True)
        results.append((best_size, "baseline", clone(best_p)))

        def try_cand(note: str, p: dict) -> int | None:
            nonlocal trial, best_size, best_p
            if time.time() >= deadline:
                print(f"deadline hit before {note}", flush=True)
                return None
            size = eval_slice(p)
            trial += 1
            log_row(w, trial, t0, size, best_size, p, note)
            f.flush()
            delta = size - best_size
            print(f"t{trial} {size} ({delta:+d}) {note}", flush=True)
            results.append((size, note, clone(p)))
            if size < best_size:
                best_size = size
                best_p = clone(p)
                BEST_JSON.write_text(json.dumps(best_p, indent=2) + "\n")
            return size

        # Phase A1: STRUCT_MIX_WEIGHT sweep around 14
        print("=== STRUCT_MIX_WEIGHT sweep ===", flush=True)
        for sm in [8, 10, 12, 13, 14, 15, 16, 18, 20, 24]:
            p = clone(seed)
            p["STRUCT_MIX_WEIGHT"] = sm
            if try_cand(f"struct={sm}", p) is None:
                break

        # Phase A2: ORDER2_RESCALE around 100 (at current best struct)
        print("=== ORDER2_RESCALE sweep ===", flush=True)
        for res in [90, 95, 98, 100, 102, 105, 110, 115]:
            p = clone(best_p)
            p["ORDER2_RESCALE"] = res
            if try_cand(f"res={res}", p) is None:
                break

        # Phase A3: MIX_WEIGHT[0] ∈ {118..125}
        print("=== MIX_WEIGHT[0] sweep ===", flush=True)
        for m0 in range(118, 126):
            p = clone(best_p)
            p["MIX_WEIGHT"][0] = m0
            if try_cand(f"m0={m0}", p) is None:
                break

        # Phase A4: MIX_WEIGHT[2] ∈ {186..194}
        print("=== MIX_WEIGHT[2] sweep ===", flush=True)
        for m2 in range(186, 195):
            p = clone(best_p)
            p["MIX_WEIGHT"][2] = m2
            if try_cand(f"m2={m2}", p) is None:
                break

        # Phase A5: light MIX[1]/MIX[3] nudges
        print("=== MIX[1]/MIX[3] nudges ===", flush=True)
        for i, vals in [(1, [155, 158, 160, 162, 165]), (3, [225, 228, 230, 232, 235])]:
            for v in vals:
                p = clone(best_p)
                p["MIX_WEIGHT"][i] = v
                if try_cand(f"m{i}={v}", p) is None:
                    break

        # Phase B: combo top m0 x m2 at best STRUCT/RESCALE
        # Pick unique top-3 m0 and m2 from results that only changed those dims,
        # plus current best dims.
        print("=== combo m0 x m2 ===", flush=True)
        m0_scores: dict[int, int] = {}
        m2_scores: dict[int, int] = {}
        for size, note, p in results:
            if note.startswith("m0="):
                m0_scores[p["MIX_WEIGHT"][0]] = size
            elif note.startswith("m2="):
                m2_scores[p["MIX_WEIGHT"][2]] = size
        # Always include current best and neighborhood
        m0_cands = sorted(
            set(
                [best_p["MIX_WEIGHT"][0], seed["MIX_WEIGHT"][0]]
                + sorted(m0_scores, key=m0_scores.get)[:3]
                + [118, 120, 122, 125]
            )
        )
        m2_cands = sorted(
            set(
                [best_p["MIX_WEIGHT"][2], seed["MIX_WEIGHT"][2]]
                + sorted(m2_scores, key=m2_scores.get)[:3]
                + [186, 188, 190, 192, 194]
            )
        )
        # Cap combo size for time
        m0_cands = m0_cands[:6]
        m2_cands = m2_cands[:6]
        print(f"combo space m0={m0_cands} m2={m2_cands}", flush=True)
        for m0 in m0_cands:
            for m2 in m2_cands:
                if time.time() >= deadline:
                    print("deadline hit in combo", flush=True)
                    break
                p = clone(best_p)
                p["MIX_WEIGHT"][0] = m0
                p["MIX_WEIGHT"][2] = m2
                # skip if identical to already-tried exact params
                key = json.dumps(p, sort_keys=True)
                if any(json.dumps(r[2], sort_keys=True) == key for r in results):
                    continue
                if try_cand(f"combo m0={m0},m2={m2}", p) is None:
                    break
            else:
                continue
            break

        # Phase B2: STRUCT x RESCALE cross at best mix (small)
        print("=== struct x res cross ===", flush=True)
        for sm in [best_p["STRUCT_MIX_WEIGHT"] - 2, best_p["STRUCT_MIX_WEIGHT"], best_p["STRUCT_MIX_WEIGHT"] + 2]:
            if sm < 0:
                continue
            for res in [best_p["ORDER2_RESCALE"] - 5, best_p["ORDER2_RESCALE"], best_p["ORDER2_RESCALE"] + 5]:
                if res < 50:
                    continue
                p = clone(best_p)
                p["STRUCT_MIX_WEIGHT"] = sm
                p["ORDER2_RESCALE"] = res
                key = json.dumps(p, sort_keys=True)
                if any(json.dumps(r[2], sort_keys=True) == key for r in results):
                    continue
                if try_cand(f"cross sm={sm},res={res}", p) is None:
                    break

    # Rank + write candidates
    results.sort(key=lambda x: x[0])
    top = results[:8]
    RESULTS.write_text(
        json.dumps([{"size": s, "note": n, "params": p} for s, n, p in top], indent=2) + "\n"
    )
    BEST_JSON.write_text(json.dumps(results[0][2], indent=2) + "\n")
    apply(results[0][2])

    for i, (s, n, p) in enumerate(top[:5]):
        name = ["A", "B", "C", "D", "E"][i]
        path = CAND_DIR / f"R3_{name}.json"
        path.write_text(json.dumps(p, indent=2) + "\n")
        print(f"WROTE {path} slice={s} {n}", flush=True)

    print(f"DONE trials={trial} best_slice={results[0][0]} elapsed={time.time()-t0:.0f}s", flush=True)
    print("BEST", json.dumps(results[0][2]), flush=True)


if __name__ == "__main__":
    main()
