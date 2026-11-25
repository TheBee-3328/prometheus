#!/usr/bin/env python3
"""A1-RETUNE: focused param sweep on 25MB enwik8 slice (word-dict stream).

Resumes from tune/a1_trials.csv (skips completed notes). Waits for a free
compress slot before every trial.
"""
from __future__ import annotations

import copy
import csv
import json
import os
import pathlib
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tune"))
from write_params_full import DEFAULTS, write_params  # noqa: E402

EVAL = ROOT / "tune" / "a1_slice_eval.sh"
OUT_CSV = ROOT / "tune" / "a1_trials.csv"
OUT_JSON = ROOT / "tune" / "a1_sweep_results.json"
CAND_DIR = ROOT / "tune" / "candidates"

BASELINE = 0
DONE: set[str] = set()


def active_compress_pids() -> list[str]:
    """PIDs whose argv is the prometheus-compress binary (ignore bash wrappers)."""
    r = subprocess.run(["ps", "-eo", "pid=,args="], capture_output=True, text=True)
    pids: list[str] = []
    my_marker = str(ROOT / "target-a1")
    for line in r.stdout.splitlines():
        line = line.strip()
        if not line:
            continue
        parts = line.split(None, 1)
        if len(parts) < 2:
            continue
        pid, args = parts[0], parts[1]
        if "bash" in args or "nix develop" in args:
            continue
        if "/prometheus-compress " in args or args.startswith("prometheus-compress "):
            # Allow our own in-flight compress during eval
            if my_marker in args:
                continue
            pids.append(pid)
        elif args.endswith("/prometheus-compress") or args == "prometheus-compress":
            if my_marker in args:
                continue
            pids.append(pid)
    return pids


def wait_for_clear(max_wait: int = 10800) -> None:
    t0 = time.time()
    while True:
        pids = active_compress_pids()
        if not pids:
            return
        if time.time() - t0 > max_wait:
            raise SystemExit(f"timeout waiting for compress clear: {pids}")
        print(f"waiting for compress pids={pids} ...", flush=True)
        time.sleep(20)


def load_done() -> None:
    global BASELINE, DONE
    if not OUT_CSV.exists():
        OUT_CSV.write_text(
            "note,elapsed_s,slice_compressed,delta_vs_baseline,params_json\n"
        )
        return
    with OUT_CSV.open() as f:
        reader = csv.DictReader(f)
        for row in reader:
            note = row["note"]
            DONE.add(note)
            if note == "baseline":
                BASELINE = int(row["slice_compressed"])
    print(f"resume: {len(DONE)} trials done; baseline={BASELINE}", flush=True)


def eval_params(p: dict, note: str) -> int | None:
    if note in DONE:
        print(f"skip {note}", flush=True)
        # recover size from csv for ranking
        with OUT_CSV.open() as f:
            for row in csv.DictReader(f):
                if row["note"] == note:
                    return int(row["slice_compressed"])
        return None
    wait_for_clear()
    write_params(p)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(ROOT / "target-a1")
    env["A1_ALLOW_COMPRESS"] = "1"
    t0 = time.time()
    out = subprocess.check_output(["bash", str(EVAL)], cwd=str(ROOT), text=True, env=env)
    elapsed = time.time() - t0
    size = None
    for line in out.splitlines():
        if line.startswith("COMPRESSED="):
            size = int(line.split("=", 1)[1])
    if size is None:
        raise RuntimeError(out)
    delta = size - BASELINE if BASELINE else 0
    # CSV-safe: escape quotes in json
    pj = json.dumps(p, separators=(",", ":")).replace('"', '""')
    with OUT_CSV.open("a") as f:
        f.write(f'{note},{elapsed:.1f},{size},{delta},"{pj}"\n')
    DONE.add(note)
    print(f"{size:8d}  Δ={delta:+7d}  {elapsed:5.1f}s  {note}", flush=True)
    return size


def main() -> None:
    global BASELINE
    os.chdir(ROOT)
    CAND_DIR.mkdir(exist_ok=True)
    load_done()

    print("=== waiting for initial clear slot ===", flush=True)
    wait_for_clear()

    base = copy.deepcopy(DEFAULTS)
    print("=== A1 baseline ===", flush=True)
    b = eval_params(base, "baseline")
    if b is not None:
        BASELINE = b

    results: list[tuple[int, str, dict]] = []
    # Seed results from CSV
    with OUT_CSV.open() as f:
        for row in csv.DictReader(f):
            # params_json may be broken by naive quoting; rebuild from note when needed
            results.append((int(row["slice_compressed"]), row["note"], {}))

    best_1d: dict[str, tuple[int, object]] = {}

    # Lean grid (~25 new 1D trials) under multi-agent compress contention.
    sweeps: list[tuple[str, list]] = [
        ("ORDER2_RESCALE", [200, 220, 235, 280, 320]),  # 200/220 already done (worse)
        ("APM_RATE_SHIFT", [6, 8]),
        ("APM_MIX_WEIGHT", [208, 240]),
        ("SSE_MIX_WEIGHT", [208, 240]),
        ("SSE2_MIX_WEIGHT", [176, 208]),
        ("WORD_MIX_WEIGHT", [48, 56, 72, 80]),
        ("ORDER0_MIX_WEIGHT", [64, 96]),
        ("ORDER1_MIX_WEIGHT", [4, 12]),
        ("ORDER3_MIX_WEIGHT", [144, 176]),
        ("ORDER4_MIX_WEIGHT", [40, 56]),
        ("ORDER5_MIX_WEIGHT", [72, 104]),
        ("ORDER6_MIX_WEIGHT", [24, 40]),
        ("ORDER7_MIX_WEIGHT", [8, 24]),
        ("STRUCT_MIX_WEIGHT", [16, 32]),
        ("INDIRECT_MIX_WEIGHT", [32, 48]),
        ("MATCH_IND_CROSS_WEIGHT", [16, 28]),
        ("MIN_MIX_LEN", [5, 7]),
    ]

    for key, values in sweeps:
        print(f"=== sweep {key} ===", flush=True)
        for v in values:
            if base.get(key) == v and f"{key}={v}" != "ORDER2_RESCALE=250":
                # baseline covers default value
                if key == list(sweeps[0])[0] and False:
                    pass
            note = f"{key}={v}"
            if base.get(key) == v:
                # treat baseline as measurement for this value
                best_1d.setdefault(key, (BASELINE, v))
                if best_1d[key][0] > BASELINE:
                    best_1d[key] = (BASELINE, v)
                continue
            p = copy.deepcopy(base)
            p[key] = v
            size = eval_params(p, note)
            if size is None:
                continue
            results.append((size, note, p))
            prev = best_1d.get(key)
            if prev is None or size < prev[0]:
                best_1d[key] = (size, v)
        # Ensure default is considered best_1d candidate
        if key not in best_1d or BASELINE < best_1d[key][0]:
            best_1d[key] = (BASELINE, base[key])

    print("=== MIX_WEIGHT buckets (lean) ===", flush=True)
    for bi, values in enumerate(
        [[110, 130], [150, 170], [180, 200], [220, 240], [240, 250]]
    ):
        for v in values:
            if base["MIX_WEIGHT"][bi] == v:
                continue
            p = copy.deepcopy(base)
            p["MIX_WEIGHT"] = list(base["MIX_WEIGHT"])
            p["MIX_WEIGHT"][bi] = v
            note = f"MIX_WEIGHT[{bi}]={v}"
            size = eval_params(p, note)
            if size is not None:
                results.append((size, note, p))

    # Reload all results with params for ranking (re-parse CSV carefully)
    ranked: list[tuple[int, str, dict]] = []
    with OUT_CSV.open() as f:
        for row in csv.DictReader(f):
            size = int(row["slice_compressed"])
            note = row["note"]
            try:
                # DictReader handles quoted fields
                pj = row["params_json"]
                params = json.loads(pj) if pj else {}
            except json.JSONDecodeError:
                params = {}
            ranked.append((size, note, params))

    gains = sorted(
        (
            (BASELINE - sz, k, v)
            for k, (sz, v) in best_1d.items()
            if sz < BASELINE - 20
        ),
        reverse=True,
    )
    print(f"=== 1D gains (>{20}B): {gains[:8]} ===", flush=True)
    for nstack in (1, 2, 3, 4):
        if len(gains) < nstack:
            break
        p = copy.deepcopy(base)
        parts = []
        for _, k, v in gains[:nstack]:
            p[k] = v
            parts.append(f"{k}={v}")
        note = f"top{nstack}:" + "+".join(parts)
        size = eval_params(p, note)
        if size is not None:
            ranked.append((size, note, p))

    # Refresh ranked from CSV
    ranked = []
    with OUT_CSV.open() as f:
        for row in csv.DictReader(f):
            try:
                params = json.loads(row["params_json"])
            except json.JSONDecodeError:
                params = copy.deepcopy(base)
            ranked.append((int(row["slice_compressed"]), row["note"], params))
    ranked.sort(key=lambda x: x[0])
    top = ranked[:10]
    OUT_JSON.write_text(
        json.dumps(
            {
                "baseline": BASELINE,
                "top": [{"size": s, "note": n, "params": p} for s, n, p in top],
                "all_count": len(ranked),
                "gains_1d": [
                    {"key": k, "value": v, "size": s, "delta": s - BASELINE}
                    for k, (s, v) in best_1d.items()
                ],
            },
            indent=2,
        )
        + "\n"
    )

    seen = set()
    written = 0
    for s, n, p in ranked:
        if n == "baseline" or s >= BASELINE or not p:
            continue
        key = json.dumps(p, sort_keys=True)
        if key in seen:
            continue
        seen.add(key)
        name = ["A", "B", "C"][written]
        path = CAND_DIR / f"A1_{name}.json"
        path.write_text(
            json.dumps(
                {"note": n, "slice": s, "delta": s - BASELINE, "params": p}, indent=2
            )
            + "\n"
        )
        print(f"CANDIDATE {name}: slice={s} Δ={s - BASELINE} {n}", flush=True)
        written += 1
        if written >= 3:
            break

    write_params(DEFAULTS)
    print(f"BASELINE_SLICE={BASELINE}", flush=True)
    if top:
        print(f"BEST_SLICE={top[0][0]} {top[0][1]}", flush=True)
    if written == 0:
        print("NO_IMPROVING_CANDIDATES", flush=True)
    print("SWEEP_DONE", flush=True)


if __name__ == "__main__":
    main()
