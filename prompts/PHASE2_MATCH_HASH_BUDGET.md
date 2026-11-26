# Agent brief — Phase 2: match/hash cost vs bits (budget rule)

**Branch / worktree:** `agent/p2-match` from B5 tip.  
**CARGO_TARGET_DIR:** local. **Nix:** mandatory.

## Why this lane
Static profile: HASH_BITS=26, BUF_BITS=25 dominate RAM. Ablation did **not** turn match off (match is length-gated mix, not a single MIX const). Need measured **bytes saved per added GB / CPU-hour** before growing contexts.

## Goal
Find match/hash configurations that improve **SCORE** (or free ≥100 MB RSS / ≥5% CPU at ≤0 SCORE loss on enwik8), staying under working ceilings (4 h/dir, 9 GB) and official 10 GB.

## Method
1. Instrument or time match vs order paths on 25 MiB (optional `prometheus-diag`).
2. Slice sweep only: HASH_BITS, BUF_BITS, match mix schedule — one knob family at a time.
3. Gate: 25 MiB Δ ≤ **−15000** vs 5680631 before full enwik8 judge.
4. Record for every trial: Δ bytes, wall_s, peak RSS, **bytes per added CPU-s**, **bytes per added MB RSS**.

## Forbidden
- Neural / FP / threads.
- enwik9 until orch asks.
- Merging without enwik8 ROUNDTRIP=OK + SCORE improve (or equal + smaller decomp).

## Report
Top 3 hyps with budget ratios; KEEP only if enwik8 SCORE improves and limits PASS.
