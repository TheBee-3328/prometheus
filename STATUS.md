# STATUS

**Current phase:** LIVE — DIAG-R6 toward ≤20.5e6  
**Current best SCORE:** **21583876** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=21546316 DECOMPRESSOR=37560 SCORE=21583876`  
**bpB:** ≈ **1.724**  
**Gap to 20.5e6:** ≈ 1.05 MB  
**Stop policy:** Operator manual stop.  

## Session SCORE path (abbrev)

28080343 → … → 21887167 → 21778410 → **21583876**

## Merged

- DIAG-R5 (`agent/diag-r5`): ind2 cross + MIX=22 + soft-reset n→1 → SCORE 21778410

## In flight

- DIAG-R6: asymptotic StateMap n+=1-(n>>3) → SCORE 21583876; mixer retune / word hybrid next

## Next (DIAG-R6)

1. Mixer retune after asymptotic schedule (ORDER3/5/6, SSE, APM, CROSS)
2. Word / o5 Laplace→hybrid dual-rate
3. RSS trim before enwik9 re-measure
