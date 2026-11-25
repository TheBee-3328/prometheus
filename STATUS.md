# STATUS

**Current phase:** LIVE — cleaned; C1 mine + B2 capped toward ~10–12 MB enwik8  
**Poll cadence:** **2 minutes**  
**enwik9:** NEVER auto-start (manual only)

## Locked best enwik8

```
ROUNDTRIP=OK COMPRESSED=20765227 DECOMPRESSOR=86880 SCORE=20852107
```
**Gap to ~11e6:** ≈ 9.7 MB

## Process hygiene (2026-11-25)

- Removed **12** obsolete worktrees (a1, b1, diag*, p5*, tuner*).
- Kept: `main`, `b2-mixer`, `c1-tmpl`, `pre-v2`.
- No zombie compress/wait loops. Live compute: B2 25MB slices only (+ new C1-MINE).
- Stuck UI agents (old C1/B2) may still show “running” but have no extra procs — superseded by orch + C1-MINE.

## In flight

- **C1-MINE** — template mine/transform (gate PASS)
- **B2** — SSE2 sweep; orch will stop if best slice Δ > −15KB
