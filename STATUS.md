# STATUS

**Current phase:** LIVE — DIAG-R6 toward ≤20.5e6  
**Current best SCORE:** **21778410** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=21740866 DECOMPRESSOR=37544 SCORE=21778410`  
**bpB:** ≈ **1.739**  
**Gap to 20.5e6:** ≈ 1.24 MB  
**Stop policy:** Operator manual stop.  

## Session SCORE path (abbrev)

28080343 → … → 23069165 → 22075936 → 21887167 → **21778410**

## Merged

- DIAG-R5 (`agent/diag-r5`): ind2 cross + MIX=22 + soft-reset n→1 → SCORE 21778410

## In flight

- enwik9 judge on **pre-R5** main binary (DECOMP=37512); first-pass COMPRESSED≈186549457; final ROUNDTRIP pending
- DIAG-R6 next (soft-reset schedule / mixer retune / word-o5 hybrid / RSS trim)

## Next (DIAG-R6)

1. Soft-reset schedule (per-order or decay) vs fixed n→1
2. Mixer retune after soft-reset=1 (ORDER3/5/6, SSE, APM)
3. Word / o5 Laplace→hybrid dual-rate
4. RSS trim before enwik9 re-measure
