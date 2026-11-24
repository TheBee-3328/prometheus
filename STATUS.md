# STATUS

**Current phase:** LIVE — DIAGNOSE-AND-FIX in flight  
**Current best SCORE:** **28011126** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=27981374 DECOMPRESSOR=29752 SCORE=28011126 …`  
**Bits/byte (approx):** 27981374 * 8 / 1e8 ≈ **2.239**  
**Stop policy:** Operator manual stop; no 40/8 auto-stop.  

## Resume notes

- TUNER-R3 claimed 28010892 **did not reproduce** (re-measure 28013201). FAIL report on `agent/tuner-r3`. Do not merge.
- Likely cause: overlapping validates in one worktree.

## In flight

| Task | Branch | Goal |
|---|---|---|
| DIAG-FIX | `agent/diag-fix` | Measure vs xz/zstd; ablation; fix hypotheses one-by-one; target ≤20.5 MB enwik8 |

## Merged this session

| Task | SCORE |
|---|---|
| TUNER-R2 | 28042709 |
| P5-SPARSE-XML | 28011126 |
| P5-REVIEW | docs |
