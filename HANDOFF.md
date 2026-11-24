# Orchestrator handoff — PROMETHEUS

**Repo:** https://github.com/TheBee-3328/prometheus  
**Date:** 2026-11-24  
**Role:** You are the ORCHESTRATOR. Do not rewrite compressor code yourself; plan, spawn agents, merge only on judge PASS.

## Operator notice — ready to monitor

**Status for the human operator:** the tree is handoff-ready. **Do not start** the long loop until the operator explicitly says so (e.g. “start monitoring”, “go”, “begin unattended”).

When the operator **does** start it: say that they can monitor, then run **continuous monitoring + agent management** (spawn, poll, watchdog, merge, log) for **several hours** without pausing for confirmation.

### Unattended runtime rules (after explicit start only)

- **Stay up for hours.** Do not end the turn to “wait for the user.” Poll agents, apply `prompts/AGENT_WATCHDOG.md` timeouts, merge on judge PASS, update `LOG.md`/`STATUS.md`, push `main`, spawn the next best gain/cost task.
- **Do not stop** the loop unless there is a **risk that must not be left to AI judgment**, for example: credentials/secrets exposure, destructive disk wipe outside `/tmp` and abandoned worktrees, force-push to `main`, payment/legal commitment, or host instability that could lose the only copy of work (root disk full with no safe cleanup left).
- Soft failures (judge FAIL, SCORE regression, zombie agent, mid-flight resume blocked) are **normal** — log them, change approach per rules, continue.
- Budget: **operator override — no 40-run / 8-hour auto-stop.** Keep going until the operator stops the session manually (~15 h window). Track run counts in STATUS for visibility only. Still stop on critical risk (secrets, destructive wipe, force-push main, disk-full with no cleanup left).
- **Gate:** never auto-start on handoff alone — wait for the operator’s go.

## Current best (judge-measured, enwik8 only)

```
ROUNDTRIP=OK COMPRESSED=28051471 DECOMPRESSOR=28872 SCORE=28080343
COMPRESS_SECONDS=35.15 DECOMPRESS_SECONDS=27.11 PEAK_RSS_MB=583
```

- **Commit:** `fa7446b` (size pass) / `main` tip includes docs after  
- **Rough enwik9 extrapolation (NOT measured):** ~280 MB compressed (~3.6×). Prior zstd ~205 MB — still behind; do not claim enwik9 without `./judge.sh enwik9`.

## Checkpoints

| ID | Commit | SCORE | Meaning |
|---|---|---|---|
| CHKP-store | 8d6ba5d | 100299876 | Phase 0 baseline |
| CHKP-order2 | d170a6a | 38254897 | arithmetic + order-2 |
| CHKP-match-pre | c44bb4b | 36483469 | match + preprocessor |
| CHKP-tuned | a61e773 | 28355431 | params search |
| **CHKP-size** | **fa7446b** | **28080343** | build-std tiny decompressor |

## Failed twice / do not retry same brief

- **P2-WORD** — worse than order-2 alone  
- **P4-ORDERS-MIX** — SCORE 30.9M then 28.8M vs 28.4M best; abandoned (branch may linger; do not merge)  
- Parallel Phase-2 alone (mixer/orders) beat P1 but lost to match; only useful if *combined* carefully with tuned match+pre

## Time sinks

- Parallel builders fighting one worktree / dirty `main`  
- Mid-flight `Task.resume` **impossible** while subagent running (Cursor platform limit) — see `prompts/AGENT_WATCHDOG.md`  
- Full enwik8 judge ~1–3 min; don’t stack many compressors on 14 GB RAM hosts  

## Environment (mandatory)

- **Nix:** `nix develop -c …` (`flake.nix`). Nightly fenix for `-Z build-std`.  
- Data: do **not** expect committed `enwik8`/`enwik9`. Run `./scripts/reconstruct_data.sh both`.  
- Score: `SCORE = COMPRESSED + DECOMPRESSOR` (aux files if any).  

## Next recommended tasks (gain/cost)

1. **Tuner round 2** on `params.rs` (after size pass, <0.5% from size alone → REVIEWER+TUNER rule applies before big new features).  
2. **Sparse/XML contexts** that *ensemble with* match — not replace it (P4-ORDERS lesson).  
3. **enwik9 milestone** only after another clear enwik8 win.  
4. Optional: streaming I/O to cut peak RSS on enwik9.

## Unstick

- Frozen orch: paste `prompts/ORCHESTRATOR_UNSTICK.md`  
- Soft/hard timeouts: `prompts/AGENT_WATCHDOG.md`  
- Setup / auto-approve / unattended: `SETUP.md`

## Merge rule

`nix develop -c ./judge.sh enwik8 .` → `ROUNDTRIP=OK` and SCORE improved (or equal + smaller code). Update `LOG.md` + `STATUS.md`, push `main`.
