# PAUSE — resume after tmux reload

**When:** 2026-11-24 ~04:20 UTC+1  
**Why:** Operator interrupted to load session into tmux.

## Do first

1. Read `STATUS.md` pause snapshot.
2. In `/tmp/workspace/prometheus-tuner-r3` on `agent/tuner-r3`:
   - Apply `tune/candidates/R3_r102_only.json` via `tune/write_params.py`
   - Confirm `nix develop -c ./judge.sh enwik8 .` → SCORE **28010892** (or better)
   - `nix develop -c ./tests/adversarial_roundtrip.sh` → 10/10
   - Commit + push branch; orch merge to main; update LOG/STATUS
3. Continue orch loop (no 40/8 auto-stop). Next after merge: feature ideas from P5 (gated sparse / SSE) or further tune.

## Do not

- Retry P2-WORD / P4-ORDERS-MIX
- Lose `R3_r102_only` validate out / JSON
- Auto-start assuming clean TUNER-R3 commit (there isn’t one yet)

**RESUMED** 2026-11-24 ~04:25 — see STATUS.md / DIAG agent.
