# STATUS

**Current phase:** Track 0 refresh done (2026-11-25) — re-measure on main best; re-rank  
**Poll cadence:** **2 minutes**  
**enwik9:** NEVER auto-start (manual only)

## Locked best enwik8

```
ROUNDTRIP=OK COMPRESSED=20765227 DECOMPRESSOR=86880 SCORE=20852107
```
**Gap to ~11e6:** ≈ 9.7 MB  
**Tip (this branch):** see latest commit on `agent/t0-refresh`

## Track 0 refresh — ranked next experiments

Ranked by **max possible gain** from full-enwik8 cost shares (A1+word dict+B1). Caps = share × COMPRESSED (upper bound if class cost → 0).

| Rank | Track | Evidence | Max possible (cap) | Notes |
|-----:|-------|----------|-------------------:|-------|
| 1 | **Residual NL / prose contexts** | structure `other` 81.4%; byte `alpha` 35.8% + class-`other` 22.9% | **≲16.9 MB** (struct other) / **≲12.2 MB** (alpha+other bytes) | Largest headroom; new CM/word/order contexts — not rejected lanes |
| 2 | **Whitespace model** | byte `ws` **24.0%** | **≲5.0 MB** | Still expensive after match (62% bits match-active) |
| 3 | **Hibyte / token CM** | byte `hibyte` **13.3%** (markup+word tokens) | **≲2.8 MB** | Improve modeling of existing dict tokens |
| 4 | **URL preprocessor / URL CM** | structure `url` **2.38%** | **≲0.50 MB** | Bounded; cheap if reversible |
| 5 | **Digit / numeric contexts** | byte `digit` **3.57%** | **≲0.74 MB** | Modest |
| 6 | **Entity / XML chrome** | structure entity+xml **0.82%** | **≲0.17 MB** | Near floor after markup dict |
| 7 | **Coder / p precision** | 16-bit p; **p_pin=0/0**; flush ≪1KB | **~0** | No free bug; do not chase |

### Closed / do not reopen

| Lane | Why |
|------|-----|
| **C1 templates** | THROW — still **15.36%** cost share (~3.2 MB cap) but transform path failed SCORE |
| **B2 SSE grids** | Rejected |
| **B3 match/APM** | Rejected (match already 82% hit bytes / 62% active bits) |
| **B4 hierarchical mix** | Rejected |

### Orch note

Do **not** chase SCORE wins from Track 0 unless a coder fix is obviously free (none found). Prefer rank-1/2 probes over reopening C1/B2/B3/B4.

## Process hygiene (2026-11-25)

- Removed **12** obsolete worktrees (a1, b1, diag*, p5*, tuner*).
- Kept: `main`, `b2-mixer`, `c1-tmpl`, `pre-v2` (+ `t0-refresh`).
- No zombie compress/wait loops.

## In flight (orch)

- Superseded for planning by Track 0 ranking above; C1 THROW / B2 capped per orch.
