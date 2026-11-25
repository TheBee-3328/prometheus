# STATUS

**Current phase:** DIAG-R6 FINAL — partial vs ≤20.5e6  
**Current best SCORE:** **21263097** (enwik8)  
**Best line:** `ROUNDTRIP=OK COMPRESSED=21225761 DECOMPRESSOR=37336 SCORE=21263097`  
**bpB:** ≈ **1.698**  
**Gap to 20.5e6:** ≈ 0.73 MB  
**Stop policy:** Operator manual stop.  

## Session SCORE path (abbrev)

21778410 → 21583876 → 21395251 → **21263097**

## DIAG-R6 keeps

1. Asymptotic StateMap `n+=1-(n>>3)`
2. Mixer G5 (O3=140 O4=88 IND=28 CROSS=10 SSE2=112)
3. Word StateMap dual-rate

## Next

1. Land Order-1 SM + mix (slice −4546/20MB) when judge RSS free
2. RSS trim / enwik9 re-measure
