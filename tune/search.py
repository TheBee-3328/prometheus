#!/usr/bin/env python3
"""Coordinate descent + seeded evolutionary search over params.rs.

Uses a fixed 25 MB enwik8 slice (offset 8_000_000). Logs every trial to
tune/trials.csv. Budget controlled by --minutes / --max-trials.

Seeds from tune/best_params.json when present (else params.rs / DEFAULTS).
"""
from __future__ import annotations

import argparse
import csv
import json
import os
import pathlib
import random
import re
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
TRIALS = ROOT / "tune" / "trials.csv"
BEST_JSON = ROOT / "tune" / "best_params.json"
PARAMS_RS = ROOT / "crates" / "prometheus-core" / "src" / "params.rs"
WRITE = ROOT / "tune" / "write_params.py"
SLICE_EVAL = ROOT / "tune" / "slice_eval.sh"

# Legacy fallback only — never use as search origin when best_params.json exists.
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
# Tuned around known-best: BUF=25 HASH=26 HASH_ORDER=6 MIN_MIX_LEN=6 max-conf.
DOMAINS = {
    "ORDER2_RESCALE": [80, 100, 110, 120, 130, 140, 160, 200, 250],
    "BUF_BITS": [24, 25, 26],
    "HASH_BITS": [24, 25, 26],
    "HASH_ORDER": [5, 6],  # 7+ already lost on this slice; avoid P4-style thrash
    "MIN_MIX_LEN": [5, 6, 7],
}


def clamp_conf(v: int) -> int:
    return max(2000, min(32700, int(v)))


def clamp_mix(v: int) -> int:
    return max(20, min(250, int(v)))


def load_seed() -> dict:
    """Prefer best_params.json, then parse params.rs, else DEFAULTS."""
    if BEST_JSON.exists():
        p = json.loads(BEST_JSON.read_text())
        print(f"seed: loaded {BEST_JSON}", flush=True)
        return normalize(p)
    text = PARAMS_RS.read_text()
    try:
        p = {
            "ORDER2_RESCALE": int(re.search(r"ORDER2_RESCALE:\s*u16\s*=\s*(\d+)", text).group(1)),
            "BUF_BITS": int(re.search(r"BUF_BITS:\s*usize\s*=\s*(\d+)", text).group(1)),
            "HASH_BITS": int(re.search(r"HASH_BITS:\s*usize\s*=\s*(\d+)", text).group(1)),
            "HASH_ORDER": int(re.search(r"HASH_ORDER:\s*usize\s*=\s*(\d+)", text).group(1)),
            "MIN_MIX_LEN": int(re.search(r"MIN_MIX_LEN:\s*u32\s*=\s*(\d+)", text).group(1)),
            "MATCH_CONF": [int(x) for x in re.search(
                r"MATCH_CONF:\s*\[u32;\s*5\]\s*=\s*\[([^\]]+)\]", text
            ).group(1).split(",")],
            "MIX_WEIGHT": [int(x) for x in re.search(
                r"MIX_WEIGHT:\s*\[u32;\s*5\]\s*=\s*\[([^\]]+)\]", text
            ).group(1).split(",")],
        }
        print(f"seed: parsed {PARAMS_RS}", flush=True)
        return normalize(p)
    except Exception as e:
        print(f"seed: fallback DEFAULTS ({e})", flush=True)
        return normalize(DEFAULTS)


def normalize(p: dict) -> dict:
    q = dict(DEFAULTS)
    q.update(p)
    q["MATCH_CONF"] = [clamp_conf(x) for x in list(q["MATCH_CONF"])]
    q["MIX_WEIGHT"] = [clamp_mix(x) for x in list(q["MIX_WEIGHT"])]
    for k in ("ORDER2_RESCALE", "BUF_BITS", "HASH_BITS", "HASH_ORDER", "MIN_MIX_LEN"):
        q[k] = int(q[k])
    return q


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
    # Bias toward MIX_WEIGHT / ORDER2_RESCALE — MATCH_CONF already maxed at best.
    kind = rng.choice(
        [
            "ORDER2_RESCALE",
            "ORDER2_RESCALE",
            "BUF_BITS",
            "HASH_BITS",
            "HASH_ORDER",
            "MIN_MIX_LEN",
            "MATCH_CONF",
            "MIX_WEIGHT",
            "MIX_WEIGHT",
            "MIX_WEIGHT",
            "MIX_WEIGHT",
        ]
    )
    if kind in DOMAINS:
        q[kind] = rng.choice(DOMAINS[kind])
    elif kind == "MATCH_CONF":
        i = rng.randrange(5)
        # Prefer high-conf region near 32700.
        step = rng.choice([-800, -400, -200, -100, 100, 200])
        q["MATCH_CONF"][i] = clamp_conf(q["MATCH_CONF"][i] + step)
        for j in range(1, 5):
            if q["MATCH_CONF"][j] < q["MATCH_CONF"][j - 1] - 2000:
                q["MATCH_CONF"][j] = q["MATCH_CONF"][j - 1]
    else:
        i = rng.randrange(5)
        step = rng.choice([-30, -20, -10, -5, 5, 10, 20, 30])
        q["MIX_WEIGHT"][i] = clamp_mix(q["MIX_WEIGHT"][i] + step)
        for j in range(1, 5):
            if q["MIX_WEIGHT"][j] < q["MIX_WEIGHT"][j - 1] - 20:
                q["MIX_WEIGHT"][j] = q["MIX_WEIGHT"][j - 1]
    # Soft RAM guard.
    if q["BUF_BITS"] + q["HASH_BITS"] > 51:
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
    # Finer MIX / near-max CONF steps around current.
    for i in range(5):
        for step in (-400, -200, 200):
            q = json.loads(json.dumps(p))
            q["MATCH_CONF"][i] = clamp_conf(q["MATCH_CONF"][i] + step)
            if q["MATCH_CONF"] != p["MATCH_CONF"]:
                out.append((f"MATCH_CONF[{i}]{step:+d}", q))
        for step in (-15, -10, -5, 5, 10, 15):
            q = json.loads(json.dumps(p))
            q["MIX_WEIGHT"][i] = clamp_mix(q["MIX_WEIGHT"][i] + step)
            if q["MIX_WEIGHT"] != p["MIX_WEIGHT"]:
                out.append((f"MIX_WEIGHT[{i}]{step:+d}", q))
    return out


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=3328)
    ap.add_argument("--minutes", type=float, default=45.0)
    ap.add_argument("--max-trials", type=int, default=80)
    ap.add_argument(
        "--skip-baseline",
        action="store_true",
        help="Skip re-logging baseline (still measures seed once for deltas).",
    )
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

    best_p = load_seed()
    trial = 0

    # Always apply + measure seed so deltas are vs real best, not DEFAULTS.
    apply_params(best_p)
    best_size = eval_slice()
    trial += 1
    note = "seed_baseline" if args.skip_baseline else "baseline"
    log_trial(trial, t0, best_size, best_size, best_p, note)
    print(f"seed baseline slice={best_size} params={json.dumps(best_p)}", flush=True)
    BEST_JSON.write_text(json.dumps(best_p, indent=2))

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
