#!/usr/bin/env python3
"""Coordinate descent + focused grids + evolutionary search over params.rs.

Round-2 tuner: starts from the known full-file winner, explores MIX_WEIGHT /
ORDER2_RESCALE / short-match CONF more densely, then mutates. Uses a fixed
25 MB enwik8 slice (offset 8_000_000). Logs every trial to tune/trials.csv.
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

# Known full-file winner (SCORE 28080343 era).
WINNER = {
    "ORDER2_RESCALE": 120,
    "BUF_BITS": 25,
    "HASH_BITS": 26,
    "HASH_ORDER": 6,
    "MIN_MIX_LEN": 6,
    "MATCH_CONF": [32700, 32700, 32700, 32700, 32700],
    "MIX_WEIGHT": [100, 160, 200, 230, 245],
}

DEFAULTS = dict(WINNER)

DOMAINS = {
    "ORDER2_RESCALE": [80, 90, 100, 110, 115, 120, 125, 130, 140, 160, 200, 250],
    "BUF_BITS": [24, 25, 26],
    "HASH_BITS": [24, 25, 26],
    "HASH_ORDER": [4, 5, 6, 7, 8],
    "MIN_MIX_LEN": [4, 5, 6, 7, 8],
}


def clamp_conf(v: int) -> int:
    return max(2000, min(32700, int(v)))


def clamp_mix(v: int) -> int:
    return max(20, min(250, int(v)))


def copy_params(p: dict) -> dict:
    q = dict(p)
    q["MATCH_CONF"] = list(p["MATCH_CONF"])
    q["MIX_WEIGHT"] = list(p["MIX_WEIGHT"])
    return q


def ensure_header(reset: bool) -> None:
    if reset or not TRIALS.exists():
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
    out = subprocess.check_output(
        ["bash", str(SLICE_EVAL)], cwd=str(ROOT), env=e, text=True
    )
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


def ram_ok(p: dict) -> bool:
    buf = 1 << int(p["BUF_BITS"])
    ht = (1 << int(p["HASH_BITS"])) * 4
    return (buf + ht) <= 1_600_000_000 and (int(p["BUF_BITS"]) + int(p["HASH_BITS"])) <= 52


def mutate(rng: random.Random, p: dict) -> dict:
    q = copy_params(p)
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
            "MIX_WEIGHT",
            "MIX_ALL",
            "CONF_SHORT",
        ]
    )
    if kind in DOMAINS:
        q[kind] = rng.choice(DOMAINS[kind])
    elif kind == "MATCH_CONF":
        i = rng.randrange(5)
        step = rng.choice([-4000, -2000, -1000, -500, 500, 1000, 2000, 4000])
        q["MATCH_CONF"][i] = clamp_conf(q["MATCH_CONF"][i] + step)
        for j in range(1, 5):
            if q["MATCH_CONF"][j] < q["MATCH_CONF"][j - 1] - 2000:
                q["MATCH_CONF"][j] = q["MATCH_CONF"][j - 1]
    elif kind == "CONF_SHORT":
        base = rng.choice([20000, 24000, 28000, 30000, 31000, 32000, 32700])
        q["MATCH_CONF"] = [
            clamp_conf(base),
            clamp_conf(base + rng.choice([0, 500, 1000, 2000])),
            32700,
            32700,
            32700,
        ]
        for j in range(1, 5):
            if q["MATCH_CONF"][j] < q["MATCH_CONF"][j - 1]:
                q["MATCH_CONF"][j] = q["MATCH_CONF"][j - 1]
    elif kind == "MIX_ALL":
        step = rng.choice([-30, -20, -10, 10, 20, 30])
        q["MIX_WEIGHT"] = [clamp_mix(x + step) for x in q["MIX_WEIGHT"]]
    else:
        i = rng.randrange(5)
        step = rng.choice([-40, -20, -10, -5, 5, 10, 20, 40])
        q["MIX_WEIGHT"][i] = clamp_mix(q["MIX_WEIGHT"][i] + step)
        for j in range(1, 5):
            if q["MIX_WEIGHT"][j] < q["MIX_WEIGHT"][j - 1] - 20:
                q["MIX_WEIGHT"][j] = q["MIX_WEIGHT"][j - 1]
    if q["BUF_BITS"] + q["HASH_BITS"] > 51:
        if rng.random() < 0.5:
            q["BUF_BITS"] = min(q["BUF_BITS"], 25)
        else:
            q["HASH_BITS"] = min(q["HASH_BITS"], 25)
    if not ram_ok(q):
        q["BUF_BITS"] = 25
        q["HASH_BITS"] = 26
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
                q = copy_params(p)
                q[key] = vals[j]
                if ram_ok(q):
                    out.append((f"{key}={vals[j]}", q))
    for i in range(5):
        for step in (-2000, -500, 500, 2000):
            q = copy_params(p)
            q["MATCH_CONF"][i] = clamp_conf(q["MATCH_CONF"][i] + step)
            out.append((f"MATCH_CONF[{i}]{step:+d}", q))
        for step in (-30, -10, 10, 30):
            q = copy_params(p)
            q["MIX_WEIGHT"][i] = clamp_mix(q["MIX_WEIGHT"][i] + step)
            out.append((f"MIX_WEIGHT[{i}]{step:+d}", q))
    return out


def focused_candidates(base: dict) -> list[tuple[str, dict]]:
    """Dense local grid around the known winner — highest prior value."""
    out: list[tuple[str, dict]] = []

    for r in [85, 95, 105, 110, 115, 118, 120, 122, 125, 128, 135, 145, 155]:
        q = copy_params(base)
        q["ORDER2_RESCALE"] = r
        out.append((f"r2:res{r}", q))

    mixes = [
        [80, 140, 190, 220, 240],
        [90, 150, 195, 225, 245],
        [100, 160, 200, 230, 245],
        [100, 160, 200, 230, 250],
        [100, 160, 200, 240, 250],
        [110, 170, 210, 235, 248],
        [120, 180, 215, 240, 250],
        [130, 185, 220, 240, 250],
        [140, 190, 220, 240, 250],
        [150, 200, 225, 245, 250],
        [60, 120, 180, 220, 245],
        [100, 140, 180, 220, 245],
        [100, 180, 210, 235, 250],
        [70, 130, 190, 230, 250],
        [110, 160, 200, 230, 245],
        [100, 160, 210, 235, 250],
        [90, 160, 200, 230, 245],
        [100, 150, 200, 230, 245],
        [100, 160, 190, 230, 245],
        [100, 160, 200, 220, 245],
        [120, 160, 200, 230, 245],
        [100, 170, 200, 230, 245],
        [100, 160, 200, 230, 240],
        [160, 200, 230, 245, 250],
        [200, 220, 235, 245, 250],
        [40, 100, 160, 210, 245],
        [250, 250, 250, 250, 250],
        [180, 200, 220, 235, 245],
    ]
    for m in mixes:
        q = copy_params(base)
        q["MIX_WEIGHT"] = [clamp_mix(x) for x in m]
        out.append((f"r2:mix{'-'.join(map(str, m))}", q))

    for c0, c1 in [
        (32700, 32700),
        (32000, 32700),
        (31000, 32700),
        (30000, 32000),
        (28000, 31000),
        (26000, 30000),
        (24000, 28000),
        (20000, 26000),
        (16000, 24000),
        (12000, 20000),
        (8000, 16000),
        (31000, 32000),
        (32500, 32700),
        (32700, 32000),
    ]:
        q = copy_params(base)
        q["MATCH_CONF"] = [
            clamp_conf(c0),
            clamp_conf(c1),
            32700,
            32700,
            32700,
        ]
        out.append((f"r2:conf{c0}-{c1}", q))

    for ho, mml in [
        (5, 5),
        (5, 6),
        (6, 5),
        (6, 6),
        (6, 7),
        (7, 6),
        (7, 7),
        (4, 5),
        (8, 6),
    ]:
        q = copy_params(base)
        q["HASH_ORDER"] = ho
        q["MIN_MIX_LEN"] = mml
        out.append((f"r2:ho{ho}_m{mml}", q))

    for buf, hb in [(25, 26), (26, 25), (26, 26), (24, 26), (25, 25)]:
        q = copy_params(base)
        q["BUF_BITS"] = buf
        q["HASH_BITS"] = hb
        if ram_ok(q):
            out.append((f"r2:b{buf}_h{hb}", q))

    for r in [110, 115, 120, 125]:
        for m in (
            [110, 170, 210, 235, 248],
            [120, 180, 215, 240, 250],
            [90, 150, 195, 225, 245],
        ):
            q = copy_params(base)
            q["ORDER2_RESCALE"] = r
            q["MIX_WEIGHT"] = list(m)
            out.append((f"r2:joint_r{r}_mix{m[0]}", q))

    for m0 in (80, 100, 130, 160):
        q = copy_params(base)
        q["MIN_MIX_LEN"] = 5
        q["MIX_WEIGHT"] = [m0, 160, 200, 230, 245]
        out.append((f"r2:mml5_mix0={m0}", q))

    seen: set[str] = set()
    uniq: list[tuple[str, dict]] = []
    for note, p in out:
        key = json.dumps(p, sort_keys=True)
        if key in seen:
            continue
        seen.add(key)
        uniq.append((note, p))
    return uniq


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=9328)
    ap.add_argument("--minutes", type=float, default=38.0)
    ap.add_argument("--max-trials", type=int, default=220)
    ap.add_argument("--reset-trials", action="store_true")
    ap.add_argument("--skip-baseline", action="store_true")
    args = ap.parse_args()

    ensure_header(args.reset_trials)
    rng = random.Random(args.seed)
    t0 = time.time()
    deadline = t0 + args.minutes * 60.0

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

    best_p = copy_params(WINNER)
    trial = 0
    evaluated: set[str] = set()

    def try_one(cand: dict, note: str) -> None:
        nonlocal trial, best_p, best_size
        if trial >= args.max_trials or time.time() >= deadline:
            return
        key = json.dumps(cand, sort_keys=True)
        if key in evaluated:
            return
        if not ram_ok(cand):
            return
        evaluated.add(key)
        apply_params(cand)
        try:
            size = eval_slice()
        except subprocess.CalledProcessError as e:
            print(f"trial failed: {e}", flush=True)
            return
        trial += 1
        log_trial(trial, t0, size, best_size, cand, note)
        mark = ""
        if size < best_size:
            best_size = size
            best_p = copy_params(cand)
            BEST_JSON.write_text(json.dumps(best_p, indent=2))
            mark = " *NEW BEST*"
        print(f"t{trial} {size} (best {best_size}) {note}{mark}", flush=True)

    if not args.skip_baseline:
        apply_params(best_p)
        best_size = eval_slice()
        trial += 1
        evaluated.add(json.dumps(best_p, sort_keys=True))
        log_trial(trial, t0, best_size, best_size, best_p, "r2:baseline_winner")
        BEST_JSON.write_text(json.dumps(best_p, indent=2))
        print(f"baseline slice={best_size}", flush=True)
    else:
        best_size = 10**18

    for note, cand in focused_candidates(best_p):
        if trial >= args.max_trials or time.time() >= deadline:
            break
        try_one(cand, note)

    improved = True
    while improved and trial < args.max_trials and time.time() < deadline:
        improved = False
        for note, cand in coordinate_neighbors(best_p):
            if trial >= args.max_trials or time.time() >= deadline:
                break
            before = best_size
            try_one(cand, f"cd:{note}")
            if best_size < before:
                improved = True
                break

    while trial < args.max_trials and time.time() < deadline:
        cand = mutate(rng, best_p)
        try_one(cand, "evo")

    apply_params(best_p)
    BEST_JSON.write_text(json.dumps(best_p, indent=2))
    print(f"DONE trials={trial} best_slice={best_size}", flush=True)
    print(json.dumps(best_p), flush=True)


if __name__ == "__main__":
    main()
