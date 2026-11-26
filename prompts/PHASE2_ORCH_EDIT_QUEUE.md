# Phase 2 — edit / spawn queue (orch)

Use these briefs under `prompts/` when spawning agents. **Do not** edit `judge_frozen_enwik9.sh` while enwik9 runs.

## Order (budget rule)
1. `PHASE2_SPEED_BITIDENTICAL.md` — free CPU for later growth  
2. `PHASE2_MATCH_HASH_BUDGET.md` — RAM/CPU vs bits on largest tables  
3. `PHASE2_PREPROCESS_SCORE.md` — SCORE including decoder cost  
4. `PHASE2_CM_SCALE.md` — bigger CM only with headroom  
5. Neural/LSTM — **blocked** until §2.4 full determinism PASS + speed/CM ratios known  

## Ablation CSV
`/tmp/agent-home/.prometheus/parallel/ablate_mix.csv` (local; summarize into DIAG when committing).

## Attribution
Commits: author **TheBee-3328** `<thebee3328@gmail.com>`; Co-authored-by Adam Grman. **Strip** `Co-authored-by: Cursor <cursoragent@cursor.com>` if injected.
