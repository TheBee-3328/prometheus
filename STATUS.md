# STATUS

**Current phase:** LIVE — DIAG-R4 toward ≤20.5e6  
**Current best SCORE:** **22075936** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=22039256 DECOMPRESSOR=36680 SCORE=22075936 …`  
**bpB:** ≈ **1.763**  
**Gap to 20.5e6:** ≈ 1.54 MB  
**Stop policy:** Operator manual stop.  

## Session SCORE path (abbrev)

28080343 → … → 23069165 → 22574501 → 22482437 → 22081312 → **22075936**

## Next (DIAG-R3 leftovers)

1. State-map / dual-rate bit-history  
2. Match secondary hash beyond chain  
3. Deeper indirect / match×indirect cross  
4. Fix STRETCH drift (+2 KB COMPRESSED)  
5. Trim match-chain RSS (~1.7 GB) before enwik9  

Rejected: sparse-gap, logistic mixer, APM2, u16 hash counters, paq SSE cells.
