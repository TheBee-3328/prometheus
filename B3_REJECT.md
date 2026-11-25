# B3-CTX REJECT (orch)

**Branch:** `agent/b3-ctx`  
**Base:** main `0c583bb` (A1 mix SCORE 20852107)  
**Slice:** first 25 MiB of enwik8 (`tune/b3_25mb.bin`)  
**Slice baseline:** **5699220**

## Gate

Need slice Δ ≤ **−15000**. Best observed: **−3774** → **ABORT** (no full enwik8 judge).

## Trials (`tune/b3_trials.csv`)

| # | Label | Compressed | Δ | Verdict |
|--:|-------|----------:|--:|---------|
| 0 | baseline | 5699220 | 0 | — |
| 1 | soft early match (len 3–5) | 5699220 | 0 | noop (match starts at HASH_ORDER=6) |
| 2 | accept≥4 + soft blend | 5699233 | +13 | REJECT |
| 3 | **MATCH_CANDIDATES 4→8** | **5695446** | **−3774** | best; still short of gate |
| 4 | adaptive match StateMap | 5813312 | +114092 | REJECT (dilutes max conf) |
| 5 | APM2 class×mb×bp | 5705899 | +6679 | REJECT |
| 6 | prev_whash into word hash | 5809992 | +110772 | REJECT |
| 7 | HASH_ORDER 6→7 | 5702956 | +3736 | REJECT |
| 8 | APM fold match-distance | 5703583 | +4363 | REJECT |
| 9 | sparse blend under match (w/ cand8) | 5698401 | −819 | worse than cand8 alone |

## Notes

- Did **not** resume SSE2/mixer 1D grids (B2 rejected).
- Did **not** redo B1 hash/checksum table growth.
- Deeper APM / match-conf learning / word-adjacent folding all hurt or were noise vs −15KB bar.
- Source restored to main tip; no KEEP candidate.

## Next

Lane closed. Orch: try another angle (template/C1, or non-1D structural elsewhere).
