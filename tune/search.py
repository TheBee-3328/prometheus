#!/usr/bin/env python3
"""Coordinate descent + seeded evolutionary search over params.rs.

Uses a fixed 25 MB enwik8 slice (offset 8_000_000). Logs every trial to
tune/trials.csv. Budget controlled by --minutes / --max-trials.
"""
from __future__ import annotations

import argparse
import csv
import json
import os
import pathlib
import random
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
TRIALS = ROOT / "tune" / "trials.csv"
BEST_JSON = ROOT / "tune" / "best_params.json"
WRITE = ROOT / "tune" / "write_params.py"
SLICE_EVAL = ROOT / "tune" / "slice_eval.sh"

DEFAULTS = {
    "ORDER2_RESCALE": 250,
    "BUF_BITS": 24,
    "HASH_BITS": 23,
    "HASH_ORDER": 4,
    "MIN_MIX_LEN": 5,
    "MATCH_CONF": [14000, 20000, 26000, 30000, 32000],
    "MIX_WEIGHT": [100, 160, 200, 230, 245],
}

# Discrete search domains (RAM-safe: BUF<=26, HASH<=26 → ~64+256 MB tables).
DOMAINS = {
    "ORDER2_RESCALE": [120, 160, 200, 220, 250, 280, 320, 400],
    "BUF_BITS": [22, 23, 24, 25, 26],
    "HASH_BITS": [21, 22, 23, 24, 25, 26],
    "HASH_ORDER": [3, 4, 5, 6],
    "MIN_MIX_LEN": [3, 4, 5, 6, 7, 8],
}


def clamp_conf(v: int) -> int:
    return max(2000, min(32700, int(v)))


def clamp_mix(v: int) -> int:
    return max(20, min(250, int(v)))


def ensure_header() -> None:
    if not TRIALS.exists():
        with TRIALS.open("w", newline="") as f:
            w = csv.writer(f)
            w.writerow(
                [
                    "trial",
                    "elapsed_s",
                    "slice_compressed",
                    "delta_vs_best",
                    "params_json",
                    "note",
                ]
            )


def apply_params(p: dict) -> None:
    subprocess.check_call(
        [sys.executable, str(WRITE), "--json", json.dumps(p)],
        cwd=str(ROOT),
    )


def eval_slice(env: dict | None = None) -> int:
    e = os.environ.copy()
    if env:
        e.update(env)
    out = subprocess.check_output(["bash", str(SLICE_EVAL)], cwd=str(ROOT), env=e, text=True)
    for line in out.strip().splitlines():
        if line.startswith("COMPRESSED="):
            return int(line.split("=", 1)[1])
    raise RuntimeError(f"no COMPRESSED= in output:\n{out}")


def log_trial(trial: int, t0: float, size: int, best: int, p: dict, note: str) -> None:
    with TRIALS.open("a", newline="") as f:
        w = csv.writer(f)
        w.writerow(
            [
                trial,
                f"{time.time() - t0:.1f}",
                size,
                size - best,
                json.dumps(p, separators=(",", ":")),
                note,
            ]
        )


def mutate(rng: random.Random, p: dict) -> dict:
    q = json.loads(json.dumps(p))
    kind = rng.choice(
        [
            "ORDER2_RESCALE",
            "BUF_BITS",
            "HASH_BITS",
            "HASH_ORDER",
            "MIN_MIX_LEN",
            "MATCH_CONF",
            "MIX_WEIGHT",
            "MATCH_CONF",
            "MIX_WEIGHT",
        ]
    )
    if kind in DOMAINS:
        q[kind] = rng.choice(DOMAINS[kind])
    elif kind == "MATCH_CONF":
        i = rng.randrange(5)
        step = rng.choice([-4000, -2000, -1000, 1000, 2000, 4000])
        q["MATCH_CONF"][i] = clamp_conf(q["MATCH_CONF"][i] + step)
        # Keep non-decreasing-ish for longer matches (soft).
        for j in range(1, 5):
            if q["MATCH_CONF"][j] < q["MATCH_CONF"][j - 1] - 2000:
                q["MATCH_CONF"][j] = q["MATCH_CONF"][j - 1]
    else:
        i = rng.randrange(5)
        step = rng.choice([-40, -20, -10, 10, 20, 40])
        q["MIX_WEIGHT"][i] = clamp_mix(q["MIX_WEIGHT"][i] + step)
        for j in range(1, 5):
            if q["MIX_WEIGHT"][j] < q["MIX_WEIGHT"][j - 1] - 20:
                q["MIX_WEIGHT"][j] = q["MIX_WEIGHT"][j - 1]
    # Soft RAM guard: huge tables only with smaller partner.
    if q["BUF_BITS"] + q["HASH_BITS"] > 50:
        if rng.random() < 0.5:
            q["BUF_BITS"] = min(q["BUF_BITS"], 25)
        else:
            q["HASH_BITS"] = min(q["HASH_BITS"], 25)
    return q


def coordinate_neighbors(p: dict) -> list[tuple[str, dict]]:
    out: list[tuple[str, dict]] = []
    for key, vals in DOMAINS.items():
        cur = p[key]
        if cur not in vals:
            vals = sorted(set(vals + [cur]))
        idx = vals.index(cur) if cur in vals else -1
        for j in (idx - 1, idx + 1):
            if 0 <= j < len(vals) and vals[j] != cur:
                q = json.loads(json.dumps(p))
                q[key] = vals[j]
                out.append((f"{key}={vals[j]}", q))
    # MIX / CONF steps around current.
    for i in range(5):
        for step in (-2000, 2000):
            q = json.loads(json.dumps(p))
            q["MATCH_CONF"][i] = clamp_conf(q["MATCH_CONF"][i] + step)
            out.append((f"MATCH_CONF[{i}]{step:+d}", q))
        for step in (-20, 20):
            q = json.loads(json.dumps(p))
            q["MIX_WEIGHT"][i] = clamp_mix(q["MIX_WEIGHT"][i] + step)
            out.append((f"MIX_WEIGHT[{i}]{step:+d}", q))
    return out


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=3328)
    ap.add_argument("--minutes", type=float, default=45.0)
    ap.add_argument("--max-trials", type=int, default=80)
    ap.add_argument("--skip-baseline", action="store_true")
    args = ap.parse_args()

    ensure_header()
    rng = random.Random(args.seed)
    t0 = time.time()
    deadline = t0 + args.minutes * 60.0

    # Extract slice once.
    slice_path = ROOT / "tune" / "enwik8.slice"
    if not slice_path.exists():
        subprocess.check_call(
            [
                "dd",
                "if=enwik8",
                f"of={slice_path}",
                "bs=1000000",
                "skip=8",
                "count=25",
                "status=none",
            ],
            cwd=str(ROOT),
        )

    best_p = dict(DEFAULTS)
    best_p["MATCH_CONF"] = list(DEFAULTS["MATCH_CONF"])
    best_p["MIX_WEIGHT"] = list(DEFAULTS["MIX_WEIGHT"])
    trial = 0

    if not args.skip_baseline:
        apply_params(best_p)
        best_size = eval_slice()
        trial += 1
        log_trial(trial, t0, best_size, best_size, best_p, "baseline")
        print(f"baseline slice={best_size}", flush=True)
    else:
        best_size = 10**18

    # Phase 1: coordinate descent
    improved = True
    while improved and trial < args.max_trials and time.time() < deadline:
        improved = False
        for note, cand in coordinate_neighbors(best_p):
            if trial >= args.max_trials or time.time() >= deadline:
                break
            apply_params(cand)
            try:
                size = eval_slice()
            except subprocess.CalledProcessError as e:
                print(f"trial failed: {e}", flush=True)
                continue
            trial += 1
            log_trial(trial, t0, size, best_size, cand, f"cd:{note}")
            print(f"t{trial} {size} (best {best_size}) {note}", flush=True)
            if size < best_size:
                best_size = size
                best_p = cand
                improved = True
                BEST_JSON.write_text(json.dumps(best_p, indent=2))
                # Restart neighbors from new best.
                break

    # Phase 2: evolutionary mutations from best
    while trial < args.max_trials and time.time() < deadline:
        cand = mutate(rng, best_p)
        apply_params(cand)
        try:
            size = eval_slice()
        except subprocess.CalledProcessError as e:
            print(f"trial failed: {e}", flush=True)
            continue
        trial += 1
        note = "evo"
        log_trial(trial, t0, size, best_size, cand, note)
        print(f"t{trial} {size} (best {best_size}) evo", flush=True)
        if size < best_size:
            best_size = size
            best_p = cand
            BEST_JSON.write_text(json.dumps(best_p, indent=2))

    apply_params(best_p)
    BEST_JSON.write_text(json.dumps(best_p, indent=2))
    print(f"DONE trials={trial} best_slice={best_size}", flush=True)
    print(json.dumps(best_p), flush=True)


if __name__ == "__main__":
    main()
