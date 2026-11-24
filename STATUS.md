# STATUS

**Current phase:** LIVE — post DIAG-FIX; next toward 20.5e6  
**Current best SCORE:** **23069165** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=23036149 DECOMPRESSOR=33016 SCORE=23069165 …`  
**bpB:** ≈ **1.843**  
**vs floors:** beats zstd 26395074 and xz 24831656  
**Gap to 20.5e6 compressed:** ≈ 2.54 MB  
**Stop policy:** Operator manual stop.  

## Merged this session

| Task | SCORE | Notes |
|---|---|---|
| TUNER-R2 | 28042709 | params |
| P5-SPARSE-XML | 28011126 | struct CM |
| P5-REVIEW | docs | |
| **DIAG-FIX** | **23069165** | o0–o6, word, SSE×2; −17.6% |

## Next (gain/cost from DIAG.md)

1. **DIAG-R2 / MIXER-APM** — working adaptive mixer + APM chain (naive mixer rejected; need careful design)
2. u16 counters / bit-history states
3. Stronger preprocessor / multi-match
4. enwik9 only after clear path under time/RSS

## In flight

Spawning next builder toward 20.5e6.
